use std::collections::HashSet;

use super::{Board, freedom_to_move_rule, one_hive_rule};
use crate::hive::error::HiveError;
use crate::hive::history::{History, MoveType};
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

    fn get_pillbug_special_moves(
        &self,
        board: &mut Board,
        position: &Position,
        history: &History,
        turn: Color,
    ) -> Result<Vec<Position>, HiveError> {
        let mut legal_moves: Vec<Position> = vec![];
        if board.get_pieces_copy(position).len() > 1 {
            return Ok(vec![]);
        }
        let last_move = history.moves.last();

        if let Some(last_move) = last_move {
            if last_move.piece_type == self.piece_type && &last_move.end_position == position {
                return Ok(vec![]);
            }
        }

        let neighbours_with_piece = board.get_neighbours_with_piece(position);
        let mut pillbug_positions: Vec<Position> = vec![];
        let pillbug_turn_positions = neighbours_with_piece
            .iter()
            .filter(|neighbour| {
                let top_piece = board.get_top_piece(neighbour).unwrap();
                return top_piece.color == turn && top_piece.piece_type == PieceType::Pillbug;
            })
            .map(|neighbour| neighbour.clone())
            .collect::<Vec<Position>>();

        pillbug_positions.extend(pillbug_turn_positions);

        let mosquito_turn_positions = neighbours_with_piece
            .iter()
            .filter(|neighbour| {
                let top_piece = board.get_top_piece(neighbour).unwrap();
                return top_piece.color == turn
                    && top_piece.piece_type == PieceType::Mosquito
                    && board
                        .get_neighbours_with_piece(neighbour)
                        .iter()
                        .any(|mosq_neighbour| {
                            board.get_top_piece(mosq_neighbour).unwrap().piece_type == PieceType::Pillbug
                        });
            })
            .map(|neighbour| neighbour.clone())
            .collect::<Vec<Position>>();

        pillbug_positions.extend(mosquito_turn_positions);

        for pillbug_position in pillbug_positions {
            if freedom_to_move_rule(board, position, &pillbug_position, 2)? {
                legal_moves.extend(board.get_neighbours_without_piece(&pillbug_position));
            }
        }
        return Ok(legal_moves);
    }

    pub fn get_legal_moves(
        &self,
        board: &mut Board,
        position: &Position,
        piece_type: Option<PieceType>,
        history: &History,
    ) -> Result<Vec<Position>, HiveError> {
        let last_move = history.moves.last();

        if let Some(last_move) = last_move {
            if last_move.move_type == MoveType::PillbugSpecialMove
                && last_move.piece_type == self.piece_type
                && &last_move.end_position == position
            {
                return Ok(vec![]);
            }
        }

        if !one_hive_rule(board, position)? {
            return Ok(vec![]);
        }
        // External callers always provide None. Internally the method can call
        //  it self with a different piece type for Mosquito moves.
        let piece_type = piece_type.unwrap_or(self.piece_type);
        let piece_height = board.get_pieces_copy(position).len();

        let top_piece_of_position = board
            .get_top_piece(position)
            .ok_or(HiveError::PieceNotFound)?;

        if top_piece_of_position != self {
            return Ok(vec![]);
        }
        let neighbours = position.get_neighbours();
        let mut legal_moves = vec![];
        let neighbours_with_piece = board.get_neighbours_with_piece(position);

        match piece_type {
            PieceType::Queen | PieceType::Pillbug => {
                for neighbour in neighbours {
                    if neighbours_with_piece.contains(&neighbour) {
                        continue;
                    }
                    if !freedom_to_move_rule(board, position, &neighbour, 1)? {
                        continue;
                    }
                    if position.get_min_distance_from_positions(&neighbours_with_piece) > 1 {
                        continue;
                    }
                    legal_moves.push(neighbour);
                }
            }
            PieceType::Ant => {
                let piece = board
                    .pieces
                    .get_mut(position)
                    .ok_or(HiveError::PieceNotFound)?
                    .pop()
                    .ok_or(HiveError::PieceNotFound)?
                    .clone();

                let mut visited: HashSet<Position> = HashSet::new();

                fn dfs(
                    position: &Position,
                    visited: &mut HashSet<Position>,
                    board: &mut Board,
                ) -> Result<(), HiveError> {
                    visited.insert(position.clone());
                    let neighbours_with_piece = board.get_neighbours_with_piece(position);
                    for neighbour in position.get_neighbours() {
                        if !neighbours_with_piece.contains(&neighbour)
                            && !visited.contains(&neighbour)
                            && freedom_to_move_rule(board, position, &neighbour, 1)?
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
                let mut beetle_height = piece_height;
                if beetle_height == 1 {
                    beetle_height += 1;
                }
                for neighbour in neighbours {
                    if position.get_min_distance_from_positions(&neighbours_with_piece) <= 1
                        && freedom_to_move_rule(board, position, &neighbour, beetle_height)?
                    {
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
                    .ok_or(HiveError::PieceNotFound)?
                    .pop()
                    .ok_or(HiveError::PieceNotFound)?
                    .clone();

                let mut visited: HashSet<Position> = HashSet::new();
                let mut legal_moves_set: HashSet<Position> = HashSet::new();

                fn dfs(
                    position: &Position,
                    visited: &mut HashSet<Position>,
                    board: &mut Board,
                    move_num: u8,
                    legal_moves_set: &mut HashSet<Position>,
                ) -> Result<(), HiveError> {
                    if move_num > 3 {
                        legal_moves_set.insert(position.clone());
                        return Ok(());
                    }
                    visited.insert(position.clone());
                    let neighbours_with_piece = board.get_neighbours_with_piece(position);
                    for neighbour in position.get_neighbours() {
                        if !neighbours_with_piece.contains(&neighbour)
                            && !visited.contains(&neighbour)
                            && freedom_to_move_rule(board, position, &neighbour, 1)?
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
            PieceType::Mosquito => {
                if neighbours_with_piece.len() == 1
                    && board
                        .get_top_piece(neighbours_with_piece.first().unwrap())
                        .unwrap()
                        .piece_type
                        == PieceType::Mosquito
                {
                    return Ok(vec![]);
                }
                if board.pieces.get(position).unwrap().len() > 1 {
                    return self.get_legal_moves(board, position, Some(PieceType::Beetle), history);
                }
                for neighbour in neighbours {
                    if !neighbours_with_piece.contains(&neighbour) {
                        continue;
                    }
                    let neighbour_top_piece = board.get_top_piece(&neighbour).unwrap();
                    let legal_moves_for_neighbour = self.get_legal_moves(
                        board,
                        &position,
                        Some(neighbour_top_piece.piece_type),
                        history,
                    )?;
                    legal_moves.extend(legal_moves_for_neighbour);
                }
            }
            PieceType::Ladybug => {
                let piece = board
                    .pieces
                    .get_mut(position)
                    .ok_or(HiveError::PieceNotFound)?
                    .pop()
                    .ok_or(HiveError::PieceNotFound)?
                    .clone();

                let mut visited: HashSet<Position> = HashSet::new();
                let mut legal_moves_set: HashSet<Position> = HashSet::new();

                fn dfs(
                    position: &Position,
                    visited: &mut HashSet<Position>,
                    move_num: u8,
                    board: &mut Board,
                    legal_moves_set: &mut HashSet<Position>,
                ) -> Result<(), HiveError> {
                    let neighbours_with_piece = board.get_neighbours_with_piece(position);
                    let piece_height = board.get_pieces_copy(position).len() + 1;
                    if move_num == 0 {
                        visited.insert(position.clone());
                    }
                    if move_num < 3
                        && board.get_top_piece(position).is_some()
                        && !visited.contains(&position)
                    {
                        visited.insert(position.clone());
                    }
                    if move_num == 3
                        && board.get_top_piece(position).is_none()
                        && !visited.contains(&position)
                    {
                        visited.insert(position.clone());
                        legal_moves_set.insert(position.clone());
                        return Ok(());
                    }
                    if move_num > 3 {
                        return Ok(());
                    }

                    for neighbour in position.get_neighbours() {
                        if move_num == 2 && neighbours_with_piece.contains(&neighbour) {
                            continue;
                        }
                        if move_num < 2 && !neighbours_with_piece.contains(&neighbour) {
                            continue;
                        }
                        if !visited.contains(&neighbour)
                            && neighbour.get_min_distance_from_positions(&neighbours_with_piece)
                                <= 1
                            && freedom_to_move_rule(board, position, &neighbour, piece_height)?
                        {
                            dfs(&neighbour, visited, move_num + 1, board, legal_moves_set)?;
                        }
                    }
                    Ok(())
                }
                dfs(position, &mut visited, 0, board, &mut legal_moves_set)?;
                board.pieces.insert(*position, vec![piece]);
                legal_moves.extend(legal_moves_set.iter().map(|position| position.clone()));
            }
        }
        return Ok(legal_moves);
    }
}
