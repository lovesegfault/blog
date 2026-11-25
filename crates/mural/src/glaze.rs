//! Glazes, and the colours they fire to.

use std::ops::{Add, Mul};

/// Linear light.
#[derive(Clone, Copy, Default)]
pub struct Rgb(pub [f32; 3]);

impl Rgb {
    pub fn hex(code: u32) -> Self {
        Self([16, 8, 0].map(|shift| {
            let c = (code >> shift & 0xFF) as f32 / 255.0;
            if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        }))
    }

    pub fn mix(self, other: Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
    }

    pub fn srgb(self) -> [u8; 3] {
        self.0.map(|c| {
            let c = c.clamp(0.0, 1.0);
            let c = if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
            (c * 255.0 + 0.5) as u8
        })
    }
}

impl Add for Rgb {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self([0, 1, 2].map(|i| self.0[i] + other.0[i]))
    }
}

impl Mul for Rgb {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self([0, 1, 2].map(|i| self.0[i] * other.0[i]))
    }
}

impl Mul<f32> for Rgb {
    type Output = Self;
    fn mul(self, k: f32) -> Self {
        Self(self.0.map(|c| c * k))
    }
}

/// A recipe. A glaze changes colour with how thickly it lies.
#[derive(Clone, Copy)]
pub struct Glaze {
    pub name: &'static str,
    thin: u32,
    even: u32,
    pooled: u32,
    /// What floats to the surface in clouds.
    bloom: u32,
    pub blooming: f32,
    /// How much the thickness wanders.
    pub mottle: f32,
    pub brush: f32,
    pub specks: f32,
    pub crazing: f32,
    pub gloss: f32,
}

const fn glaze(name: &'static str, colours: [u32; 4], character: [f32; 6]) -> Glaze {
    let [thin, even, pooled, bloom] = colours;
    let [blooming, mottle, brush, specks, crazing, gloss] = character;
    Glaze { name, thin, even, pooled, bloom, blooming, mottle, brush, specks, crazing, gloss }
}

#[rustfmt::skip]
pub const GLAZES: &[Glaze] = &[
    //                  thin      even      pooled    bloom       bloom mottle brush specks craze gloss
    glaze("sand",      [0xbf9260, 0xdfcca2, 0xc9b184, 0xefe4c8], [0.60, 0.70, 0.30, 1.00, 0.45, 0.45]),
    glaze("cream",     [0xd2b484, 0xf0e6cc, 0xddd3b4, 0xfaf4e2], [0.55, 0.55, 0.22, 0.85, 1.00, 0.70]),
    glaze("ochre",     [0xe6ba5c, 0xc98322, 0x80450e, 0xeccb84], [0.45, 0.95, 0.60, 0.70, 0.45, 0.90]),
    glaze("sienna",    [0xc9743f, 0x9a3c19, 0x48160a, 0xc2662e], [0.50, 1.00, 0.65, 0.55, 0.15, 0.80]),
    glaze("salmon",    [0xe2b08a, 0xc67e58, 0x8c4a2e, 0xe8c6a8], [0.50, 0.80, 0.45, 0.80, 0.50, 0.60]),
    glaze("olive",     [0xbcac62, 0x84853f, 0x464d1f, 0xb4843a], [0.55, 1.00, 0.55, 0.75, 0.40, 0.85]),
    glaze("moss",      [0x888a48, 0x43572b, 0x172914, 0x7a5520], [0.45, 1.00, 0.55, 0.45, 0.30, 0.95]),
    glaze("slate",     [0xb8baa4, 0x789398, 0x3e5d68, 0xc6d3ca], [0.60, 0.95, 0.45, 0.70, 0.80, 0.80]),
    glaze("manganese", [0x84492a, 0x2e1b15, 0x110a09, 0x6e3b1c], [0.50, 0.85, 0.35, 0.00, 0.00, 1.00]),
    glaze("cobalt",    [0x4a80d6, 0x1338a2, 0x050d48, 0x3468ca], [0.34, 0.72, 0.70, 0.12, 0.50, 1.00]),
    // A darker firing.
    glaze("char",      [0x8a5532, 0x33221a, 0x1c120e, 0x573623], [0.60, 0.85, 0.40, 0.00, 0.20, 0.75]),
    glaze("ash",       [0xa88a60, 0xcdbb94, 0xa3906c, 0xe0d2ae], [0.55, 0.75, 0.25, 0.90, 1.00, 0.70]),
    // For behind text: the same clay, kept quiet.
    glaze("slab",      [0xd6b98c, 0xf6f0de, 0xd9d6be, 0xfbf7ea], [0.40, 0.20, 0.06, 0.55, 0.85, 0.95]),
    glaze("deep",      [0x24439a, 0x0c1a62, 0x060e40, 0x12267c], [0.70, 0.85, 0.40, 0.00, 0.25, 0.80]),
    glaze("umber",     [0x704428, 0x31221b, 0x271a14, 0x3a281e], [0.40, 0.30, 0.18, 0.00, 0.40, 0.42]),
];

pub fn find(name: &str) -> Option<usize> {
    GLAZES.iter().position(|glaze| glaze.name == name)
}

pub fn named(name: &str) -> usize {
    find(name).expect("a known glaze")
}

/// A recipe's colours in linear light.
#[derive(Clone, Copy)]
pub struct Fired {
    thin: Rgb,
    even: Rgb,
    pooled: Rgb,
    pub bloom: Rgb,
}

impl Glaze {
    pub fn fired(&self) -> Fired {
        Fired {
            thin: Rgb::hex(self.thin),
            even: Rgb::hex(self.even),
            pooled: Rgb::hex(self.pooled),
            bloom: Rgb::hex(self.bloom),
        }
    }
}

impl Fired {
    /// `thickness` is 0 where the glaze breaks, 1 where it lies evenly, and 2 where it pools.
    pub fn colour(&self, thickness: f32) -> Rgb {
        // Mixing square roots keeps the middle of a ramp from looking washed out.
        let blend =
            |a: Rgb, b: Rgb, t: f32| Rgb([0, 1, 2].map(|i| (a.0[i].sqrt() * (1.0 - t) + b.0[i].sqrt() * t).powi(2)));
        let t = thickness.clamp(0.0, 2.0);
        if t < 1.0 { blend(self.thin, self.even, t) } else { blend(self.even, self.pooled, t - 1.0) }
    }
}
