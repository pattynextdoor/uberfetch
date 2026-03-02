use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math::{self, DepthRange};
use super::Animation;

const NUM_PARTICLES: usize = 1200;

/// Lorenz system parameters.
const SIGMA: f64 = 10.0;
const RHO: f64 = 28.0;
const BETA: f64 = 8.0 / 3.0;

/// The attractor's natural z-center is around 25; offset so it orbits the origin.
const Z_CENTER: f64 = 25.0;

/// Integration timestep — small enough for numerical stability.
const DT_STEP: f64 = 0.005;

struct Particle {
    x: f64,
    y: f64,
    z: f64,
}

/// Number of Euler steps each successive particle is pre-simulated beyond the
/// previous one.  With 1200 particles × 20 steps × 0.005 dt ≈ 120 s of
/// simulated spread, which fills both lobes of the attractor.
const WARMUP_STEPS_PER_PARTICLE: usize = 20;

/// Chaotic Lorenz attractor particle animation.
///
/// ~1200 particles independently trace the classic butterfly-shaped strange
/// attractor, rendered as braille dots with warm-to-cool depth coloring.
pub struct Lorenz {
    particles: Vec<Particle>,
    angle_x: f64,
    angle_y: f64,
}

impl Lorenz {
    #[expect(clippy::cast_precision_loss, reason = "animation indices are small")]
    pub fn new() -> Self {
        let mut particles: Vec<Particle> = (0..NUM_PARTICLES)
            .map(|i| {
                let frac = i as f64 / NUM_PARTICLES as f64;
                let angle = frac * std::f64::consts::TAU;
                Particle {
                    x: 1.0 + 0.1 * angle.cos(),
                    y: 1.0 + 0.1 * angle.sin(),
                    z: 1.0 + 0.05 * frac,
                }
            })
            .collect();

        // Pre-simulate so particles are already spread across the attractor.
        // Each particle runs progressively more steps than the last, producing
        // a trail-like distribution across both lobes.
        for (i, p) in particles.iter_mut().enumerate() {
            for _ in 0..i * WARMUP_STEPS_PER_PARTICLE {
                let dx = SIGMA * (p.y - p.x);
                let dy = p.x * (RHO - p.z) - p.y;
                let dz = p.x * p.y - BETA * p.z;
                p.x += dx * DT_STEP;
                p.y += dy * DT_STEP;
                p.z += dz * DT_STEP;
            }
        }

        Self {
            particles,
            angle_x: 0.3,
            angle_y: 0.0,
        }
    }
}

impl Animation for Lorenz {
    fn update(&mut self, dt: f32) {
        let dt64 = f64::from(dt);
        self.angle_y += 0.3 * dt64;
        self.angle_x = 0.3 + 0.15 * (self.angle_y * 0.4).sin();

        // Each particle takes multiple small Euler steps per frame for stability.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "dt/DT_STEP is always a small positive value; fits in usize"
        )]
        let steps = ((dt64 / DT_STEP).ceil() as usize).max(1);
        #[expect(
            clippy::cast_precision_loss,
            reason = "step count is always small enough for exact f64 representation"
        )]
        let h = dt64 / steps as f64;

        for p in &mut self.particles {
            for _ in 0..steps {
                let dx = SIGMA * (p.y - p.x);
                let dy = p.x * (RHO - p.z) - p.y;
                let dz = p.x * p.y - BETA * p.z;
                p.x += dx * h;
                p.y += dy * h;
                p.z += dz * h;
            }
        }
    }

    fn draw(&self, ctx: &mut Context, viewport: (f64, f64)) {
        let (vw, vh) = viewport;
        let vmin = vw.min(vh);

        // The attractor spans roughly ±20 in x/y and 0..50 in z.
        // Scale so the butterfly fills the viewport nicely.
        let scale = vmin / 25.0;
        let distance = 4.0 * vmin;

        let mut data = Vec::with_capacity(NUM_PARTICLES);
        let mut z_min = f64::INFINITY;
        let mut z_max = f64::NEG_INFINITY;

        for p in &self.particles {
            let point = [p.x * scale, (p.z - Z_CENTER) * scale, p.y * scale];
            let point = math::rotate_x(point, self.angle_x);
            let point = math::rotate_y(point, self.angle_y);
            let z = point[2];
            z_min = z_min.min(z);
            z_max = z_max.max(z);
            let proj = math::project(point, distance);
            data.push((proj, z));
        }
        let depth_range = DepthRange::new(z_min, z_max);

        // Four depth layers: warm (near) → cool (far)
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

        // Draw back-to-front: deep blue (far) → amber (near)
        if !far.is_empty() {
            ctx.draw(&Points {
                coords: &far,
                color: Color::Rgb(40, 70, 160),
            });
        }
        if !mid_far.is_empty() {
            ctx.draw(&Points {
                coords: &mid_far,
                color: Color::Rgb(100, 120, 200),
            });
        }
        if !mid_near.is_empty() {
            ctx.draw(&Points {
                coords: &mid_near,
                color: Color::Rgb(200, 150, 100),
            });
        }
        if !near.is_empty() {
            ctx.draw(&Points {
                coords: &near,
                color: Color::Rgb(255, 180, 80),
            });
        }
    }

    fn name(&self) -> &'static str {
        "lorenz"
    }

    fn description(&self) -> &'static str {
        "Chaotic Lorenz attractor"
    }
}
