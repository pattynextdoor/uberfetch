use ratatui::style::Color;
use ratatui::widgets::canvas::Context;

use super::math::{self, Vec3};
use super::Animation;

const VERTICES: [Vec3; 6] = [
    [0.0, 1.0, 0.0],   // top
    [0.0, -1.0, 0.0],  // bottom
    [1.0, 0.0, 0.0],   // right
    [-1.0, 0.0, 0.0],  // left
    [0.0, 0.0, 1.0],   // front
    [0.0, 0.0, -1.0],  // back
];

const EDGES: [(usize, usize); 12] = [
    (0, 2), (0, 3), (0, 4), (0, 5), // top to equator
    (1, 2), (1, 3), (1, 4), (1, 5), // bottom to equator
    (2, 4), (4, 3), (3, 5), (5, 2), // equatorial ring
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

    fn draw(&self, ctx: &mut Context) {
        let pulse = 1.0 + 0.2 * (self.time * 2.5).sin();
        let base_scale = 30.0 * pulse;
        let distance = 4.0 * base_scale;

        let projected: Vec<[f64; 2]> = VERTICES
            .iter()
            .map(|&v| {
                let v = math::scale(v, base_scale);
                let v = math::rotate_x(v, self.angle_x);
                let v = math::rotate_y(v, self.angle_y);
                let v = math::rotate_z(v, self.angle_z);
                math::project(v, distance)
            })
            .collect();

        for (idx, &(i, j)) in EDGES.iter().enumerate() {
            let brightness = ((self.time * 1.5 + idx as f64 * 0.5).sin() * 0.3 + 0.7)
                .clamp(0.3, 1.0);
            let gray = (brightness * 255.0) as u8;
            let color = Color::Rgb(gray, gray, (f64::from(gray) * 0.8) as u8);

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
        "diamond"
    }

    fn description(&self) -> &'static str {
        "Rotating pulsating octahedron"
    }
}
