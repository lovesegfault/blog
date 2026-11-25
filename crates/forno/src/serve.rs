//! A preview that follows the files as they change.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result, anyhow};
use maud::{PreEscaped, html};

use crate::content::NOT_FOUND;
use crate::links::{decode, find};
use crate::output::{Files, walk};
use crate::programs;
use crate::site::{self, Tools};

#[derive(Clone, Default)]
struct Edition {
    version: u128,
    files: Arc<Files>,
    problems: Vec<String>,
}

#[derive(Default)]
struct Press {
    latest: Mutex<Edition>,
    changed: Condvar,
}

fn survey(roots: &[&str]) -> Vec<(Option<SystemTime>, PathBuf)> {
    let files = roots.iter().map(Path::new).flat_map(|root| walk(root).unwrap_or_else(|_| vec![root.to_owned()]));
    files.map(|file| (file.metadata().and_then(|metadata| metadata.modified()).ok(), file)).collect()
}

fn watch(tools: &Tools, press: &Press) -> ! {
    let (mut seen, mut programs) = ([Vec::new(), Vec::new()], Err(anyhow!("nothing is compiled yet")));
    loop {
        let now =
            [survey(&["crates/collision", "crates/explainers"]), survey(&["assets", "content", "static", "site.toml"])];
        if now == seen {
            thread::sleep(Duration::from_millis(150));
            continue;
        }
        let started = Instant::now();
        if now[0] != seen[0] {
            programs = programs::compile(&tools.cache);
        }
        seen = now;
        let programs = programs.as_ref().map_err(|error| anyhow!("{error:#}"));
        let build = AssertUnwindSafe(|| site::build(tools, programs?, true));
        let built = catch_unwind(build).unwrap_or_else(|_| Err(anyhow!("forno crashed: see the terminal")));

        let mut latest = press.latest.lock().unwrap();
        match built {
            Ok(site) => (latest.files, latest.problems) = (site.files.into(), site.problems.into_iter().collect()),
            Err(error) => latest.problems = vec![format!("{error:#}")],
        }
        latest.version = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis();
        press.changed.notify_all();
        latest.problems.iter().for_each(|problem| eprintln!("{problem}"));
        eprintln!("built in {:.2?}", started.elapsed());
    }
}

fn media_type(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap().to_lowercase().as_str() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "wasm" => "application/wasm",
        "pdf" => "application/pdf",
        "svg" => "image/svg+xml",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

fn respond(mut stream: TcpStream, press: &Press) -> Result<()> {
    let lines = BufReader::new(&stream).lines().map_while(Result::ok).take_while(|line| !line.is_empty());
    let request: Vec<String> = lines.collect();
    let target = request.first().and_then(|line| line.split([' ', '?']).nth(1)).context("not a request")?;
    let after = |version, patience| {
        let unchanged = |latest: &mut Edition| latest.version == version;
        press.changed.wait_timeout_while(press.latest.lock().unwrap(), patience, unchanged).unwrap().0.clone()
    };
    let mut edition = after(0, Duration::from_secs(600));

    if target == "/.edition" {
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n")?;
        loop {
            write!(stream, "data: {}\n\n", edition.version)?;
            edition = after(edition.version, Duration::from_secs(20));
        }
    }

    let found = find(&edition.files, &decode(target));
    if found.is_some_and(|(name, _)| *name != decode(target)) && !target.ends_with('/') {
        return Ok(write!(stream, "HTTP/1.1 302 Found\r\nLocation: {target}/\r\nContent-Length: 0\r\n\r\n")?);
    }
    let mut status = if found.is_some() { "200 OK" } else { "404 Not Found" }.to_owned();
    let (name, mut body) = match found.or_else(|| find(&edition.files, NOT_FOUND)) {
        Some((name, file)) => (name.as_str(), file.bytes()?.into_owned()),
        None => (NOT_FOUND, Vec::new()),
    };
    if name.ends_with(".html") {
        // Only while the tab is in view, as browsers allow few connections to one server.
        let reload = format!(
            r#"let news; (document.onvisibilitychange = () => document.hidden ? news?.close()
            : (news = new EventSource("/.edition")).onmessage = e => e.data == {} || location.reload())()"#,
            edition.version
        );
        let extra = html! {
            @if !edition.problems.is_empty() { pre.problem.banner { (edition.problems.join("\n")) } }
            script { (PreEscaped(reload)) }
        };
        body.extend(extra.0.bytes());
    }
    // Safari only plays video in parts.
    let range = request.iter().find_map(|line| line.to_lowercase().strip_prefix("range: bytes=").map(str::to_owned));
    if let Some((first, last)) = range.as_ref().and_then(|range| range.split_once('-')) {
        let end = body.len().saturating_sub(1);
        let (first, last) = (first.parse()?, last.parse().unwrap_or(end).min(end));
        status = format!("206 Partial Content\r\nContent-Range: bytes {first}-{last}/{}", body.len());
        body = body.get(first..=last).context("not a range")?.to_vec();
    }
    let keep = if name.starts_with("/assets/") { "max-age=31536000, immutable" } else { "no-store" };
    let (kind, length) = (media_type(name), body.len());
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {length}\r\n")?;
    write!(stream, "Cache-Control: {keep}\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n")?;
    Ok(stream.write_all(&body)?)
}

pub fn serve(tools: &Tools, address: &str) -> Result<()> {
    let listener = TcpListener::bind(address).with_context(|| format!("cannot listen on {address}"))?;
    eprintln!("http://{}/", listener.local_addr()?);
    let press = Press::default();
    thread::scope(|scope| {
        scope.spawn(|| watch(tools, &press));
        for stream in listener.incoming().flatten() {
            scope.spawn(|| respond(stream, &press).ok());
        }
    });
    Ok(())
}
