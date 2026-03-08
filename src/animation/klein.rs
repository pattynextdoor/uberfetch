use std::f64::consts::TAU;

use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math::{self, DepthRange};
use super::Animation;

const NUM_PARTICLES: usize = 1500;
/// Radius parameter for the figure-8 immersion.
const A: f64 = 2.5;

struct Particle {
    u: f64,
    v: f64,
    speed: f64,
}

/// Klein bottle (figure-8 immersion) particle flow animation.
///
/// ~1500 particles on the surface of a Klein bottle using the figure-8
/// immersion parametrization. Self-intersection creates natural visual
/// density at crossing points. Rendered with deep emerald/jade greens.
pub struct Klein {
    particles: Vec<Particle>,
    angle_x: f64,
    angle_y: f64,
    angle_z: f64,
}

impl Klein {
    #[expect(clippy::cast_precision_loss, reason = "animation indices are small")]
    pub fn new() -> Self {
        let particles = (0..NUM_PARTICLES)
            .map(|i| {
                let frac = i as f64 / NUM_PARTICLES as f64;
                Particle {
                    u: frac * TAU,
                    v: ((i * 13) as f64 % NUM_PARTICLES as f64) / NUM_PARTICLES as f64 * TAU,
                    speed: 0.2 + (i % 11) as f64 * 0.08,
                }
            })
            .collect();
        Self {
            particles,
            angle_x: 0.3,
            angle_y: 0.0,
            angle_z: 0.0,
        }
    }
}

impl Animation for Klein {
    fn update(&mut self, dt: f32) {
        let dt = f64::from(dt);
        // Slow tumbling rotation — all 3 axes at different rates
        self.angle_x += 0.2 * dt;
        self.angle_y += 0.3 * dt;
        self.angle_z += 0.13 * dt;

        for p in &mut self.particles {
            p.u = (p.u + p.speed * dt) % TAU;
            p.v = (p.v + p.speed * 0.7 * dt) % TAU;
        }
    }

    #[expect(
        clippy::similar_names,
        reason = "sin_v and sin_dv are natural math names"
    )]
    fn draw(&self, ctx: &mut Context, viewport: (f64, f64)) {
        let (vw, vh) = viewport;
        let vmin = vw.min(vh);
        let scale = vmin * 0.22;
        let distance = 5.0 * scale;

        let mut data = Vec::with_capacity(NUM_PARTICLES);
        let mut z_min = f64::INFINITY;
        let mut z_max = f64::NEG_INFINITY;

        for p in &self.particles {
            let half_u = p.u / 2.0;
            let cos_hu = half_u.cos();
            let sin_hu = half_u.sin();
            let sin_v = p.v.sin();
            let sin_dv = (2.0 * p.v).sin();

            let r = A + cos_hu * sin_v - sin_hu * sin_dv;
            let point = [
                r * p.u.cos() * scale,
                r * p.u.sin() * scale,
                (sin_hu * sin_v + cos_hu * sin_dv) * scale,
            ];
            let point = math::rotate_x(point, self.angle_x);
            let point = math::rotate_y(point, self.angle_y);
            let point = math::rotate_z(point, self.angle_z);
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

        // Deep emerald/jade green palette — back-to-front
        if !far.is_empty() {
            ctx.draw(&Points {
                coords: &far,
                color: Color::Rgb(15, 60, 40),
            });
        }
        if !mid_far.is_empty() {
            ctx.draw(&Points {
                coords: &mid_far,
                color: Color::Rgb(30, 110, 70),
            });
        }
        if !mid_near.is_empty() {
            ctx.draw(&Points {
                coords: &mid_near,
                color: Color::Rgb(50, 170, 100),
            });
        }
        if !near.is_empty() {
            ctx.draw(&Points {
                coords: &near,
                color: Color::Rgb(90, 220, 140),
            });
        }
    }

    fn name(&self) -> &'static str {
        "klein"
    }

    fn description(&self) -> &'static str {
        "Klein bottle figure-8 immersion"
    }
}
