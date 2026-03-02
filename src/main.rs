mod animation;
mod renderer;
mod sysinfo;

use std::io;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use tachyonfx::{fx, Shader};

use animation::diamond::Diamond;
use animation::Animation;

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

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.list {
        println!("Available animations:");
        for (name, desc) in animation::list_animations() {
            println!("  {name:<12} {desc}");
        }
        return Ok(());
    }

    let mut anim: Box<dyn Animation> = match cli.animation {
        AnimationChoice::Diamond => Box::new(Diamond::new()),
        AnimationChoice::Hypercube => Box::new(animation::hypercube::Hypercube::new()),
        AnimationChoice::Toroid => Box::new(animation::toroid::Toroid::new()),
        AnimationChoice::Geodesic => Box::new(animation::geodesic::Geodesic::new()),
    };

    let info = sysinfo::SystemInfo::collect();
    let terminal = ratatui::init();
    let result = run(terminal, anim.as_mut(), &info, cli.fps);
    ratatui::restore();
    result
}

fn run(
    mut terminal: DefaultTerminal,
    animation: &mut dyn Animation,
    info: &sysinfo::SystemInfo,
    fps: u32,
) -> io::Result<()> {
    let tick_rate = Duration::from_secs_f64(1.0 / f64::from(fps));
    let mut last_tick = Instant::now();

    // A subtle, never-ending HSL color shift applied as a post-processing
    // effect over the animation panel.  `ping_pong` plays the shift forward
    // then backward so the hue oscillates smoothly, and `never_complete`
    // keeps it looping forever.
    let mut glow = fx::never_complete(fx::ping_pong(fx::hsl_shift_fg(
        [30.0, 0.0, 0.05],
        3000u32, // 3 s per half-cycle, linear interpolation
    )));

    loop {
        // Capture elapsed time *before* drawing so the effect can advance by
        // the real wall-clock delta since the last frame.
        let elapsed = last_tick.elapsed();

        terminal.draw(|frame| {
            renderer::layout::draw(frame, animation, info);

            // Apply the glow effect to just the animation panel region.
            let anim_area = renderer::layout::animation_rect(frame.area());
            glow.process(elapsed.into(), frame.buffer_mut(), anim_area);
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let dt = last_tick.elapsed().as_secs_f32();
            animation.update(dt);
            last_tick = Instant::now();
        }
    }
}
