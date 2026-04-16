use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::types::{App, GameState, PendingInput, Tile, VimMotion, Zone, TOTAL_LEVELS};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    if !app.started {
        render_title(frame);
        return;
    }

    if app.game_state == GameState::Won {
        render_win(frame, app);
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(frame.area());

    render_map(frame, app, layout[0]);
    render_sidebar(frame, app, layout[1]);
}

fn render_title(frame: &mut Frame<'_>) {
    let green_bold = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let cyan_bold = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let yellow_bold = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let gray = Style::default().fg(Color::Gray);
    let dark_gray = Style::default().fg(Color::DarkGray);

    let banner = vec![
        Line::from(""),
        Line::from(Span::styled(" ██╗   ██╗██╗███╗   ███╗███████╗", green_bold)),
        Line::from(Span::styled(" ██║   ██║██║████╗ ████║██╔════╝", green_bold)),
        Line::from(Span::styled(" ██║   ██║██║██╔████╔██║███████╗", green_bold)),
        Line::from(Span::styled(" ╚██╗ ██╔╝██║██║╚██╔╝██║╚════██║", green_bold)),
        Line::from(Span::styled("  ╚████╔╝ ██║██║ ╚═╝ ██║███████║", green_bold)),
        Line::from(Span::styled("   ╚═══╝  ╚═╝╚═╝     ╚═╝╚══════╝", green_bold)),
        Line::from(""),
        Line::from(Span::styled(
            " ██████╗ ██╗   ██╗██████╗ ██╗   ██╗███████╗██████╗ ",
            green_bold,
        )),
        Line::from(Span::styled(
            "██╔═══██╗██║   ██║██╔══██╗██║   ██║██╔════╝██╔══██╗",
            green_bold,
        )),
        Line::from(Span::styled(
            "██║   ██║██║   ██║██████╔╝██║   ██║█████╗  ██████╔╝",
            green_bold,
        )),
        Line::from(Span::styled(
            "██║▄▄ ██║██║   ██║██╔══██╗██║   ██║██╔══╝  ██╔══██╗",
            green_bold,
        )),
        Line::from(Span::styled(
            "╚██████╔╝╚██████╔╝██║  ██║╚██████╔╝███████╗██║  ██║",
            green_bold,
        )),
        Line::from(Span::styled(
            " ╚══▀▀═╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝",
            green_bold,
        )),
        Line::from(""),
    ];

    let separator = Line::from(Span::styled(
        "─── Motions ───",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    let cross = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("         ", Style::default()),
            Span::styled("k", cyan_bold),
            Span::styled(" ↑", gray),
            Span::raw("              "),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("h", cyan_bold),
            Span::styled(" ←", gray),
            Span::raw("     "),
            Span::styled("→ ", gray),
            Span::styled("l", cyan_bold),
            Span::raw("   "),
            Span::styled("Basic Movement", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("         ", Style::default()),
            Span::styled("j", cyan_bold),
            Span::styled(" ↓", gray),
            Span::raw("              "),
        ]),
        Line::from(""),
    ];

    let zone1_color = zone_accent_color(Zone::Zone1);
    let zone2_color = zone_accent_color(Zone::Zone2);
    let zone3_color = zone_accent_color(Zone::Zone3);
    let zone4_color = zone_accent_color(Zone::Zone4);
    let zone5_color = zone_accent_color(Zone::Zone5);

    let motions = vec![
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("w", cyan_bold),
            Span::styled("/", Style::default()),
            Span::styled("b", cyan_bold),
            Span::raw("  Word Jumps       "),
            Span::styled("0", cyan_bold),
            Span::styled("/", Style::default()),
            Span::styled("$", cyan_bold),
            Span::raw("  Line Ends        "),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("f", cyan_bold),
            Span::styled("/", Style::default()),
            Span::styled("t", cyan_bold),
            Span::raw("  Find/Till        "),
            Span::styled("dd", cyan_bold),
            Span::raw("   Delete Obstacle "),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Zone 1: ", Style::default().fg(zone1_color)),
            Span::styled("hjkl", cyan_bold),
            Span::raw("  "),
            Span::styled("Zone 2: ", Style::default().fg(zone2_color)),
            Span::styled("wb", cyan_bold),
            Span::raw("  "),
            Span::styled("Zone 3: ", Style::default().fg(zone3_color)),
            Span::styled("0$", cyan_bold),
        ]),
        Line::from(vec![
            Span::styled("  Zone 4: ", Style::default().fg(zone4_color)),
            Span::styled("ft", cyan_bold),
            Span::raw("  "),
            Span::styled("Zone 5: ", Style::default().fg(zone5_color)),
            Span::styled("dd", cyan_bold),
        ]),
        Line::from(""),
    ];

    let instructions = vec![
        Line::from(Span::styled(
            "  Navigate from @ to > across five zones.",
            gray,
        )),
        Line::from(""),
        Line::from(Span::styled(
            "       ► Press any key to start ◄",
            yellow_bold,
        )),
        Line::from(Span::styled("           Esc or q quits", dark_gray)),
    ];

    let mut lines = Vec::new();
    lines.extend(banner);
    lines.push(separator);
    lines.extend(cross);
    lines.extend(motions);
    lines.extend(instructions);

    let title = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Welcome"))
        .wrap(Wrap { trim: true });

    frame.render_widget(title, centered_rect(80, 70, frame.area()));
}

fn render_win(frame: &mut Frame<'_>, app: &App) {
    let duration = format_duration(app.final_time.unwrap_or(app.elapsed));
    let gold = Color::Rgb(255, 215, 0);
    let green_bold = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let gold_bold = Style::default().fg(gold).add_modifier(Modifier::BOLD);
    let yellow_style = Style::default().fg(Color::Yellow);

    let trophy_lines: Vec<Line> = vec![
        Line::from(Span::styled("        ╔═══╗         ", gold_bold)),
        Line::from(Span::styled("        ║   ║         ", gold_bold)),
        Line::from(Span::styled(
            "        ║ ★ ║         ",
            Style::default().fg(gold).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("        ║   ║         ", gold_bold)),
        Line::from(Span::styled("        ╚═╤═╝         ", gold_bold)),
        Line::from(Span::styled("         _|_          ", gold_bold)),
        Line::from(Span::styled("        |   |         ", gold_bold)),
        Line::from(Span::styled("        |___|         ", gold_bold)),
        Line::from(Span::styled("        \\_____/        ", gold_bold)),
    ];

    let header = Line::from(vec![
        Span::styled("★  ★  ★  ", yellow_style),
        Span::styled("V I C T O R Y", green_bold),
        Span::styled("  ★  ★  ★", yellow_style),
    ]);

    let stats = Line::from(format!(
        "  Level: {} / {}    Time: {duration}    Moves: {}",
        app.level, TOTAL_LEVELS, app.motion_count
    ));

    let mut zone_lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ── Zone Completion ──",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (zone, motions) in phase_definitions() {
        let total = motions.len();
        let discovered = motions
            .iter()
            .filter(|m| app.discovered_motions.contains(m))
            .count();
        let bar_width = 8;
        let filled = if total == 0 {
            0
        } else {
            let f = ((discovered as f64 / total as f64) * bar_width as f64).round() as usize;
            f.min(bar_width)
        };
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_width - filled));
        let complete = discovered == total;
        let check = if complete { " ✓" } else { "" };
        let check_span = if complete {
            Span::styled(check.to_string(), Style::default().fg(Color::Green))
        } else {
            Span::raw("")
        };
        let zone_color = zone_accent_color(zone);
        zone_lines.push(Line::from(vec![
            Span::styled(
                format!("  {}  ", zone.title()),
                Style::default().fg(zone_color),
            ),
            Span::styled(format!("[{bar}] "), Style::default().fg(zone_color)),
            Span::raw(format!("{discovered}/{total}")),
            check_span,
        ]));
    }

    let total_discovered = app.unique_motions();
    let rating = match total_discovered {
        0..=3 => "Novice",
        4..=6 => "Apprentice",
        7..=9 => "Journeyman",
        10 => "Adept",
        _ => "Master",
    };
    let rating_color = match total_discovered {
        0..=3 => Color::Rgb(205, 127, 50),
        4..=6 => Color::Rgb(192, 192, 192),
        7..=9 => Color::Rgb(255, 215, 0),
        _ => Color::Rgb(255, 255, 100),
    };

    let mastery_line = Line::from(vec![
        Span::styled("  Motion Mastery: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{total_discovered}/11"),
            Style::default()
                .fg(rating_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  — ", Style::default().fg(Color::White)),
        Span::styled(rating, Style::default().fg(rating_color)),
    ]);

    let footer = Line::from(Span::styled(
        "       Press q or Esc to leave the dungeon.",
        Style::default().fg(Color::DarkGray),
    ));

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(header);
    lines.push(Line::from(""));
    lines.extend(trophy_lines);
    lines.push(Line::from(""));
    lines.push(stats);
    lines.extend(zone_lines);
    lines.push(Line::from(""));
    lines.push(mastery_line);
    lines.push(Line::from(""));
    lines.push(footer);
    lines.push(Line::from(""));

    let body = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("★ Victory — Level {} ★", app.level)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(body, centered_rect(80, 70, frame.area()));
}

fn render_map(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Dungeon");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let view_width = inner.width as usize;
    let view_height = inner.height as usize;
    let half_w = view_width / 2;
    let half_h = view_height / 2;

    let mut left = app.player.position.x.saturating_sub(half_w);
    let mut top = app.player.position.y.saturating_sub(half_h);

    if left + view_width > app.map.width {
        left = app.map.width.saturating_sub(view_width);
    }
    if top + view_height > app.map.height {
        top = app.map.height.saturating_sub(view_height);
    }

    let trail_positions: Vec<crate::types::Position> = app.trail.iter().copied().collect();

    let mut lines = Vec::with_capacity(view_height);
    for y in top..(top + view_height).min(app.map.height) {
        let mut spans = Vec::with_capacity(view_width);
        for x in left..(left + view_width).min(app.map.width) {
            if app.player.position.x == x && app.player.position.y == y {
                spans.push(Span::styled(
                    "@",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
                continue;
            }

            if let Some(trail_idx) = trail_positions.iter().position(|p| p.x == x && p.y == y) {
                let (glyph, style) = trail_style(trail_idx, trail_positions.len());
                spans.push(Span::styled(glyph.to_string(), style));
                continue;
            }

            let tile = app.map.get_tile(x, y);
            let zone = app.map.zone_at(crate::types::Position { x, y });

            let (glyph, style) = match tile {
                Tile::Wall => (
                    wall_display_glyph(x, y, &app.map).to_string(),
                    Style::default().fg(zone_wall_color(zone)),
                ),
                Tile::Floor => (".".to_string(), Style::default().fg(zone_floor_color(zone))),
                Tile::Exit => {
                    let (g, s) = exit_glow_style(app.elapsed);
                    (g.to_string(), s)
                }
                Tile::Obstacle => {
                    let (g, s) = obstacle_display_style();
                    (g.to_string(), s)
                }
            };
            spans.push(Span::styled(glyph, style));
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_sidebar(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Vim Motions");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from(Span::styled(
            format!("Current zone: {}", app.current_zone()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("Level: {} / {}", app.level, TOTAL_LEVELS)),
        Line::from(format!("Time: {}", format_duration(app.elapsed))),
        Line::from(format!("Moves: {}", app.motion_count)),
        Line::from(format!("Unique: {}", app.unique_motions())),
        Line::from(""),
    ];

    for (zone, motions) in phase_definitions() {
        lines.push(Line::from(Span::styled(
            zone.title(),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for motion in motions {
            let used = app.player.used_motions.contains(motion);
            let marker = if used { "✓" } else { "·" };
            let style = if used {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(Span::styled(
                format!(
                    "{marker} {:<7} {}",
                    motion.key_label(),
                    motion.display_name()
                ),
                style,
            )));
        }
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "Hint",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(app.status_message.clone()));

    if let Some(pending) = app.pending_input {
        let prompt = match pending {
            PendingInput::Find => "Awaiting target for f",
            PendingInput::Till => "Awaiting target for t",
            PendingInput::Delete => "Awaiting second d",
        };
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            prompt,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let panel = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(panel, inner);
}

fn phase_definitions() -> Vec<(Zone, &'static [VimMotion])> {
    const ZONE1: &[VimMotion] = &[VimMotion::H, VimMotion::J, VimMotion::K, VimMotion::L];
    const ZONE2: &[VimMotion] = &[VimMotion::W, VimMotion::B];
    const ZONE3: &[VimMotion] = &[VimMotion::Zero, VimMotion::Dollar];
    const ZONE4: &[VimMotion] = &[VimMotion::Find, VimMotion::Till];
    const ZONE5: &[VimMotion] = &[VimMotion::DeleteLine];

    vec![
        (Zone::Zone1, ZONE1),
        (Zone::Zone2, ZONE2),
        (Zone::Zone3, ZONE3),
        (Zone::Zone4, ZONE4),
        (Zone::Zone5, ZONE5),
    ]
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn zone_wall_color(zone: Zone) -> Color {
    match zone {
        Zone::Zone1 => Color::DarkGray,
        Zone::Zone2 => Color::Rgb(0, 0, 139),
        Zone::Zone3 => Color::Rgb(139, 0, 139),
        Zone::Zone4 => Color::Rgb(139, 0, 0),
        Zone::Zone5 => Color::Rgb(139, 139, 0),
    }
}

fn zone_floor_color(zone: Zone) -> Color {
    match zone {
        Zone::Zone1 => Color::Gray,
        Zone::Zone2 => Color::Cyan,
        Zone::Zone3 => Color::LightMagenta,
        Zone::Zone4 => Color::LightRed,
        Zone::Zone5 => Color::Yellow,
    }
}

fn zone_accent_color(zone: Zone) -> Color {
    match zone {
        Zone::Zone1 => Color::White,
        Zone::Zone2 => Color::Blue,
        Zone::Zone3 => Color::Magenta,
        Zone::Zone4 => Color::Red,
        Zone::Zone5 => Color::Yellow,
    }
}

fn wall_display_glyph(x: usize, y: usize, map: &crate::map::Map) -> char {
    let mut non_wall_neighbors = 0u8;
    let directions: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for (dx, dy) in directions {
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        if nx >= 0 && ny >= 0 {
            let nx = nx as usize;
            let ny = ny as usize;
            if nx < map.width && ny < map.height {
                if !matches!(map.get_tile(nx, ny), Tile::Wall) {
                    non_wall_neighbors += 1;
                }
            }
        }
    }
    match non_wall_neighbors {
        0 => '█',
        1 => '▓',
        2 => '▒',
        _ => '#',
    }
}

fn exit_glow_style(elapsed: std::time::Duration) -> (char, Style) {
    let phase = (elapsed.as_millis() % 1000) as f64 / 1000.0;
    let pulse = (phase * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5;
    let r = (200.0 + 55.0 * pulse) as u8;
    let g = (200.0 + 55.0 * pulse) as u8;
    let glyph = if pulse > 0.5 { '►' } else { '>' };
    (
        glyph,
        Style::default()
            .fg(Color::Rgb(r, g, 0))
            .add_modifier(Modifier::BOLD),
    )
}

fn trail_style(index: usize, total: usize) -> (char, Style) {
    let fade = if total <= 1 {
        1.0
    } else {
        1.0 - (index as f64 / (total as f64 - 1.0)) * 0.7
    };
    let g = (180.0 * fade) as u8;
    ('·', Style::default().fg(Color::Rgb(0, g, 0)))
}

fn obstacle_display_style() -> (char, Style) {
    (
        '▓',
        Style::default()
            .fg(Color::LightRed)
            .add_modifier(Modifier::SLOW_BLINK),
    )
}
