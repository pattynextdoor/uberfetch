use std::f64::consts::TAU;

use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math;
use super::Animation;

const NUM_PARTICLES: usize = 600;

struct Particle {
    theta: f64,
    phi: f64,
    speed: f64,
}

/// Toroidal particle flow animation.
///
/// ~600 particles flow along the surface of a parametric torus,
/// rendered as braille dots with two colour layers for depth.
pub struct Toroid {
    particles: Vec<Particle>,
    angle_x: f64,
    angle_y: f64,
    major_radius: f64,
    minor_radius: f64,
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
                    speed: 0.5 + (i % 5) as f64 * 0.2,
                }
            })
            .collect();
        Self {
            particles,
            angle_x: 0.4,
            angle_y: 0.0,
            major_radius: 20.0,
            minor_radius: 8.0,
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

    fn draw(&self, ctx: &mut Context) {
        let distance = 4.0 * self.major_radius;

        let coords: Vec<(f64, f64)> = self
            .particles
            .iter()
            .map(|p| {
                let r = self.major_radius + self.minor_radius * p.phi.cos();
                let point = [
                    r * p.theta.cos(),
                    self.minor_radius * p.phi.sin(),
                    r * p.theta.sin(),
                ];
                let point = math::rotate_x(point, self.angle_x);
                let point = math::rotate_y(point, self.angle_y);
                let proj = math::project(point, distance);
                (proj[0], proj[1])
            })
            .collect();

        ctx.draw(&Points {
            coords: &coords,
            color: Color::Rgb(200, 160, 255),
        });

        // Second layer with different color for depth
        let coords2: Vec<(f64, f64)> = self
            .particles
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 3 == 0)
            .map(|(_, p)| {
                let offset_phi = p.phi + 0.3;
                let r = self.major_radius + self.minor_radius * offset_phi.cos();
                let point = [
                    r * p.theta.cos(),
                    self.minor_radius * offset_phi.sin(),
                    r * p.theta.sin(),
                ];
                let point = math::rotate_x(point, self.angle_x);
                let point = math::rotate_y(point, self.angle_y);
                let proj = math::project(point, distance);
                (proj[0], proj[1])
            })
            .collect();

        ctx.draw(&Points {
            coords: &coords2,
            color: Color::Rgb(140, 100, 200),
        });
    }

    fn name(&self) -> &'static str {
        "toroid"
    }

    fn description(&self) -> &'static str {
        "Toroidal particle flow"
    }
}
