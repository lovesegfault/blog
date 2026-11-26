//! Runs MathJax here, so that readers do not have to.

use std::cell::OnceCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Result, bail};
use rquickjs::{Context, Function, Promise, Runtime};
use typst::foundations::Bytes;
use typst::layout::Em;
use typst::text::Font;

use crate::output::{read, read_text};

pub struct Math {
    vendor: PathBuf,
    done: Mutex<HashMap<String, Arc<Formula>>>,
}

pub struct Formula {
    pub html: String,
    /// For screen readers, and for where the stylesheet does not reach.
    pub mathml: String,
    /// What it adds to the stylesheet.
    pub rules: Vec<String>,
}

thread_local! {
    static ENGINE: OnceCell<Context> = const { OnceCell::new() };
}

/// The width of `text`, in ems. MathJax brings the spaces.
fn measure(font: &Path, text: &str) -> Option<f64> {
    let font = Font::new(Bytes::new(read(font).ok()?), 0)?.instantiate(<_>::default(), <_>::default(), &<_>::default());
    let advance = |letter| font.x_advance(font.ttf().glyph_index(letter)?.0);
    Some(text.chars().map(|letter| if letter == ' ' { 0.25 } else { advance(letter).map_or(1.0, Em::get) }).sum())
}

impl Math {
    pub fn new(vendor: &Path) -> Self {
        Self { vendor: vendor.into(), done: Mutex::default() }
    }

    fn start(&self) -> rquickjs::Result<Context> {
        let runtime = Runtime::new()?;
        runtime.set_max_stack_size(8 << 20);
        let context = Context::full(&runtime)?;
        let (scripts, fonts) = (self.vendor.clone(), self.vendor.clone());
        let missing = |error: anyhow::Error| format!("throw Error({:?})", error.to_string());
        let read = move |path: String| read_text(&scripts.join(path)).unwrap_or_else(missing);
        let measure = move |font: String, text: String| measure(&fonts.join(font), &text);
        context.with(|js| {
            js.globals().set("read", Function::new(js.clone(), read))?;
            js.globals().set("measure", Function::new(js.clone(), measure))?;
            js.eval::<(), _>(include_str!("math.js"))
        })?;
        Ok(context)
    }

    fn call(&self, function: &str, argument: &str) -> rquickjs::Result<Vec<String>> {
        ENGINE.with(|engine| {
            let context = engine.get_or_init(|| self.start().expect("MathJax loads"));
            context.with(|js| js.globals().get::<_, Function>(function)?.call::<_, Promise>((argument,))?.finish())
        })
    }

    /// The styles that every formula shares.
    pub fn sheet(&self) -> Result<String> {
        Ok(self.call("sheet", "")?.concat())
    }

    pub fn render(&self, mathml: &str) -> Result<Arc<Formula>> {
        if let Some(formula) = self.done.lock().unwrap().get(mathml) {
            return Ok(formula.clone());
        }
        let formula = match &*self.call("typeset", mathml)? {
            [html, mathml, rules @ ..] => Formula { html: html.into(), mathml: mathml.into(), rules: rules.into() },
            error => bail!(error.concat()),
        };
        Ok(self.done.lock().unwrap().entry(mathml.into()).or_insert(formula.into()).clone())
    }
}
