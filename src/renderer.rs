use bracket_lib::prelude::*;

use crate::animation::AttackEffectKind;
use crate::types::{
    App, GameState, MAX_HP, PauseOption, PendingInput, Position, TOTAL_LEVELS, Tile, VimMotion,
    Zone,
};
use crate::visibility::VisibilityState;

pub const SCREEN_WIDTH: u32 = 80;
pub const SCREEN_HEIGHT: u32 = 50;

pub fn render(ctx: &mut BTerm, app: &App) {
    ctx.cls();

    let (screen_width, screen_height) = ctx.get_char_size();
    if !screen_meets_minimum_size(screen_width, screen_height) {
        render_resize_notice(ctx, screen_width, screen_height);
        return;
    }

    if !app.session.started {
        render_title(ctx);
        return;
    }

    if app.session.game_state == GameState::Won {
        render_win(ctx, app);
        return;
    }

    if app.session.game_state == GameState::Lost {
        render_lost(ctx, app);
        return;
    }

    if app.session.game_state == GameState::Paused {
        render_gameplay(ctx, app);
        render_pause_overlay(ctx, app);
        return;
    }

    if app.session.game_state == GameState::Dying {
        render_gameplay(ctx, app);
        return;
    }

    render_gameplay(ctx, app);
}

fn render_resize_notice(ctx: &mut BTerm, width: u32, height: u32) {
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);
    let yellow: RGB = RGB::named(YELLOW);
    let dark_gray: RGB = rgb8(140, 140, 140);
    let lines = [
        ("Window too small", white),
        ("Please resize the window.", yellow),
        ("Needs at least 80x50 cells.", dark_gray),
    ];
    let start_y = center_y_for(height, lines.len());

    for (index, (line, color)) in lines.iter().enumerate() {
        let y = start_y + index as i32;
        if y >= height as i32 {
            break;
        }
        ctx.print_color(center_x_for(width, line.len()), y, *color, black, line);
    }
}

pub fn screen_meets_minimum_size(width: u32, height: u32) -> bool {
    width >= SCREEN_WIDTH && height >= SCREEN_HEIGHT
}

fn render_gameplay(ctx: &mut BTerm, app: &App) {
    let map_width: i32 = 60;
    let sidebar_x: i32 = 61;

    render_map_viewport(ctx, app, map_width);
    render_sidebar(ctx, app, sidebar_x);
}

fn render_map_viewport(ctx: &mut BTerm, app: &App, map_width: i32) {
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);
    let mw = map_width as usize;
    let player_draw_pos = visual_player_position(app);
    let enemy_draw_positions = visual_enemy_positions(app);

    ctx.print_color(0, 0, white, black, "┌");
    for x in 1..mw + 1 {
        ctx.print_color(x as i32, 0, white, black, "─");
    }
    ctx.print_color(map_width + 1, 0, white, black, "┐");

    ctx.print_color(0, 49, white, black, "└");
    for x in 1..mw + 1 {
        ctx.print_color(x as i32, 49, white, black, "─");
    }
    ctx.print_color(map_width + 1, 49, white, black, "┘");

    for y in 1..49 {
        ctx.print_color(0, y, white, black, "│");
        ctx.print_color(map_width + 1, y, white, black, "│");
    }
    ctx.print_color(2, 0, white, black, " Dungeon ");

    let view_width = mw.saturating_sub(2);
    let view_height: usize = 48;
    let half_w = view_width / 2;
    let half_h = view_height / 2;

    let mut left = player_draw_pos.x.saturating_sub(half_w);
    let mut top = player_draw_pos.y.saturating_sub(half_h);

    if left + view_width > app.world.map.width {
        left = app.world.map.width.saturating_sub(view_width);
    }
    if top + view_height > app.world.map.height {
        top = app.world.map.height.saturating_sub(view_height);
    }

    let trail_positions: Vec<crate::types::Position> = app.player.trail.iter().copied().collect();

    for screen_y in 0..view_height {
        let map_y = top + screen_y;
        if map_y >= app.world.map.height {
            break;
        }
        for screen_x in 0..view_width {
            let map_x = left + screen_x;
            if map_x >= app.world.map.width {
                break;
            }

            let draw_x = (screen_x + 1) as i32;
            let draw_y = (screen_y + 1) as i32;

            if player_draw_pos.x == map_x && player_draw_pos.y == map_y {
                ctx.print_color(draw_x, draw_y, RGB::named(GREEN), black, "@");
                continue;
            }

            let vis = app.world.visibility.get(Position { x: map_x, y: map_y });

            if vis == VisibilityState::Hidden {
                ctx.print_color(draw_x, draw_y, black, black, " ");
                continue;
            }

            if vis == VisibilityState::Visible {
                if enemy_draw_positions
                    .iter()
                    .any(|enemy_pos| enemy_pos.x == map_x && enemy_pos.y == map_y)
                {
                    ctx.print_color(draw_x, draw_y, RGB::named(RED), black, "e");
                    continue;
                }

                if let Some(trail_idx) =
                    trail_positions.iter().position(|p| p.x == map_x && p.y == map_y)
                {
                    let (glyph, color) = trail_color(trail_idx, trail_positions.len());
                    ctx.print_color(draw_x, draw_y, color, black, glyph.to_string());
                    continue;
                }
            }

            let tile = app.world.map.get_tile(map_x, map_y);
            let zone = app.world.map.zone_at(Position { x: map_x, y: map_y });
            let wall_glyph = wall_display_glyph(map_x, map_y, &app.world.map);

            if let Some((glyph, fg)) =
                tile_fog_appearance(tile, zone, vis, app.session.elapsed, wall_glyph)
            {
                ctx.print_color(draw_x, draw_y, fg, black, glyph.to_string());
            }
        }
    }

    for effect in &app.attack_effects {
        let screen_x = effect.x as isize - left as isize;
        let screen_y = effect.y as isize - top as isize;
        if screen_x >= 0
            && screen_x < view_width as isize
            && screen_y >= 0
            && screen_y < view_height as isize
        {
            let draw_x = (screen_x + 1) as i32;
            let draw_y = (screen_y + 1) as i32;
            let (glyph, fg) = attack_effect_display(effect.kind, effect.timer.progress());
            ctx.print_color(draw_x, draw_y, fg, black, glyph.to_string());
        }
    }
}

pub fn visual_player_position(app: &App) -> Position {
    let (x, y) = app
        .player_animation
        .map(|animation| animation.current_position())
        .unwrap_or((app.player.inner.position.x as f64, app.player.inner.position.y as f64));

    Position {
        x: x.round().clamp(0.0, app.world.map.width.saturating_sub(1) as f64) as usize,
        y: y.round().clamp(0.0, app.world.map.height.saturating_sub(1) as f64) as usize,
    }
}

pub fn visual_enemy_positions(app: &App) -> Vec<Position> {
    app.world
        .enemies
        .iter()
        .enumerate()
        .map(|(enemy_index, enemy)| {
            let (x, y) = app
                .enemy_animations
                .iter()
                .find_map(|(animated_index, animation)| {
                    (*animated_index == enemy_index).then(|| animation.current_position())
                })
                .unwrap_or((enemy.position.x as f64, enemy.position.y as f64));

            Position {
                x: x.round().clamp(0.0, app.world.map.width.saturating_sub(1) as f64) as usize,
                y: y.round().clamp(0.0, app.world.map.height.saturating_sub(1) as f64) as usize,
            }
        })
        .collect()
}

fn render_sidebar(ctx: &mut BTerm, app: &App, sidebar_x: i32) {
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);
    let green: RGB = RGB::named(GREEN);
    let dark_gray: RGB = rgb8(140, 140, 140);
    let magenta: RGB = RGB::named(MAGENTA);

    ctx.print_color(sidebar_x - 1, 0, white, black, "┌");
    for x in sidebar_x..80 {
        ctx.print_color(x, 0, white, black, "─");
    }
    ctx.print_color(sidebar_x - 1, 49, white, black, "└");
    for x in sidebar_x..80 {
        ctx.print_color(x, 49, white, black, "─");
    }
    for y in 1..49 {
        ctx.print_color(sidebar_x - 1, y, white, black, "│");
    }
    ctx.print_color(sidebar_x + 2, 0, white, black, " Vim Motions ");

    let mut y: i32 = 2;

    let zone = app.current_zone();
    let zone_color = zone_accent_color(zone);
    ctx.print_color(sidebar_x, y, zone_color, black, zone.title());
    y += 2;

    ctx.print_color(
        sidebar_x,
        y,
        white,
        black,
        format!("Level: {} / {}", app.player.level, TOTAL_LEVELS),
    );
    y += 1;
    ctx.print_color(
        sidebar_x,
        y,
        white,
        black,
        format!("Time:  {}", format_duration(app.session.elapsed)),
    );
    y += 1;
    ctx.print_color(sidebar_x, y, white, black, format!("Moves: {}", app.player.motion_count));
    y += 1;
    let hp_ratio = app.player.hp as f32 / MAX_HP as f32;
    let hp_filled = (hp_ratio * 10.0).round() as usize;
    let hp_color = if hp_ratio > 0.5 {
        RGB::named(GREEN)
    } else if hp_ratio > 0.25 {
        RGB::named(YELLOW)
    } else {
        RGB::named(RED)
    };
    let hp_text = format!("HP: {}/{}", app.player.hp, MAX_HP);
    ctx.print_color(sidebar_x, y, hp_color, black, &hp_text);
    y += 1;
    let mut bar = String::with_capacity(12);
    bar.push('[');
    for i in 0..10usize {
        if i < hp_filled {
            bar.push('█');
        } else {
            bar.push(' ');
        }
    }
    bar.push(']');
    ctx.print_color(sidebar_x, y, hp_color, black, &bar);
    y += 1;
    ctx.print_color(sidebar_x, y, white, black, format!("Unique: {}", app.unique_motions()));
    y += 2;

    for (zone, motions) in phase_definitions() {
        if y >= 47 {
            break;
        }
        let zc = zone_accent_color(zone);
        ctx.print_color(sidebar_x, y, zc, black, zone.title());
        y += 1;
        for motion in motions {
            if y >= 47 {
                break;
            }
            let used = app.player.inner.used_motions.contains(motion);
            let marker = if used { "✓" } else { "·" };
            let color = if used { green } else { dark_gray };
            let label = format!("{} {:<7} {}", marker, motion.key_label(), motion.display_name());
            ctx.print_color(sidebar_x, y, color, black, &label);
            y += 1;
        }
        y += 1;
    }

    if y < 47 {
        ctx.print_color(sidebar_x, y, white, black, "Hint");
        y += 1;
    }
    if y < 48 {
        let max_width = (80 - sidebar_x) as usize;
        let msg = if app.session.status_message.len() > max_width {
            &app.session.status_message[..max_width]
        } else {
            &app.session.status_message
        };
        ctx.print_color(sidebar_x, y, dark_gray, black, msg);
        y += 2;
    }

    if let Some(pending) = app.input.pending_input
        && y < 49
    {
        let prompt = match pending {
            PendingInput::Find => "Awaiting target for f",
            PendingInput::Till => "Awaiting target for t",
            PendingInput::Delete => "Awaiting second d",
            PendingInput::GotoLine => "Awaiting second g",
        };
        ctx.print_color(sidebar_x, y, magenta, black, prompt);
        y += 1;
    }

    render_minimap(ctx, app, sidebar_x, y, &mut y);
}

pub const MINIMAP_WIDTH: usize = 18;
pub const MINIMAP_HEIGHT: usize = 9;

pub fn minimap_map_coords(mx: usize, my: usize) -> (usize, usize) {
    let map_x = (mx as f64 * 80.0 / MINIMAP_WIDTH as f64) as usize;
    let map_y = (my as f64 * 40.0 / MINIMAP_HEIGHT as f64) as usize;
    (map_x.min(79), map_y.min(39))
}

pub fn minimap_player_pos(player_x: usize, player_y: usize) -> (i32, i32) {
    let mm_x = (player_x as f64 * MINIMAP_WIDTH as f64 / 80.0) as i32;
    let mm_y = (player_y as f64 * MINIMAP_HEIGHT as f64 / 40.0) as i32;
    (mm_x.min(MINIMAP_WIDTH as i32 - 1), mm_y.min(MINIMAP_HEIGHT as i32 - 1))
}

fn render_minimap(ctx: &mut BTerm, app: &App, x: i32, start_y: i32, y_out: &mut i32) {
    let black: RGB = RGB::named(BLACK);
    let white: RGB = RGB::named(WHITE);

    let title_y = start_y + 1;
    let mm_y = title_y + 1;

    if mm_y + MINIMAP_HEIGHT as i32 + 2 > 49 {
        *y_out = start_y;
        return;
    }

    ctx.print_color(x, title_y, white, black, "Map");

    ctx.print_color(x, mm_y, white, black, "┌");
    for mx in 0..MINIMAP_WIDTH {
        ctx.print_color(x + 1 + mx as i32, mm_y, white, black, "─");
    }
    ctx.print_color(x + 1 + MINIMAP_WIDTH as i32, mm_y, white, black, "┐");

    let bottom_y = mm_y + MINIMAP_HEIGHT as i32 + 1;
    ctx.print_color(x, bottom_y, white, black, "└");
    for mx in 0..MINIMAP_WIDTH {
        ctx.print_color(x + 1 + mx as i32, bottom_y, white, black, "─");
    }
    ctx.print_color(x + 1 + MINIMAP_WIDTH as i32, bottom_y, white, black, "┘");

    for side_y in (mm_y + 1)..bottom_y {
        ctx.print_color(x, side_y, white, black, "│");
        ctx.print_color(x + 1 + MINIMAP_WIDTH as i32, side_y, white, black, "│");
    }

    for my in 0..MINIMAP_HEIGHT {
        for mx in 0..MINIMAP_WIDTH {
            let (map_x, map_y) = minimap_map_coords(mx, my);
            let pos = Position { x: map_x, y: map_y };
            let vis = app.world.visibility.get(pos);

            if vis == VisibilityState::Hidden {
                continue;
            }

            let tile = app.world.map.get_tile(map_x, map_y);
            let zone = app.world.map.zone_at(pos);

            let (glyph, color) = match tile {
                Tile::Wall => ('█', dim_color(zone_wall_color(zone), 0.6)),
                Tile::Floor => ('·', dim_color(zone_floor_color(zone), 0.6)),
                Tile::Exit => ('>', RGB::named(YELLOW)),
                Tile::Obstacle => ('▒', dim_color(rgb8(255, 100, 100), 0.6)),
                Tile::Torchlight => ('i', RGB::named(YELLOW)),
            };

            let final_color =
                if vis == VisibilityState::Explored { dim_color(color, 0.5) } else { color };

            ctx.print_color(
                x + 1 + mx as i32,
                mm_y + 1 + my as i32,
                final_color,
                black,
                glyph.to_string(),
            );
        }
    }

    let (px, py) = minimap_player_pos(app.player.inner.position.x, app.player.inner.position.y);
    ctx.print_color(x + 1 + px, mm_y + 1 + py, RGB::named(GREEN), black, "@");

    *y_out = bottom_y + 1;
}

fn render_title(ctx: &mut BTerm) {
    let black: RGB = RGB::named(BLACK);
    let green: RGB = RGB::named(GREEN);
    let cyan: RGB = RGB::named(CYAN);
    let yellow: RGB = RGB::named(YELLOW);
    let white: RGB = RGB::named(WHITE);
    let gray: RGB = rgb8(160, 160, 160);
    let dark_gray: RGB = rgb8(130, 130, 130);

    let vim_banner: &[&str] = &[
        " ██╗   ██╗██╗███╗   ███╗███████╗",
        " ██║   ██║██║████╗ ████║██╔════╝",
        " ██║   ██║██║██╔████╔██║███████╗",
        " ╚██╗ ██╔╝██║██║╚██╔╝██║╚════██║",
        "  ╚████╔╝ ██║██║ ╚═╝ ██║███████║",
        "   ╚═══╝  ╚═╝╚═╝     ╚═╝╚══════╝",
    ];

    let quake_banner: &[&str] = &[
        " ██████╗ ██╗   ██╗██████╗ ██╗   ██╗███████╗██████╗ ",
        "██╔═══██╗██║   ██║██╔══██╗██║   ██║██╔════╝██╔══██╗",
        "██║   ██║██║   ██║██████╔╝██║   ██║█████╗  ██████╔╝",
        "██║▄▄ ██║██║   ██║██╔══██╗██║   ██║██╔══╝  ██╔══██╗",
        "╚██████╔╝╚██████╔╝██║  ██║╚██████╔╝███████╗██║  ██║",
        " ╚══▀▀═╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝",
    ];

    let start_y: i32 = 1;
    for (i, line) in vim_banner.iter().enumerate() {
        ctx.print_color(center_x(line.len()), start_y + i as i32, green, black, line);
    }

    let quake_y = start_y + vim_banner.len() as i32 + 1;
    for (i, line) in quake_banner.iter().enumerate() {
        ctx.print_color(center_x(line.len()), quake_y + i as i32, green, black, line);
    }

    let mut y: i32 = quake_y + quake_banner.len() as i32 + 1;

    ctx.print_color(center_x(15), y, white, black, "─── Motions ───");
    y += 2;

    let cx = center_x(20);
    ctx.print_color(cx + 9, y, cyan, black, "k ↑");
    y += 1;
    ctx.print_color(cx + 4, y, cyan, black, "h ←");
    ctx.print_color(cx + 11, y, gray, black, "→ ");
    ctx.print_color(cx + 13, y, cyan, black, "l");
    ctx.print_color(cx + 16, y, white, black, "Basic Movement");
    y += 1;
    ctx.print_color(cx + 9, y, cyan, black, "j ↓");
    y += 2;

    ctx.print_color(4, y, cyan, black, "w");
    ctx.print_color(5, y, white, black, "/");
    ctx.print_color(6, y, cyan, black, "b");
    ctx.print_color(8, y, white, black, "Word Jumps       ");
    ctx.print_color(27, y, cyan, black, "0");
    ctx.print_color(28, y, white, black, "/");
    ctx.print_color(29, y, cyan, black, "$");
    ctx.print_color(31, y, white, black, "Line Ends        ");
    y += 1;
    ctx.print_color(4, y, cyan, black, "f");
    ctx.print_color(5, y, white, black, "/");
    ctx.print_color(6, y, cyan, black, "t");
    ctx.print_color(8, y, white, black, "Find/Till        ");
    ctx.print_color(27, y, cyan, black, "dd");
    ctx.print_color(29, y, white, black, "  Delete Obstacle ");
    y += 1;
    ctx.print_color(4, y, cyan, black, "G");
    ctx.print_color(5, y, white, black, "/");
    ctx.print_color(6, y, cyan, black, "gg");
    ctx.print_color(9, y, white, black, "Row Jumps         ");
    y += 2;

    let z1 = zone_accent_color(Zone::Zone1);
    let z2 = zone_accent_color(Zone::Zone2);
    let z3 = zone_accent_color(Zone::Zone3);
    let z4 = zone_accent_color(Zone::Zone4);
    let z5 = zone_accent_color(Zone::Zone5);

    ctx.print_color(2, y, z1, black, "  Zone 1: ");
    ctx.print_color(12, y, cyan, black, "hjkl");
    ctx.print_color(17, y, z2, black, "  Zone 2: ");
    ctx.print_color(27, y, cyan, black, "wb");
    y += 1;
    ctx.print_color(2, y, z3, black, "  Zone 3: ");
    ctx.print_color(12, y, cyan, black, "0$Ggg");
    ctx.print_color(18, y, z4, black, " Zone 4: ");
    ctx.print_color(27, y, cyan, black, "ft");
    ctx.print_color(30, y, z5, black, "  Zone 5: ");
    ctx.print_color(40, y, cyan, black, "dd");
    y += 2;

    ctx.print_color(center_x(36), y, gray, black, "Navigate from @ to > across five zones.");
    y += 2;
    let prompt = "► Press any key to start ◄";
    ctx.print_color(center_x(prompt.len()), y, yellow, black, prompt);
    y += 1;
    ctx.print_color(center_x(18), y, dark_gray, black, "Esc or q quits");
}

fn render_win(ctx: &mut BTerm, app: &App) {
    let black: RGB = RGB::named(BLACK);
    let green: RGB = RGB::named(GREEN);
    let gold: RGB = rgb8(255, 215, 0);
    let white: RGB = RGB::named(WHITE);
    let dark_gray: RGB = rgb8(130, 130, 130);

    let trophy: &[&str] = &[
        "        ╔═══╗         ",
        "        ║   ║         ",
        "        ║ ★ ║         ",
        "        ║   ║         ",
        "        ╚═╤═╝         ",
        "         _|_          ",
        "        |   |         ",
        "        |___|         ",
        "        \\_____/        ",
    ];

    let mut y: i32 = 2;

    let header = "★  ★  ★  V I C T O R Y  ★  ★  ★";
    ctx.print_color(center_x(header.len()), y, green, black, header);
    y += 2;

    for line in trophy {
        let x = center_x(line.len());
        ctx.print_color(x, y, gold, black, line);
        y += 1;
    }
    y += 1;

    let duration = format_duration(app.session.final_time.unwrap_or(app.session.elapsed));
    let stats = format!(
        "  Level: {} / {}    Time: {}    Moves: {}",
        app.player.level, TOTAL_LEVELS, duration, app.player.motion_count
    );
    ctx.print_color(center_x(stats.len()), y, white, black, &stats);
    y += 2;

    let zhdr = "── Zone Completion ──";
    ctx.print_color(center_x(zhdr.len()), y, white, black, zhdr);
    y += 2;

    for (zone, motions) in phase_definitions() {
        let total = motions.len();
        let discovered =
            motions.iter().filter(|m| app.player.discovered_motions.contains(m)).count();
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
        let zone_color = zone_accent_color(zone);
        let line = format!("  {}  [{}] {}/{}{}", zone.title(), bar, discovered, total, check);
        ctx.print_color(center_x(line.len()), y, zone_color, black, &line);
        y += 1;
    }
    y += 1;

    let total_discovered = app.unique_motions();
    let (rating, rating_color) = motion_mastery(total_discovered);
    let num_str = format!("{}/13", total_discovered);
    let after_num = format!("  — {}", rating);
    let mastery_x = center_x(format!("  Motion Mastery: {}{}", num_str, after_num).len());
    ctx.print_color(mastery_x, y, white, black, "  Motion Mastery: ");
    let prefix_len = "  Motion Mastery: ".len() as i32;
    ctx.print_color(mastery_x + prefix_len, y, rating_color, black, &num_str);
    ctx.print_color(
        mastery_x + prefix_len + num_str.len() as i32,
        y,
        rating_color,
        black,
        &after_num,
    );
    y += 2;

    let footer = "Press q or Esc to leave the dungeon.";
    ctx.print_color(center_x(footer.len()), y, dark_gray, black, footer);
}

fn render_lost(ctx: &mut BTerm, app: &App) {
    let black: RGB = RGB::named(BLACK);
    let red: RGB = RGB::named(RED);
    let light_red: RGB = rgb8(255, 100, 100);
    let yellow: RGB = RGB::named(YELLOW);
    let white: RGB = RGB::named(WHITE);
    let dark_gray: RGB = rgb8(130, 130, 130);

    let skull: &[&str] = &[
        "          ████          ",
        "        ██    ██        ",
        "       ██ █  █ ██       ",
        "       ██      ██       ",
        "        ████████        ",
        "          █  █          ",
        "        ██    ██        ",
    ];

    let mut y: i32 = 3;

    let header = "☠  G A M E   O V E R  ☠";
    ctx.print_color(center_x(header.len()), y, red, black, header);
    y += 2;

    for line in skull {
        let x = center_x(line.len());
        ctx.print_color(x, y, red, black, line);
        y += 1;
    }
    y += 1;

    let duration = format_duration(app.session.final_time.unwrap_or(app.session.elapsed));
    let stats = format!(
        "  Level: {} / {}    Time: {}    Moves: {}",
        app.player.level, TOTAL_LEVELS, duration, app.player.motion_count
    );
    ctx.print_color(center_x(stats.len()), y, white, black, &stats);
    y += 2;

    let hp_msg = "HP depleted — an enemy caught you.";
    ctx.print_color(center_x(hp_msg.len()), y, light_red, black, hp_msg);
    y += 2;

    let prompt = "► Press any key to retry the level ◄";
    ctx.print_color(center_x(prompt.len()), y, yellow, black, prompt);
    y += 1;
    ctx.print_color(center_x(18), y, dark_gray, black, "Esc or q quits");
}

fn render_pause_overlay(ctx: &mut BTerm, app: &App) {
    let white = RGB::named(WHITE);
    let yellow = RGB::named(YELLOW);
    let cyan = RGB::named(CYAN);
    let dark_gray = rgb8(130, 130, 130);
    let box_bg = rgb8(20, 20, 40);
    let box_width = 36i32;
    let box_height = 11i32;
    let left = center_x(box_width as usize);
    let top = center_y_for(SCREEN_HEIGHT, box_height as usize);
    let text_x = left + 2;
    let text_width = (box_width - 4) as usize;

    ctx.print_color(left, top, white, box_bg, "┌");
    for x in (left + 1)..(left + box_width - 1) {
        ctx.print_color(x, top, white, box_bg, "─");
    }
    ctx.print_color(left + box_width - 1, top, white, box_bg, "┐");

    for y in (top + 1)..(top + box_height - 1) {
        ctx.print_color(left, y, white, box_bg, "│");
        for x in (left + 1)..(left + box_width - 1) {
            ctx.print_color(x, y, white, box_bg, " ");
        }
        ctx.print_color(left + box_width - 1, y, white, box_bg, "│");
    }

    ctx.print_color(left, top + box_height - 1, white, box_bg, "└");
    for x in (left + 1)..(left + box_width - 1) {
        ctx.print_color(x, top + box_height - 1, white, box_bg, "─");
    }
    ctx.print_color(left + box_width - 1, top + box_height - 1, white, box_bg, "┘");

    let header = "P  A  U  S  E";
    ctx.print_color(center_x(header.len()), top + 2, cyan, box_bg, header);

    let options = [
        (PauseOption::Resume, "Resume"),
        (PauseOption::RetryLevel, "Retry Level"),
        (PauseOption::QuitGame, "Quit Game"),
    ];

    for (index, (option, label)) in options.iter().enumerate() {
        let selected = app.session.pause_selection == *option;
        let prefix = if selected { "► " } else { "  " };
        let line = format!("{prefix}{label}");
        let color = if selected { yellow } else { dark_gray };
        ctx.print_color(
            text_x,
            top + 4 + index as i32,
            color,
            box_bg,
            format!("{line:<text_width$}"),
        );
    }

    let nav_hint = "↑↓ j/k navigate  Enter select";
    ctx.print_color(text_x, top + 8, dark_gray, box_bg, format!("{nav_hint:<text_width$}"));
    let esc_hint = "Esc resume";
    ctx.print_color(text_x, top + 9, dark_gray, box_bg, format!("{esc_hint:<text_width$}"));
}

pub fn center_x(text_len: usize) -> i32 {
    center_x_for(SCREEN_WIDTH, text_len)
}

pub fn center_x_for(screen_width: u32, text_len: usize) -> i32 {
    (screen_width as i32 - text_len as i32).max(0) / 2
}

pub fn center_y_for(screen_height: u32, content_height: usize) -> i32 {
    (screen_height as i32 - content_height as i32).max(0) / 2
}

pub fn rgb8(r: u8, g: u8, b: u8) -> RGB {
    RGB { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0 }
}

pub fn dim_color(color: RGB, factor: f32) -> RGB {
    RGB {
        r: (color.r * factor).min(1.0),
        g: (color.g * factor).min(1.0),
        b: (color.b * factor).min(1.0),
    }
}

pub fn tile_fog_appearance(
    tile: Tile,
    zone: Zone,
    vis: VisibilityState,
    elapsed: std::time::Duration,
    wall_glyph: char,
) -> Option<(char, RGB)> {
    if vis == VisibilityState::Hidden {
        return None;
    }

    let (glyph, color) = match tile {
        Tile::Wall => (wall_glyph, zone_wall_color(zone)),
        Tile::Floor => ('.', zone_floor_color(zone)),
        Tile::Exit => exit_glow(elapsed),
        Tile::Obstacle => obstacle_display(elapsed),
        Tile::Torchlight => ('i', RGB::named(YELLOW)),
    };

    let fg = if vis == VisibilityState::Explored { dim_color(color, 0.5) } else { color };

    Some((glyph, fg))
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{minutes:02}:{seconds:02}")
}

pub fn phase_definitions() -> Vec<(Zone, &'static [VimMotion])> {
    const ZONE1: &[VimMotion] = &[VimMotion::H, VimMotion::J, VimMotion::K, VimMotion::L];
    const ZONE2: &[VimMotion] = &[VimMotion::W, VimMotion::B];
    const ZONE3: &[VimMotion] =
        &[VimMotion::Zero, VimMotion::Dollar, VimMotion::G, VimMotion::GotoLine];
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

pub fn zone_wall_color(zone: Zone) -> RGB {
    match zone {
        Zone::Zone1 => rgb8(170, 175, 185),
        Zone::Zone2 => rgb8(100, 160, 230),
        Zone::Zone3 => rgb8(190, 130, 220),
        Zone::Zone4 => rgb8(220, 100, 100),
        Zone::Zone5 => rgb8(220, 210, 80),
    }
}

pub fn zone_floor_color(zone: Zone) -> RGB {
    match zone {
        Zone::Zone1 => rgb8(160, 165, 170),
        Zone::Zone2 => RGB::named(CYAN),
        Zone::Zone3 => rgb8(255, 100, 255),
        Zone::Zone4 => rgb8(255, 100, 100),
        Zone::Zone5 => RGB::named(YELLOW),
    }
}

pub fn zone_accent_color(zone: Zone) -> RGB {
    match zone {
        Zone::Zone1 => RGB::named(WHITE),
        Zone::Zone2 => RGB::named(BLUE),
        Zone::Zone3 => RGB::named(MAGENTA),
        Zone::Zone4 => RGB::named(RED),
        Zone::Zone5 => RGB::named(YELLOW),
    }
}

pub fn wall_display_glyph(x: usize, y: usize, map: &crate::map::Map) -> char {
    let mut non_wall_neighbors = 0u8;
    let directions: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for (dx, dy) in directions {
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        if nx >= 0 && ny >= 0 {
            let nx = nx as usize;
            let ny = ny as usize;
            if nx < map.width && ny < map.height && !matches!(map.get_tile(nx, ny), Tile::Wall) {
                non_wall_neighbors += 1;
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

pub fn exit_glow(elapsed: std::time::Duration) -> (char, RGB) {
    let phase = (elapsed.as_millis() % 1000) as f64 / 1000.0;
    let pulse = (phase * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5;
    let r = (200.0 + 55.0 * pulse) as u8;
    let g = (200.0 + 55.0 * pulse) as u8;
    let glyph = if pulse > 0.5 { '►' } else { '>' };
    (glyph, rgb8(r, g, 0))
}

pub fn trail_color(index: usize, total: usize) -> (char, RGB) {
    let fade = if total <= 1 { 1.0 } else { 1.0 - (index as f64 / (total as f64 - 1.0)) * 0.7 };
    let g = (230.0 * fade) as u8;
    ('·', rgb8(60, g, 60))
}

pub fn obstacle_display(elapsed: std::time::Duration) -> (char, RGB) {
    let visible = (elapsed.as_millis() % 1000) < 500;
    if visible { ('▒', rgb8(255, 100, 100)) } else { (' ', RGB::named(BLACK)) }
}

pub fn attack_effect_display(kind: AttackEffectKind, progress: f64) -> (char, RGB) {
    match kind {
        AttackEffectKind::PlayerStrike => {
            if progress < 0.5 {
                ('*', rgb8(255, 255, 100))
            } else {
                ('/', rgb8(200, 180, 50))
            }
        }
        AttackEffectKind::EnemyHit => {
            if progress < 0.5 {
                ('!', RGB::named(RED))
            } else {
                ('·', rgb8(180, 40, 40))
            }
        }
    }
}

pub fn motion_mastery(total_discovered: usize) -> (&'static str, RGB) {
    match total_discovered {
        0..=3 => ("Novice", rgb8(205, 127, 50)),
        4..=6 => ("Apprentice", rgb8(192, 192, 192)),
        7..=9 => ("Journeyman", rgb8(255, 215, 0)),
        10..=11 => ("Adept", rgb8(100, 200, 255)),
        12 => ("Expert", rgb8(200, 100, 255)),
        _ => ("Master", rgb8(255, 255, 100)),
    }
}
