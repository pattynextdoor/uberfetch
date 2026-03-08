use std::f64::consts::TAU;

use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math::{self, DepthRange};
use super::Animation;

const NUM_PARTICLES: usize = 1200;
/// Trefoil knot parameters: winds p times around the torus hole
/// while winding q times around the torus tube.
const P: f64 = 2.0;
const Q: f64 = 3.0;

struct Particle {
    t: f64,
    speed: f64,
}

/// Torus knot (trefoil) particle flow animation.
///
/// ~1200 particles flow along a (2,3) torus knot curve,
/// rendered with four depth-based colour layers in cool blue-green tones.
pub struct TorusKnot {
    particles: Vec<Particle>,
    angle_x: f64,
    angle_y: f64,
}

impl TorusKnot {
    #[expect(clippy::cast_precision_loss, reason = "animation indices are small")]
    pub fn new() -> Self {
        let particles = (0..NUM_PARTICLES)
            .map(|i| {
                let frac = i as f64 / NUM_PARTICLES as f64;
                Particle {
                    t: frac * TAU,
                    speed: 0.25 + (i % 9) as f64 * 0.1,
                }
            })
            .collect();
        Self {
            particles,
            angle_x: 0.5,
            angle_y: 0.0,
        }
    }
}

impl Animation for TorusKnot {
    fn update(&mut self, dt: f32) {
        let dt = f64::from(dt);
        self.angle_y += 0.35 * dt;
        self.angle_x = 0.5 + 0.25 * (self.angle_y * 0.2).sin();

        for p in &mut self.particles {
            p.t = (p.t + p.speed * dt) % TAU;
        }
    }

    fn draw(&self, ctx: &mut Context, viewport: (f64, f64)) {
        let (vw, vh) = viewport;
        let vmin = vw.min(vh);
        let scale = vmin * 0.35;
        let distance = 5.0 * scale;

        let mut data = Vec::with_capacity(NUM_PARTICLES);
        let mut z_min = f64::INFINITY;
        let mut z_max = f64::NEG_INFINITY;

        for p in &self.particles {
            let r = 2.0 + (Q * p.t).cos();
            let point = [
                r * (P * p.t).cos() * scale,
                r * (P * p.t).sin() * scale,
                (Q * p.t).sin() * scale,
            ];
            let point = math::rotate_x(point, self.angle_x);
            let point = math::rotate_y(point, self.angle_y);
            let z = point[2];
            z_min = z_min.min(z);
            z_max = z_max.max(z);
            let proj = math::project(point, distance);
            data.push((proj, z));
        }
        let depth_range = DepthRange::new(z_min, z_max);

        let mut near = Vec::new();
        let mut mid_near = Vec::new();
        let mut mid_far = Vec::new();
        let mut far = Vec::new();

        for &(proj, z) in &data {
            if !math::is_visible(proj, vw, vh) {
                continue;
            }
            let depth = depth_range.normalize(z);
            let coord = (proj[0], proj[1]);

            if depth > 0.75 {
                near.push(coord);
            } else if depth > 0.50 {
                mid_near.push(coord);
            } else if depth > 0.25 {
                mid_far.push(coord);
            } else {
                far.push(coord);
            }
        }

        // Cool blue-green palette — back-to-front
        if !far.is_empty() {
            ctx.draw(&Points {
                coords: &far,
                color: Color::Rgb(20, 60, 80),
            });
        }
        if !mid_far.is_empty() {
            ctx.draw(&Points {
                coords: &mid_far,
                color: Color::Rgb(40, 120, 140),
            });
        }
        if !mid_near.is_empty() {
            ctx.draw(&Points {
                coords: &mid_near,
                color: Color::Rgb(70, 180, 200),
            });
        }
        if !near.is_empty() {
            ctx.draw(&Points {
                coords: &near,
                color: Color::Rgb(120, 220, 240),
            });
        }
    }

    fn name(&self) -> &'static str {
        "torus-knot"
    }

    fn description(&self) -> &'static str {
        "Trefoil torus knot flow"
    }
}
