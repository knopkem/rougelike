//! Headless simulation driver: feeds the same `do_turn`/event path the UI
//! uses, without curses. This is the driver behind the binary's
//! `--headless` flag and the home for the simulation test suite (issue 23).

use crate::core::action::Action;
use crate::core::events::GameEvent;
use crate::core::game::Game;

/// Why a headless run stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnd {
    /// The player died.
    Death,
    /// The player won (amulet raised).
    Victory,
    /// The action source returned `None` before the run ended.
    Exhausted,
    /// `max_turns` turns ran without the run ending.
    TurnLimit,
}

/// Drive the game headlessly, through the exact `Game::do_turn` / event
/// path the interactive UI uses (minus rendering, input, and autosaves):
/// each turn the `next_action` closure supplies an [`Action`], it is
/// resolved by `do_turn`, and every drained event is forwarded to
/// `on_event` (the UI forwards them to the SFX engine; tests can assert
/// on them).
///
/// Stops at the first of: the player dying, the player winning,
/// `next_action` returning `None`, or `max_turns` turns elapsed.
pub fn run_turns<F, E>(
    game: &mut Game,
    max_turns: u64,
    mut next_action: F,
    mut on_event: E,
) -> RunEnd
where
    F: FnMut(u64, &Game) -> Option<Action>,
    E: FnMut(&GameEvent),
{
    for _ in 0..max_turns {
        let action = match next_action(game.turn, game) {
            Some(action) => action,
            None => return RunEnd::Exhausted,
        };
        game.do_turn(action);
        for ev in game.drain_events() {
            on_event(&ev);
        }
        if !game.alive {
            return RunEnd::Death;
        }
        if game.won {
            return RunEnd::Victory;
        }
    }
    RunEnd::TurnLimit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::GameEvent;
    use crate::tests_harness::new_game;

    #[test]
    fn turn_limit_is_reached_when_the_run_survives() {
        let mut game = new_game(42);
        game.monsters.clear();
        let end = run_turns(&mut game, 3, |_turn, _game| Some(Action::Wait), |_ev| {});
        assert_eq!(end, RunEnd::TurnLimit);
        assert!(game.alive && !game.won);
        assert_eq!(game.turn, 3);
    }

    #[test]
    fn exhausted_when_the_action_source_runs_out() {
        let mut game = new_game(42);
        game.monsters.clear();
        let end = run_turns(
            &mut game,
            100,
            |turn, _game| if turn < 2 { Some(Action::Wait) } else { None },
            |_ev| {},
        );
        assert_eq!(end, RunEnd::Exhausted);
        assert_eq!(game.turn, 2);
    }

    #[test]
    fn death_ends_the_run() {
        use crate::tests_harness::with_isolated_data_dir;
        // Death records a hiscore entry, so this test must not share the
        // data dir with the hiscore suite.
        with_isolated_data_dir("sim_death", || {
            death_ends_the_run_inner();
        });
    }

    fn death_ends_the_run_inner() {
        let mut game = new_game(42);
        game.monsters.clear();
        // Starving: no well-fed regen, so the 0 HP is not healed back up.
        game.player.hunger = 0;
        game.player.hp = 0;
        let saw_death = std::cell::Cell::new(false);
        let end = run_turns(
            &mut game,
            10,
            |_turn, _game| Some(Action::Wait),
            |ev| {
                if matches!(ev, GameEvent::PlayerDeath) {
                    saw_death.set(true);
                }
            },
        );
        assert_eq!(end, RunEnd::Death);
        assert!(!game.alive);
        assert!(saw_death.get());
    }

    #[test]
    fn events_are_forwarded_to_the_callback() {
        let mut game = new_game(42);
        game.monsters.clear();
        game.emit(GameEvent::LevelUp);
        let mut events = Vec::new();
        run_turns(
            &mut game,
            1,
            |_turn, _game| Some(Action::Wait),
            |ev| events.push(ev.clone()),
        );
        assert_eq!(game.turn, 1);
        assert!(events.contains(&GameEvent::LevelUp));
    }
}
