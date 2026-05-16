//! CLI binary for interactive Hive play-testing. Domain logic lives in `MyHiveGame::hive`; a future
//! Axum/Actix server would depend on the same library crate and omit this binary.

mod cli;

use MyHiveGame::hive::Game;

fn main() {
    let mut game = Game::new(None);
    cli::run_game_loop(&mut game);
}
