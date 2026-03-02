use ratatui::style::Color;

/// The 16 standard ANSI colors for the palette strip.
pub const PALETTE: [Color; 16] = [
    Color::Indexed(0),
    Color::Indexed(1),
    Color::Indexed(2),
    Color::Indexed(3),
    Color::Indexed(4),
    Color::Indexed(5),
    Color::Indexed(6),
    Color::Indexed(7),
    Color::Indexed(8),
    Color::Indexed(9),
    Color::Indexed(10),
    Color::Indexed(11),
    Color::Indexed(12),
    Color::Indexed(13),
    Color::Indexed(14),
    Color::Indexed(15),
];

/// Accent color for labels.
pub const LABEL_COLOR: Color = Color::Indexed(4);

/// Separator line color.
pub const SEPARATOR_COLOR: Color = Color::Indexed(8);
