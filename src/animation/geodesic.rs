use std::collections::{HashMap, HashSet};

use ratatui::style::Color;
use ratatui::widgets::canvas::Context;

use super::math::{self, Vec3};
use super::Animation;

fn icosahedron_vertices() -> Vec<Vec3> {
    let phi = f64::midpoint(1.0, 5.0_f64.sqrt());
    let raw = vec![
        [-1.0, phi, 0.0],
        [1.0, phi, 0.0],
        [-1.0, -phi, 0.0],
        [1.0, -phi, 0.0],
        [0.0, -1.0, phi],
        [0.0, 1.0, phi],
        [0.0, -1.0, -phi],
        [0.0, 1.0, -phi],
        [phi, 0.0, -1.0],
        [phi, 0.0, 1.0],
        [-phi, 0.0, -1.0],
        [-phi, 0.0, 1.0],
    ];
    raw.into_iter()
        .map(|v| {
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            [v[0] / len, v[1] / len, v[2] / len]
        })
        .collect()
}

fn icosahedron_faces() -> Vec<[usize; 3]> {
    vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ]
}

fn subdivide_icosahedron() -> (Vec<Vec3>, Vec<(usize, usize)>) {
    let base_verts = icosahedron_vertices();
    let faces = icosahedron_faces();

    let mut verts = base_verts;
    let mut edge_set = HashSet::new();
    let mut midpoint_cache = HashMap::new();

    for face in &faces {
        let a = face[0];
        let b = face[1];
        let c = face[2];
        let ab = get_midpoint(a, b, &mut verts, &mut midpoint_cache);
        let bc = get_midpoint(b, c, &mut verts, &mut midpoint_cache);
        let ca = get_midpoint(c, a, &mut verts, &mut midpoint_cache);

        for &(i, j) in &[
            (a, ab),
            (ab, b),
            (b, bc),
            (bc, c),
            (c, ca),
            (ca, a),
            (ab, bc),
            (bc, ca),
            (ca, ab),
        ] {
            let edge = if i < j { (i, j) } else { (j, i) };
            edge_set.insert(edge);
        }
    }

    let edges: Vec<(usize, usize)> = edge_set.into_iter().collect();
    (verts, edges)
}

fn get_midpoint(
    a: usize,
    b: usize,
    verts: &mut Vec<Vec3>,
    cache: &mut HashMap<(usize, usize), usize>,
) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }
    let mid = [
        f64::midpoint(verts[a][0], verts[b][0]),
        f64::midpoint(verts[a][1], verts[b][1]),
        f64::midpoint(verts[a][2], verts[b][2]),
    ];
    let len = (mid[0] * mid[0] + mid[1] * mid[1] + mid[2] * mid[2]).sqrt();
    let normalized = [mid[0] / len, mid[1] / len, mid[2] / len];
    let idx = verts.len();
    verts.push(normalized);
    cache.insert(key, idx);
    idx
}

/// Breathing geodesic sphere animation.
pub struct Geodesic {
    base_vertices: Vec<Vec3>,
    edges: Vec<(usize, usize)>,
    angle_x: f64,
    angle_y: f64,
    time: f64,
}

impl Geodesic {
    pub fn new() -> Self {
        let (verts, edges) = subdivide_icosahedron();
        Self {
            base_vertices: verts,
            edges,
            angle_x: 0.0,
            angle_y: 0.0,
            time: 0.0,
        }
    }
}

impl Animation for Geodesic {
    fn update(&mut self, dt: f32) {
        let dt = f64::from(dt);
        self.time += dt;
        self.angle_x += 0.3 * dt;
        self.angle_y += 0.5 * dt;
    }

    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "animation indices are small; color values are clamped to 0-255"
    )]
    fn draw(&self, ctx: &mut Context) {
        let base_scale = 25.0;
        let distance = 4.0 * base_scale;

        let projected: Vec<[f64; 2]> = self
            .base_vertices
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let phase = i as f64 * 0.15;
                let breath = 1.0 + 0.15 * (self.time * 2.0 + phase).sin();
                let v = math::scale(v, base_scale * breath);
                let v = math::rotate_x(v, self.angle_x);
                let v = math::rotate_y(v, self.angle_y);
                math::project(v, distance)
            })
            .collect();

        for &(i, j) in &self.edges {
            let avg_phase = (i + j) as f64 * 0.075;
            let brightness = ((self.time * 1.5 + avg_phase).sin() * 0.3 + 0.7).clamp(0.3, 1.0);
            let g = (brightness * 255.0) as u8;
            let b = (brightness * 200.0) as u8;
            let color = Color::Rgb((f64::from(g) * 0.6) as u8, g, b);

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
        "geodesic"
    }

    fn description(&self) -> &'static str {
        "Breathing geodesic sphere"
    }
}
