pub mod diamond;
pub mod geodesic;
pub mod hypercube;
pub mod math;
pub mod toroid;

use ratatui::widgets::canvas::Context;

/// Trait for all uberfetch animations.
pub trait Animation {
    /// Advance the animation state by dt seconds.
    fn update(&mut self, dt: f32);
    /// Render the current frame into a ratatui Canvas context.
    /// `viewport` is `(half_width, half_height)` — the canvas coordinate bounds.
    fn draw(&self, ctx: &mut Context, viewport: (f64, f64));
    /// Human-readable name for CLI selection.
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    /// Short description of the animation.
    #[allow(dead_code)]
    fn description(&self) -> &'static str;
}

/// Returns a list of all available animation names and descriptions.
pub fn list_animations() -> Vec<(&'static str, &'static str)> {
    vec![
        ("diamond", "Rotating pulsating octahedron"),
        ("hypercube", "4D tesseract rotation"),
        ("toroid", "Toroidal particle flow"),
        ("geodesic", "Breathing geodesic sphere"),
    ]
}
