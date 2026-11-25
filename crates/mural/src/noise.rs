//! Deterministic noise. Wavelengths are in design pixels.

/// A 32-bit mix of three integers.
pub fn hash(x: i32, y: i32, seed: u32) -> u32 {
    let mut h =
        (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^ (h >> 16)
}

/// Uniform in `0..1`.
pub fn unit(x: i32, y: i32, seed: u32) -> f32 {
    (hash(x, y, seed) >> 8) as f32 / (1 << 24) as f32
}

/// Uniform in `-1..1`.
pub fn signed(x: i32, y: i32, seed: u32) -> f32 {
    unit(x, y, seed) * 2.0 - 1.0
}

/// A noise source, which can repeat so that a texture tiles.
#[derive(Clone, Copy)]
pub struct Noise {
    pub seed: u32,
    pub period: Option<f32>,
}

impl Noise {
    /// The number of cells in a period, and the wavelength that makes it whole.
    fn fit(self, wavelength: f32) -> (i32, f32) {
        match self.period {
            Some(period) => {
                let cells = (period / wavelength).round().max(1.0);
                (cells as i32, period / cells)
            }
            None => (i32::MAX, wavelength),
        }
    }

    /// Smooth noise in about `-1..1`, stretched to different wavelengths along each axis.
    pub fn streak(self, salt: u32, x: f32, y: f32, along_x: f32, along_y: f32) -> f32 {
        let ((nx, wx), (ny, wy)) = (self.fit(along_x), self.fit(along_y));
        let (x, y) = (x / wx, y / wy);
        let (i, j) = (x.floor() as i32, y.floor() as i32);
        let (u, v) = (x - x.floor(), y - y.floor());
        let seed = self.seed.wrapping_add(salt.wrapping_mul(0x27D4_EB2F));
        let slope = |a: i32, b: i32| {
            let h = hash((i + a).rem_euclid(nx), (j + b).rem_euclid(ny), seed);
            let (gx, gy) = ((h & 0xFFFF) as f32 / 32768.0 - 1.0, (h >> 16) as f32 / 32768.0 - 1.0);
            gx * (u - a as f32) + gy * (v - b as f32)
        };
        let fade = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
        let (fu, fv) = (fade(u), fade(v));
        let top = slope(0, 0) + (slope(1, 0) - slope(0, 0)) * fu;
        let bottom = slope(0, 1) + (slope(1, 1) - slope(0, 1)) * fu;
        (top + (bottom - top) * fv) * 2.2
    }

    pub fn smooth(self, salt: u32, x: f32, y: f32, wavelength: f32) -> f32 {
        self.streak(salt, x, y, wavelength, wavelength)
    }

    /// Octaves of smooth noise, each half the size of the last.
    pub fn cloud(self, salt: u32, x: f32, y: f32, wavelength: f32, octaves: u32) -> f32 {
        let (mut sum, mut gain, mut wavelength) = (0.0, 0.62, wavelength);
        for octave in 0..octaves {
            sum += gain * self.smooth(salt + octave * 101, x, y, wavelength);
            gain *= 0.55;
            wavelength *= 0.5;
        }
        sum
    }

    /// Distances to the two nearest of a scatter of points, and a hash of the nearest.
    pub fn cells(self, salt: u32, x: f32, y: f32, spacing: f32) -> (f32, f32, u32) {
        let (n, spacing) = self.fit(spacing);
        let (x, y) = (x / spacing, y / spacing);
        let (i, j) = (x.floor() as i32, y.floor() as i32);
        let seed = self.seed.wrapping_add(salt.wrapping_mul(0x27D4_EB2F));
        let (mut first, mut second, mut id) = (f32::MAX, f32::MAX, 0);
        for b in j - 1..=j + 1 {
            for a in i - 1..=i + 1 {
                let h = hash(a.rem_euclid(n), b.rem_euclid(n), seed);
                let dx = a as f32 + (h & 0xFFFF) as f32 / 65536.0 - x;
                let dy = b as f32 + (h >> 16) as f32 / 65536.0 - y;
                let d = dx * dx + dy * dy;
                if d < first {
                    (second, first, id) = (first, d, hash(h as i32, 1, 7));
                } else if d < second {
                    second = d;
                }
            }
        }
        (first.sqrt() * spacing, second.sqrt() * spacing, id)
    }
}
