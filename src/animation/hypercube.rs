use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math::{self, DepthRange, Vec3, Vec4};
use super::Animation;

/// Generate the 16 vertices of a unit tesseract centered at the origin.
///
/// Each vertex is a combination of ±1 in four dimensions. We use the
/// bit pattern of the index (0..16) to pick the sign for each axis:
/// bit 0 → x, bit 1 → y, bit 2 → z, bit 3 → w.
fn tesseract_vertices() -> [Vec4; 16] {
    let mut verts = [[0.0; 4]; 16];
    for (i, vert) in verts.iter_mut().enumerate() {
        *vert = [
            if i & 1 != 0 { 1.0 } else { -1.0 },
            if i & 2 != 0 { 1.0 } else { -1.0 },
            if i & 4 != 0 { 1.0 } else { -1.0 },
            if i & 8 != 0 { 1.0 } else { -1.0 },
        ];
    }
    verts
}

/// The 32 edges of a tesseract.
///
/// Two vertices are connected if and only if they differ in exactly one
/// coordinate — i.e. their XOR has exactly one bit set.
const EDGES: [(usize, usize); 32] = [
    // x-edges (bit 0)
    (0, 1),
    (2, 3),
    (4, 5),
    (6, 7),
    (8, 9),
    (10, 11),
    (12, 13),
    (14, 15),
    // y-edges (bit 1)
    (0, 2),
    (1, 3),
    (4, 6),
    (5, 7),
    (8, 10),
    (9, 11),
    (12, 14),
    (13, 15),
    // z-edges (bit 2)
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
    (8, 12),
    (9, 13),
    (10, 14),
    (11, 15),
    // w-edges (bit 3)
    (0, 8),
    (1, 9),
    (2, 10),
    (3, 11),
    (4, 12),
    (5, 13),
    (6, 14),
    (7, 15),
];

/// 4D tesseract (hypercube) rotating through hyperplanes.
///
/// The tesseract is simultaneously rotated in three 4D planes (XW, YZ, XZ)
/// at different speeds, then double-projected (4D → 3D → 2D) for display.
/// Edges are colour-coded by the dimension they span: blue for X, pink for Y,
/// green for Z, and gold for W.
pub struct Hypercube {
    angle_xw: f64,
    angle_yz: f64,
    angle_xz: f64,
    vertices: [Vec4; 16],
}

impl Hypercube {
    pub fn new() -> Self {
        Self {
            angle_xw: 0.0,
            angle_yz: 0.0,
            angle_xz: 0.0,
            vertices: tesseract_vertices(),
        }
    }
}

impl Animation for Hypercube {
    fn update(&mut self, dt: f32) {
        let dt = f64::from(dt);
        self.angle_xw += 0.6 * dt;
        self.angle_yz += 0.4 * dt;
        self.angle_xz += 0.3 * dt;
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "color values are clamped to 0-255"
    )]
    fn draw(&self, ctx: &mut Context, viewport: (f64, f64)) {
        let (vw, vh) = viewport;
        let base_scale = vw.min(vh) * 0.55;
        let distance_4d = 3.0;
        let distance_3d = 4.0;

        let transformed_3d: [Vec3; 16] = self.vertices.map(|v| {
            let v = [
                v[0] * base_scale,
                v[1] * base_scale,
                v[2] * base_scale,
                v[3] * base_scale,
            ];
            let v = math::rotate_xw(v, self.angle_xw);
            let v = math::rotate_yz(v, self.angle_yz);
            let v = math::rotate_xz(v, self.angle_xz);
            math::project_4d_to_3d(v, distance_4d * base_scale)
        });

        let projected = transformed_3d.map(|v| math::project(v, distance_3d * base_scale));
        let visible = projected.map(|p| math::is_visible(p, vw, vh));
        let depth_range = DepthRange::from_z_iter(transformed_3d.iter().map(|v| v[2]));

        for &(i, j) in &EDGES {
            if !visible[i] || !visible[j] {
                continue;
            }

            // Color based on which dimension the edge spans
            let diff = i ^ j;
            let base_color: (f64, f64, f64) = match diff {
                1 => (100.0, 180.0, 255.0), // x-edges: blue
                2 => (255.0, 100.0, 180.0), // y-edges: pink
                4 => (100.0, 255.0, 180.0), // z-edges: green
                8 => (255.0, 220.0, 100.0), // w-edges: gold
                _ => (255.0, 255.0, 255.0),
            };

            // Modulate color brightness by depth (closer = brighter)
            let avg_z = f64::midpoint(transformed_3d[i][2], transformed_3d[j][2]);
            let brightness = 0.3 + 0.7 * depth_range.normalize(avg_z);

            let color = Color::Rgb(
                (base_color.0 * brightness) as u8,
                (base_color.1 * brightness) as u8,
                (base_color.2 * brightness) as u8,
            );

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
            let b = ((0.4 + 0.6 * depth_range.normalize(transformed_3d[i][2])) * 255.0) as u8;
            ctx.draw(&Points {
                coords: &[(proj[0], proj[1])],
                color: Color::Rgb(b, b, b),
            });
        }
    }

    fn name(&self) -> &'static str {
        "hypercube"
    }

    fn description(&self) -> &'static str {
        "4D tesseract rotation"
    }
}
