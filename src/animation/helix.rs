use std::f64::consts::{PI, TAU};

use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math::{self, DepthRange};
use super::Animation;

const PARTICLES_PER_STRAND: usize = 400;
const NUM_STRANDS: usize = 2;
const TOTAL_PARTICLES: usize = PARTICLES_PER_STRAND * NUM_STRANDS;

/// Number of full helix turns visible at once.
const TURNS: f64 = 4.0;

/// Cyan strand depth palette: dark → bright.
const CYAN: [(u8, u8, u8); 4] = [
    (20, 80, 100),
    (40, 140, 180),
    (60, 190, 230),
    (80, 220, 255),
];

/// Magenta strand depth palette: dark → bright.
const MAGENTA: [(u8, u8, u8); 4] = [
    (100, 20, 80),
    (180, 40, 140),
    (230, 60, 190),
    (255, 80, 220),
];

/// Vertical travel speed — one full height traversal in ~4 seconds.
const FLOW_SPEED: f64 = 0.25;

struct Particle {
    /// Normalized position along the helix height [0, 1).
    t: f64,
    /// Which strand this particle belongs to (0 or 1).
    strand: usize,
}

/// Double helix particle animation.
///
/// Two intertwined helices with particles flowing upward, rendered as
/// braille dots with per-strand coloring and depth-based brightness.
pub struct Helix {
    particles: Vec<Particle>,
    angle_y: f64,
    time: f64,
}

impl Helix {
    #[expect(clippy::cast_precision_loss, reason = "animation indices are small")]
    pub fn new() -> Self {
        let particles = (0..TOTAL_PARTICLES)
            .map(|i| {
                let strand = i / PARTICLES_PER_STRAND;
                let idx = i % PARTICLES_PER_STRAND;
                let t = idx as f64 / PARTICLES_PER_STRAND as f64;
                Particle { t, strand }
            })
            .collect();
        Self {
            particles,
            angle_y: 0.0,
            time: 0.0,
        }
    }
}

impl Animation for Helix {
    fn update(&mut self, dt: f32) {
        let dt64 = f64::from(dt);
        self.angle_y += 0.5 * dt64;
        self.time += dt64;

        for p in &mut self.particles {
            p.t = (p.t + FLOW_SPEED * dt64) % 1.0;
        }
    }

    fn draw(&self, ctx: &mut Context, viewport: (f64, f64)) {
        let (vw, vh) = viewport;
        let vmin = vw.min(vh);

        let radius = vmin * 0.3;
        let half_height = vmin * 0.7;
        let distance = 4.0 * vmin;

        // Gentle tilt oscillation so it's not always perfectly vertical
        let tilt_x = 0.3 + 0.15 * (self.time * 0.6).sin();

        let mut data = Vec::with_capacity(TOTAL_PARTICLES);
        let mut z_min = f64::INFINITY;
        let mut z_max = f64::NEG_INFINITY;

        for p in &self.particles {
            // Phase offset: strand 1 is shifted by π for the classic double-helix look
            let phase = if p.strand == 1 { PI } else { 0.0 };
            let theta = p.t * TURNS * TAU + phase;
            let y = (p.t - 0.5) * 2.0 * half_height;

            let point = [radius * theta.cos(), y, radius * theta.sin()];
            let point = math::rotate_x(point, tilt_x);
            let point = math::rotate_y(point, self.angle_y);
            let z = point[2];
            z_min = z_min.min(z);
            z_max = z_max.max(z);
            let proj = math::project(point, distance);
            data.push((proj, z, p.strand));
        }
        let depth_range = DepthRange::new(z_min, z_max);

        // Each strand has its own 4-layer depth bucket for distinct coloring.
        // Strand 0: cyan, Strand 1: magenta
        let mut buckets: [Vec<(f64, f64)>; 8] = Default::default();

        for &(proj, z, strand) in &data {
            if !math::is_visible(proj, vw, vh) {
                continue;
            }
            let depth = depth_range.normalize(z);
            let coord = (proj[0], proj[1]);

            // Map depth to layer index: 0 (far) .. 3 (near)
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "depth is clamped to 0.0..=1.0; multiplied by 3.0 fits in usize"
            )]
            let layer = (depth * 3.99).min(3.0) as usize;
            buckets[strand * 4 + layer].push(coord);
        }

        // Draw back-to-front across both strands, interleaving by depth layer
        for layer in 0..4 {
            for (strand, palette) in [&CYAN, &MAGENTA].iter().enumerate() {
                let bucket = &buckets[strand * 4 + layer];
                if !bucket.is_empty() {
                    let (r, g, b) = palette[layer];
                    ctx.draw(&Points {
                        coords: bucket,
                        color: Color::Rgb(r, g, b),
                    });
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "helix"
    }

    fn description(&self) -> &'static str {
        "Double helix particle flow"
    }
}
