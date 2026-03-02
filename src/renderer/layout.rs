use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::animation::Animation;
use crate::renderer::color;
use crate::sysinfo::SystemInfo;

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;

/// Draw the full uberfetch display: animation (left) + system info (right).
/// Returns the animation panel `Rect` so callers can apply post-processing effects.
pub fn draw(frame: &mut Frame, animation: &dyn Animation, info: &SystemInfo) -> Option<Rect> {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let msg = Paragraph::new("Terminal too small (need 80x24)")
            .style(Style::default().fg(ratatui::style::Color::Red));
        frame.render_widget(msg, area);
        return None;
    }

    let chunks = split_layout(area);
    draw_animation(frame, animation, chunks[0]);
    draw_info(frame, info, chunks[1]);
    Some(chunks[0])
}

fn split_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).split(area)
}

fn draw_animation(frame: &mut Frame, animation: &dyn Animation, area: Rect) {
    let half_w = f64::from(area.width);
    let half_h = f64::from(area.height);
    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .x_bounds([-half_w, half_w])
        .y_bounds([-half_h, half_h])
        .paint(|ctx| {
            animation.draw(ctx, (half_w, half_h));
        });
    frame.render_widget(canvas, area);
}

fn draw_info(frame: &mut Frame, info: &SystemInfo, area: Rect) {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "user".into());
    let title = format!("{username}@{}", info.hostname);
    let separator = "\u{2500}".repeat(title.len());

    let fields = [
        ("OS", info.os.as_str()),
        ("Kernel", info.kernel.as_str()),
        ("Uptime", info.uptime.as_str()),
        ("CPU", info.cpu.as_str()),
        ("Memory", info.memory.as_str()),
        ("Shell", info.shell.as_str()),
        ("Terminal", info.terminal.as_str()),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        title,
        Style::default().fg(color::LABEL_COLOR).bold(),
    )));
    lines.push(Line::from(Span::styled(
        separator,
        Style::default().fg(color::SEPARATOR_COLOR),
    )));

    for (label, value) in &fields {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{label}: "),
                Style::default().fg(color::LABEL_COLOR).bold(),
            ),
            Span::raw(*value),
        ]));
    }

    lines.push(Line::from(""));

    // Color palette strip — two rows of 8 colored blocks
    let palette_top: Vec<Span> = color::PALETTE[..8]
        .iter()
        .map(|&c| Span::styled("\u{2588}\u{2588}\u{2588}", Style::default().fg(c)))
        .collect();
    let palette_bottom: Vec<Span> = color::PALETTE[8..]
        .iter()
        .map(|&c| Span::styled("\u{2588}\u{2588}\u{2588}", Style::default().fg(c)))
        .collect();
    lines.push(Line::from(palette_top));
    lines.push(Line::from(palette_bottom));

    // Vertically center
    #[expect(
        clippy::cast_possible_truncation,
        reason = "info lines always fit in u16"
    )]
    let content_height = lines.len() as u16;
    let top_padding = area.height.saturating_sub(content_height) / 2;
    let mut centered_lines: Vec<Line> = vec![Line::from(""); top_padding as usize];
    centered_lines.extend(lines);

    let paragraph = Paragraph::new(centered_lines);
    frame.render_widget(paragraph, area);
}
