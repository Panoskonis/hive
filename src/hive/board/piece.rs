use std::collections::HashSet;

use super::{Board, freedom_to_move_rule, one_hive_rule};
use crate::hive::error::HiveError;
use crate::hive::history::{ActionType, History};
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

    pub(crate) fn get_pillbug_special_moves(
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
        let last_move = history.actions.last();

        if let Some(last_move) = last_move
            && last_move.action_type != ActionType::CannotMove
        {
            if last_move.piece_type.unwrap() == self.piece_type
                && &last_move.end_position.unwrap() == position
            {
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
                            board.get_top_piece(mosq_neighbour).unwrap().piece_type
                                == PieceType::Pillbug
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
        let last_move = history.actions.last();

        if let Some(last_move) = last_move {
            if last_move.action_type == ActionType::PillbugSpecialMove
                && last_move.piece_type.unwrap() == self.piece_type
                && &last_move.end_position.unwrap() == position
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
                    if neighbour.get_min_distance_from_positions(&neighbours_with_piece) > 1 {
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
                    if neighbour.get_min_distance_from_positions(&neighbours_with_piece) <= 1
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::hive::history::{Action, ActionType};

    fn pos(q: i8, s: i8, r: i8) -> Position {
        Position::new(q, s, r).unwrap()
    }

    fn empty_history() -> History {
        History::new(None)
    }

    fn place(
        board: &mut Board,
        q: i8,
        s: i8,
        r: i8,
        color: Color,
        piece_type: PieceType,
    ) {
        board
            .pieces
            .insert(pos(q, s, r), vec![Piece::new(color, piece_type)]);
    }

    fn stack(
        board: &mut Board,
        q: i8,
        s: i8,
        r: i8,
        pieces: &[(Color, PieceType)],
    ) {
        board.pieces.insert(
            pos(q, s, r),
            pieces
                .iter()
                .map(|&(color, piece_type)| Piece::new(color, piece_type))
                .collect(),
        );
    }

    fn legal_moves(
        board: &mut Board,
        q: i8,
        s: i8,
        r: i8,
        color: Color,
        piece_type: PieceType,
        history: &History,
    ) -> Result<Vec<Position>, HiveError> {
        let piece = Piece::new(color, piece_type);
        piece.get_legal_moves(board, &pos(q, s, r), None, history)
    }

    fn pillbug_special_moves(
        board: &mut Board,
        q: i8,
        s: i8,
        r: i8,
        color: Color,
        piece_type: PieceType,
        turn: Color,
        history: &History,
    ) -> Result<Vec<Position>, HiveError> {
        let piece = Piece::new(color, piece_type);
        piece.get_pillbug_special_moves(board, &pos(q, s, r), history, turn)
    }

    fn assert_moves(
        actual: Vec<Position>,
        expected: &[Position],
        scenario: &str,
    ) {
        let actual_set: HashSet<_> = actual.into_iter().collect();
        let expected_set: HashSet<_> = expected.iter().copied().collect();
        assert_eq!(
            actual_set, expected_set,
            "{scenario}: expected {} moves, got {}",
            expected_set.len(),
            actual_set.len()
        );
    }

    fn assert_contains(actual: &[Position], target: Position, scenario: &str) {
        assert!(
            actual.contains(&target),
            "{scenario}: expected {:?} in {:?}",
            (target.q, target.s, target.r),
            actual
                .iter()
                .map(|p| (p.q, p.s, p.r))
                .collect::<Vec<_>>()
        );
    }

    fn assert_empty(actual: Vec<Position>, scenario: &str) {
        assert_moves(actual, &[], scenario);
    }

    /// White queen alone — sliding requires contact with the hive perimeter.
    #[test]
    fn lone_queen_cannot_slide() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Queen);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Queen,
            &empty_history(),
        )
        .unwrap();
        assert_empty(moves, "lone queen");
    }

    /// Two-piece hive: queen can step to empty cells along the shared edge.
    #[test]
    fn queen_on_small_hive_has_slide_moves() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Queen);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Queen,
            &empty_history(),
        )
        .unwrap();
        assert_moves(
            moves,
            &[
                pos(0, -1, 1),
                pos(1, 0, -1),
            ],
            "queen beside one ant",
        );
    }

    /// Queen completely ringed by enemy pieces.
    #[test]
    fn queen_surrounded_by_six_pieces_has_no_moves() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Queen);
        for (q, s, r) in [
            (-1, 1, 0),
            (1, -1, 0),
            (-1, 0, 1),
            (1, 0, -1),
            (0, 1, -1),
            (0, -1, 1),
        ] {
            place(&mut board, q, s, r, Color::Black, PieceType::Beetle);
        }
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Queen,
            &empty_history(),
        )
        .unwrap();
        assert_empty(moves, "surrounded queen");
    }

    /// Moving would split the hive into two components.
    #[test]
    fn queen_bridge_piece_cannot_break_one_hive() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Queen);
        place(&mut board, 2, -2, 0, Color::Black, PieceType::Ant);
        place(&mut board, 0, -2, 2, Color::White, PieceType::Ant);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Queen,
            &empty_history(),
        )
        .unwrap();
        assert_empty(moves, "bridge queen");
    }

    /// Grasshopper jumps in a straight line over the adjacent stack.
    #[test]
    fn grasshopper_jumps_over_line_of_pieces() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Grasshopper);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        place(&mut board, 2, -2, 0, Color::White, PieceType::Beetle);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Grasshopper,
            &empty_history(),
        )
        .unwrap();
        assert_moves(moves, &[pos(3, -3, 0)], "grasshopper line jump");
    }

    /// No piece to jump over in a direction → no landing there.
    #[test]
    fn grasshopper_needs_adjacent_piece_to_jump() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Grasshopper);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Grasshopper,
            &empty_history(),
        )
        .unwrap();
        assert_contains(
            &moves,
            pos(2, -2, 0),
            "grasshopper over one ant",
        );
        assert!(
            !moves.contains(&pos(-1, 1, 0)),
            "grasshopper should not jump into empty directions"
        );
    }

    /// Beetle climbs onto an adjacent occupied cell.
    #[test]
    fn beetle_climbs_onto_adjacent_piece() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Beetle);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Beetle,
            &empty_history(),
        )
        .unwrap();
        assert_contains(&moves, pos(1, -1, 0), "beetle climb");
    }

    /// Beetle on top of a stack uses height 2 for freedom-to-move.
    #[test]
    fn beetle_on_stack_moves_with_height_two() {
        let mut board = Board::new();
        stack(
            &mut board,
            1,
            -1,
            0,
            &[(Color::Black, PieceType::Ant), (Color::White, PieceType::Beetle)],
        );
        place(&mut board, 0, 0, 0, Color::Black, PieceType::Queen);
        place(&mut board, 2, -2, 0, Color::Black, PieceType::Spider);
        let moves = legal_moves(
            &mut board,
            1,
            -1,
            0,
            Color::White,
            PieceType::Beetle,
            &empty_history(),
        )
        .unwrap();
        assert_contains(&moves, pos(2, -2, 0), "beetle on stack");
        assert_contains(&moves, pos(0, 0, 0), "beetle climbs onto adjacent queen");
    }

    /// Spider must take exactly three sliding steps along the hive surface.
    #[test]
    fn spider_exactly_three_steps_on_ring() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Spider);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        place(&mut board, 0, -1, 1, Color::Black, PieceType::Ant);
        place(&mut board, -1, 0, 1, Color::Black, PieceType::Ant);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Spider,
            &empty_history(),
        )
        .unwrap();
        assert!(
            !moves.contains(&pos(1, -1, 0)),
            "spider cannot stop on occupied cells"
        );
        assert!(
            !moves.contains(&pos(0, 0, 0)),
            "spider cannot stay on its start cell"
        );
        assert_moves(
            moves,
            &[pos(2, -2, 0), pos(-2, 0, 2)],
            "spider three-step on partial ring",
        );
    }

    /// Spider on a minimal hive only reaches cells exactly three perimeter steps away.
    #[test]
    fn spider_on_two_piece_hive_has_one_three_step_destination() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Spider);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Spider,
            &empty_history(),
        )
        .unwrap();
        assert_eq!(moves.len(), 1, "only one three-step path off a two-piece hive");
        assert_contains(&moves, pos(2, -2, 0), "spider three-step");
    }

    /// Ant slides around the outside of a compact hive.
    #[test]
    fn ant_circles_two_piece_hive() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Ant);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Queen);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Ant,
            &empty_history(),
        )
        .unwrap();
        assert_moves(
            moves,
            &[
                pos(0, -1, 1),
                pos(1, -2, 1),
                pos(2, -2, 0),
                pos(1, 0, -1),
                pos(2, -1, -1),
            ],
            "ant around two-piece hive",
        );
    }

    /// Buried piece is not the top of stack → no legal moves as that piece.
    #[test]
    fn buried_ant_under_beetle_cannot_move() {
        let mut board = Board::new();
        stack(
            &mut board,
            0,
            0,
            0,
            &[(Color::White, PieceType::Ant), (Color::Black, PieceType::Beetle)],
        );
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Queen);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Ant,
            &empty_history(),
        )
        .unwrap();
        assert_empty(moves, "buried ant");
    }

    /// Mosquito copies movement of adjacent piece types.
    #[test]
    fn mosquito_copies_adjacent_grasshopper_jump() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Mosquito);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Grasshopper);
        place(&mut board, 2, -2, 0, Color::White, PieceType::Ant);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Mosquito,
            &empty_history(),
        )
        .unwrap();
        assert_contains(&moves, pos(3, -3, 0), "mosquito grasshopper jump");
    }

    /// Mosquito on a stack moves as a beetle.
    #[test]
    fn mosquito_on_stack_moves_as_beetle() {
        let mut board = Board::new();
        stack(
            &mut board,
            0,
            0,
            0,
            &[(Color::Black, PieceType::Ant), (Color::White, PieceType::Mosquito)],
        );
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Queen);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Mosquito,
            &empty_history(),
        )
        .unwrap();
        assert_contains(&moves, pos(1, -1, 0), "mosquito beetle climb");
    }

    /// Only mosquito touching another mosquito → no moves.
    #[test]
    fn mosquito_touching_only_mosquito_is_stuck() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Mosquito);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Mosquito);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Mosquito,
            &empty_history(),
        )
        .unwrap();
        assert_empty(moves, "mosquito pair");
    }

    /// Pillbug slides like a queen on a small hive.
    #[test]
    fn pillbug_slides_like_queen_on_perimeter() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Pillbug);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Pillbug,
            &empty_history(),
        )
        .unwrap();
        assert_contains(&moves, pos(1, 0, -1), "pillbug slide");
        assert!(
            !moves.contains(&pos(1, -1, 0)),
            "pillbug cannot enter occupied cell"
        );
    }

    /// Pillbug relocates a neighbouring single-level piece to an empty hex by the pillbug.
    #[test]
    fn pillbug_special_moves_adjacent_ant() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Pillbug);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        place(&mut board, -1, 1, 0, Color::White, PieceType::Queen);
        let moves = pillbug_special_moves(
            &mut board,
            1,
            -1,
            0,
            Color::Black,
            PieceType::Ant,
            Color::White,
            &empty_history(),
        )
        .unwrap();
        assert_contains(&moves, pos(0, 1, -1), "pillbug special");
        assert_contains(&moves, pos(-1, 0, 1), "pillbug special");
        assert!(
            !moves.contains(&pos(0, 0, 0)),
            "cannot drop on the pillbug"
        );
    }

    /// Stacked target cannot be pillbugged.
    #[test]
    fn pillbug_special_cannot_move_covered_piece() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Pillbug);
        stack(
            &mut board,
            1,
            -1,
            0,
            &[(Color::Black, PieceType::Ant), (Color::White, PieceType::Beetle)],
        );
        let moves = pillbug_special_moves(
            &mut board,
            1,
            -1,
            0,
            Color::Black,
            PieceType::Ant,
            Color::White,
            &empty_history(),
        )
        .unwrap();
        assert_empty(moves, "covered ant");
    }

    /// Piece moved last turn cannot immediately be pillbugged again.
    #[test]
    fn pillbug_special_blocked_after_that_piece_moved() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Pillbug);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        let mut history = empty_history();
        history.actions.push(Action {
            action_type: ActionType::MovePiece,
            piece_type: Some(PieceType::Ant),
            start_position: Some(pos(2, -2, 0)),
            end_position: Some(pos(1, -1, 0)),
            turn: Color::Black,
        });
        let moves = pillbug_special_moves(
            &mut board,
            1,
            -1,
            0,
            Color::Black,
            PieceType::Ant,
            Color::White,
            &history,
        )
        .unwrap();
        assert_empty(moves, "just moved ant");
    }

    /// Piece relocated by pillbug cannot move on the following turn.
    #[test]
    fn piece_cannot_move_after_pillbug_special_relocation() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Pillbug);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        let mut history = empty_history();
        history.actions.push(Action {
            action_type: ActionType::PillbugSpecialMove,
            piece_type: Some(PieceType::Ant),
            start_position: Some(pos(2, -2, 0)),
            end_position: Some(pos(1, -1, 0)),
            turn: Color::White,
        });
        let moves = legal_moves(
            &mut board,
            1,
            -1,
            0,
            Color::Black,
            PieceType::Ant,
            &history,
        )
        .unwrap();
        assert_empty(moves, "ant after pillbug");
    }

    /// Mosquito adjacent to pillbug can initiate pillbug special on neighbours.
    #[test]
    fn mosquito_as_pillbug_special_moves_neighbour() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Mosquito);
        place(&mut board, 1, -1, 0, Color::White, PieceType::Pillbug);
        place(&mut board, 2, -2, 0, Color::Black, PieceType::Ant);
        let moves = pillbug_special_moves(
            &mut board,
            2,
            -2,
            0,
            Color::Black,
            PieceType::Ant,
            Color::White,
            &empty_history(),
        )
        .unwrap();
        assert!(
            !moves.is_empty(),
            "mosquito-as-pillbug should relocate the ant"
        );
    }

    /// Ladybug: three steps on top of hive, then down to an empty cell.
    #[test]
    fn ladybug_finishes_on_empty_cell_after_three_on_hive_steps() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Ladybug);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        place(&mut board, 0, -1, 1, Color::Black, PieceType::Ant);
        place(&mut board, -1, 0, 1, Color::Black, PieceType::Ant);
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Ladybug,
            &empty_history(),
        )
        .unwrap();
        assert!(
            moves.iter().all(|p| board.get_top_piece(p).is_none()),
            "ladybug must end on empty cells"
        );
        assert!(!moves.is_empty(), "ladybug should have at least one path");
    }

    /// Two-high gate blocks a queen from sliding through a narrow passage.
    #[test]
    fn queen_blocked_by_two_high_gate() {
        let mut board = Board::new();
        place(&mut board, 0, 0, 0, Color::White, PieceType::Queen);
        place(&mut board, 1, -1, 0, Color::Black, PieceType::Ant);
        stack(
            &mut board,
            0,
            1,
            -1,
            &[(Color::Black, PieceType::Beetle), (Color::Black, PieceType::Beetle)],
        );
        stack(
            &mut board,
            0,
            -1,
            1,
            &[(Color::Black, PieceType::Beetle), (Color::Black, PieceType::Beetle)],
        );
        let moves = legal_moves(
            &mut board,
            0,
            0,
            0,
            Color::White,
            PieceType::Queen,
            &empty_history(),
        )
        .unwrap();
        assert!(
            !moves.contains(&pos(-1, 1, 0)),
            "queen cannot pass through a two-high gate"
        );
    }
}
