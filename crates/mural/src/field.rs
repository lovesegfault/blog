//! Scalar fields over pixels.

use rayon::prelude::*;

pub struct Field {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

const FAR: f32 = 1e12;

impl Field {
    /// The distance from each pixel to the nearest one that `is_seed`, after Felzenszwalb and Huttenlocher.
    pub fn distance(width: usize, height: usize, is_seed: impl Fn(usize) -> bool + Sync) -> Self {
        let mut rows: Vec<f32> =
            (0..width * height).into_par_iter().map(|i| if is_seed(i) { 0.0 } else { FAR }).collect();
        rows.par_chunks_mut(width).for_each(envelope);
        let mut columns = transpose(&rows, width, height);
        columns.par_chunks_mut(height).for_each(envelope);
        let mut data = transpose(&columns, height, width);
        data.par_iter_mut().for_each(|d| *d = d.sqrt());
        Self { width, height, data }
    }

    /// Positive where `is_inside`, negative elsewhere.
    pub fn signed_distance(width: usize, height: usize, is_inside: impl Fn(usize) -> bool + Sync) -> Self {
        let mut inside = Self::distance(width, height, |i| !is_inside(i));
        let outside = Self::distance(width, height, &is_inside);
        inside
            .data
            .par_iter_mut()
            .zip(&outside.data)
            .for_each(|(a, b)| *a = if *a > 0.0 { *a - 0.5 } else { 0.5 - *b });
        inside
    }

    /// Rounds off the staircase that measuring from whole pixels leaves.
    pub fn soften(mut self, passes: usize) -> Self {
        let (w, h) = (self.width, self.height);
        for _ in 0..passes {
            let from = self.data.clone();
            self.data.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
                let (up, down) = (y.saturating_sub(1) * w, (y + 1).min(h - 1) * w);
                for (x, out) in row.iter_mut().enumerate() {
                    let (left, right) = (x.saturating_sub(1), (x + 1).min(w - 1));
                    *out = (from[y * w + left]
                        + from[y * w + right]
                        + from[up + x]
                        + from[down + x]
                        + 2.0 * from[y * w + x])
                        / 6.0;
                }
            });
        }
        self
    }

    pub fn at(&self, x: f32, y: f32) -> f32 {
        let x = x.clamp(0.0, self.width as f32 - 1.001);
        let y = y.clamp(0.0, self.height as f32 - 1.001);
        let (i, u, v) = (y as usize * self.width + x as usize, x.fract(), y.fract());
        let top = self.data[i] + (self.data[i + 1] - self.data[i]) * u;
        let bottom = self.data[i + self.width] + (self.data[i + self.width + 1] - self.data[i + self.width]) * u;
        top + (bottom - top) * v
    }
}

fn transpose(from: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut to = vec![0.0; from.len()];
    to.par_chunks_mut(height).enumerate().for_each(|(x, column)| {
        for (y, out) in column.iter_mut().enumerate() {
            *out = from[y * width + x];
        }
    });
    to
}

/// Replaces each value by the lowest of the parabolas rooted at every other.
fn envelope(line: &mut [f32]) {
    let from = line.to_vec();
    let mut roots: Vec<usize> = Vec::new();
    let mut starts: Vec<f64> = Vec::new();
    for q in (0..from.len()).filter(|&q| from[q] < FAR) {
        let mut start = f64::NEG_INFINITY;
        while let Some(&p) = roots.last() {
            let lift = |i: usize| f64::from(from[i]) + (i * i) as f64;
            start = (lift(q) - lift(p)) / (2 * (q - p)) as f64;
            if start > starts[starts.len() - 1] {
                break;
            }
            roots.pop();
            starts.pop();
            start = f64::NEG_INFINITY;
        }
        roots.push(q);
        starts.push(start);
    }
    if roots.is_empty() {
        return;
    }
    let mut k = 0;
    for (q, out) in line.iter_mut().enumerate() {
        while k + 1 < roots.len() && starts[k + 1] < q as f64 {
            k += 1;
        }
        let d = q.abs_diff(roots[k]) as f32;
        *out = d * d + from[roots[k]];
    }
}
