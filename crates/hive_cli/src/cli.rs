//! Interactive CLI for local play-testing the Hive engine.

use std::fmt;
use std::io;

use hive_engine::{ActionType, Game, GameStatus, HiveError, PieceType, Position};
use std::path::PathBuf;

#[derive(Debug)]
enum ReadLineError {
    Io(io::Error),
    Parse(HiveError),
}

impl fmt::Display for ReadLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadLineError::Io(e) => write!(f, "{e}"),
            ReadLineError::Parse(e) => write!(f, "{e}"),
        }
    }
}

fn get_input_cli() -> Result<String, io::Error> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line)
}

fn read_move_type_cli() -> Result<ActionType, ReadLineError> {
    let line = get_input_cli().map_err(ReadLineError::Io)?;
    ActionType::try_from(line.trim()).map_err(ReadLineError::Parse)
}

fn read_move_type_cli_with_retry() -> ActionType {
    println!("Insert your move type: 'move', 'place', or 'pb' (pillbug special move).");
    loop {
        match read_move_type_cli() {
            Ok(t) => return t,
            Err(e) => eprintln!(
                "Error: {}. Please select 'move', 'place', or 'pb' (pillbug special move).",
                e
            ),
        }
    }
}

fn read_position_cli() -> Result<Position, ReadLineError> {
    let line = get_input_cli().map_err(ReadLineError::Io)?;
    Position::try_from(line.trim()).map_err(ReadLineError::Parse)
}

fn read_position_cli_with_retry() -> Position {
    println!("Insert your position: '0,0,0'.");
    loop {
        match read_position_cli() {
            Ok(p) => return p,
            Err(e) => eprintln!("Error: {}. Please enter a position like '0,0,0'.", e),
        }
    }
}

fn read_piece_type_cli() -> Result<PieceType, ReadLineError> {
    let line = get_input_cli().map_err(ReadLineError::Io)?;
    PieceType::try_from(line.trim()).map_err(ReadLineError::Parse)
}

fn read_piece_type_cli_with_retry() -> PieceType {
    println!("Insert your piece type: 'queen', 'grasshopper', 'beetle', 'spider' or 'Ant'.");
    loop {
        match read_piece_type_cli() {
            Ok(p) => return p,
            Err(e) => eprintln!(
                "Error: {}. Please enter a piece type like 'queen', 'grasshopper', 'beetle', 'spider' or 'Ant'.",
                e
            ),
        }
    }
}

fn move_piece_w_retry(game: &mut Game) {
    loop {
        let starting_position = read_position_cli_with_retry();
        let ending_position = read_position_cli_with_retry();
        match game.move_piece_with_checks(starting_position, ending_position) {
            Ok(()) => break,
            Err(e) => eprintln!("Error: {}. Please enter a valid move.", e),
        }
    }
}

fn pillbug_special_move_w_retry(game: &mut Game) {
    loop {
        let starting_position = read_position_cli_with_retry();
        let ending_position = read_position_cli_with_retry();
        match game.pillbug_special_move_with_checks(starting_position, ending_position) {
            Ok(()) => break,
            Err(e) => eprintln!("Error: {}. Please enter a valid pillbug special move.", e),
        }
    }
}

fn place_piece_w_retry(game: &mut Game) {
    loop {
        let piece_type = read_piece_type_cli_with_retry();
        let position = read_position_cli_with_retry();

        match game.place_piece_with_checks(piece_type, position) {
            Ok(()) => break,
            Err(e) => eprintln!("Error: {}. Please enter a valid placement.", e),
        }
    }
}

pub fn run_game_loop(game: &mut Game) {
    while game.get_status().unwrap() == GameStatus::InProgress {
        println!("Turn: {:?}. Insert your move.", game.turn());
        let move_type = read_move_type_cli_with_retry();
        match move_type {
            ActionType::MovePiece => {
                move_piece_w_retry(game);
            }
            ActionType::PlacePiece => {
                place_piece_w_retry(game);
            }
            ActionType::PillbugSpecialMove => {
                pillbug_special_move_w_retry(game);
            }
            _ => {
                eprintln!(
                    "Please enter a valid move type. 'move', 'place', or 'pb' (pillbug special move)."
                );
            }
        }
        let png_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../hive_view_rust.png");
        hive_visualize::save_hive_png(&game.board, "Hive", png_path).unwrap_or_else(|e| {
            eprintln!("Visualization failed: {e}");
        });
    }
}
