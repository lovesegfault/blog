//! Fires drawings into ceramic: tiles, glazes, joints and light.
//!
//! Lengths are in design pixels, which `density` turns into real ones.

mod field;
mod glaze;
mod noise;
mod pieces;
mod valley;

use std::f32::consts::{FRAC_PI_2, TAU};

use rayon::prelude::*;

use crate::glaze::{Fired, GLAZES, Rgb};
use crate::noise::{Noise, signed, unit};
pub use crate::pieces::{Firing, HOME, LABELS, NARROW, POST, Scene, firings};
pub use crate::valley::Valley;

/// Everything that decides what comes out of the kiln, besides the drawing.
pub const RECIPE: [&str; 6] = [
    include_str!("field.rs"),
    include_str!("glaze.rs"),
    include_str!("lib.rs"),
    include_str!("noise.rs"),
    include_str!("pieces.rs"),
    include_str!("valley.rs"),
];

/// What is painted at one point of a wall, before firing.
#[derive(Clone, Copy)]
struct Paint {
    /// Two glazes and how much of the second, for where they meet.
    glazes: [usize; 2],
    blend: f32,
    /// Extra thickness the drawing asks for.
    body: f32,
    /// The distance to the nearest brushed line.
    edge: f32,
    /// The distance into the mural from where it is cut. Negative on the bare wall.
    inside: f32,
    /// The distance into the lake. Negative on land.
    offshore: f32,
    /// From 0 to 1, how much to hold back whatever would get in the way of text.
    calm: f32,
}

impl Paint {
    fn plain(glaze: &str) -> Self {
        let glazes = [glaze::named(glaze); 2];
        Self { glazes, blend: 0.0, body: 0.0, edge: f32::MAX, inside: f32::MAX, offshore: -1.0, calm: 0.0 }
    }
}

#[derive(Clone, Copy)]
enum Light {
    Day,
    /// Dim all over, and lit by a lamp if there is one.
    Night {
        ambient: Rgb,
        lamp: Option<Lamp>,
    },
}

#[derive(Clone, Copy)]
struct Lamp {
    at: [f32; 2],
    /// Where its light has fallen by half.
    reach: f32,
    power: f32,
}

#[derive(Clone, Copy)]
struct Wall {
    size: [f32; 2],
    density: f32,
    tile: [f32; 2],
    /// A corner of some tile.
    origin: [f32; 2],
    joint: f32,
    /// Grout fills the joints, or else the page shows through them.
    grout: Option<Rgb>,
    /// After how many tiles the pattern repeats.
    repeat: [i32; 2],
    /// For one large tile whose middle has to repeat.
    period: Option<f32>,
    /// The tile that starts at `origin` has come away.
    missing: bool,
    seed: u32,
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    /// Straight alpha.
    pub rgba: Vec<u8>,
}

/// A point of fired surface, waiting for light.
#[derive(Clone, Copy, Default)]
struct Surface {
    colour: Rgb,
    height: f32,
    tilt: [f32; 2],
    gloss: f32,
    shade: f32,
    cover: f32,
    shadow: f32,
    /// How well something drawn on the water would take.
    wet: f32,
}

/// A fired wall.
struct Panel {
    width: usize,
    height: usize,
    density: f32,
    surfaces: Vec<Surface>,
}

fn step(from: f32, to: f32, x: f32) -> f32 {
    let t = ((x - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn normalize([x, y, z]: [f32; 3]) -> [f32; 3] {
    let length = (x * x + y * y + z * z).sqrt();
    [x / length, y / length, z / length]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

struct Kiln<'a> {
    wall: &'a Wall,
    fired: Vec<Fired>,
    clay: Rgb,
    iron: Rgb,
    manganese: Rgb,
}

impl Kiln<'_> {
    /// What a tile leaves behind: the ridges of the trowel, pressed flat where it held.
    fn mortar(&self, u: f32, v: f32, noise: Noise) -> Surface {
        let [tw, th] = self.wall.tile;
        let k = tw / 64.0;
        let sweep = (u - tw * 0.2).hypot(v - th * 2.4)
            + k * (2.0 * noise.smooth(1, u, v, 34.0 * k) + 0.5 * noise.smooth(5, u, v, 6.0 * k));
        let ridge = (sweep * TAU / (8.5 * k)).sin();
        let pressed = 0.1 + 0.9 * noise.cloud(2, u, v, 24.0 * k, 3);
        let grit = noise.cloud(3, u, v, 2.6, 2);
        let border = u.min(tw - u).min(v).min(th - v);
        let crust = step(2.6 * k, 0.8 * k, border - 2.2 * k * noise.smooth(6, u, v, 8.0 * k));
        let cast = step(0.0, 7.0 * k, u).min(step(0.0, 11.0 * k, v));
        Surface {
            colour: Rgb::hex(0xb0a28c) * (0.90 + 0.14 * noise.cloud(4, u, v, 18.0 * k, 3) + 0.10 * grit - 0.25 * crust),
            height: -1.2 + k * (0.62 * ridge.min(pressed) + 0.7 * crust) + 0.10 * grit,
            shade: (0.34 + 0.66 * cast) * (0.6 + 0.4 * step(0.0, 2.4, border)) * (0.80 + 0.20 * step(-1.0, 0.1, ridge)),
            cover: 1.0,
            ..Surface::default()
        }
    }

    fn surface(&self, x: f32, y: f32, paint: &(impl Fn(f32, f32, [i32; 2]) -> Paint + Sync)) -> Surface {
        let wall = self.wall;
        let soft = 0.6 / wall.density;
        let [tw, th] = wall.tile;
        let cell = [((x - wall.origin[0]) / tw).floor(), ((y - wall.origin[1]) / th).floor()];
        let (u, v) = (x - wall.origin[0] - cell[0] * tw, y - wall.origin[1] - cell[1] * th);
        let tile = [0, 1].map(|i| (cell[i] as i32).rem_euclid(wall.repeat[i]));
        let chance = |salt: u32| unit(tile[0], tile[1], wall.seed + salt);
        let swing = |salt: u32| signed(tile[0], tile[1], wall.seed + salt);
        let noise = Noise { seed: noise::hash(tile[0], tile[1], wall.seed), period: wall.period };

        if wall.missing && cell == [0.0; 2] {
            return self.mortar(u, v, noise);
        }

        // No two tiles are cut alike, and some have lost a corner.
        let slack = wall.joint * 0.22;
        let radius = 1.6 + chance(1) * 1.2;
        let reach = |at: f32, size: f32, salt: u32| {
            (at - size / 2.0 - swing(salt) * slack).abs() - (size - wall.joint) / 2.0 - swing(salt + 1) * slack + radius
        };
        let (qx, qy) = (reach(u, tw, 2), reach(v, th, 4));
        let from_joint = radius - qx.max(0.0).hypot(qy.max(0.0)) - qx.max(qy).min(0.0)
            + 0.32 * noise.smooth(1, u, v, 15.0)
            + 0.14 * noise.smooth(2, u, v, 4.0);
        let corner = 1.0 + 1.6 * step(-9.0, 0.0, qx.min(qy) - radius);

        // The cut edge follows the wall. The drawing follows the tile, which was set a little off.
        let Paint { inside: cut, calm, .. } = paint(x, y, tile);
        let bite = (noise.cloud(3, u, v, 9.0, 2) - 0.50 - 0.3 * calm).max(0.0) * 7.0 * corner;
        let wander = Noise { seed: wall.seed, period: None };
        let drift = |salt: u32| {
            swing(salt) * 1.1 + 1.0 * wander.smooth(salt, x, y, 26.0) + 0.45 * wander.smooth(salt + 1, x, y, 6.0)
        };
        let paint_here = paint(x + drift(10), y + drift(12), tile);
        let from_cut = cut + 0.4 * wander.smooth(14, x, y, 11.0);
        let rim = from_joint.min(from_cut) - bite * 0.3;
        let chipped = step(soft, -soft, rim - bite * 0.7) * step(0.0, 0.4, bite);

        // A slab keeps its character for the rim, where there is nothing to read.
        let lively = 1.0 - if wall.period.is_some() { step(5.0, 19.0, rim) } else { calm };

        let behind = paint(x - 1.2, y - 2.6, tile).inside;
        let mut out = Surface { shadow: 0.40 * step(-7.5, 1.5, behind).powi(2), ..Surface::default() };
        let tiled = step(-soft, soft, rim);
        match wall.grout {
            Some(_) => out.cover = step(-soft, soft, from_cut),
            None => {
                out.cover = tiled;
                out.shadow = 0.34 * step(-wall.joint * 0.5, 0.6, from_joint + 0.5 * step(0.4, 1.0, v / th)).powi(2);
            }
        }
        if out.cover <= 0.0 {
            return out;
        }
        out.wet = out.cover * step(1.0, 2.2, paint_here.offshore) * (0.28 + 0.72 * step(0.0, 0.8, rim));

        // How thickly the glaze lies.
        let cloud = noise.cloud(20, u, v, 52.0, 4);
        let fine = noise.cloud(30, u, v, 7.0, 2);
        let turn = if wall.period.is_some() { 0.0 } else { (chance(20) * 2.0).floor() * FRAC_PI_2 + swing(21) * 0.3 };
        let (along, across) = (u * turn.cos() + v * turn.sin(), v * turn.cos() - u * turn.sin());
        let across = across + 2.2 * noise.smooth(40, u, v, 46.0);
        let worked = 0.35 + 0.65 * chance(30);
        let brush = worked
            * (0.75 * noise.streak(41, along, across, 96.0, 8.5) + 0.34 * noise.streak(42, along, across, 44.0, 2.4));
        let sag = if wall.period.is_some() { 0.0 } else { (v / th - 0.5) * 0.30 };
        let bead = 0.30 * (-((rim - 4.4) / 2.6).powi(2)).exp() * (0.6 + sag * 2.4);
        let breaking = (0.80 + 0.35 * noise.smooth(43, u, v, 18.0)) * (-rim.max(0.0) / 0.95).exp();
        let edge = paint_here.edge;
        let pool = 0.42 * (-edge / 6.0).exp();
        let bloom = step(0.05, 0.75, noise.cloud(50, u, v, 34.0, 3) + 0.25 * fine);
        let batch = swing(22) * if wall.period.is_some() { 0.0 } else { 0.30 };

        let (mut thickness, mut ridges) = (0.0, 0.0);
        let (mut specks, mut crazing) = (0.0, 0.0);
        for (index, share) in [(paint_here.glazes[0], 1.0 - paint_here.blend), (paint_here.glazes[1], paint_here.blend)]
        {
            if share <= 0.0 {
                continue;
            }
            let (glaze, fired) = (&GLAZES[index], &self.fired[index]);
            let t = 1.0
                + paint_here.body
                + glaze.mottle * (1.0 - 0.6 * calm) * (0.62 * cloud + 0.26 * fine + batch + sag + bead * 1.6)
                + glaze.brush * 0.42 * brush
                + pool
                - breaking;
            let clouded = glaze.blooming * (1.0 - 0.7 * calm) * bloom * step(1.9, 1.0, t) * step(0.0, 0.7, t);
            out.colour = out.colour + fired.colour(t).mix(fired.bloom, clouded) * share;
            out.gloss += glaze.gloss * share;
            thickness += t * share;
            ridges += (glaze.brush * 0.42 * brush + glaze.mottle * 0.26 * fine) * share;
            specks += glaze.specks * share;
            crazing += glaze.crazing * share;
        }
        // Where it has run off altogether, the clay shows.
        out.colour = out.colour.mix(self.clay, step(0.22, -0.30, thickness) * 0.8);

        // Iron bleeds out of the clay.
        for (salt, spacing, odds, smallest, largest) in [(60, 6.5, 0.20, 0.28, 0.85), (61, 29.0, 0.16, 0.8, 1.9)] {
            let (near, _, id) = noise.cells(salt, u, v, spacing);
            if ((id >> 8) as f32 / (1 << 24) as f32) < odds * specks {
                let size = smallest + (largest - smallest) * ((id & 0xFF) as f32 / 255.0).powi(2);
                let near = near * (1.0 + 0.25 * fine);
                let stain = 0.72 * step(size + soft, size * 0.4, near) + 0.22 * (-(near / (size * 2.4)).powi(2)).exp();
                out.colour = out.colour.mix(self.iron, stain.min(0.85) * lively);
            }
        }

        // The glass cracks as it cools.
        let (near, next, _) = noise.cells(70, u + 2.0 * fine, v + 1.6 * cloud, 9.0 + chance(23) * 6.0);
        let crack =
            step(0.42, 0.06, (next - near) * 0.5) * step(-0.25, 0.45, noise.smooth(71, u, v, 60.0) + swing(24) * 0.5);
        out.colour = out.colour.mix(out.colour * self.iron * 3.0, crack * crazing * (0.10 + 0.32 * lively));

        // The drawing, in manganese. The brush swells, thins and sometimes runs dry.
        let mut groove = 0.0;
        if edge < 14.0 {
            let load = wander.smooth(80, x, y, 62.0) + 0.4 * wander.smooth(81, x, y, 17.0);
            let edge = edge + 0.28 * fine;
            let width = (1.0 + 0.85 * load).max(0.0) * (tw / 62.0).sqrt();
            let dry = 0.74 + 0.20 * step(-0.6, 0.5, wander.smooth(82, x, y, 9.0) + load);
            let ink = step(width + 0.7, width - 0.6, edge) * step(0.0, 0.5, width);
            out.colour = out.colour.mix(self.iron, 0.24 * (-edge / 3.4).exp()).mix(self.manganese, ink * dry);
            groove = ink * 0.10;
        }

        // Each tile sat somewhere else in the kiln.
        if wall.period.is_none() {
            let side = chance(25) * TAU;
            let toward =
                ((u / tw - 0.5) * side.cos() + (v / th - 0.5) * side.sin() + 0.35 + 0.2 * cloud).clamp(0.0, 1.0);
            let flash = toward * step(0.45, 1.0, chance(26)) * 0.50 * lively;
            out.colour = out.colour.mix(out.colour * Rgb([0.98, 0.66, 0.42]), flash) * (1.0 + 0.10 * swing(27));
            out.tilt = [swing(28) * 0.040, swing(29) * 0.040];
        }
        out.colour = out.colour.mix(self.clay * (1.05 + 0.2 * fine), chipped);
        out.gloss *= (1.0 - chipped) * (1.0 - 0.5 * crack * crazing) * lively;

        out.height = 1.5 * (1.0 - (-rim.max(0.0) / 1.5).exp())
            + 0.42 * (thickness - 0.72 * ridges).clamp(0.0, 2.2)
            + 0.75 * noise.smooth(90, u, v, 40.0)
            + 0.10 * noise.smooth(92, u, v, 11.0)
            + 0.022 * noise.smooth(91, u, v, 3.2)
            - 0.8 * groove
            - 0.5 * chipped;
        out.shade = 0.72 + 0.28 * step(-0.4, 1.8, rim);

        if let Some(grout) = wall.grout {
            let sand = 0.86 + 0.20 * noise.smooth(95, u, v, 1.6) + 0.10 * noise.smooth(96, u, v, 9.0);
            out.colour = (out.colour * 0.3).mix(grout * sand, lively * lively).mix(out.colour, tiled);
            out.gloss *= tiled;
            out.height = out.height * tiled - 0.6 * (1.0 - tiled);
        }
        out
    }
}

fn shine(surface: &Surface, slope: [f32; 2], x: f32, y: f32, light: Light) -> Rgb {
    let normal = normalize([surface.tilt[0] - slope[0], surface.tilt[1] - slope[1], 1.0]);
    let glint = |toward: [f32; 3], sharpness: i32| {
        dot(normal, normalize([toward[0], toward[1], toward[2] + 1.0])).max(0.0).powi(sharpness) * surface.gloss
    };
    // A window, up and to the left, that only some tiles lean far enough to catch.
    let window = step(0.020, 0.100, -0.55 * normal[0] - 0.83 * normal[1]) * surface.gloss;
    let sun = normalize([-0.45, -0.62, 0.80]);
    let facing = dot(normal, sun).max(0.0) / sun[2];
    match light {
        Light::Day => {
            surface.colour * ((0.42 + 0.58 * facing) * surface.shade)
                + Rgb([0.92, 0.96, 1.0]) * (0.040 * window + 0.30 * glint(sun, 70))
        }
        Light::Night { ambient, lamp } => {
            // Pale glazes would glare in the dark.
            let colour = surface.colour * (1.0 / (1.0 + 0.8 * dot(surface.colour.0, [0.21, 0.72, 0.07])));
            let mut lit = colour * ambient * ((0.6 + 0.4 * facing) * surface.shade) + ambient * (0.030 * window);
            if let Some(Lamp { at, reach, power }) = lamp {
                let (dx, dy) = ((at[0] - x) / reach, (at[1] - y) / reach);
                let toward = normalize([dx, dy, 0.6]);
                let falloff = 1.0 / (1.0 + dx.hypot(dy).powi(4));
                let facing = dot(normal, toward).max(0.0) / toward[2];
                // It reddens as it fades, so that it meets the dark in violet and not in grey.
                let warm = Rgb([1.0, 0.30, 0.17]).mix(Rgb([1.0, 0.76, 0.46]), step(0.02, 0.55, falloff));
                let mirrored = glint(toward, 300);
                lit = lit + colour * warm * (power * falloff * (0.30 + 0.70 * facing).min(2.4) * surface.shade);
                lit = lit.mix(
                    Rgb([0.95, 0.36, 0.08]),
                    falloff.sqrt() * (0.40 * step(0.60, 0.95, mirrored) + 0.04 * mirrored),
                );
            }
            lit
        }
    }
}

/// A limewashed wall that repeats every `side` pixels. `grain` is how far it strays from `colour`.
fn limewash(side: u32, colour: Rgb, grain: f32) -> Image {
    let noise = Noise { seed: 40, period: Some(side as f32) };
    let mut rgba = vec![255; (side * side * 4) as usize];
    rgba.par_chunks_mut(4).enumerate().for_each(|(i, pixel)| {
        let (x, y) = ((i as u32 % side) as f32, (i as u32 / side) as f32);
        let bend = 22.0 * noise.smooth(1, x, y, 256.0);
        let stroke = |salt: u32, along: f32, across: f32| {
            0.6 * noise.streak(salt, along, across + bend, 256.0, 30.0)
                + 0.6 * noise.streak(salt + 1, along, across + bend, 128.0, 5.0)
        };
        let which = step(-0.3, 0.3, noise.smooth(2, x, y, 170.0));
        let wash = stroke(3, x + y, x - y) * which + stroke(5, x - y, x + y) * (1.0 - which);
        let value = 0.55 * wash + 0.35 * noise.cloud(7, x, y, 170.0, 3) + 0.45 * signed(x as i32, y as i32, 8);
        pixel[..3].copy_from_slice(&(colour * (1.0 + grain * value)).srgb());
    });
    Image { width: side, height: side, rgba }
}

fn fire(wall: &Wall, paint: impl Fn(f32, f32, [i32; 2]) -> Paint + Sync) -> Panel {
    let [width, height] = wall.size.map(|side| (side * wall.density).round() as usize);
    let kiln = Kiln {
        wall,
        fired: GLAZES.iter().map(glaze::Glaze::fired).collect(),
        clay: Rgb::hex(0xc08a5e),
        iron: Rgb::hex(0x4a2a14),
        manganese: Rgb::hex(0x24130d),
    };
    let centre = |pixel: usize| (pixel as f32 + 0.5) / wall.density;
    let surface = |i: usize| kiln.surface(centre(i % width), centre(i / width), &paint);
    Panel { width, height, density: wall.density, surfaces: (0..width * height).into_par_iter().map(surface).collect() }
}

impl Panel {
    fn photograph(&self, light: Light) -> Image {
        let &Self { width, height, density, ref surfaces } = self;
        let dark = matches!(light, Light::Night { .. });
        let shadow_colour = if dark { Rgb::default() } else { Rgb::hex(0x3a2616) };
        let mut rgba = vec![0; width * height * 4];
        rgba.par_chunks_mut(4).enumerate().for_each(|(i, pixel)| {
            let surface = &surfaces[i];
            let (column, row) = (i % width, i / width);
            let rise = |before: usize, after: usize| (surfaces[after].height - surfaces[before].height) * density / 2.0;
            let slope = [
                rise(i - usize::from(column > 0), i + usize::from(column + 1 < width)),
                rise(i - if row > 0 { width } else { 0 }, i + if row + 1 < height { width } else { 0 }),
            ];
            let [x, y] = [column, row].map(|pixel| (pixel as f32 + 0.5) / density);
            let shadow = surface.shadow * (1.0 - surface.cover);
            let alpha = surface.cover + shadow;
            if alpha > 0.0 {
                let colour = shadow_colour.mix(shine(surface, slope, x, y, light), surface.cover / alpha);
                pixel[..3].copy_from_slice(&colour.srgb());
                pixel[3] = (alpha * 255.0 + 0.5) as u8;
            }
        });
        Image { width: width as u32, height: height as u32, rgba }
    }

    /// A half-size mask for what is drawn on the lake. Like lustre, it breaks at every joint and stops at the shore.
    fn water(&self) -> Image {
        let (width, height) = (self.width / 2, self.height / 2);
        let pixel = |i: usize| {
            let corner = i / width * 2 * self.width + i % width * 2;
            let wet: f32 = [0, 1, self.width, self.width + 1].iter().map(|step| self.surfaces[corner + step].wet).sum();
            // A few levels do, and compress far better.
            [0, 0, 0, (wet / 4.0 * 5.0).round() as u8 * 51]
        };
        Image { width: width as u32, height: height as u32, rgba: (0..width * height).flat_map(pixel).collect() }
    }
}
