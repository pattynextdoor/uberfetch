pub mod diamond;
pub mod geodesic;
pub mod helix;
pub mod hypercube;
pub mod klein;
pub mod lorenz;
pub mod math;
pub mod mobius;
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
    fn name(&self) -> &'static str;
    /// Short description of the animation.
    fn description(&self) -> &'static str;
}

/// Returns a list of all available animation names and descriptions.
///
/// Built from actual trait implementations so names/descriptions stay in sync.
pub fn list_animations() -> Vec<(&'static str, &'static str)> {
    let all: Vec<Box<dyn Animation>> = vec![
        Box::new(diamond::Diamond::new()),
        Box::new(hypercube::Hypercube::new()),
        Box::new(toroid::Toroid::new()),
        Box::new(geodesic::Geodesic::new()),
        Box::new(lorenz::Lorenz::new()),
        Box::new(helix::Helix::new()),
        Box::new(mobius::Mobius::new()),
        Box::new(klein::Klein::new()),
    ];
    all.iter().map(|a| (a.name(), a.description())).collect()
}
