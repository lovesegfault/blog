//! Writing, drawn the way a detector draws a collision.
//!
//! Every piece of text is a charged particle in a magnetic field: the longer it is, the straighter it flies. The event
//! is turned so that the momentum nobody saw, which would balance the rest, lies along `+x`. The beam runs along `z`.

use std::f32::consts::{PI, TAU};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Text,
    Code,
    Figure,
}

/// A paragraph, or a sentence.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Piece {
    pub kind: Kind,
    pub words: u32,
    pub notes: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Post {
    pub name: String,
    pub pieces: Vec<Piece>,
}

/// What collides. It is written out as text to reach the browser.
#[derive(Clone, PartialEq, Debug)]
pub enum Source {
    /// One jet for each post, oldest first.
    Site(Vec<Post>),
    /// One post, spread all around.
    Post(Post),
}

pub struct Track {
    pub jet: usize,
    /// From 0 for the straightest to 2 for the ones that curl up.
    pub class: u8,
    pub width: f32,
    pub opacity: f32,
    points: Vec<[f32; 3]>,
}

/// Where something struck the calorimeter.
pub struct Hit {
    pub jet: usize,
    /// From 0 to 2.
    pub size: u8,
    at: [f32; 3],
}

#[derive(Default)]
pub struct Event {
    pub tracks: Vec<Track>,
    pub hits: Vec<Hit>,
}

/// How far the event has been turned about `x`.
#[derive(Clone, Copy)]
pub struct Camera {
    sin: f32,
    cos: f32,
}

/// The angle every page starts from.
pub const START: f32 = 1.02;

const RADIUS: f32 = 1.0;
const HALF_LENGTH: f32 = 2.5;
/// Nothing is sent within this angle of the arrow.
const GAP: f32 = 1.2;

impl Post {
    /// Keeps about `most` pieces, evenly spread, and every one that has a note.
    pub fn thinned(&self, most: usize) -> Self {
        let step = (self.pieces.len() as f32 / most as f32).max(1.0);
        let starts = |k: usize| (k as f32 / step).floor() != ((k as f32 - 1.0) / step).floor();
        let kept = self.pieces.iter().enumerate().filter(|&(k, piece)| piece.notes > 0 || starts(k));
        Self { name: self.name.clone(), pieces: kept.map(|(_, piece)| *piece).collect() }
    }
}

/// `site|home 120' 45 c30 f8|about 12`
impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (label, posts) = match self {
            Self::Site(posts) => ("site", posts.as_slice()),
            Self::Post(post) => ("post", std::slice::from_ref(post)),
        };
        f.write_str(label)?;
        for post in posts {
            write!(f, "|{}", post.name)?;
            for piece in &post.pieces {
                let kind = match piece.kind {
                    Kind::Text => "",
                    Kind::Code => "c",
                    Kind::Figure => "f",
                };
                write!(f, " {kind}{}{}", piece.words, "'".repeat(piece.notes as usize))?;
            }
        }
        Ok(())
    }
}

impl FromStr for Source {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        let piece = |text: &str| {
            let plain = text.trim_end_matches('\'');
            let (kind, words) = match plain.split_at_checked(1) {
                Some(("c", words)) => (Kind::Code, words),
                Some(("f", words)) => (Kind::Figure, words),
                _ => (Kind::Text, plain),
            };
            Ok(Piece { kind, words: words.parse().map_err(drop)?, notes: (text.len() - plain.len()) as u32 })
        };
        let post = |text: &str| {
            let mut parts = text.split(' ');
            let name = parts.next().ok_or(())?.to_owned();
            Ok(Post { name, pieces: parts.map(piece).collect::<Result<_, ()>>()? })
        };
        let mut parts = text.split('|');
        match parts.next() {
            Some("site") => parts.map(post).collect::<Result<_, _>>().map(Self::Site),
            Some("post") => post(parts.next().ok_or(())?).map(Self::Post),
            _ => Err(()),
        }
    }
}

struct Dice(u32);

impl Dice {
    fn new(name: &str, salt: &str) -> Self {
        let start = 1_779_033_703 ^ (name.len() + salt.len()) as u32;
        let mix = |h: u32, byte: u8| (h ^ u32::from(byte)).wrapping_mul(3_432_918_353).rotate_left(13);
        Self(name.bytes().chain(salt.bytes()).fold(start, mix))
    }

    /// Uniform in `0..1`.
    fn roll(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x6D2B_79F5);
        let t = (self.0 ^ self.0 >> 15).wrapping_mul(1 | self.0);
        let t = t.wrapping_add((t ^ t >> 7).wrapping_mul(61 | t)) ^ t;
        ((t ^ t >> 14) >> 8) as f32 / (1 << 24) as f32
    }

    fn normal(&mut self) -> f32 {
        (-2.0 * (1.0 - self.roll()).ln()).sqrt() * (TAU * self.roll()).cos()
    }

    fn sign(&mut self) -> f32 {
        if self.roll() < 0.5 { 1.0 } else { -1.0 }
    }
}

struct Particle {
    piece: Piece,
    /// The radius it bends with.
    bend: f32,
    charge: f32,
    /// Its heading around the beam, and along it.
    phi: f32,
    eta: f32,
    hits: u32,
}

fn sector(i: usize, offset: f32) -> f32 {
    GAP + (i as f32 * 0.618_034 + offset).fract() * (TAU - 2.0 * GAP)
}

/// From -1 for the first of `count` to 1 for the last.
fn across(i: usize, count: usize) -> f32 {
    if count > 1 { 2.0 * i as f32 / (count - 1) as f32 - 1.0 } else { 0.0 }
}

fn jet(post: &Post, i: usize, count: usize) -> Vec<Particle> {
    let mut dice = Dice::new(&post.name, "");
    let along = across(i, count);
    let (phi, eta) = (sector(i, 0.37), 1.75 * along.abs().powf(0.8).copysign(along));
    let particle = |&piece: &Piece| Particle {
        piece: Piece { notes: piece.notes.min(1), ..piece },
        bend: 0.14 + 2.5 * (1.0 - (piece.words as f32 / -90.0).exp()),
        charge: dice.sign(),
        phi: phi + dice.normal() * 0.3,
        eta: eta + dice.normal() * 0.24,
        hits: 2 + (piece.words + 11) / 22,
    };
    post.pieces.iter().map(particle).collect()
}

fn spray(post: &Post) -> Vec<Particle> {
    let mut dice = Dice::new(&post.name, "");
    let offset = dice.roll();
    let particle = |(i, &piece): (usize, &Piece)| Particle {
        piece,
        bend: 0.1 + 2.6 * (1.0 - (piece.words as f32 / -38.0).exp()),
        charge: dice.sign(),
        phi: sector(i, offset),
        eta: 1.5 * across(i, post.pieces.len()) + dice.normal() * 0.15,
        hits: 1 + (piece.words + 3) / 7,
    };
    post.pieces.iter().enumerate().map(particle).collect()
}

fn escaped([x, y, z]: [f32; 3]) -> bool {
    x.hypot(y) >= RADIUS || z.abs() >= HALF_LENGTH
}

/// Follows a helix until it leaves the tracker, or has turned through `limit`. Slow particles spiral in.
fn fly(mut at: [f32; 3], mut phi: f32, eta: f32, bend: f32, charge: f32, limit: f32) -> Vec<[f32; 3]> {
    let (climb, loss) = (eta.sinh(), if bend < 0.5 { 0.15 } else { 0.0 });
    let (mut points, mut turned) = (vec![at], 0.0_f32);
    while !escaped(at) && turned <= limit {
        let radius = bend * (-loss * turned).exp();
        let step = (radius * 0.2).min(0.05);
        phi += charge * step / radius / 2.0;
        at = [at[0] + step * phi.cos(), at[1] + step * phi.sin(), at[2] + step * climb];
        phi += charge * step / radius / 2.0;
        turned += step / radius;
        points.push(at);
    }
    points
}

impl Event {
    pub fn new(source: &Source) -> Self {
        let jets: Vec<(&str, Vec<Particle>)> = match source {
            Source::Site(posts) => {
                posts.iter().enumerate().map(|(i, post)| (post.name.as_str(), jet(post, i, posts.len()))).collect()
            }
            Source::Post(post) => vec![(&post.name, spray(post))],
        };
        let seen = |along: fn(f32) -> f32| -> f32 {
            jets.iter().flat_map(|(_, particles)| particles).map(|p| p.piece.words as f32 * along(p.phi)).sum()
        };
        let turn = PI - seen(f32::sin).atan2(seen(f32::cos));

        let mut event = Self::default();
        for (jet, (name, particles)) in jets.iter().enumerate() {
            let mut dice = Dice::new(name, "hits");
            for particle in particles {
                event.detect(jet, particle, particle.phi + turn, &mut dice);
            }
        }
        event
    }

    fn detect(&mut self, jet: usize, particle: &Particle, phi: f32, dice: &mut Dice) {
        let Particle { piece, bend, eta, .. } = *particle;
        let ((sin, cos), climb) = (phi.sin_cos(), eta.sinh());
        match piece.kind {
            // Neutral: nothing in the tracker, and a cluster past it.
            Kind::Figure => {
                for i in 0..9 {
                    let (out, angle) = (1.06 + dice.roll() * 0.3, phi + dice.normal() * 0.05);
                    let at = [out * angle.cos(), out * angle.sin(), out * climb + dice.normal() * 0.06];
                    self.hits.push(Hit { jet, size: if i < 3 { 2 } else { 1 }, at });
                }
            }
            // A muon: straight through everything.
            Kind::Code => {
                let far = 1.75 / (1.0 + climb * climb * 0.35).sqrt();
                let points = vec![[0.0; 3], [far * cos, far * sin, far * climb]];
                self.tracks.push(Track { jet, class: 0, width: 0.7, opacity: 0.83, points });
            }
            Kind::Text => {
                // Tight tracks are made to bend away from the arrow.
                let charge = if bend < 0.9 && cos > -0.35 { 1.0_f32.copysign(sin) } else { particle.charge };
                let curls = bend < 0.5;
                let limit = PI * (2.2 + dice.roll() * 1.5);
                let points = fly([0.0; 3], phi, if curls { eta * 0.24 } else { eta }, bend, charge, limit);
                let end @ [x, y, z] = points[points.len() - 1];
                self.tracks.push(if curls {
                    Track { jet, class: 2, width: 0.85, opacity: 0.9, points }
                } else {
                    let width = 0.75 + (bend * 0.14).min(0.5);
                    Track { jet, class: u8::from(bend < 1.25), width, opacity: 0.98, points }
                });
                for _ in 0..if escaped(end) { particle.hits } else { 0 } {
                    let out = 1.04 + dice.normal().abs() * 0.17;
                    let (sin, cos) = (dice.normal() * 0.075).sin_cos();
                    let at = [out * (x * cos - y * sin), out * (x * sin + y * cos), out * z + dice.normal() * 0.1];
                    let size = if dice.roll() < 0.16 { 2 } else { u8::from(dice.roll() < 0.55) };
                    self.hits.push(Hit { jet, size, at });
                }
                // A note is a decay away from the vertex.
                for _ in 0..piece.notes {
                    let (out, angle) = (0.38 + dice.roll() * 0.2, phi + dice.normal() * 0.3);
                    let from = [out * angle.cos(), out * angle.sin(), out * climb];
                    for charge in [1.0, -1.0] {
                        let points = fly(from, angle, eta, 0.55 + dice.roll() * 0.3, charge, 1.15);
                        self.tracks.push(Track { jet, class: 2, width: 0.9, opacity: 1.0, points });
                    }
                }
            }
        }
    }
}

impl Camera {
    pub fn new(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self { sin, cos }
    }

    /// In tracker radii, along the arrow and across it.
    fn project(self, [x, y, z]: [f32; 3]) -> [f32; 2] {
        let near = 1.0 / (1.0 - 0.085 * (y * self.sin + z * self.cos));
        [near * x, near * (z * self.sin - y * self.cos)]
    }
}

impl Track {
    /// The track up to `grown`, a fraction of its length.
    pub fn seen(&self, camera: Camera, grown: f32) -> impl ExactSizeIterator<Item = [f32; 2]> {
        let end = grown.clamp(0.0, 1.0) * (self.points.len() - 1) as f32;
        (0..end.ceil() as usize + 1).map(move |i| {
            let (at, before) = (self.points[i], self.points[i.saturating_sub(1)]);
            let short = (i as f32 - end).max(0.0);
            camera.project([0, 1, 2].map(|k| at[k] + (before[k] - at[k]) * short))
        })
    }
}

impl Hit {
    pub fn seen(&self, camera: Camera) -> [f32; 2] {
        camera.project(self.at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources_survive_the_trip_to_the_browser() {
        for text in ["site", "site|home 120' 45 c30 f8|about 12''", "post|empty"] {
            assert_eq!(text.parse::<Source>().unwrap().to_string(), text);
        }
        assert!("site|home x".parse::<Source>().is_err());
    }

    #[test]
    fn what_was_seen_points_away_from_the_arrow() {
        let event = Event::new(&"site|a c20 c20|b c20|c c20".parse().unwrap());
        let total = |axis: usize| -> f32 {
            event.tracks.iter().map(|track| track.points[1]).map(|end| end[axis] / end[0].hypot(end[1])).sum()
        };
        assert!(total(0) < 0.0 && total(1).abs() < 1e-4);
    }

    #[test]
    fn tracks_grow_from_the_vertex() {
        let event = Event::new(&"post|a c20".parse().unwrap());
        let [whole, half] = [1.0, 0.5].map(|grown| event.tracks[0].seen(Camera::new(0.0), grown).last().unwrap());
        assert!((whole[0] / half[0] - 2.0).abs() < 0.3);
        assert_eq!(event.tracks[0].seen(Camera::new(0.0), 0.0).len(), 1);
    }
}
