use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math::{self, Vec3, Vec4};
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

/// Generate the 32 edges of a tesseract.
///
/// Two vertices are connected if and only if they differ in exactly one
/// coordinate — i.e. their XOR has exactly one bit set.
fn tesseract_edges() -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for i in 0u32..16 {
        for j in (i + 1)..16 {
            if (i ^ j).is_power_of_two() {
                edges.push((i as usize, j as usize));
            }
        }
    }
    edges
}

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
    edges: Vec<(usize, usize)>,
}

impl Hypercube {
    pub fn new() -> Self {
        Self {
            angle_xw: 0.0,
            angle_yz: 0.0,
            angle_xz: 0.0,
            vertices: tesseract_vertices(),
            edges: tesseract_edges(),
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

        // Transform 4D → 3D, keeping the intermediate 3D result for depth
        let transformed_3d: Vec<Vec3> = self
            .vertices
            .iter()
            .map(|v| {
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
            })
            .collect();

        // Project 3D → 2D
        let projected: Vec<[f64; 2]> = transformed_3d
            .iter()
            .map(|&v| math::project(v, distance_3d * base_scale))
            .collect();

        let visible: Vec<bool> = projected
            .iter()
            .map(|p| math::is_visible(*p, vw, vh))
            .collect();

        // Z-range for depth-based brightness
        let (z_min, z_max) = transformed_3d
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), v| {
                (min.min(v[2]), max.max(v[2]))
            });
        let z_range = (z_max - z_min).max(0.001);

        for &(i, j) in &self.edges {
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
            let depth = 1.0 - (avg_z - z_min) / z_range;
            let brightness = 0.3 + 0.7 * depth;

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

        // Vertex dots with depth-based brightness
        for (i, (&vis, proj)) in visible.iter().zip(projected.iter()).enumerate() {
            if !vis {
                continue;
            }
            let depth = 1.0 - (transformed_3d[i][2] - z_min) / z_range;
            let b = ((0.4 + 0.6 * depth) * 255.0) as u8;
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
