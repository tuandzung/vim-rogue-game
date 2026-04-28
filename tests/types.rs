use vim_rogue::types::*;

#[test]
fn tile_glyph_wall() {
    assert_eq!(Tile::Wall.glyph(), '#');
}

#[test]
fn tile_glyph_floor() {
    assert_eq!(Tile::Floor.glyph(), '.');
}

#[test]
fn tile_glyph_exit() {
    assert_eq!(Tile::Exit.glyph(), '>');
}

#[test]
fn tile_glyph_obstacle() {
    assert_eq!(Tile::Obstacle.glyph(), '▒');
}

#[test]
fn tile_glyph_torchlight() {
    assert_eq!(Tile::Torchlight.glyph(), 'i');
}

#[test]
fn vim_motion_key_labels() {
    let cases = [
        (VimMotion::H, "h"),
        (VimMotion::J, "j"),
        (VimMotion::K, "k"),
        (VimMotion::L, "l"),
        (VimMotion::W, "w"),
        (VimMotion::B, "b"),
        (VimMotion::Zero, "0"),
        (VimMotion::Dollar, "$"),
        (VimMotion::Find, "f<char>"),
        (VimMotion::Till, "t<char>"),
        (VimMotion::DeleteLine, "dd"),
        (VimMotion::G, "G"),
        (VimMotion::GotoLine, "gg"),
    ];

    for (motion, expected) in cases {
        assert_eq!(motion.key_label(), expected);
    }
}

#[test]
fn zone_titles_exist() {
    let zones = [Zone::Zone1, Zone::Zone2, Zone::Zone3, Zone::Zone4, Zone::Zone5];

    for zone in zones {
        assert!(!zone.title().trim().is_empty());
    }
}

#[test]
fn direction_deltas() {
    assert_eq!(Direction::Left.delta(), (-1, 0));
    assert_eq!(Direction::Down.delta(), (0, 1));
    assert_eq!(Direction::Up.delta(), (0, -1));
    assert_eq!(Direction::Right.delta(), (1, 0));
}

#[test]
fn position_is_copy() {
    let original = Position { x: 3, y: 7 };
    let _copied = original;
    assert_eq!(original.x, 3);
}

#[test]
fn new_motion_display_names() {
    assert_eq!(VimMotion::G.display_name(), "Column bottom");
    assert_eq!(VimMotion::GotoLine.display_name(), "Column top");
}

#[test]
fn new_motion_descriptions_non_empty() {
    assert!(!VimMotion::G.description().is_empty());
    assert!(!VimMotion::GotoLine.description().is_empty());
}

#[test]
fn enemy_struct_fields() {
    let enemy = Enemy {
        position: Position { x: 5, y: 10 },
        glyph: 'X',
        hp: None,
        ..Enemy::new(Position { x: 5, y: 10 })
    };
    assert_eq!(enemy.position.x, 5);
    assert_eq!(enemy.position.y, 10);
    assert_eq!(enemy.glyph, 'X');
    assert_eq!(enemy.hp, None);
}

#[test]
fn enemy_hp_with_value() {
    let enemy = Enemy {
        position: Position { x: 3, y: 4 },
        glyph: 'g',
        hp: Some(5),
        ..Enemy::new(Position { x: 3, y: 4 })
    };
    assert_eq!(enemy.hp, Some(5));
}

#[test]
fn total_levels_is_four() {
    assert_eq!(TOTAL_LEVELS, 4);
}

#[test]
fn max_hp_constant() {
    assert_eq!(MAX_HP, 30);
}

#[test]
fn torchlight_fov_radius_constant() {
    assert_eq!(TORCHLIGHT_FOV_RADIUS, 6);
}

#[test]
fn render_cell_new() {
    let cell = RenderCell::new('@', (1, 2, 3), (4, 5, 6));
    assert_eq!(cell.glyph, '@');
    assert_eq!(cell.fg, (1, 2, 3));
    assert_eq!(cell.bg, (4, 5, 6));
    assert!(!cell.blink);
}

#[test]
fn render_cell_with_blink() {
    let cell = RenderCell::new('@', (1, 2, 3), (4, 5, 6)).with_blink();
    assert!(cell.blink);
}

#[test]
fn render_grid_new_dimensions() {
    let default = RenderCell::new('.', (1, 1, 1), (0, 0, 0));
    let grid = RenderGrid::new(3, 2, default);
    assert_eq!(grid.width(), 3);
    assert_eq!(grid.height(), 2);
}

#[test]
fn render_grid_get_set() {
    let default = RenderCell::new('.', (1, 1, 1), (0, 0, 0));
    let mut grid = RenderGrid::new(2, 2, default);
    let cell = RenderCell::new('@', (10, 20, 30), (40, 50, 60));
    grid.set(1, 0, cell.clone());
    assert_eq!(grid.get(1, 0), &cell);
}

#[test]
fn render_grid_fill() {
    let default = RenderCell::new('.', (1, 1, 1), (0, 0, 0));
    let mut grid = RenderGrid::new(2, 2, default);
    let fill_cell = RenderCell::new('#', (9, 9, 9), (8, 8, 8)).with_blink();
    grid.fill(fill_cell.clone());

    for y in 0..grid.height() {
        for x in 0..grid.width() {
            assert_eq!(grid.get(x, y), &fill_cell);
        }
    }
}

#[test]
fn render_grid_default_cells() {
    let default = RenderCell::new('.', (1, 1, 1), (0, 0, 0));
    let grid = RenderGrid::new(2, 2, default.clone());
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            assert_eq!(grid.get(x, y), &default);
        }
    }
}

#[test]
fn screen_model_covers_all_states() {
    let cases = [
        (ScreenModel::Title, "title"),
        (ScreenModel::Gameplay, "gameplay"),
        (ScreenModel::Win, "win"),
        (ScreenModel::Lost, "lost"),
    ];

    for (screen, expected) in cases {
        let label = match screen {
            ScreenModel::Title => "title",
            ScreenModel::Gameplay => "gameplay",
            ScreenModel::Win => "win",
            ScreenModel::Lost => "lost",
        };
        assert_eq!(label, expected);
    }
}

#[test]
fn view_model_frame_advances() {
    let mut view = ViewModel::new(ScreenModel::Gameplay);
    view.advance_frame();
    view.advance_frame();
    assert_eq!(view.frame_number, 2);
}

#[test]
fn view_model_new_starts_at_frame_zero() {
    let view = ViewModel::new(ScreenModel::Title);
    assert_eq!(view.frame_number, 0);
}
