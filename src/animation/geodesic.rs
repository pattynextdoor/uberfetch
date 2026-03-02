use std::collections::{HashMap, HashSet};

use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};

use super::math::{self, DepthRange, Vec3};
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
    raw.into_iter().map(math::normalize).collect()
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
    let mid = math::normalize([
        f64::midpoint(verts[a][0], verts[b][0]),
        f64::midpoint(verts[a][1], verts[b][1]),
        f64::midpoint(verts[a][2], verts[b][2]),
    ]);
    let idx = verts.len();
    verts.push(mid);
    cache.insert(key, idx);
    idx
}

/// Subdivide each triangular face into 4 smaller triangles by adding
/// edge midpoints, then project new vertices onto the unit sphere.
fn subdivide_mesh(
    verts: &mut Vec<Vec3>,
    faces: &[[usize; 3]],
    cache: &mut HashMap<(usize, usize), usize>,
) -> Vec<[usize; 3]> {
    let mut new_faces = Vec::new();
    for face in faces {
        let a = face[0];
        let b = face[1];
        let c = face[2];
        let ab = get_midpoint(a, b, verts, cache);
        let bc = get_midpoint(b, c, verts, cache);
        let ca = get_midpoint(c, a, verts, cache);

        new_faces.push([a, ab, ca]);
        new_faces.push([b, bc, ab]);
        new_faces.push([c, ca, bc]);
        new_faces.push([ab, bc, ca]);
    }
    new_faces
}

/// Build a geodesic sphere by subdividing an icosahedron `levels` times.
/// Returns (vertices, edges).
fn build_geodesic(levels: usize) -> (Vec<Vec3>, Vec<(usize, usize)>) {
    let mut verts = icosahedron_vertices();
    let mut faces = icosahedron_faces();

    for _ in 0..levels {
        let mut cache = HashMap::new();
        faces = subdivide_mesh(&mut verts, &faces, &mut cache);
    }

    // Extract unique edges from final face list
    let mut edge_set = HashSet::new();
    for face in &faces {
        for &(i, j) in &[(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            let edge = if i < j { (i, j) } else { (j, i) };
            edge_set.insert(edge);
        }
    }

    (verts, edge_set.into_iter().collect())
}

/// Breathing geodesic sphere animation.
///
/// Two levels of icosahedron subdivision (~162 vertices, ~480 edges)
/// for a dense, high-fidelity wireframe.
pub struct Geodesic {
    base_vertices: Vec<Vec3>,
    edges: Vec<(usize, usize)>,
    angle_x: f64,
    angle_y: f64,
    time: f64,
}

impl Geodesic {
    pub fn new() -> Self {
        let (verts, edges) = build_geodesic(2);
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
    fn draw(&self, ctx: &mut Context, viewport: (f64, f64)) {
        let (vw, vh) = viewport;
        let base_scale = vw.min(vh) * 0.7;
        let distance = 4.0 * base_scale;

        // Transform each vertex with per-vertex breathing phase.
        // Phase factor is reduced for the denser mesh so ~1 wave cycle
        // spans all vertices, keeping the organic feel without chaos.
        let transformed: Vec<Vec3> = self
            .base_vertices
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let phase = i as f64 * 0.04;
                let breath = 1.0 + 0.08 * (self.time * 2.0 + phase).sin();
                let v = math::scale(v, base_scale * breath);
                let v = math::rotate_x(v, self.angle_x);
                math::rotate_y(v, self.angle_y)
            })
            .collect();

        let projected: Vec<[f64; 2]> = transformed
            .iter()
            .map(|&v| math::project(v, distance))
            .collect();

        let visible: Vec<bool> = projected
            .iter()
            .map(|p| math::is_visible(*p, vw, vh))
            .collect();

        let depth_range = DepthRange::from_z_iter(transformed.iter().map(|v| v[2]));

        for &(i, j) in &self.edges {
            if !visible[i] || !visible[j] {
                continue;
            }

            let avg_z = f64::midpoint(transformed[i][2], transformed[j][2]);
            let depth_brightness = 0.25 + 0.75 * depth_range.normalize(avg_z);

            let avg_phase = (i + j) as f64 * 0.02;
            let shimmer = ((self.time * 1.5 + avg_phase).sin() * 0.15 + 0.85).clamp(0.3, 1.0);
            let brightness = depth_brightness * shimmer;

            let g = (brightness * 255.0) as u8;
            let b = (brightness * 220.0) as u8;
            let color = Color::Rgb((f64::from(g) * 0.5) as u8, g, b);

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
            let b = ((0.4 + 0.6 * depth_range.normalize(transformed[i][2])) * 255.0) as u8;
            ctx.draw(&Points {
                coords: &[(proj[0], proj[1])],
                color: Color::Rgb((f64::from(b) * 0.5) as u8, b, (f64::from(b) * 0.85) as u8),
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
