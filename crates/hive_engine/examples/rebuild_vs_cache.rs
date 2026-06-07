use std::collections::HashMap;
use std::hint::black_box;
use std::thread;
use std::time::{Duration, Instant};

use hive_engine::{Action, ActionType, Color, Game, PieceType};

const TARGET_HISTORY_SIZES: [usize; 5] = [10, 50, 100, 200, 500];
const GAME_ID: i32 = 1;

struct Fixture {
    history_size: usize,
    actions: Vec<Action>,
    cached_game: Game,
}

fn main() {
    thread::Builder::new()
        .name("rebuild-vs-cache".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .expect("benchmark thread should start")
        .join()
        .expect("benchmark thread should finish");
}

fn run() {
    let fixtures = build_fixtures();

    println!(
        "{:>8} {:>10} {:>16} {:>16} {:>12}",
        "history", "iters", "rebuild avg", "cache avg", "speedup"
    );

    for fixture in fixtures {
        let iterations = iterations_for(fixture.history_size);
        let rebuild = bench(iterations, || {
            let game = rebuild_game(&fixture.actions);
            black_box((game.move_num, game.history.actions.len()));
        });

        let mut cache = HashMap::new();
        cache.insert(GAME_ID, fixture.cached_game.clone());
        let cache_hit = bench(iterations, || {
            let game = cache.get(&GAME_ID).unwrap().clone();
            black_box((game.move_num, game.history.actions.len()));
        });

        println!(
            "{:>8} {:>10} {:>16} {:>16} {:>11.1}x",
            fixture.history_size,
            iterations,
            format_duration(rebuild),
            format_duration(cache_hit),
            rebuild.as_nanos() as f64 / cache_hit.as_nanos().max(1) as f64
        );
    }
}

fn build_fixtures() -> Vec<Fixture> {
    let mut game = Game::new(true, true, true);
    let mut fixtures = Vec::new();
    let mut next_target = 0;

    while next_target < TARGET_HISTORY_SIZES.len() {
        let action = choose_action(&mut game);
        game.apply_action(action)
            .expect("generated action should apply");

        while next_target < TARGET_HISTORY_SIZES.len()
            && game.history.actions.len() >= TARGET_HISTORY_SIZES[next_target]
        {
            fixtures.push(Fixture {
                history_size: game.history.actions.len(),
                actions: game.history.actions.clone(),
                cached_game: game.clone(),
            });
            next_target += 1;
        }
    }

    fixtures
}

fn choose_action(game: &mut Game) -> Action {
    let turn = game.turn();

    if let Some(action) = choose_simple_move(game, turn) {
        return action;
    }

    choose_simple_placement(game, turn)
}

fn choose_simple_move(game: &mut Game, turn: Color) -> Option<Action> {
    let mut positions = game.board.pieces.keys().copied().collect::<Vec<_>>();
    positions.sort_by_key(|position| (position.q, position.s, position.r));

    for from in positions {
        let top_piece = match game.board.get_top_piece(&from) {
            Some(piece) if piece.color == turn && is_simple_move_piece(piece.piece_type) => *piece,
            _ => continue,
        };
        let mut moves = game.get_legal_moves(from).ok()?;
        moves.sort_by_key(|position| (position.q, position.s, position.r));
        if let Some(to) = moves.first().copied() {
            return Some(Action {
                action_type: ActionType::MovePiece,
                piece_type: Some(top_piece.piece_type),
                start_position: Some(from),
                end_position: Some(to),
                turn,
            });
        }
    }

    None
}

fn choose_simple_placement(game: &mut Game, turn: Color) -> Action {
    let inventory = if turn == Color::White {
        &game.white_inventory
    } else {
        &game.black_inventory
    };
    let piece = SIMPLE_PLACEMENT_PIECES
        .iter()
        .copied()
        .find(|piece| inventory.count(*piece) > 0)
        .expect("fixture should have a simple piece left to place");

    let mut positions = if game.move_num == 1 && turn == Color::White {
        vec![hive_engine::Position::new(0, 0, 0).unwrap()]
    } else if game.move_num == 1 && turn == Color::Black {
        hive_engine::Position::new(0, 0, 0)
            .unwrap()
            .get_neighbours()
    } else {
        game.get_legal_placement_positions()
    };
    positions.sort_by_key(|position| (position.q, position.s, position.r));

    Action {
        action_type: ActionType::PlacePiece,
        piece_type: Some(piece),
        start_position: None,
        end_position: positions.first().copied(),
        turn,
    }
}

const SIMPLE_PLACEMENT_PIECES: [PieceType; 6] = [
    PieceType::Queen,
    PieceType::Beetle,
    PieceType::Grasshopper,
    PieceType::Ladybug,
    PieceType::Mosquito,
    PieceType::Pillbug,
];

fn is_simple_move_piece(piece: PieceType) -> bool {
    matches!(
        piece,
        PieceType::Queen | PieceType::Beetle | PieceType::Grasshopper | PieceType::Ladybug
    )
}

fn rebuild_game(actions: &[Action]) -> Game {
    let mut game = Game::new(true, true, true);
    for action in actions {
        if action.action_type != ActionType::CannotMove {
            game.apply_action(*action)
                .expect("persisted action should replay");
        }
    }
    game
}

fn bench(iterations: usize, mut f: impl FnMut()) -> Duration {
    let started = Instant::now();
    for _ in 0..iterations {
        f();
    }
    started.elapsed() / iterations as u32
}

fn iterations_for(history_size: usize) -> usize {
    match history_size {
        0..=10 => 10_000,
        11..=50 => 2_000,
        51..=100 => 1_000,
        101..=200 => 500,
        _ => 100,
    }
}

fn format_duration(duration: Duration) -> String {
    if duration.as_micros() >= 1_000 {
        format!("{:.3} ms", duration.as_secs_f64() * 1_000.0)
    } else if duration.as_nanos() >= 1_000 {
        format!("{:.3} us", duration.as_secs_f64() * 1_000_000.0)
    } else {
        format!("{} ns", duration.as_nanos())
    }
}
