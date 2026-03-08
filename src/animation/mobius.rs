use std::f64::consts::TAU;

use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math::{self, DepthRange};
use super::Animation;

const NUM_PARTICLES: usize = 1500;

struct Particle {
    u: f64,
    v: f64,
    speed: f64,
}

/// Möbius strip particle flow animation.
///
/// ~1500 particles flow along the surface of a Möbius strip,
/// rendered as braille dots with four depth-based colour layers
/// in warm amber/copper tones.
pub struct Mobius {
    particles: Vec<Particle>,
    angle_x: f64,
    angle_y: f64,
}

impl Mobius {
    #[expect(clippy::cast_precision_loss, reason = "animation indices are small")]
    pub fn new() -> Self {
        let particles = (0..NUM_PARTICLES)
            .map(|i| {
                let frac = i as f64 / NUM_PARTICLES as f64;
                Particle {
                    u: frac * TAU,
                    v: -1.0 + 2.0 * ((i * 7) as f64 % NUM_PARTICLES as f64) / NUM_PARTICLES as f64,
                    speed: 0.3 + (i % 7) as f64 * 0.12,
                }
            })
            .collect();
        Self {
            particles,
            angle_x: 0.3,
            angle_y: 0.0,
        }
    }
}

impl Animation for Mobius {
    fn update(&mut self, dt: f32) {
        let dt = f64::from(dt);
        self.angle_y += 0.4 * dt;
        self.angle_x = 0.3 + 0.15 * (self.angle_y * 0.25).sin();

        for p in &mut self.particles {
            p.u = (p.u + p.speed * dt) % TAU;
        }
    }

    fn draw(&self, ctx: &mut Context, viewport: (f64, f64)) {
        let (vw, vh) = viewport;
        let vmin = vw.min(vh);
        let scale = vmin * 0.55;
        let distance = 4.0 * scale;

        let mut data = Vec::with_capacity(NUM_PARTICLES);
        let mut z_min = f64::INFINITY;
        let mut z_max = f64::NEG_INFINITY;

        for p in &self.particles {
            let half_u = p.u / 2.0;
            let r = 1.0 + p.v / 2.0 * half_u.cos();
            let point = [
                r * p.u.cos() * scale,
                r * p.u.sin() * scale,
                p.v / 2.0 * half_u.sin() * scale,
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

        // Warm amber/copper palette — back-to-front
        if !far.is_empty() {
            ctx.draw(&Points {
                coords: &far,
                color: Color::Rgb(100, 55, 20),
            });
        }
        if !mid_far.is_empty() {
            ctx.draw(&Points {
                coords: &mid_far,
                color: Color::Rgb(160, 90, 30),
            });
        }
        if !mid_near.is_empty() {
            ctx.draw(&Points {
                coords: &mid_near,
                color: Color::Rgb(210, 140, 50),
            });
        }
        if !near.is_empty() {
            ctx.draw(&Points {
                coords: &near,
                color: Color::Rgb(245, 190, 80),
            });
        }
    }

    fn name(&self) -> &'static str {
        "mobius"
    }

    fn description(&self) -> &'static str {
        "Möbius strip particle flow"
    }
}
