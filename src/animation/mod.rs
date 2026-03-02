pub mod math;
mod diamond;

use ratatui::widgets::canvas::Context;

/// Trait for all uberfetch animations.
pub trait Animation {
    /// Advance the animation state by dt seconds.
    fn update(&mut self, dt: f32);
    /// Render the current frame into a ratatui Canvas context.
    fn draw(&self, ctx: &mut Context);
    /// Human-readable name for CLI selection.
    fn name(&self) -> &'static str;
    /// Short description of the animation.
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
