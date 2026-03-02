use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math::{self, DepthRange, Vec3};
use super::Animation;

const VERTICES: [Vec3; 6] = [
    [0.0, 1.0, 0.0],  // top
    [0.0, -1.0, 0.0], // bottom
    [1.0, 0.0, 0.0],  // right
    [-1.0, 0.0, 0.0], // left
    [0.0, 0.0, 1.0],  // front
    [0.0, 0.0, -1.0], // back
];

const EDGES: [(usize, usize); 12] = [
    (0, 2),
    (0, 3),
    (0, 4),
    (0, 5), // top to equator
    (1, 2),
    (1, 3),
    (1, 4),
    (1, 5), // bottom to equator
    (2, 4),
    (4, 3),
    (3, 5),
    (5, 2), // equatorial ring
];

/// Rotating pulsating octahedron animation.
pub struct Diamond {
    angle_x: f64,
    angle_y: f64,
    angle_z: f64,
    time: f64,
}

impl Diamond {
    pub fn new() -> Self {
        Self {
            angle_x: 0.0,
            angle_y: 0.0,
            angle_z: 0.0,
            time: 0.0,
        }
    }
}

impl Animation for Diamond {
    fn update(&mut self, dt: f32) {
        let dt = f64::from(dt);
        self.time += dt;
        self.angle_x += 0.8 * dt;
        self.angle_y += 1.2 * dt;
        self.angle_z += 0.3 * dt;
    }

    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "animation indices are small; color values are clamped to 0-255"
    )]
    fn draw(&self, ctx: &mut Context, viewport: (f64, f64)) {
        let (vw, vh) = viewport;
        let pulse = 1.0 + 0.15 * (self.time * 2.5).sin();
        // Scale relative to viewport so the shape fills available space
        let base_scale = vw.min(vh) * 0.7 * pulse;
        let distance = 4.0 * base_scale;

        let transformed: [Vec3; 6] = VERTICES.map(|v| {
            let v = math::scale(v, base_scale);
            let v = math::rotate_x(v, self.angle_x);
            let v = math::rotate_y(v, self.angle_y);
            math::rotate_z(v, self.angle_z)
        });

        let projected = transformed.map(|v| math::project(v, distance));
        let visible = projected.map(|p| math::is_visible(p, vw, vh));
        let depth_range = DepthRange::from_z_iter(transformed.iter().map(|v| v[2]));

        for (idx, &(i, j)) in EDGES.iter().enumerate() {
            if !visible[i] || !visible[j] {
                continue;
            }

            let avg_z = f64::midpoint(transformed[i][2], transformed[j][2]);
            let depth_brightness = 0.35 + 0.65 * depth_range.normalize(avg_z);

            let shimmer =
                ((self.time * 1.5 + idx as f64 * 0.5).sin() * 0.15 + 0.85).clamp(0.4, 1.0);
            let brightness = depth_brightness * shimmer;
            let gray = (brightness * 255.0) as u8;
            let color = Color::Rgb(gray, gray, (f64::from(gray) * 0.85) as u8);

            ctx.draw(&ratatui::widgets::canvas::Line {
                x1: projected[i][0],
                y1: projected[i][1],
                x2: projected[j][0],
                y2: projected[j][1],
                color,
            });
        }

        for (i, (&vis, proj)) in visible.iter().zip(projected.iter()).enumerate() {
            if !vis {
                continue;
            }
            let b = ((0.5 + 0.5 * depth_range.normalize(transformed[i][2])) * 255.0) as u8;
            ctx.draw(&Points {
                coords: &[(proj[0], proj[1])],
                color: Color::Rgb(b, b, (f64::from(b) * 0.9) as u8),
            });
        }
    }

    fn name(&self) -> &'static str {
        "diamond"
    }

    fn description(&self) -> &'static str {
        "Rotating pulsating octahedron"
    }
}
