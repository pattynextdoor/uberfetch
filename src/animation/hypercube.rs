use ratatui::style::Color;
use ratatui::widgets::canvas::Context;

use super::math::{self, Vec4};
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

    fn draw(&self, ctx: &mut Context) {
        let distance_4d = 3.0;
        let distance_3d = 4.0;
        let base_scale = 20.0;

        let projected: Vec<[f64; 2]> = self
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
                let v3 = math::project_4d_to_3d(v, distance_4d * base_scale);
                math::project(v3, distance_3d * base_scale)
            })
            .collect();

        for &(i, j) in &self.edges {
            // Color based on which dimension the edge spans.
            // The XOR of two adjacent vertex indices has exactly one bit set,
            // telling us which axis (x=1, y=2, z=4, w=8) the edge runs along.
            let diff = i ^ j;
            let color = match diff {
                1 => Color::Rgb(100, 180, 255), // x-edges: blue
                2 => Color::Rgb(255, 100, 180), // y-edges: pink
                4 => Color::Rgb(100, 255, 180), // z-edges: green
                8 => Color::Rgb(255, 220, 100), // w-edges: gold
                _ => Color::White,
            };

            ctx.draw(&ratatui::widgets::canvas::Line {
                x1: projected[i][0],
                y1: projected[i][1],
                x2: projected[j][0],
                y2: projected[j][1],
                color,
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
