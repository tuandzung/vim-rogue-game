mod common;

use common::{assert_approx_eq, tick_state, tick_timer};
use vim_rogue::animation::*;

#[test]
fn timer_at_zero_is_zero_progress() {
    let timer = AnimationTimer::new(100.0);
    assert_eq!(timer.progress(), 0.0);
    assert!(!timer.is_complete());
}

#[test]
fn timer_at_duration_is_complete() {
    let mut timer = AnimationTimer::new(100.0);
    timer.update(100.0);
    assert_eq!(timer.progress(), 1.0);
    assert!(timer.is_complete());
}

#[test]
fn timer_progress_clamps_at_one() {
    let mut timer = AnimationTimer::new(100.0);
    timer.update(250.0);
    assert_eq!(timer.progress(), 1.0);
}

#[test]
fn interpolator_linear_midpoint() {
    assert_eq!(Interpolator::linear(0.5), 0.5);
}

#[test]
fn interpolator_ease_in_out_starts_slow() {
    assert!(Interpolator::ease_in_out(0.25) < 0.25);
}

#[test]
fn interpolator_ease_in_out_ends_fast() {
    assert!(Interpolator::ease_in_out(0.75) > 0.75);
}

#[test]
fn test_clock_deterministic() {
    let mut first = TestClock::new();
    let mut second = TestClock::new();

    for delta in [16.0, 32.0, 8.0] {
        first.tick(delta);
        second.tick(delta);
    }

    assert_eq!(first.elapsed_ms(), second.elapsed_ms());
}

#[test]
fn animation_state_start_position() {
    let state = AnimationState::new(100.0, (2.0, 3.0), (8.0, 9.0));
    assert_eq!(state.current_position(), (2.0, 3.0));
}

#[test]
fn animation_state_end_position() {
    let mut state = AnimationState::new(100.0, (2.0, 3.0), (8.0, 9.0));
    state.update(100.0);
    assert_eq!(state.current_position(), (8.0, 9.0));
}

#[test]
fn animation_state_midpoint_interpolation() {
    let mut state = AnimationState::new(100.0, (2.0, 3.0), (8.0, 9.0));
    state.update(50.0);
    assert_eq!(state.current_position(), (5.0, 6.0));
}

#[test]
fn timer_progresses_with_small_delta_from_test_clock() {
    let mut clock = TestClock::new();
    let mut timer = AnimationTimer::new(100.0);

    tick_timer(&mut timer, &mut clock, 10.0);

    assert_approx_eq(clock.elapsed_ms(), 10.0);
    assert_approx_eq(timer.progress(), 0.1);
    assert!(!timer.is_complete());
}

#[test]
fn timer_progresses_with_medium_delta_from_test_clock() {
    let mut clock = TestClock::new();
    let mut timer = AnimationTimer::new(200.0);

    tick_timer(&mut timer, &mut clock, 100.0);

    assert_approx_eq(clock.elapsed_ms(), 100.0);
    assert_approx_eq(timer.progress(), 0.5);
    assert!(!timer.is_complete());
}

#[test]
fn timer_progresses_with_large_delta_from_test_clock() {
    let mut clock = TestClock::new();
    let mut timer = AnimationTimer::new(120.0);

    tick_timer(&mut timer, &mut clock, 90.0);

    assert_approx_eq(clock.elapsed_ms(), 90.0);
    assert_approx_eq(timer.progress(), 0.75);
    assert!(!timer.is_complete());
}

#[test]
fn timer_exactly_reaches_complete_progress_at_duration_boundary() {
    let mut clock = TestClock::new();
    let mut timer = AnimationTimer::new(100.0);

    tick_timer(&mut timer, &mut clock, 30.0);
    tick_timer(&mut timer, &mut clock, 20.0);
    tick_timer(&mut timer, &mut clock, 50.0);

    assert_approx_eq(clock.elapsed_ms(), 100.0);
    assert_eq!(timer.progress(), 1.0);
    assert!(timer.is_complete());
}

#[test]
fn timer_overshoot_with_multiple_large_deltas_stays_clamped() {
    let mut clock = TestClock::new();
    let mut timer = AnimationTimer::new(100.0);

    tick_timer(&mut timer, &mut clock, 80.0);
    tick_timer(&mut timer, &mut clock, 60.0);
    tick_timer(&mut timer, &mut clock, 40.0);

    assert_approx_eq(clock.elapsed_ms(), 180.0);
    assert_eq!(timer.progress(), 1.0);
    assert!(timer.is_complete());
}

#[test]
fn interpolator_linear_matches_endpoints_and_midpoint() {
    let mut clock = TestClock::new();

    clock.tick(0.0);

    assert_eq!(clock.elapsed_ms(), 0.0);
    assert_eq!(Interpolator::linear(0.0), 0.0);
    assert_eq!(Interpolator::linear(0.5), 0.5);
    assert_eq!(Interpolator::linear(1.0), 1.0);
}

#[test]
fn interpolator_ease_in_out_matches_known_checkpoints() {
    let mut clock = TestClock::new();

    clock.tick(25.0);

    assert_eq!(clock.elapsed_ms(), 25.0);
    assert_eq!(Interpolator::ease_in_out(0.0), 0.0);
    assert_eq!(Interpolator::ease_in_out(0.25), 0.15625);
    assert_eq!(Interpolator::ease_in_out(0.5), 0.5);
    assert_eq!(Interpolator::ease_in_out(0.75), 0.84375);
    assert_eq!(Interpolator::ease_in_out(1.0), 1.0);
}

#[test]
fn interpolator_ease_in_out_is_symmetric_around_midpoint() {
    let mut clock = TestClock::new();

    clock.tick(50.0);

    let early = Interpolator::ease_in_out(0.25);
    let late = Interpolator::ease_in_out(0.75);

    assert_eq!(clock.elapsed_ms(), 50.0);
    assert_approx_eq(early, 1.0 - late);
}

#[test]
fn test_clock_same_fixed_delta_produces_same_elapsed() {
    let mut first = TestClock::new();
    let mut second = TestClock::new();

    for _ in 0..4 {
        first.tick(16.0);
        second.tick(16.0);
    }

    assert_eq!(first.elapsed_ms(), 64.0);
    assert_eq!(first.elapsed_ms(), second.elapsed_ms());
}

#[test]
fn test_clock_cumulative_ticks_match_total_elapsed() {
    let mut clock = TestClock::new();

    clock.tick(5.0);
    clock.tick(15.0);
    clock.tick(30.0);

    assert_eq!(clock.elapsed_ms(), 50.0);
}

#[test]
fn test_clock_zero_delta_keeps_elapsed_at_zero() {
    let mut clock = TestClock::new();

    clock.tick(0.0);

    assert_eq!(clock.elapsed_ms(), 0.0);
}

#[test]
fn animation_state_creation_preserves_start_and_end_positions() {
    let mut clock = TestClock::new();
    let state = AnimationState::new(160.0, (1.0, 2.0), (7.0, 8.0));

    clock.tick(0.0);

    assert_eq!(clock.elapsed_ms(), 0.0);
    assert_eq!(state.current_position(), (1.0, 2.0));
    assert_eq!((state.start_x, state.start_y), (1.0, 2.0));
    assert_eq!((state.end_x, state.end_y), (7.0, 8.0));
}

#[test]
fn animation_state_update_tracks_quarter_progress_position() {
    let mut clock = TestClock::new();
    let mut state = AnimationState::new(100.0, (1.0, 2.0), (5.0, 10.0));

    tick_state(&mut state, &mut clock, 25.0);

    let (x, y) = state.current_position();
    assert_approx_eq(x, 1.625);
    assert_approx_eq(y, 3.25);
}

#[test]
fn animation_state_update_tracks_halfway_position() {
    let mut clock = TestClock::new();
    let mut state = AnimationState::new(100.0, (1.0, 2.0), (5.0, 10.0));

    tick_state(&mut state, &mut clock, 50.0);

    assert_eq!(state.current_position(), (3.0, 6.0));
}

#[test]
fn animation_state_update_tracks_three_quarter_position() {
    let mut clock = TestClock::new();
    let mut state = AnimationState::new(100.0, (1.0, 2.0), (5.0, 10.0));

    tick_state(&mut state, &mut clock, 75.0);

    let (x, y) = state.current_position();
    assert_approx_eq(x, 4.375);
    assert_approx_eq(y, 8.75);
}

#[test]
fn animation_state_is_complete_when_progress_reaches_one() {
    let mut clock = TestClock::new();
    let mut state = AnimationState::new(120.0, (2.0, 4.0), (8.0, 10.0));

    tick_state(&mut state, &mut clock, 120.0);

    assert!(state.is_complete());
    assert_eq!(state.current_position(), (8.0, 10.0));
}

#[test]
fn animation_state_current_position_is_exact_start_at_zero_progress() {
    let mut clock = TestClock::new();
    let state = AnimationState::new(100.0, (3.0, 4.0), (9.0, 12.0));

    clock.tick(0.0);

    assert_eq!(clock.elapsed_ms(), 0.0);
    assert_eq!(state.current_position(), (3.0, 4.0));
}

#[test]
fn animation_state_current_position_is_exact_end_at_full_progress() {
    let mut clock = TestClock::new();
    let mut state = AnimationState::new(100.0, (3.0, 4.0), (9.0, 12.0));

    tick_state(&mut state, &mut clock, 100.0);

    assert_eq!(state.current_position(), (9.0, 12.0));
}

#[test]
fn animation_states_can_run_sequentially_without_sharing_elapsed_time() {
    let mut clock = TestClock::new();
    let mut first = AnimationState::new(100.0, (0.0, 0.0), (4.0, 0.0));

    tick_state(&mut first, &mut clock, 100.0);
    assert!(first.is_complete());
    assert_eq!(first.current_position(), (4.0, 0.0));

    let mut second = AnimationState::new(80.0, (4.0, 0.0), (4.0, 6.0));
    tick_state(&mut second, &mut clock, 40.0);

    assert_eq!(clock.elapsed_ms(), 140.0);
    assert!(!second.is_complete());
    assert_eq!(second.current_position(), (4.0, 3.0));

    tick_state(&mut second, &mut clock, 40.0);

    assert!(second.is_complete());
    assert_eq!(second.current_position(), (4.0, 6.0));
}

#[test]
fn attack_effect_starts_at_zero_progress() {
    let effect = AttackEffect::new(AttackEffectKind::PlayerStrike, 5, 10);
    assert_eq!(effect.timer.progress(), 0.0);
    assert!(!effect.is_complete());
}

#[test]
fn attack_effect_completes_after_duration() {
    let mut effect = AttackEffect::new(AttackEffectKind::PlayerStrike, 5, 10);
    effect.update(ATTACK_EFFECT_MS);
    assert!(effect.is_complete());
}

#[test]
fn attack_effect_progress_midpoint() {
    let mut effect = AttackEffect::new(AttackEffectKind::EnemyHit, 3, 7);
    effect.update(ATTACK_EFFECT_MS / 2.0);
    assert_approx_eq(effect.timer.progress(), 0.5);
    assert!(!effect.is_complete());
}

#[test]
fn attack_effect_preserves_position() {
    let effect = AttackEffect::new(AttackEffectKind::PlayerStrike, 12, 34);
    assert_eq!(effect.x, 12);
    assert_eq!(effect.y, 34);
}

#[test]
fn attack_effect_kind_is_stored() {
    let strike = AttackEffect::new(AttackEffectKind::PlayerStrike, 0, 0);
    let hit = AttackEffect::new(AttackEffectKind::EnemyHit, 0, 0);
    assert_eq!(strike.kind, AttackEffectKind::PlayerStrike);
    assert_eq!(hit.kind, AttackEffectKind::EnemyHit);
}
