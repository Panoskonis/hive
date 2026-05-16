use std::collections::HashSet;

use super::{freedom_to_move_rule, one_hive_rule, Board};
use crate::hive::position::Position;
use crate::hive::types::{Color, PieceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

impl Piece {
    pub fn new(color: Color, piece_type: PieceType) -> Self {
        Self { color, piece_type }
    }

    pub fn get_legal_moves(
        &self,
        board: &mut Board,
        position: &Position,
    ) -> Result<Vec<Position>, String> {
        if !one_hive_rule(board, position)? {
            println!("One hive rule not satisfied");
            return Ok(vec![]);
        }

        let top_piece_of_position = board
            .get_top_piece(position)
            .ok_or("No piece found in position".to_string())?;

        if top_piece_of_position != self {
            println!("Top piece of position does not match the piece being moved");
            return Ok(vec![]);
        }
        let neighbours = position.get_neighbours();
        let mut legal_moves = vec![];
        let neighbours_with_piece = board.get_neighbours_with_piece(position);

        match self.piece_type {
            PieceType::Queen => {
                for neighbour in neighbours {
                    if neighbours_with_piece.contains(&neighbour) {
                        continue;
                    }
                    if !freedom_to_move_rule(board, position, &neighbour)? {
                        continue;
                    }
                    if position.get_min_distance_from_positions(&neighbours_with_piece) > 1 {
                        continue;
                    }
                    legal_moves.push(neighbour);
                }
            }
            PieceType::SoldierAnt => {
                let piece = board
                    .pieces
                    .get_mut(position)
                    .ok_or("No piece found in position".to_string())?
                    .pop()
                    .ok_or("No piece found in position".to_string())?
                    .clone();

                let mut visited: HashSet<Position> = HashSet::new();

                fn dfs(
                    position: &Position,
                    visited: &mut HashSet<Position>,
                    board: &mut Board,
                ) -> Result<(), String> {
                    visited.insert(position.clone());
                    let neighbours_with_piece = board.get_neighbours_with_piece(position);
                    for neighbour in position.get_neighbours() {
                        if !neighbours_with_piece.contains(&neighbour)
                            && !visited.contains(&neighbour)
                            && freedom_to_move_rule(board, position, &neighbour)?
                            && neighbour.get_min_distance_from_positions(&neighbours_with_piece)
                                == 1
                        {
                            dfs(&neighbour, visited, board)?;
                        }
                    }
                    Ok(())
                }

                dfs(position, &mut visited, board)?;
                visited.remove(position);
                board.pieces.insert(*position, vec![piece]);
                legal_moves.extend(visited.iter().map(|position| position.clone()));
            }
            PieceType::Beetle => {
                for neighbour in neighbours {
                    if position.get_min_distance_from_positions(&neighbours_with_piece) <= 1 {
                        legal_moves.push(neighbour);
                    }
                }
            }
            PieceType::Grasshopper => {
                for neighbour in neighbours {
                    if !neighbours_with_piece.contains(&neighbour) {
                        continue;
                    }
                    let unit_vec = position.unit_vec(&neighbour);
                    let mut current_position = neighbour.add(&unit_vec);
                    while board.has_piece(&current_position) {
                        current_position = current_position.add(&unit_vec);
                    }
                    legal_moves.push(current_position);
                }
            }
            PieceType::Spider => {
                let piece = board
                    .pieces
                    .get_mut(position)
                    .ok_or("No piece found in position".to_string())?
                    .pop()
                    .ok_or("No piece found in position".to_string())?
                    .clone();

                let mut visited: HashSet<Position> = HashSet::new();
                let mut legal_moves_set: HashSet<Position> = HashSet::new();

                fn dfs(
                    position: &Position,
                    visited: &mut HashSet<Position>,
                    board: &mut Board,
                    move_num: u8,
                    legal_moves_set: &mut HashSet<Position>,
                ) -> Result<(), String> {
                    if move_num > 3 {
                        legal_moves_set.insert(position.clone());
                        return Ok(());
                    }
                    visited.insert(position.clone());
                    let neighbours_with_piece = board.get_neighbours_with_piece(position);
                    for neighbour in position.get_neighbours() {
                        if !neighbours_with_piece.contains(&neighbour)
                            && !visited.contains(&neighbour)
                            && freedom_to_move_rule(board, position, &neighbour)?
                            && neighbour.get_min_distance_from_positions(&neighbours_with_piece)
                                == 1
                        {
                            dfs(&neighbour, visited, board, move_num + 1, legal_moves_set)?;
                        }
                    }
                    Ok(())
                }

                dfs(position, &mut visited, board, 1, &mut legal_moves_set)?;
                board.pieces.insert(*position, vec![piece]);
                legal_moves.extend(legal_moves_set.iter().map(|position| position.clone()));
            }
        }
        return Ok(legal_moves);
    }
}
