//! Everything the site has fired.

use anyhow::Result;

use crate::glaze::{Rgb, named};
use crate::valley::{Cartoon, DEEP_CALM, Stencil, View};
use crate::{Image, Lamp, Light, Paint, Panel, Valley, Wall, fire, limewash, step};

/// A mural with the collision in it. The arrow leaves the vertex, passes over the farm, and ends at `tip`.
pub struct Scene {
    pub name: &'static str,
    cut: &'static str,
    /// In metres from the farm.
    collision: [f32; 2],
    /// From the vertex to the farm, in tiles, so that both fall in the middle of one.
    tiles: [f32; 2],
    /// The arrow's length, and how many metres that is.
    pub length: f32,
    metres: f32,
    pub size: [f32; 2],
    pub tip: [f32; 2],
    /// The radius of the tracker.
    pub radius: f32,
    joint: f32,
    density: f32,
    lamp: f32,
}

pub const HOME: Scene = Scene {
    name: "home",
    cut: "wide",
    collision: [349.0, 32.0],
    tiles: [4.0, -3.0],
    length: 620.0,
    metres: 660.0,
    size: [1060.0, 1180.0],
    tip: [1044.0, 200.0],
    radius: 291.0,
    joint: 1.45,
    density: 2.0,
    lamp: 341.0,
};

pub const POST: Scene = Scene {
    name: "post",
    cut: "tight",
    collision: [346.0, 26.0],
    tiles: [4.0, -2.0],
    length: 294.0,
    metres: 560.0,
    size: [420.0, 1010.0],
    tip: [402.0, 310.0],
    radius: 153.0,
    joint: 1.0,
    density: 2.5,
    lamp: 194.0,
};

pub const NARROW: Scene = Scene {
    name: "narrow",
    cut: "tight",
    collision: [341.0, 55.0],
    tiles: [-2.0, 4.0],
    length: 252.0,
    metres: 575.0,
    size: [800.0, 352.0],
    tip: [180.0, 346.0],
    radius: 126.0,
    joint: 1.0,
    density: 2.5,
    lamp: 166.0,
};

/// The glazes that labels and small tiles come in, and whether they take dark text.
pub const LABELS: [(&str, bool); 8] = [
    ("ochre", true),
    ("sienna", false),
    ("olive", true),
    ("slate", true),
    ("cobalt", false),
    ("cream", true),
    ("moss", false),
    ("salmon", true),
];

const QUOTE: &str = "M25.4 6.2C14.6 9.8 6.2 18.4 6.2 28.4C6.2 34.6 10.8 38.8 16.4 38.8C22.2 38.8 26.6 34.6 26.6 29\
    C26.6 23.8 22.6 19.8 17.2 19.6C16.6 19.6 16 19.6 15.6 19.7C17.6 15.6 21.8 12.4 27 10.4Z\
    M50.2 5.8C39.2 9.2 30.6 17.8 30.6 28C30.6 34.2 35.4 38.4 41 38.4C46.8 38.4 51 34 51 28.6\
    C51 23.4 47 19.4 41.6 19.2C41 19.2 40.4 19.2 40 19.3C42.2 15.2 46.6 12 51.8 10Z";

impl Scene {
    /// The way the arrow points, clockwise.
    pub fn turn(&self) -> f32 {
        self.tiles[1].atan2(self.tiles[0])
    }

    pub fn vertex(&self) -> [f32; 2] {
        [self.tip[0] - self.turn().cos() * self.length, self.tip[1] - self.turn().sin() * self.length]
    }

    /// How far along the arrow the farm is.
    pub fn farm(&self) -> f32 {
        self.collision[0].hypot(self.collision[1]) * self.length / self.metres
    }

    fn fire(&self, valley: &Valley) -> Result<Vec<Image>> {
        let (vertex, tile) = (self.vertex(), self.farm() / self.tiles[0].hypot(self.tiles[1]));
        let view = View {
            cut: self.cut,
            anchor: self.collision,
            at: vertex,
            scale: self.length / self.metres,
            turn: self.turn() - (-self.collision[1]).atan2(-self.collision[0]),
            dark: self.length * 0.5,
            sink: None,
        };
        let wall = Wall {
            origin: [0, 1].map(|i| vertex[i] + (self.tiles[i] - 0.5) * tile),
            joint: self.joint,
            missing: true,
            ..tiled(self.size, self.density, tile, 7)
        };
        let lamp = Lamp { at: vertex, reach: self.lamp, power: 0.58 };
        landscape(valley, view, &wall, Light::Night { ambient: DUSK, lamp: Some(lamp) })
    }
}

/// The light at night, on the murals and on what is near the text.
const DUSK: Rgb = Rgb([0.50, 0.43, 0.41]);
const NEAR: Rgb = Rgb([0.84, 0.78, 0.77]);

fn dim(ambient: Rgb) -> Light {
    Light::Night { ambient, lamp: None }
}

fn tiled(size: [f32; 2], density: f32, tile: f32, seed: u32) -> Wall {
    Wall {
        size,
        density,
        tile: [tile; 2],
        origin: [0.0; 2],
        joint: 1.5,
        grout: Some(Rgb::hex(0x665646)),
        repeat: [i32::MAX; 2],
        period: None,
        missing: false,
        seed,
    }
}

/// Tiles set straight on the page.
fn loose(tile: [f32; 2], count: [i32; 2], joint: f32, seed: u32) -> Wall {
    let size = [tile[0] * count[0] as f32, tile[1] * count[1] as f32];
    Wall { tile, joint, grout: None, repeat: count, ..tiled(size, 3.0, 0.0, seed) }
}

/// By day, by night on black clay, and where the water is.
fn landscape(valley: &Valley, view: View, wall: &Wall, night: Light) -> Result<Vec<Image>> {
    let cartoon = Cartoon::trace(valley, view, wall.size, wall.density)?;
    let (day, water) = {
        let panel = fire(wall, |x, y, _| cartoon.paint(x, y));
        (panel.photograph(Light::Day), panel.water())
    };
    let darker = [("sand", "char"), ("cream", "ash")].map(|(pale, dark)| (named(pale), named(dark)));
    let blacken = |glaze| darker.iter().find(|(pale, _)| *pale == glaze).map_or(glaze, |(_, dark)| *dark);
    let night = fire(&Wall { grout: Some(Rgb::hex(0x201712)), ..*wall }, |x, y, _| {
        let paint = cartoon.paint(x, y);
        Paint { glazes: paint.glazes.map(blacken), ..paint }
    })
    .photograph(night);
    Ok(vec![day, night, water])
}

fn both(panel: Panel, ambient: Rgb) -> Vec<Image> {
    vec![panel.photograph(Light::Day), panel.photograph(dim(ambient))]
}

fn coast(valley: &Valley) -> Result<Vec<Image>> {
    let view = View {
        cut: "tight",
        anchor: [1010.0, -490.0],
        at: [720.0, 196.0],
        scale: 1440.0 / 1640.0,
        turn: 0.335,
        dark: 1.0,
        sink: Some([205.0, 296.0]),
    };
    landscape(valley, view, &tiled([1440.0, 300.0], 1.7, 60.0, 9), dim(DUSK * 1.4))
}

fn wall() -> Vec<Image> {
    vec![limewash(512, Rgb::hex(0xf3efe6), 0.026), limewash(512, Rgb::hex(0x1e1613), 0.11)]
}

/// What the coast runs into.
fn pool() -> Vec<Image> {
    let deep = Paint { calm: DEEP_CALM, ..Paint::plain("deep") };
    both(fire(&Wall { repeat: [6; 2], ..tiled([360.0; 2], 2.0, 60.0, 9) }, |_, _, _| deep), DUSK * 1.4)
}

/// What code is written on.
fn slab() -> Vec<Image> {
    let wall = Wall { density: 2.0, period: Some(256.0), ..loose([304.0; 2], [1; 2], 5.0, 21) };
    let glazes = [Paint::plain("slab"), Paint { calm: 0.7, ..Paint::plain("umber") }];
    glazes.map(|glaze| fire(&wall, |_, _, _| glaze).photograph(Light::Day)).into()
}

fn row<const N: usize>(wall: Wall, glazes: [&str; N]) -> Vec<Image> {
    let glazes = glazes.map(Paint::plain);
    both(fire(&wall, |_, _, [x, y]| glazes[(x + y) as usize]), NEAR)
}

fn label(glaze: &str, pale: bool, seed: u32) -> Vec<Image> {
    let body = match (glaze, pale) {
        ("cream", _) => 0.0,
        (_, true) => -0.55,
        (_, false) => 0.42,
    };
    let paint = Paint { body, calm: 0.6, ..Paint::plain(glaze) };
    both(fire(&loose([120.0, 44.0], [1; 2], 4.0, seed), |_, _, _| paint), Rgb([1.0, 0.94, 0.90]))
}

/// Half day and half night.
fn dusk() -> Vec<Image> {
    let wall = Wall { density: 4.0, ..loose([24.0; 2], [1; 2], 3.0, 12) };
    let split = Paint { glazes: [named("cream"), named("cobalt")], ..Paint::plain("cream") };
    both(fire(&wall, |x, y, _| Paint { blend: step(23.6, 24.4, x + y), ..split }), NEAR * 1.8)
}

/// Made larger than it is shown, so that it looks as thin as a tile.
fn quote() -> Vec<Image> {
    let wall = Wall { origin: [-40.0; 2], ..tiled([144.0, 120.0], 1.7, 260.0, 17) };
    let mark = Stencil::trace(QUOTE, [60.0, 50.0], 8.0);
    let cut = |glaze, body| {
        let paint = Paint { body, calm: 0.2, ..Paint::plain(glaze) };
        fire(&wall, |x, y, _| Paint { inside: 2.4 * mark.inside(x / 2.4, y / 2.4), ..paint })
    };
    vec![cut("sienna", -0.15).photograph(Light::Day), cut("ochre", -0.3).photograph(dim(NEAR * 1.2))]
}

/// One load of the kiln.
pub struct Firing<'a> {
    pub name: String,
    /// What to call the pictures that come out.
    pub takes: &'static [&'static str],
    /// How carefully they have to be compressed, out of 100.
    pub quality: f32,
    pub fire: Box<dyn Fn() -> Result<Vec<Image>> + 'a>,
}

impl<'a> Firing<'a> {
    fn new(name: &str, quality: f32, fire: impl Fn() -> Result<Vec<Image>> + 'a) -> Self {
        Self { name: name.to_owned(), takes: &["day", "night"], quality, fire: Box::new(fire) }
    }
}

pub fn firings(valley: &Valley) -> Vec<Firing<'_>> {
    let trim = |seed, glazes: [_; 7]| Ok(row(loose([11.0, 26.0], [1, 7], 2.6, seed), glazes));
    let mut all = vec![
        Firing::new("coast", 75.0, || coast(valley)),
        Firing::new("wall", 98.0, || Ok(wall())),
        Firing::new("pool", 76.0, || Ok(pool())),
        Firing::new("slab", 90.0, || Ok(slab())),
        Firing::new("dusk", 84.0, || Ok(dusk())),
        Firing::new("quote", 88.0, || Ok(quote())),
        Firing::new("rule", 84.0, || Ok(row(loose([24.0; 2], [3, 1], 4.0, 3), ["ochre", "cobalt", "sienna"]))),
        Firing::new("chips", 84.0, || Ok(row(loose([20.0; 2], [8, 1], 3.0, 8), LABELS.map(|(glaze, _)| glaze)))),
        Firing::new("trim", 84.0, move || trim(5, ["olive", "moss", "olive", "ochre", "moss", "olive", "slate"])),
        Firing::new("warm", 84.0, move || trim(6, ["ochre", "sienna", "ochre", "salmon", "sienna", "ochre", "sienna"])),
    ];
    for (seed, (glaze, pale)) in (30..).zip(LABELS) {
        all.push(Firing::new(&format!("label-{glaze}"), 84.0, move || Ok(label(glaze, pale, seed))));
    }
    for scene in [&HOME, &POST, &NARROW] {
        all.push(Firing { takes: &["day", "night", "water"], ..Firing::new(scene.name, 75.0, || scene.fire(valley)) });
    }
    all
}
