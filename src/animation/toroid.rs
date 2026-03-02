use std::f64::consts::TAU;

use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math;
use super::Animation;

const NUM_PARTICLES: usize = 1500;

struct Particle {
    theta: f64,
    phi: f64,
    speed: f64,
}

/// Toroidal particle flow animation.
///
/// ~1500 particles flow along the surface of a parametric torus,
/// rendered as braille dots with four depth-based colour layers.
pub struct Toroid {
    particles: Vec<Particle>,
    angle_x: f64,
    angle_y: f64,
}

impl Toroid {
    #[expect(clippy::cast_precision_loss, reason = "animation indices are small")]
    pub fn new() -> Self {
        let particles = (0..NUM_PARTICLES)
            .map(|i| {
                let frac = i as f64 / NUM_PARTICLES as f64;
                Particle {
                    theta: frac * TAU,
                    phi: (frac * 7.0 * TAU) % TAU,
                    speed: 0.3 + (i % 7) as f64 * 0.15,
                }
            })
            .collect();
        Self {
            particles,
            angle_x: 0.4,
            angle_y: 0.0,
        }
    }
}

impl Animation for Toroid {
    fn update(&mut self, dt: f32) {
        let dt = f64::from(dt);
        self.angle_y += 0.5 * dt;
        self.angle_x = 0.4 + 0.2 * (self.angle_y * 0.3).sin();

        for p in &mut self.particles {
            p.theta = (p.theta + p.speed * dt) % TAU;
            p.phi = (p.phi + p.speed * 1.5 * dt) % TAU;
        }
    }

    fn draw(&self, ctx: &mut Context, viewport: (f64, f64)) {
        let (vw, vh) = viewport;
        let vmin = vw.min(vh);
        // Scale torus radii relative to viewport
        let major = vmin * 0.50;
        let minor = major * 0.4;
        let distance = 4.0 * major;

        // Compute 3D position, project to 2D, and track z for depth
        let data: Vec<([f64; 2], f64)> = self
            .particles
            .iter()
            .map(|p| {
                let r = major + minor * p.phi.cos();
                let point = [r * p.theta.cos(), minor * p.phi.sin(), r * p.theta.sin()];
                let point = math::rotate_x(point, self.angle_x);
                let point = math::rotate_y(point, self.angle_y);
                let z = point[2];
                let proj = math::project(point, distance);
                (proj, z)
            })
            .collect();

        // Z-range for depth normalization
        let (z_min, z_max) = data
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &(_, z)| {
                (min.min(z), max.max(z))
            });
        let z_range = (z_max - z_min).max(0.001);

        // Bucket particles by depth layer for back-to-front drawing
        let mut near = Vec::new();
        let mut mid_near = Vec::new();
        let mut mid_far = Vec::new();
        let mut far = Vec::new();

        for &(proj, z) in &data {
            let depth = 1.0 - (z - z_min) / z_range;
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

        // Draw back-to-front so nearer particles overwrite farther ones
        if !far.is_empty() {
            ctx.draw(&Points {
                coords: &far,
                color: Color::Rgb(80, 50, 120),
            });
        }
        if !mid_far.is_empty() {
            ctx.draw(&Points {
                coords: &mid_far,
                color: Color::Rgb(140, 100, 190),
            });
        }
        if !mid_near.is_empty() {
            ctx.draw(&Points {
                coords: &mid_near,
                color: Color::Rgb(190, 150, 240),
            });
        }
        if !near.is_empty() {
            ctx.draw(&Points {
                coords: &near,
                color: Color::Rgb(240, 200, 255),
            });
        }
    }

    fn name(&self) -> &'static str {
        "toroid"
    }

    fn description(&self) -> &'static str {
        "Toroidal particle flow"
    }
}
