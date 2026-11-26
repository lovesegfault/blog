//! Figures that readers can play with.
//!
//! Each file in `src/bin` is one, and a post shows it with `#explainer("name")[what it is]`. Its first view is written
//! into the page, so there is something to see before the program loads, or if it never does.

use std::cell::RefCell;

pub use maud::{Markup, html};

pub trait Explainer {
    /// `options` is what the post gave to `explainer` under that name.
    fn new(options: &str) -> Self
    where
        Self: Sized;

    /// The reader used something in the view:
    ///
    /// - an `input` or a `select` sends its `name` and value, which is `true` or `false` for a checkbox;
    /// - a `button` sends its `name` and `value`;
    /// - an SVG element with `data-drag="name"` sends `x y` in the units of its drawing, when it is dragged or nudged
    ///   with the arrow keys.
    fn input(&mut self, name: &str, value: &str);

    /// Called for each frame after the start or an input, for as long as it returns `true`.
    fn tick(&mut self, _seconds: f32) -> bool {
        false
    }

    fn view(&self) -> Markup;
}

/// The `x y` that dragging sends.
pub fn point(value: &str) -> Option<[f32; 2]> {
    let (x, y) = value.split_once(' ')?;
    Some([x.parse().ok()?, y.parse().ok()?])
}

/// Makes a type the explainer of its file.
#[macro_export]
macro_rules! explainer {
    ($explainer:ty) => {
        #[unsafe(no_mangle)]
        extern "C" fn start() {
            $crate::start::<$explainer>();
        }

        fn main() {}
    };
}

thread_local! {
    static TEXT: RefCell<Vec<u8>> = RefCell::default();
    static RUNNING: RefCell<Option<Box<dyn Explainer>>> = RefCell::default();
}

fn received() -> String {
    String::from_utf8(TEXT.take()).expect("the page sends UTF-8")
}

fn running<T>(act: impl FnOnce(&mut dyn Explainer) -> T) -> T {
    RUNNING.with_borrow_mut(|explainer| act(explainer.as_deref_mut().expect("started")))
}

#[doc(hidden)]
pub fn start<E: Explainer + 'static>() {
    RUNNING.set(Some(Box::new(E::new(&received()))));
}

/// Room for `length` bytes of text, for the call that follows.
#[unsafe(no_mangle)]
extern "C" fn reserve(length: usize) -> *mut u8 {
    TEXT.set(vec![0; length]);
    text()
}

#[unsafe(no_mangle)]
extern "C" fn text() -> *mut u8 {
    TEXT.with_borrow_mut(Vec::as_mut_ptr)
}

/// Takes `name=value`.
#[unsafe(no_mangle)]
extern "C" fn input() {
    if let Some((name, value)) = received().split_once('=') {
        running(|explainer| explainer.input(name, value));
    }
}

#[unsafe(no_mangle)]
extern "C" fn tick(seconds: f32) -> bool {
    running(|explainer| explainer.tick(seconds))
}

/// Leaves the view in `text`, and returns its length.
#[unsafe(no_mangle)]
extern "C" fn view() -> usize {
    TEXT.set(running(|explainer| explainer.view()).into_string().into_bytes());
    TEXT.with_borrow(Vec::len)
}
