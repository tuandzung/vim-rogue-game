use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::types::{App, GameState, PendingInput, Tile, VimMotion, Zone};

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
    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "VIM QUAKE",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("A roguelike dungeon for learning Vim motions."),
        Line::from("Navigate from @ to > across five motion-focused zones."),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to start",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("Esc or q quits during the run."),
    ])
    .block(Block::default().borders(Borders::ALL).title("Welcome"))
    .wrap(Wrap { trim: true });

    frame.render_widget(title, centered_rect(70, 40, frame.area()));
}

fn render_win(frame: &mut Frame<'_>, app: &App) {
    let duration = format_duration(app.final_time.unwrap_or(app.elapsed));
    let body = Paragraph::new(vec![
        Line::from(Span::styled(
            "VIM QUAKE CLEARED",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Time: {duration}")),
        Line::from(format!("Total motions: {}", app.motion_count)),
        Line::from(format!(
            "Unique motions discovered: {}",
            app.unique_motions()
        )),
        Line::from(format!("Final zone: {}", app.current_zone())),
        Line::from(""),
        Line::from("Press q or Esc to leave the dungeon."),
    ])
    .block(Block::default().borders(Borders::ALL).title("Victory"))
    .wrap(Wrap { trim: true });

    frame.render_widget(body, centered_rect(70, 50, frame.area()));
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

            let tile = app.map.get_tile(x, y);
            spans.push(Span::styled(tile.to_string(), tile_style(tile)));
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

fn tile_style(tile: Tile) -> Style {
    match tile {
        Tile::Wall => Style::default().fg(Color::DarkGray),
        Tile::Floor => Style::default().fg(Color::White),
        Tile::Exit => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        Tile::Obstacle => Style::default().fg(Color::LightRed),
    }
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
