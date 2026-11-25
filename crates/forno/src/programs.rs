//! Other programs: the ones readers run, and the one that lays out diagrams.

use std::io::Write;
use std::path::PathBuf;
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
    Ok(target.join("wasm32-unknown-unknown/wasm"))
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

/// Lays out a Graphviz graph, in the colour of the text around it. Ids in the drawing start with `id`.
pub fn diagram(tools: &Tools, graph: &str, id: &str) -> Result<String> {
    let asked = format!("{graph} {id} {:?}", tools.vendor);
    let folder = tools.cache.entry(&[include_bytes!("programs.rs"), asked.as_bytes()], |folder| {
        // Courier is as wide as the font the stylesheet puts in its place.
        let mut dot = Command::new(tools.vendor.join("dot"))
            .args(["-Tsvg", "-Gbgcolor=transparent", "-Gnodesep=0.4", "-Granksep=0.45", &format!("-Gid={id}")])
            .args(["-Nshape=box", "-Nstyle=rounded", "-Nheight=0.4", "-Earrowsize=0.7"])
            .args(["G", "N", "E"].map(|of| format!("-{of}fontname=Courier")))
            .args(["G", "N", "E"].map(|of| format!("-{of}fontsize=11")))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        dot.stdin.take().unwrap().write_all(graph.as_bytes())?;
        let drawn = dot.wait_with_output()?;
        ensure!(drawn.status.success(), "{}", String::from_utf8_lossy(&drawn.stderr).trim());
        let lines = str::from_utf8(&drawn.stdout)?.split_inclusive('\n').skip_while(|line| !line.starts_with("<svg"));
        let svg: String = lines.filter(|line| !line.starts_with("<title>") && !line.starts_with("<!--")).collect();
        Ok(fs::write(
            folder.join("svg"),
            svg.replace("\"black\"", "\"currentColor\"").replace("<g id=", "<g data-id="),
        )?)
    })?;
    read_text(&folder.join("svg"))
}
