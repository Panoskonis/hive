mod piece;

pub use piece::Piece;

use std::collections::{HashMap, HashSet};

use crate::hive::error::HiveError;
use crate::hive::position::Position;
use crate::hive::types::Color;

#[derive(Debug, Clone)]
pub struct Board {
    pub(crate) pieces: HashMap<Position, Vec<Piece>>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            pieces: HashMap::new(),
        }
    }

    pub(crate) fn get_pieces_copy(&self, position: &Position) -> Vec<Piece> {
        self.pieces.get(position).unwrap_or(&vec![]).clone()
    }

    pub(crate) fn get_top_piece(&self, position: &Position) -> Option<&Piece> {
        self.pieces.get(position).and_then(|pieces| pieces.last())
    }

    fn get_bottom_piece(&self, position: &Position) -> Option<&Piece> {
        self.pieces.get(position).and_then(|pieces| pieces.first())
    }

    pub(crate) fn has_piece(&self, position: &Position) -> bool {
        self.pieces.contains_key(position) && !self.pieces.get(position).unwrap().is_empty()
    }

    pub(crate) fn get_neighbours_with_piece(&self, position: &Position) -> Vec<Position> {
        let mut neighbours: Vec<Position> = position
            .get_neighbours()
            .iter()
            .filter(|neighbour| self.has_piece(neighbour))
            .map(|neighbour| neighbour.clone())
            .collect();

        match self.pieces.get(position) {
            Some(pos_pieces) => {
                if pos_pieces.len() > 1 {
                    neighbours.push(position.clone());
                }
            }
            None => {}
        }

        return neighbours;
    }

    pub(crate) fn get_all_allowed_placement_positions(&self, color: Color) -> Vec<Position> {
        let other_color = if color == Color::White {
            Color::Black
        } else {
            Color::White
        };

        let all_color_empty_neighbours: HashSet<Position> = self
            .pieces
            .iter()
            .filter(|(_, pieces)| pieces.last().unwrap().color == color)
            .map(|(position, _)| position.get_neighbours())
            .flatten()
            .filter(|position| !self.has_piece(position))
            .collect();

        let all_other_color_empty_neighbours: HashSet<Position> = self
            .pieces
            .iter()
            .filter(|(_, pieces)| pieces.last().unwrap().color == other_color)
            .map(|(position, _)| position.get_neighbours())
            .flatten()
            .filter(|position| !self.has_piece(position))
            .collect();

        return all_color_empty_neighbours
            .difference(&all_other_color_empty_neighbours)
            .copied()
            .collect();
    }
}

pub(super) fn one_hive_rule(board: &mut Board, position: &Position) -> Result<bool, HiveError> {
    let neighbours = board.get_neighbours_with_piece(position);
    let mut pieces = board.get_pieces_copy(position);

    if pieces.len() > 1 {
        return Ok(true);
    }

    let start_position = neighbours.first();
    if start_position.is_none() {
        return Ok(true);
    }
    let start_position = start_position.unwrap();

    let piece = pieces.pop().ok_or(HiveError::PieceNotFound)?;
    board.pieces.remove(position);

    let mut visited: HashSet<Position> = HashSet::new();

    fn dfs(position: &Position, visited: &mut HashSet<Position>, board: &Board) {
        if visited.contains(position) {
            return;
        }
        visited.insert(position.clone());
        for neighbour in board.get_neighbours_with_piece(position) {
            dfs(&neighbour, visited, board);
        }
    }
    dfs(start_position, &mut visited, board);

    let board_size = board.pieces.len();

    board.pieces.insert(position.clone(), vec![piece]);

    return Ok(visited.len() == board_size);
}

pub(super) fn freedom_to_move_rule(
    board: &Board,
    position: &Position,
    adjacent_position: &Position,
) -> Result<bool, HiveError> {
    let position_neighbours = position.get_neighbours();
    let adjacent_position_neighbours = adjacent_position.get_neighbours();

    let common_neighbours = position_neighbours
        .iter()
        .filter(|neighbour| {
            adjacent_position_neighbours.contains(neighbour) && board.has_piece(neighbour)
        })
        .count();

    return Ok(common_neighbours < 2);
}
