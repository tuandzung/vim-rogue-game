mod common;

use common::{all_transparent, with_transparent_tiles, with_walls};
use vim_rogue::types::Position;
use vim_rogue::visibility::{VisibilityMap, VisibilityState};

#[test]
fn new_map_all_hidden() {
    let vis = VisibilityMap::new(80, 40);

    for y in 0..40 {
        for x in 0..80 {
            assert_eq!(
                vis.get(Position { x, y }),
                VisibilityState::Hidden,
                "Tile at ({x},{y}) should be Hidden"
            );
        }
    }
}

#[test]
fn fov_center_visible() {
    let mut vis = VisibilityMap::new(80, 40);
    let center = Position { x: 40, y: 20 };

    vis.compute_fov(center, 5, all_transparent);

    assert_eq!(vis.get(center), VisibilityState::Visible);

    assert_eq!(vis.get(Position { x: 41, y: 20 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 39, y: 20 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 40, y: 21 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 40, y: 19 }), VisibilityState::Visible);
}

#[test]
fn fov_wall_blocking() {
    let mut vis = VisibilityMap::new(80, 40);
    let center = Position { x: 10, y: 20 };
    let wall = Position { x: 13, y: 20 };
    vis.compute_fov(center, 10, with_walls(&[wall]));

    assert_eq!(vis.get(wall), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 14, y: 20 }), VisibilityState::Hidden);
}

#[test]
fn fov_radius_respected() {
    let mut vis = VisibilityMap::new(80, 40);
    let center = Position { x: 40, y: 20 };
    let radius = 3;

    vis.compute_fov(center, radius, all_transparent);

    assert_eq!(vis.get(Position { x: 44, y: 20 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 43, y: 20 }), VisibilityState::Visible);
}

#[test]
fn explored_persists() {
    let mut vis = VisibilityMap::new(80, 40);
    let center1 = Position { x: 40, y: 20 };
    let center2 = Position { x: 60, y: 20 };

    vis.compute_fov(center1, 5, all_transparent);
    assert_eq!(vis.get(Position { x: 41, y: 20 }), VisibilityState::Visible);

    vis.demote_visible_to_explored();
    assert_eq!(vis.get(Position { x: 41, y: 20 }), VisibilityState::Explored);

    vis.compute_fov(center2, 5, all_transparent);

    assert_eq!(vis.get(Position { x: 41, y: 20 }), VisibilityState::Explored);

    assert_eq!(vis.get(center2), VisibilityState::Visible);
}

#[test]
fn reset_clears_all() {
    let mut vis = VisibilityMap::new(80, 40);
    let center = Position { x: 40, y: 20 };

    vis.compute_fov(center, 10, all_transparent);
    vis.demote_visible_to_explored();

    assert_eq!(vis.get(Position { x: 41, y: 20 }), VisibilityState::Explored);

    vis.reset();

    for y in 0..40 {
        for x in 0..80 {
            assert_eq!(
                vis.get(Position { x, y }),
                VisibilityState::Hidden,
                "Tile at ({x},{y}) should be Hidden after reset"
            );
        }
    }
}

#[test]
fn out_of_bounds_returns_hidden() {
    let vis = VisibilityMap::new(80, 40);

    assert_eq!(vis.get(Position { x: 80, y: 0 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 0, y: 40 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 100, y: 100 }), VisibilityState::Hidden);
}

#[test]
fn demote_preserves_explored() {
    let mut vis = VisibilityMap::new(80, 40);

    let explored_pos = Position { x: 10, y: 10 };
    let hidden_pos = Position { x: 20, y: 20 };
    let visible_pos = Position { x: 30, y: 30 };

    vis.set(explored_pos, VisibilityState::Explored);
    vis.set(hidden_pos, VisibilityState::Hidden);
    vis.set(visible_pos, VisibilityState::Visible);

    vis.demote_visible_to_explored();

    assert_eq!(vis.get(explored_pos), VisibilityState::Explored);
    assert_eq!(vis.get(hidden_pos), VisibilityState::Hidden);
    assert_eq!(vis.get(visible_pos), VisibilityState::Explored);
}

#[test]
fn fov_corner_of_map() {
    let mut vis = VisibilityMap::new(80, 40);

    vis.compute_fov(Position { x: 0, y: 0 }, 5, all_transparent);

    assert_eq!(vis.get(Position { x: 0, y: 0 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 1, y: 0 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 0, y: 1 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 80, y: 0 }), VisibilityState::Hidden);
}

#[test]
fn fov_open_area_reveals_cardinal_and_diagonal_tiles() {
    let mut vis = VisibilityMap::new(25, 25);
    let center = Position { x: 12, y: 12 };

    vis.compute_fov(center, 10, all_transparent);

    assert_eq!(vis.get(Position { x: 22, y: 12 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 2, y: 12 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 12, y: 22 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 12, y: 2 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 19, y: 19 }), VisibilityState::Visible);
}

#[test]
fn fov_radius_ten_includes_boundary_but_excludes_radius_eleven() {
    let mut vis = VisibilityMap::new(30, 30);
    let center = Position { x: 15, y: 15 };

    vis.compute_fov(center, 10, all_transparent);

    assert_eq!(vis.get(Position { x: 25, y: 15 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 15, y: 25 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 26, y: 15 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 15, y: 26 }), VisibilityState::Hidden);
}

#[test]
fn fov_corridor_layout_stays_inside_long_passage() {
    let mut vis = VisibilityMap::new(21, 7);
    let center = Position { x: 10, y: 3 };
    let mut corridor_tiles = Vec::new();

    for x in 1..20 {
        corridor_tiles.push(Position { x, y: 3 });
    }

    vis.compute_fov(center, 10, with_transparent_tiles(&corridor_tiles));

    assert_eq!(vis.get(Position { x: 19, y: 3 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 10, y: 2 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 10, y: 1 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 0, y: 3 }), VisibilityState::Visible);
}

#[test]
fn fov_room_opening_allows_visibility_through_doorway() {
    let mut vis = VisibilityMap::new(20, 15);
    let center = Position { x: 9, y: 8 };
    let mut transparent_tiles = Vec::new();

    for y in 5..10 {
        for x in 5..14 {
            transparent_tiles.push(Position { x, y });
        }
    }

    transparent_tiles.extend([
        Position { x: 9, y: 4 },
        Position { x: 9, y: 3 },
        Position { x: 9, y: 2 },
        Position { x: 9, y: 1 },
    ]);

    vis.compute_fov(center, 10, with_transparent_tiles(&transparent_tiles));

    assert_eq!(vis.get(Position { x: 9, y: 4 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 9, y: 1 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 7, y: 4 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 7, y: 3 }), VisibilityState::Hidden);
}

#[test]
fn explored_tiles_persist_after_nearby_recompute() {
    let mut vis = VisibilityMap::new(30, 20);

    vis.compute_fov(Position { x: 10, y: 10 }, 3, all_transparent);
    vis.demote_visible_to_explored();
    vis.compute_fov(Position { x: 12, y: 10 }, 3, all_transparent);

    assert_eq!(vis.get(Position { x: 7, y: 10 }), VisibilityState::Explored);
    assert_eq!(vis.get(Position { x: 15, y: 10 }), VisibilityState::Visible);
}

#[test]
fn multiple_sequential_fov_updates_leave_old_tiles_explored() {
    let mut vis = VisibilityMap::new(30, 20);

    for x in 10..=15 {
        if x > 10 {
            vis.demote_visible_to_explored();
        }
        vis.compute_fov(Position { x, y: 10 }, 3, all_transparent);
    }

    assert_eq!(vis.get(Position { x: 10, y: 10 }), VisibilityState::Explored);
    assert_eq!(vis.get(Position { x: 15, y: 10 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 18, y: 10 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 6, y: 10 }), VisibilityState::Hidden);
}

#[test]
fn reset_clears_explored_and_visible_tiles_after_multiple_passes() {
    let mut vis = VisibilityMap::new(30, 20);

    vis.compute_fov(Position { x: 10, y: 10 }, 3, all_transparent);
    vis.demote_visible_to_explored();
    vis.compute_fov(Position { x: 13, y: 10 }, 3, all_transparent);
    vis.reset();

    assert_eq!(vis.get(Position { x: 10, y: 10 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 13, y: 10 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 16, y: 10 }), VisibilityState::Hidden);
}

#[test]
fn l_shaped_corridor_hides_tiles_around_corner() {
    let mut vis = VisibilityMap::new(15, 15);
    let center = Position { x: 4, y: 4 };
    let mut transparent_tiles = Vec::new();

    for y in 2..11 {
        transparent_tiles.push(Position { x: 4, y });
    }
    for x in 4..13 {
        transparent_tiles.push(Position { x, y: 10 });
    }

    vis.compute_fov(center, 10, with_transparent_tiles(&transparent_tiles));

    assert_eq!(vis.get(Position { x: 4, y: 10 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 8, y: 10 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 5, y: 9 }), VisibilityState::Visible);
}

#[test]
fn explored_walls_remain_after_moving_away() {
    let mut vis = VisibilityMap::new(20, 20);
    let wall = Position { x: 8, y: 5 };

    vis.compute_fov(Position { x: 5, y: 5 }, 5, with_walls(&[wall]));
    vis.demote_visible_to_explored();
    vis.compute_fov(Position { x: 5, y: 12 }, 2, with_walls(&[wall]));

    assert_eq!(vis.get(wall), VisibilityState::Explored);
    assert_eq!(vis.get(Position { x: 9, y: 5 }), VisibilityState::Hidden);
}

#[test]
fn recompute_after_move_demotes_old_visible_and_reveals_new_tiles() {
    let mut vis = VisibilityMap::new(30, 20);

    vis.compute_fov(Position { x: 10, y: 10 }, 2, all_transparent);
    vis.demote_visible_to_explored();
    vis.compute_fov(Position { x: 13, y: 10 }, 2, all_transparent);

    assert_eq!(vis.get(Position { x: 8, y: 10 }), VisibilityState::Explored);
    assert_eq!(vis.get(Position { x: 15, y: 10 }), VisibilityState::Visible);
}

#[test]
fn visibility_is_symmetric_in_open_space() {
    let a = Position { x: 10, y: 10 };
    let b = Position { x: 16, y: 14 };
    let mut from_a = VisibilityMap::new(30, 30);
    let mut from_b = VisibilityMap::new(30, 30);

    from_a.compute_fov(a, 10, all_transparent);
    from_b.compute_fov(b, 10, all_transparent);

    assert_eq!(from_a.get(b), VisibilityState::Visible);
    assert_eq!(from_b.get(a), VisibilityState::Visible);
}

#[test]
fn zero_radius_only_reveals_center_tile() {
    let mut vis = VisibilityMap::new(10, 10);
    let center = Position { x: 5, y: 5 };

    vis.compute_fov(center, 0, all_transparent);

    assert_eq!(vis.get(center), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 6, y: 5 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 5, y: 6 }), VisibilityState::Hidden);
}

#[test]
fn negative_radius_is_clamped_to_zero() {
    let mut vis = VisibilityMap::new(10, 10);
    let center = Position { x: 5, y: 5 };

    vis.compute_fov(center, -3, all_transparent);

    assert_eq!(vis.get(center), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 4, y: 5 }), VisibilityState::Hidden);
    assert_eq!(vis.get(Position { x: 5, y: 4 }), VisibilityState::Hidden);
}

#[test]
fn fov_top_right_corner_handles_map_edges() {
    let mut vis = VisibilityMap::new(20, 20);

    vis.compute_fov(Position { x: 19, y: 0 }, 10, all_transparent);

    assert_eq!(vis.get(Position { x: 19, y: 0 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 18, y: 0 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 19, y: 1 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 20, y: 0 }), VisibilityState::Hidden);
}

#[test]
fn fov_bottom_edge_center_handles_map_edges() {
    let mut vis = VisibilityMap::new(21, 20);

    vis.compute_fov(Position { x: 10, y: 19 }, 10, all_transparent);

    assert_eq!(vis.get(Position { x: 10, y: 19 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 10, y: 9 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 20, y: 19 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 10, y: 20 }), VisibilityState::Hidden);
}

#[test]
fn wall_at_exact_radius_is_visible_but_tiles_behind_it_are_hidden() {
    let mut vis = VisibilityMap::new(25, 15);
    let wall = Position { x: 15, y: 7 };

    vis.compute_fov(Position { x: 5, y: 7 }, 10, with_walls(&[wall]));

    assert_eq!(vis.get(wall), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 16, y: 7 }), VisibilityState::Hidden);
}

#[test]
fn multi_source_fov_unions_visibility_from_two_sources() {
    let mut vis = VisibilityMap::new(40, 40);
    vis.compute_multi_fov(
        &[(Position { x: 10, y: 10 }, 6), (Position { x: 30, y: 10 }, 6)],
        all_transparent,
    );

    assert_eq!(vis.get(Position { x: 10, y: 10 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 30, y: 10 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 12, y: 10 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 32, y: 10 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 20, y: 10 }), VisibilityState::Hidden);
}

#[test]
fn multi_source_fov_with_empty_sources_does_nothing() {
    let mut vis = VisibilityMap::new(40, 40);
    vis.compute_multi_fov(&[], all_transparent);

    assert_eq!(vis.get(Position { x: 20, y: 20 }), VisibilityState::Hidden);
}

#[test]
fn torchlight_fov_persists_when_player_moves_away() {
    let mut vis = VisibilityMap::new(40, 40);

    vis.compute_fov(Position { x: 5, y: 5 }, 10, all_transparent);
    vis.compute_multi_fov(&[(Position { x: 20, y: 20 }, 6)], all_transparent);

    vis.demote_visible_to_explored();
    vis.compute_multi_fov(&[(Position { x: 20, y: 20 }, 6)], all_transparent);

    assert_eq!(vis.get(Position { x: 20, y: 20 }), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 5, y: 5 }), VisibilityState::Explored);
}

#[test]
fn multi_source_fov_respects_walls() {
    let wall = Position { x: 13, y: 10 };
    let mut vis = VisibilityMap::new(30, 20);
    vis.compute_multi_fov(&[(Position { x: 10, y: 10 }, 6)], with_walls(&[wall]));

    assert_eq!(vis.get(wall), VisibilityState::Visible);
    assert_eq!(vis.get(Position { x: 14, y: 10 }), VisibilityState::Hidden);
}
