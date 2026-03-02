mod animation;
mod renderer;
mod sysinfo;

use clap::{Parser, ValueEnum};

/// Neofetch alternative with esoteric terminal animations.
#[derive(Parser)]
#[command(name = "uberfetch", version, about)]
struct Cli {
    /// Animation to display.
    #[arg(short, long, value_enum, default_value_t = AnimationChoice::Diamond)]
    animation: AnimationChoice,

    /// Target frames per second.
    #[arg(short, long, default_value_t = 30)]
    fps: u32,

    /// List available animations and exit.
    #[arg(short, long)]
    list: bool,
}

/// Available animation choices.
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum AnimationChoice {
    Diamond,
    Hypercube,
    Toroid,
    Geodesic,
}

fn main() {
    let cli = Cli::parse();

    if cli.list {
        println!("Available animations:");
        for (name, desc) in animation::list_animations() {
            println!("  {name:<12} {desc}");
        }
        return;
    }

    println!("Animation: {:?}", cli.animation as u8);
    println!("FPS: {}", cli.fps);
}
