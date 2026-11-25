//! The browser's side of the figure. `assets/site.js` strokes what this projects.

use std::cell::RefCell;

use collision::{Camera, Event, START};

#[derive(Default)]
struct State {
    source: Vec<u8>,
    event: Event,
    frame: Vec<f32>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

/// Room for a source of `length` bytes.
#[unsafe(no_mangle)]
extern "C" fn reserve(length: usize) -> *mut u8 {
    STATE.with_borrow_mut(|state| {
        state.source = vec![0; length];
        state.source.as_mut_ptr()
    })
}

#[unsafe(no_mangle)]
extern "C" fn collide() {
    STATE.with_borrow_mut(|state| {
        let source = str::from_utf8(&state.source).ok().and_then(|text| text.parse().ok());
        state.event = source.as_ref().map(Event::new).unwrap_or_default();
    });
}

/// The number of tracks and of hits, then `jet class width opacity count x y x y …` for each track and `jet size x y`
/// for each hit.
#[unsafe(no_mangle)]
extern "C" fn frame(turned: f32, grown: f32) -> *const f32 {
    STATE.with_borrow_mut(|State { event, frame, .. }| {
        let camera = Camera::new(START + turned);
        frame.clear();
        frame.extend([event.tracks.len() as f32, event.hits.len() as f32]);
        for track in &event.tracks {
            let points = track.seen(camera, grown);
            frame.extend([track.jet as f32, track.class.into(), track.width, track.opacity, points.len() as f32]);
            frame.extend(points.flatten());
        }
        for hit in &event.hits {
            frame.extend([hit.jet as f32, hit.size.into()]);
            frame.extend(hit.seen(camera));
        }
        frame.as_ptr()
    })
}

fn main() {}
