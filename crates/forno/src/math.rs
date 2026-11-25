//! Runs KaTeX here, so that readers do not have to.

use std::cell::OnceCell;
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use anyhow::{Result, bail};
use rquickjs::{Context, Function, Runtime};

pub type Macros = BTreeMap<String, String>;

pub struct Math {
    katex: String,
    done: Mutex<HashMap<(String, bool, Macros), String>>,
}

const RENDER: &str = "
    globalThis.render = (tex, displayMode, macros) => {
        try { return katex.renderToString(tex, { displayMode, macros, strict: false }); }
        catch (error) { return error.message; }
    }";

thread_local! {
    static ENGINE: OnceCell<Context> = const { OnceCell::new() };
}

impl Math {
    pub fn new(katex: String) -> Self {
        Self { katex, done: Mutex::default() }
    }

    fn start(&self) -> rquickjs::Result<Context> {
        let context = Context::full(&Runtime::new()?)?;
        context.with(|js| js.eval::<(), _>(self.katex.as_str()).and_then(|()| js.eval::<(), _>(RENDER)))?;
        Ok(context)
    }

    pub fn render(&self, tex: &str, display: bool, macros: &Macros) -> Result<String> {
        let key = (tex.to_owned(), display, macros.clone());
        if let Some(html) = self.done.lock().unwrap().get(&key) {
            return Ok(html.clone());
        }
        let html: String = ENGINE.with(|engine| {
            let context = engine.get_or_init(|| self.start().expect("KaTeX loads"));
            context.with(|js| js.globals().get::<_, Function>("render")?.call((tex, display, macros)))
        })?;
        if !html.starts_with('<') {
            bail!(html);
        }
        self.done.lock().unwrap().insert(key, html.clone());
        Ok(html)
    }
}

/// The MathML inside what KaTeX made, for where its stylesheet does not reach.
pub fn portable(html: &str) -> String {
    let math = html.find("<math").zip(html.find("<annotation")).map_or("", |(start, end)| &html[start..end]);
    format!("{}</math>", math.replace("<semantics>", ""))
}
