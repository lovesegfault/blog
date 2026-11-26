//! Other programs: the ones readers run, and the one that lays out diagrams.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

use anyhow::{Context, Result, ensure};
use wasmi::{Config, Engine, Linker, Module, Store};

use crate::output::{Cache, read_text};
use crate::site::Tools;

/// The folder with the figure and the explainers, compiled for browsers.
pub fn compile(cache: &Cache) -> Result<PathBuf> {
    if let Some(folder) = env::var_os("FORNO_WASM") {
        return Ok(folder.into());
    }
    let target = cache.0.join("wasm");
    let cargo = Command::new("cargo").args(["wasm", "--quiet", "--target-dir"]).arg(&target).output()?;
    ensure!(cargo.status.success(), "{}", String::from_utf8_lossy(&cargo.stderr));
    // Cargo leaves behind those that were renamed.
    let folder = target.join("wasm32-unknown-unknown/wasm");
    for program in fs::read_dir(&folder)?.flatten().map(|entry| PathBuf::from(entry.file_name())) {
        let written = |(root, kind)| Path::new(root).join(program.with_extension(kind)).exists();
        let sources = [("crates", ""), ("crates/explainers/src/bin", ""), ("crates/explainers/src/bin", "rs")];
        if program.extension().is_some_and(|it| it == "wasm") && !sources.into_iter().any(written) {
            fs::remove_file(folder.join(program))?;
        }
    }
    Ok(folder)
}

/// What an explainer shows before anyone touches it.
pub fn first_view(program: &[u8], options: &str) -> Result<String> {
    let engine = Engine::new(Config::default().consume_fuel(true));
    let mut store = Store::new(&engine, ());
    store.set_fuel(1_000_000_000)?;
    let instance = Linker::new(&engine).instantiate_and_start(&mut store, &Module::new(&engine, program)?)?;
    let memory = instance.get_memory(&store, "memory").context("it has no memory")?;
    let room = instance.get_typed_func::<u32, u32>(&store, "reserve")?.call(&mut store, options.len() as u32)?;
    memory.write(&mut store, room as usize, options.as_bytes())?;
    instance.get_typed_func::<(), ()>(&store, "start")?.call(&mut store, ())?;
    let mut view = vec![0; instance.get_typed_func::<(), u32>(&store, "view")?.call(&mut store, ())? as usize];
    let text = instance.get_typed_func::<(), u32>(&store, "text")?.call(&mut store, ())?;
    memory.read(&store, text as usize, &mut view)?;
    Ok(String::from_utf8(view)?)
}

/// Where the inside of a graph starts.
fn opening(graph: &str) -> Option<usize> {
    let at = graph.find(['{', '/', '#', '"'])? + 1;
    let end = match (&graph[at - 1..at], graph[at..].starts_with('*')) {
        ("{", _) => return Some(at),
        ("\"", _) => "\"",
        ("/", true) => "*/",
        _ => "\n",
    };
    let past = at + graph[at..].find(end)? + end.len();
    Some(past + opening(&graph[past..])?)
}

/// Lays out a Graphviz graph, in the colour of the text around it.
pub fn diagram(tools: &Tools, graph: &str) -> Result<String> {
    let asked = format!("{graph} {:?}", tools.vendor);
    let folder = tools.cache.entry(&[include_bytes!("programs.rs"), asked.as_bytes()], |folder| {
        // What is not a flag, the graph can change.
        let usual = "bgcolor=transparent fontsize=11 nodesep=0.4 ranksep=0.45 edge[fontsize=11 arrowsize=0.7] \
                     node[fontsize=11 shape=box style=rounded height=0.4] ";
        let (head, rest) = graph.split_at(opening(graph).context("a graph is written between braces")?);
        // Courier is as wide as the font the stylesheet puts in its place.
        let mut dot = Command::new(tools.vendor.join("dot"))
            .args(["-Tsvg", "-Gfontname=Courier", "-Nfontname=Courier", "-Efontname=Courier"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        dot.stdin.take().unwrap().write_all([head, usual, rest].concat().as_bytes())?;
        let drawn = dot.wait_with_output()?;
        let said = String::from_utf8_lossy(&drawn.stderr);
        ensure!(drawn.status.success() && !said.contains("Warning: "), "{}", said.trim());
        let lines = str::from_utf8(&drawn.stdout)?.split_inclusive('\n').skip_while(|line| !line.starts_with("<svg"));
        let svg: String = lines.filter(|line| !line.starts_with("<title>") && !line.starts_with("<!--")).collect();
        let svg = svg.replace("<svg", "<svg fill=\"black\"").replace("\"black\"", "\"currentColor\"");
        Ok(fs::write(folder.join("svg"), svg.replace("<g id=", "<g data-id="))?)
    })?;
    read_text(&folder.join("svg"))
}
