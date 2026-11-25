//! The point of a line that is closest to a vector.

use explainers::{Explainer, Markup, explainer, html, point};

struct Projection {
    vector: [f32; 2],
    degrees: f32,
    turning: bool,
}

const UNIT: f32 = 50.0;

impl Explainer for Projection {
    fn new(options: &str) -> Self {
        Self { vector: [90.0, -120.0], degrees: options.parse().unwrap_or(16.0), turning: false }
    }

    fn input(&mut self, name: &str, value: &str) {
        match (name, point(value), value.parse()) {
            ("vector", Some([x, y]), _) => self.vector = [x.clamp(-300.0, 300.0), y.clamp(-150.0, 150.0)],
            ("degrees", _, Ok(degrees)) => (self.degrees, self.turning) = (degrees, false),
            ("turn", ..) => self.turning ^= true,
            _ => {}
        }
    }

    fn tick(&mut self, seconds: f32) -> bool {
        if self.turning {
            self.degrees = (self.degrees + 20.0 * seconds + 90.0) % 180.0 - 90.0;
        }
        self.turning
    }

    fn view(&self) -> Markup {
        let [x, y] = self.vector;
        let (sin, cos) = (-self.degrees.to_radians()).sin_cos();
        let (along, across) = (x * cos + y * sin, y * cos - x * sin);
        let corner = 11.0_f32.min(along.abs()).min(across.abs());
        let (back, up) = (-corner.copysign(along), corner.copysign(across));
        html! {
            svg viewBox="-325 -170 650 340" {
                g transform={ "rotate(" (-self.degrees) ")" } {
                    path.faint d="M-400 0H400" {}
                    path.faint d={ "M" (along + back) " 0v" (up) "h" (-back) } {}
                    path.dashed d={ "M" (along) " 0V" (across) } {}
                    path.cool d={ "M0 0H" (along) } {}
                    circle.cool cx=(along) r="4.5" {}
                }
                path.warm d={ "M0 0L" (x) " " (y) } {}
                circle r="3" {}
                circle.warm cx=(x) cy=(y) r="9" data-drag="vector" tabindex="0" role="img" aria-label="The tip of u" {}
                text x=(x + 15.0) y=(y - 6.0) { "u" }
            }
            form {
                label {
                    "The slope of W"
                    input type="range" name="degrees" min="-90" max="90" value=(self.degrees.round());
                }
                button type="button" name="turn" { @if self.turning { "stop" } @else { "turn" } }
                output { "d(u, W) = " (format!("{:.2}", across.abs() / UNIT)) }
            }
        }
    }
}

explainer!(Projection);
