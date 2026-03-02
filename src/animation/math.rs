/// A 3D point.
pub type Vec3 = [f64; 3];

/// A 2D point (after projection).
pub type Vec2 = [f64; 2];

/// A 4D point.
pub type Vec4 = [f64; 4];

/// Rotate a point around the X axis by `angle` radians.
pub fn rotate_x(p: Vec3, angle: f64) -> Vec3 {
    let (s, c) = angle.sin_cos();
    [p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c]
}

/// Rotate a point around the Y axis by `angle` radians.
pub fn rotate_y(p: Vec3, angle: f64) -> Vec3 {
    let (s, c) = angle.sin_cos();
    [p[0] * c + p[2] * s, p[1], -p[0] * s + p[2] * c]
}

/// Rotate a point around the Z axis by `angle` radians.
pub fn rotate_z(p: Vec3, angle: f64) -> Vec3 {
    let (s, c) = angle.sin_cos();
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c, p[2]]
}

/// Perspective projection from 3D to 2D.
///
/// `distance` is the camera distance from the origin along the Z axis.
pub fn project(p: Vec3, distance: f64) -> Vec2 {
    let z = p[2] + distance;
    if z.abs() < 0.001 {
        return [0.0, 0.0];
    }
    let factor = distance / z;
    [p[0] * factor, p[1] * factor]
}

/// Scale a Vec3 by a scalar.
pub fn scale(p: Vec3, s: f64) -> Vec3 {
    [p[0] * s, p[1] * s, p[2] * s]
}

/// Rotate in the XW plane (4D rotation).
pub fn rotate_xw(p: Vec4, angle: f64) -> Vec4 {
    let (s, c) = angle.sin_cos();
    [p[0] * c - p[3] * s, p[1], p[2], p[0] * s + p[3] * c]
}

/// Rotate in the YZ plane (4D rotation).
pub fn rotate_yz(p: Vec4, angle: f64) -> Vec4 {
    let (s, c) = angle.sin_cos();
    [p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c, p[3]]
}

/// Rotate in the XZ plane (4D rotation).
pub fn rotate_xz(p: Vec4, angle: f64) -> Vec4 {
    let (s, c) = angle.sin_cos();
    [p[0] * c - p[2] * s, p[1], p[0] * s + p[2] * c, p[3]]
}

/// Project from 4D to 3D using perspective projection.
pub fn project_4d_to_3d(p: Vec4, distance: f64) -> Vec3 {
    let w = p[3] + distance;
    if w.abs() < 0.001 {
        return [0.0, 0.0, 0.0];
    }
    let factor = distance / w;
    [p[0] * factor, p[1] * factor, p[2] * factor]
}

/// Normalize a Vec3 to unit length. Returns the zero vector for near-zero inputs.
pub fn normalize(v: Vec3) -> Vec3 {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Check whether a projected 2D point is within the visible viewport.
pub fn is_visible(point: Vec2, half_w: f64, half_h: f64) -> bool {
    point[0].abs() <= half_w && point[1].abs() <= half_h
}

/// Precomputed z-range for normalizing depth values across a set of vertices.
pub struct DepthRange {
    z_min: f64,
    z_range: f64,
}

impl DepthRange {
    /// Build from an explicit min/max pair (useful when tracking z during a loop).
    pub fn new(z_min: f64, z_max: f64) -> Self {
        Self {
            z_min,
            z_range: (z_max - z_min).max(0.001),
        }
    }

    /// Build by scanning an iterator of z values.
    pub fn from_z_iter(iter: impl Iterator<Item = f64>) -> Self {
        let (z_min, z_max) = iter.fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), z| {
            (min.min(z), max.max(z))
        });
        Self::new(z_min, z_max)
    }

    /// Map a z value to 0.0..=1.0 where 1.0 is nearest to the camera.
    pub fn normalize(&self, z: f64) -> f64 {
        1.0 - (z - self.z_min) / self.z_range
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn approx_eq(a: &[f64], b: &[f64], eps: f64) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < eps)
    }

    mod rotate_x {
        use super::*;

        #[test]
        fn rotates_y_axis_to_z_axis_at_90_degrees() {
            let r = rotate_x([0.0, 1.0, 0.0], PI / 2.0);
            assert!(approx_eq(&r, &[0.0, 0.0, 1.0], 1e-10));
        }

        #[test]
        fn leaves_x_component_unchanged() {
            let r = rotate_x([5.0, 1.0, 0.0], PI / 2.0);
            assert!((r[0] - 5.0).abs() < 1e-10);
        }
    }

    mod rotate_y {
        use super::*;

        #[test]
        fn rotates_x_axis_to_negative_z_at_90_degrees() {
            let r = rotate_y([1.0, 0.0, 0.0], PI / 2.0);
            assert!(approx_eq(&r, &[0.0, 0.0, -1.0], 1e-10));
        }
    }

    mod rotate_z {
        use super::*;

        #[test]
        fn rotates_x_axis_to_y_axis_at_90_degrees() {
            let r = rotate_z([1.0, 0.0, 0.0], PI / 2.0);
            assert!(approx_eq(&r, &[0.0, 1.0, 0.0], 1e-10));
        }
    }

    mod project {
        use super::*;

        #[test]
        fn origin_projects_to_origin() {
            let r = project([0.0, 0.0, 0.0], 5.0);
            assert!(approx_eq(&r, &[0.0, 0.0], 1e-10));
        }

        #[test]
        fn point_on_projection_plane_is_unchanged() {
            let r = project([1.0, 0.0, 0.0], 5.0);
            assert!(approx_eq(&r, &[1.0, 0.0], 1e-10));
        }

        #[test]
        fn point_behind_camera_returns_origin() {
            let r = project([1.0, 1.0, -5.0], 5.0);
            assert!(approx_eq(&r, &[0.0, 0.0], 1e-10));
        }
    }

    mod scale_tests {
        use super::*;

        #[test]
        fn scales_all_components() {
            let r = scale([1.0, 2.0, 3.0], 2.0);
            assert!(approx_eq(&r, &[2.0, 4.0, 6.0], 1e-10));
        }
    }

    mod rotate_4d {
        use super::*;

        #[test]
        fn xw_rotation_at_zero_is_identity() {
            let p = [1.0, 2.0, 3.0, 4.0];
            let r = rotate_xw(p, 0.0);
            assert!(approx_eq(&r, &p, 1e-10));
        }

        #[test]
        fn yz_rotation_at_zero_is_identity() {
            let p = [1.0, 2.0, 3.0, 4.0];
            let r = rotate_yz(p, 0.0);
            assert!(approx_eq(&r, &p, 1e-10));
        }
    }

    mod project_4d {
        use super::*;

        #[test]
        fn projects_point_at_w_zero_with_unit_factor() {
            let r = project_4d_to_3d([1.0, 2.0, 3.0, 0.0], 5.0);
            assert!(approx_eq(&r, &[1.0, 2.0, 3.0], 1e-10));
        }
    }
}
