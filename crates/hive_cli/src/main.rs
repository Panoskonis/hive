//! CLI binary for interactive Hive play-testing. Domain logic lives in `hive_engine`; a future
//! Axum/Actix server would depend on `hive_engine` only and omit this binary.

mod cli;

use hive_engine::Game;

fn main() {
    let mut game = Game::new(true, true, true);
    cli::run_game_loop(&mut game);
}
