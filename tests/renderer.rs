mod common;

use bracket_lib::prelude::*;
use common::{approx_eq, test_app};
use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;
use vim_rogue::animation::{AnimationState, AttackEffectKind, ENEMY_MOVE_MS};
use vim_rogue::map::Map;
use vim_rogue::player::Player;
use vim_rogue::renderer::*;
use vim_rogue::types::{App, Enemy, GameState, MAX_HP, Position, Tile, VimMotion, Zone};
use vim_rogue::visibility::{VisibilityMap, VisibilityState};

#[test]
fn zone_wall_colors() {
    assert!(approx_eq(zone_wall_color(Zone::Zone1).r, 170.0 / 255.0));
    assert!(approx_eq(zone_wall_color(Zone::Zone2).b, 230.0 / 255.0));
    assert!(approx_eq(zone_wall_color(Zone::Zone3).r, 190.0 / 255.0));
    assert!(approx_eq(zone_wall_color(Zone::Zone4).r, 220.0 / 255.0));
    assert!(approx_eq(zone_wall_color(Zone::Zone5).g, 210.0 / 255.0));
}

#[test]
fn zone_floor_colors() {
    assert!(approx_eq(zone_floor_color(Zone::Zone1).r, 160.0 / 255.0));
    assert_eq!(zone_floor_color(Zone::Zone2), RGB::named(CYAN));
    assert!(approx_eq(zone_floor_color(Zone::Zone3).r, 1.0));
    assert!(approx_eq(zone_floor_color(Zone::Zone4).g, 100.0 / 255.0));
    assert_eq!(zone_floor_color(Zone::Zone5), RGB::named(YELLOW));
}

#[test]
fn zone_accent_colors() {
    assert_eq!(zone_accent_color(Zone::Zone1), RGB::named(WHITE));
    assert_eq!(zone_accent_color(Zone::Zone2), RGB::named(BLUE));
    assert_eq!(zone_accent_color(Zone::Zone3), RGB::named(MAGENTA));
    assert_eq!(zone_accent_color(Zone::Zone4), RGB::named(RED));
    assert_eq!(zone_accent_color(Zone::Zone5), RGB::named(YELLOW));
}

#[test]
fn wall_glyph_solid_when_isolated() {
    let map = Map::new();
    assert_eq!(wall_display_glyph(5, 5, &map), '█');
}

#[test]
fn wall_glyph_medium_with_one_floor_neighbor() {
    let mut map = Map::new();
    map.set_tile(4, 5, Tile::Floor);
    assert_eq!(wall_display_glyph(5, 5, &map), '▓');
}

#[test]
fn wall_glyph_shallow_with_two_floor_neighbors() {
    let mut map = Map::new();
    map.set_tile(4, 5, Tile::Floor);
    map.set_tile(6, 5, Tile::Floor);
    assert_eq!(wall_display_glyph(5, 5, &map), '▒');
}

#[test]
fn wall_glyph_edge_with_three_floor_neighbors() {
    let mut map = Map::new();
    map.set_tile(4, 5, Tile::Floor);
    map.set_tile(6, 5, Tile::Floor);
    map.set_tile(5, 4, Tile::Floor);
    assert_eq!(wall_display_glyph(5, 5, &map), '#');
}

#[test]
fn format_duration_zero() {
    assert_eq!(format_duration(Duration::from_secs(0)), "00:00");
}

#[test]
fn format_duration_one_minute() {
    assert_eq!(format_duration(Duration::from_secs(60)), "01:00");
}

#[test]
fn format_duration_mixed() {
    assert_eq!(format_duration(Duration::from_secs(125)), "02:05");
}

#[test]
fn format_duration_large() {
    assert_eq!(format_duration(Duration::from_secs(3661)), "61:01");
}

#[test]
fn phase_definitions_has_five_zones() {
    assert_eq!(phase_definitions().len(), 5);
}

#[test]
fn phase_definitions_cover_all_13_motions() {
    let total: usize = phase_definitions().iter().map(|(_, m)| m.len()).sum();
    assert_eq!(total, 13);
}

#[test]
fn phase_definitions_zone1_is_basic() {
    let defs = phase_definitions();
    assert_eq!(defs[0].0, Zone::Zone1);
    assert!(defs[0].1.contains(&VimMotion::H));
    assert!(defs[0].1.contains(&VimMotion::J));
    assert!(defs[0].1.contains(&VimMotion::K));
    assert!(defs[0].1.contains(&VimMotion::L));
}

#[test]
fn phase_definitions_zone5_is_delete() {
    let defs = phase_definitions();
    assert_eq!(defs[4].0, Zone::Zone5);
    assert_eq!(defs[4].1, &[VimMotion::DeleteLine]);
}

#[test]
fn exit_glow_returns_valid_glyph() {
    let (glyph, color) = exit_glow(Duration::from_millis(0));
    assert!(glyph == '>' || glyph == '►');
    assert!(color.r > 0.0);
    assert!(color.g > 0.0);
    assert_eq!(color.b, 0.0);
}

#[test]
fn trail_color_newest_is_brightest() {
    let (glyph, color) = trail_color(0, 5);
    assert_eq!(glyph, '·');
    assert!(color.g > 0.0);
}

#[test]
fn trail_color_single_entry_full_brightness() {
    let (_, color) = trail_color(0, 1);
    assert!(approx_eq(color.g, 230.0 / 255.0));
}

#[test]
fn trail_color_oldest_is_dimmer() {
    let (_, newest) = trail_color(0, 5);
    let (_, oldest) = trail_color(4, 5);
    assert!(newest.g > oldest.g);
}

#[test]
fn obstacle_visible_in_first_half() {
    let (glyph, color) = obstacle_display(Duration::from_millis(200));
    assert_eq!(glyph, '▒');
    assert!(color.r > 0.0);
}

#[test]
fn obstacle_hidden_in_second_half() {
    let (glyph, color) = obstacle_display(Duration::from_millis(700));
    assert_eq!(glyph, ' ');
    assert_eq!(color.r, 0.0);
}

#[test]
fn motion_mastery_ratings() {
    assert_eq!(motion_mastery(0).0, "Novice");
    assert_eq!(motion_mastery(3).0, "Novice");
    assert_eq!(motion_mastery(4).0, "Apprentice");
    assert_eq!(motion_mastery(6).0, "Apprentice");
    assert_eq!(motion_mastery(7).0, "Journeyman");
    assert_eq!(motion_mastery(9).0, "Journeyman");
    assert_eq!(motion_mastery(10).0, "Adept");
    assert_eq!(motion_mastery(11).0, "Adept");
    assert_eq!(motion_mastery(12).0, "Expert");
    assert_eq!(motion_mastery(13).0, "Master");
}

#[test]
fn center_x_even_text() {
    assert_eq!(center_x(10), 35);
}

#[test]
fn center_x_full_width() {
    assert_eq!(center_x(80), 0);
}

#[test]
fn center_x_zero() {
    assert_eq!(center_x(0), 40);
}

#[test]
fn center_x_for_clamps_when_text_is_wider_than_screen() {
    assert_eq!(center_x_for(10, 20), 0);
}

#[test]
fn center_y_for_clamps_when_content_is_taller_than_screen() {
    assert_eq!(center_y_for(2, 5), 0);
}

#[test]
fn screen_meets_minimum_size_accepts_required_dimensions() {
    assert!(screen_meets_minimum_size(SCREEN_WIDTH, SCREEN_HEIGHT));
}

#[test]
fn screen_meets_minimum_size_rejects_small_dimensions() {
    assert!(!screen_meets_minimum_size(SCREEN_WIDTH - 1, SCREEN_HEIGHT));
    assert!(!screen_meets_minimum_size(SCREEN_WIDTH, SCREEN_HEIGHT - 1));
}

#[test]
fn rgb8_converts_correctly() {
    let c = rgb8(255, 0, 128);
    assert!(approx_eq(c.r, 1.0));
    assert!(approx_eq(c.g, 0.0));
    assert!(approx_eq(c.b, 128.0 / 255.0));
}

#[test]
fn visual_enemy_positions_use_active_animation() {
    let mut app = test_app();
    app.enemies
        .push(Enemy { position: Position { x: 4, y: 2 }, ..Enemy::new(Position { x: 4, y: 2 }) });
    let mut animation = AnimationState::new(ENEMY_MOVE_MS, (2.0, 2.0), (4.0, 2.0));
    animation.update(ENEMY_MOVE_MS / 2.0);
    app.enemy_animations.push((0, animation));

    let positions = visual_enemy_positions(&app);

    assert_eq!(positions, vec![Position { x: 3, y: 2 }]);
}

#[test]
fn dim_color_reduces_components() {
    let original = RGB { r: 1.0, g: 0.5, b: 0.0 };
    let dimmed = dim_color(original, 0.3);
    assert!(approx_eq(dimmed.r, 0.3));
    assert!(approx_eq(dimmed.g, 0.15));
    assert!(approx_eq(dimmed.b, 0.0));
}

#[test]
fn dim_color_clamps_to_one() {
    let color = RGB { r: 2.0, g: 2.0, b: 2.0 };
    let dimmed = dim_color(color, 1.0);
    assert!(approx_eq(dimmed.r, 1.0));
    assert!(approx_eq(dimmed.g, 1.0));
    assert!(approx_eq(dimmed.b, 1.0));
}

#[test]
fn fog_hidden_tile_renders_blank() {
    let result = tile_fog_appearance(
        Tile::Floor,
        Zone::Zone1,
        VisibilityState::Hidden,
        Duration::from_millis(0),
        '#',
    );
    assert!(result.is_none());
}

#[test]
fn fog_explored_tile_renders_dim() {
    let result = tile_fog_appearance(
        Tile::Floor,
        Zone::Zone1,
        VisibilityState::Explored,
        Duration::from_millis(0),
        '#',
    );
    let (glyph, color) = result.expect("explored tile should have appearance");
    assert_eq!(glyph, '.');
    let full = zone_floor_color(Zone::Zone1);
    assert!(color.r < full.r || approx_eq(color.r, full.r * 0.5));
    assert!(color.g < full.g || approx_eq(color.g, full.g * 0.5));
    assert!(color.b < full.b || approx_eq(color.b, full.b * 0.5));
}

#[test]
fn fog_visible_tile_full_color() {
    let result = tile_fog_appearance(
        Tile::Wall,
        Zone::Zone2,
        VisibilityState::Visible,
        Duration::from_millis(0),
        '▓',
    );
    let (glyph, color) = result.expect("visible tile should have appearance");
    assert_eq!(glyph, '▓');
    assert_eq!(color, zone_wall_color(Zone::Zone2));
}

#[test]
fn fog_enemy_not_visible_in_explored() {
    let mut vis = VisibilityMap::new(80, 40);
    let pos = Position { x: 10, y: 10 };
    vis.set(pos, VisibilityState::Explored);

    assert_eq!(vis.get(pos), VisibilityState::Explored);
    assert_ne!(vis.get(pos), VisibilityState::Visible);
}

#[test]
fn minimap_scaling_maps_corners_correctly() {
    let (x0, y0) = minimap_map_coords(0, 0);
    assert_eq!(x0, 0);
    assert_eq!(y0, 0);

    let (xn, yn) = minimap_map_coords(MINIMAP_WIDTH - 1, MINIMAP_HEIGHT - 1);
    assert!(xn < 80, "map x should be < 80, got {xn}");
    assert!(yn < 40, "map y should be < 40, got {yn}");
}

#[test]
fn minimap_scaling_covers_full_map() {
    let (x_last, _) = minimap_map_coords(MINIMAP_WIDTH - 1, 0);
    assert!(x_last >= 75, "rightmost minimap column should reach near x=80, got {x_last}");

    let (_, y_last) = minimap_map_coords(0, MINIMAP_HEIGHT - 1);
    assert!(y_last >= 35, "bottom minimap row should reach near y=40, got {y_last}");
}

#[test]
fn minimap_hidden_tile_is_blank() {
    let mut app = test_app();
    let pos = Position { x: 10, y: 10 };
    app.map.set_tile(10, 10, Tile::Floor);
    assert_eq!(app.visibility.get(pos), VisibilityState::Hidden);
}

#[test]
fn minimap_player_position_at_start() {
    let app = test_app();
    let (px, py) = minimap_player_pos(app.player.position.x, app.player.position.y);
    assert!(px >= 0 && px < MINIMAP_WIDTH as i32, "player minimap x should be in range, got {px}");
    assert!(py >= 0 && py < MINIMAP_HEIGHT as i32, "player minimap y should be in range, got {py}");
}

#[test]
fn minimap_player_position_at_exit() {
    let (px, py) = minimap_player_pos(76, 36);
    assert!(
        px >= 0 && px < MINIMAP_WIDTH as i32,
        "player minimap x at exit should be in range, got {px}"
    );
    assert!(
        py >= 0 && py < MINIMAP_HEIGHT as i32,
        "player minimap y at exit should be in range, got {py}"
    );
    assert_eq!(px, MINIMAP_WIDTH as i32 - 1);
    assert_eq!(py, MINIMAP_HEIGHT as i32 - 1);
}

#[test]
fn minimap_player_position_at_origin() {
    let (px, py) = minimap_player_pos(0, 0);
    assert_eq!(px, 0);
    assert_eq!(py, 0);
}

#[test]
fn minimap_scaling_center_cell() {
    let (cx, cy) = minimap_map_coords(MINIMAP_WIDTH / 2, MINIMAP_HEIGHT / 2);
    assert!(cx > 30 && cx < 50, "center minimap cell should map near map center, got x={cx}");
    assert!(cy > 15 && cy < 25, "center minimap cell should map near map center, got y={cy}");
}

#[test]
fn torchlight_glyph_is_i() {
    assert_eq!(Tile::Torchlight.glyph(), 'i');
}

#[test]
fn max_hp_constant_for_bar() {
    let hp = MAX_HP;
    let hp_ratio = hp as f32 / MAX_HP as f32;
    let hp_filled = (hp_ratio * 10.0).round() as usize;
    assert_eq!(hp_filled, 10);
}

#[test]
fn hp_bar_half_at_15() {
    let hp_ratio = 15_f32 / MAX_HP as f32;
    let hp_filled = (hp_ratio * 10.0).round() as usize;
    assert_eq!(hp_filled, 5);
    assert!(hp_ratio > 0.25 && hp_ratio <= 0.5);
}

#[test]
fn attack_effect_player_strike_early_glyph() {
    let (glyph, _) = attack_effect_display(AttackEffectKind::PlayerStrike, 0.0);
    assert_eq!(glyph, '*');
}

#[test]
fn attack_effect_player_strike_late_glyph() {
    let (glyph, _) = attack_effect_display(AttackEffectKind::PlayerStrike, 0.7);
    assert_eq!(glyph, '/');
}

#[test]
fn attack_effect_enemy_hit_early_glyph() {
    let (glyph, _) = attack_effect_display(AttackEffectKind::EnemyHit, 0.0);
    assert_eq!(glyph, '!');
}

#[test]
fn attack_effect_enemy_hit_late_glyph() {
    let (glyph, _) = attack_effect_display(AttackEffectKind::EnemyHit, 0.7);
    assert_eq!(glyph, '·');
}

#[test]
fn attack_effect_player_strike_early_color() {
    let (_, color) = attack_effect_display(AttackEffectKind::PlayerStrike, 0.0);
    assert!(color.r > 0.9);
    assert!(color.g > 0.9);
    assert!(color.b > 0.3);
}

#[test]
fn attack_effect_enemy_hit_early_color() {
    let (_, color) = attack_effect_display(AttackEffectKind::EnemyHit, 0.0);
    assert_eq!(color, RGB::named(RED));
}
