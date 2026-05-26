use crate::hive::board::{Board, Piece};
use crate::hive::error::{HiveError, QueenPlacementContext};
use crate::hive::history::{Action, ActionType, History};
use crate::hive::inventory::Inventory;
use crate::hive::position::Position;
use crate::hive::types::{Color, PieceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    WhiteWon,
    BlackWon,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegalAction {
    Place { piece: PieceType, at: Position },
    Move { from: Position, to: Position },
    PillbugSpecial { piece_from: Position, to: Position },
}

impl LegalAction {
    pub fn apply(self, game: &mut Game) -> Result<(), HiveError> {
        match self {
            LegalAction::Place { piece, at } => game.place_piece_with_checks(piece, at),
            LegalAction::Move { from, to } => game.move_piece_with_checks(from, to),
            LegalAction::PillbugSpecial { piece_from, to } => {
                game.pillbug_special_move_with_checks(piece_from, to)
            }
        }
    }
}

pub struct Game {
    pub board: Board,
    pub move_num: u16,
    pub turn: Color,
    pub white_inventory: Inventory,
    pub black_inventory: Inventory,
    pub history: History,
}

impl Game {
    pub fn new(m: bool, l: bool, p: bool) -> Self {
        Self {
            board: Board::new(),
            move_num: 1,
            turn: Color::White,
            white_inventory: Inventory::new(m, l, p),
            black_inventory: Inventory::new(m, l, p),
            history: History::new(),
        }
    }

    pub fn turn(&self) -> Color {
        self.turn
    }

    fn update_turn(&mut self) {
        self.turn = if self.turn == Color::White {
            Color::Black
        } else {
            Color::White
        };

        if self.turn == Color::White {
            self.move_num += 1;
        }
    }

    pub fn has_legal_actions(&mut self) -> Result<bool, HiveError> {
        if !self
            .board
            .get_all_allowed_placement_positions(self.turn)
            .is_empty()
        {
            return Ok(true);
        }
        let positions: Vec<Position> = self.board.pieces.keys().copied().collect();

        for position in positions {
            let top_piece = match self.board.get_top_piece(&position) {
                Some(top_piece) => *top_piece,
                None => continue,
            };

            if top_piece.color == self.turn {
                let legal_moves = top_piece.get_legal_moves(
                    &mut self.board,
                    &position,
                    Some(top_piece.piece_type),
                    &self.history,
                )?;
                if !legal_moves.is_empty() {
                    return Ok(true);
                }
            }

            let pillbug_special_moves = top_piece.get_pillbug_special_moves(
                &mut self.board,
                &position,
                &self.history,
                self.turn,
            )?;
            if !pillbug_special_moves.is_empty() {
                return Ok(true);
            }
        }
        return Ok(false);
    }

    fn place_piece(
        &mut self,
        piece_type: PieceType,
        position: Position,
        color: Color,
    ) -> Result<(), HiveError> {
        let player_inventory = if color == Color::White {
            &mut self.white_inventory
        } else {
            &mut self.black_inventory
        };

        player_inventory.place_piece(piece_type)?;
        self.board
            .pieces
            .insert(position, vec![Piece::new(self.turn, piece_type)]);
        self.history.actions.push(Action {
            action_type: ActionType::PlacePiece,
            piece_type: Some(piece_type),
            start_position: None,
            end_position: Some(position),
            turn: self.turn,
        });
        self.update_turn();

        if !self.has_legal_actions()? {
            self.history.actions.push(Action {
                action_type: ActionType::CannotMove,
                piece_type: None,
                start_position: None,
                end_position: None,
                turn: self.turn,
            });
            self.update_turn();
        }
        return Ok(());
    }

    pub fn place_piece_with_checks(
        &mut self,
        piece_type: PieceType,
        position: Position,
    ) -> Result<(), HiveError> {
        if (self.move_num == 1) && (self.turn == Color::White) {
            self.place_piece(piece_type, Position::new(0, 0, 0).unwrap(), self.turn)?;
            return Ok(());
        }
        if self.move_num == 1 && self.turn == Color::Black {
            if Position::new(0, 0, 0)
                .unwrap()
                .get_neighbours()
                .contains(&position)
            {
                self.place_piece(piece_type, position, self.turn)?;
                return Ok(());
            }
            return Err(HiveError::IllegalPlacementPosition);
        }
        let player_inventory = if self.turn == Color::White {
            &mut self.white_inventory
        } else {
            &mut self.black_inventory
        };

        if self.move_num >= 4 && player_inventory.Queen > 0 && piece_type != PieceType::Queen {
            return Err(HiveError::QueenMustBePlaced(QueenPlacementContext::Place));
        }

        if !self
            .board
            .get_all_allowed_placement_positions(self.turn)
            .contains(&position)
        {
            return Err(HiveError::IllegalPlacementPosition);
        }

        self.place_piece(piece_type, position, self.turn)?;
        return Ok(());
    }

    pub fn move_piece_with_checks(
        &mut self,
        start_position: Position,
        end_position: Position,
    ) -> Result<(), HiveError> {
        let piece = self
            .board
            .get_top_piece(&start_position)
            .ok_or(HiveError::PieceNotFound)?
            .clone();
        if piece.color != self.turn {
            return Err(HiveError::WrongTurn);
        }
        if start_position == end_position {
            return Err(HiveError::SameStartAndEnd);
        }

        let player_inventory = if self.turn == Color::White {
            &mut self.white_inventory
        } else {
            &mut self.black_inventory
        };

        if self.move_num >= 4 && player_inventory.Queen > 0 {
            return Err(HiveError::QueenMustBePlaced(QueenPlacementContext::Move));
        }

        let legal_moves =
            piece.get_legal_moves(&mut self.board, &start_position, None, &self.history)?;

        if !legal_moves.contains(&end_position) {
            return Err(HiveError::IllegalMoveDestination);
        }

        let pieces_start = self
            .board
            .pieces
            .get_mut(&start_position)
            .ok_or(HiveError::PieceNotFound)?;
        let piece = pieces_start.pop().ok_or(HiveError::PieceNotFound)?;
        if pieces_start.is_empty() {
            self.board.pieces.remove(&start_position);
        }
        let mut pieces_end = self.board.get_pieces_copy(&end_position);
        pieces_end.push(piece);
        self.board.pieces.insert(end_position, pieces_end);
        self.history.actions.push(Action {
            action_type: ActionType::MovePiece,
            piece_type: Some(piece.piece_type),
            start_position: Some(start_position),
            end_position: Some(end_position),
            turn: self.turn,
        });
        self.update_turn();
        if !self.has_legal_actions()? {
            self.history.actions.push(Action {
                action_type: ActionType::CannotMove,
                piece_type: None,
                start_position: None,
                end_position: None,
                turn: self.turn,
            });
            self.update_turn();
        }
        return Ok(());
    }

    pub fn pillbug_special_move_with_checks(
        &mut self,
        start_position: Position,
        end_position: Position,
    ) -> Result<(), HiveError> {
        let piece = self
            .board
            .get_top_piece(&start_position)
            .ok_or(HiveError::PieceNotFound)?
            .clone();

        if start_position == end_position {
            return Err(HiveError::SameStartAndEnd);
        }

        let player_inventory = if self.turn == Color::White {
            &mut self.white_inventory
        } else {
            &mut self.black_inventory
        };

        if self.move_num >= 4 && player_inventory.Queen > 0 {
            return Err(HiveError::QueenMustBePlaced(QueenPlacementContext::Move));
        }

        let legal_moves = piece.get_pillbug_special_moves(
            &mut self.board,
            &start_position,
            &self.history,
            self.turn,
        )?;

        if !legal_moves.contains(&end_position) {
            return Err(HiveError::IllegalMoveDestination);
        }

        let pieces_start = self
            .board
            .pieces
            .get_mut(&start_position)
            .ok_or(HiveError::PieceNotFound)?;
        let piece = pieces_start.pop().ok_or(HiveError::PieceNotFound)?;
        if pieces_start.is_empty() {
            self.board.pieces.remove(&start_position);
        }
        let mut pieces_end = self.board.get_pieces_copy(&end_position);
        pieces_end.push(piece);
        self.board.pieces.insert(end_position, pieces_end);
        self.history.actions.push(Action {
            action_type: ActionType::PillbugSpecialMove,
            piece_type: Some(piece.piece_type),
            start_position: Some(start_position),
            end_position: Some(end_position),
            turn: self.turn,
        });
        self.update_turn();
        if !self.has_legal_actions()? {
            self.history.actions.push(Action {
                action_type: ActionType::CannotMove,
                piece_type: None,
                start_position: None,
                end_position: None,
                turn: self.turn,
            });
            self.update_turn();
        }
        return Ok(());
    }

    pub fn get_status(&self) -> Result<GameStatus, HiveError> {
        let white_queen_position = self
            .board
            .pieces
            .iter()
            .filter(|(pos, _)| self.board.has_piece(pos))
            .find(|(_, pieces)| {
                let bottom_piece = pieces.first().unwrap();
                bottom_piece.color == Color::White && bottom_piece.piece_type == PieceType::Queen
            });

        let black_queen_position = self
            .board
            .pieces
            .iter()
            .filter(|(pos, _)| self.board.has_piece(pos))
            .find(|(_, pieces)| {
                let bottom_piece = pieces.first().unwrap();
                bottom_piece.color == Color::Black && bottom_piece.piece_type == PieceType::Queen
            });

        if white_queen_position.is_none() && self.move_num >= 5 {
            return Err(HiveError::QueenNotFoundAfter4thMove(Color::White));
        }
        if black_queen_position.is_none() && self.move_num >= 5 {
            return Err(HiveError::QueenNotFoundAfter4thMove(Color::Black));
        }

        let white_queen_position = match white_queen_position {
            Some(position) => position.0,
            None => return Ok(GameStatus::InProgress),
        };
        let black_queen_position = match black_queen_position {
            Some(position) => position.0,
            None => return Ok(GameStatus::InProgress),
        };
        let mut white_queen_neighbours = self.board.get_neighbours_with_piece(white_queen_position);

        let mut black_queen_neighbours = self.board.get_neighbours_with_piece(black_queen_position);
        white_queen_neighbours.retain(|neighbour| neighbour != white_queen_position);
        black_queen_neighbours.retain(|neighbour| neighbour != black_queen_position);
        if white_queen_neighbours.len() == 6 && black_queen_neighbours.len() == 6 {
            return Ok(GameStatus::Draw);
        }

        if white_queen_neighbours.len() == 6 {
            return Ok(GameStatus::BlackWon);
        }
        if black_queen_neighbours.len() == 6 {
            return Ok(GameStatus::WhiteWon);
        }

        return Ok(GameStatus::InProgress);
    }

    pub fn apply_action(&mut self, action: Action) -> Result<(), HiveError> {
        if self.turn != action.turn {
            return Err(HiveError::WrongTurn);
        }
        match action.action_type {
            ActionType::PlacePiece => self.place_piece_with_checks(
                action.piece_type.unwrap(),
                action.end_position.unwrap(),
            )?,
            ActionType::MovePiece => self.move_piece_with_checks(
                action.start_position.unwrap(),
                action.end_position.unwrap(),
            )?,
            ActionType::PillbugSpecialMove => self.pillbug_special_move_with_checks(
                action.start_position.unwrap(),
                action.end_position.unwrap(),
            )?,
            _ => return Err(HiveError::InvalidMoveType),
        }
        return Ok(());
    }

    pub fn get_legal_moves(&mut self, position: Position) -> Result<Vec<Position>, HiveError> {
        let piece = *self
            .board
            .get_top_piece(&position)
            .ok_or(HiveError::PieceNotFound)?;
        let legal_moves = piece.get_legal_moves(
            &mut self.board,
            &position,
            Some(piece.piece_type),
            &self.history,
        )?;
        return Ok(legal_moves);
    }

    pub fn get_legal_pillbug_special_moves(
        &mut self,
        position: Position,
    ) -> Result<Vec<Position>, HiveError> {
        let piece = *self
            .board
            .get_top_piece(&position)
            .ok_or(HiveError::PieceNotFound)?;
        let legal_moves = piece.get_pillbug_special_moves(
            &mut self.board,
            &position,
            &self.history,
            self.turn,
        )?;
        return Ok(legal_moves);
    }

    pub fn get_legal_placement_positions(&mut self) -> Vec<Position> {
        return self.board.get_all_allowed_placement_positions(self.turn);
    }

    pub fn legal_actions(&mut self) -> Result<Vec<LegalAction>, HiveError> {
        let mut actions = self.legal_placement_actions()?;
        if self.queen_must_be_placed_before_non_place_actions() {
            return Ok(actions);
        }

        let positions: Vec<Position> = self.board.pieces.keys().copied().collect();
        for from in positions {
            let top_piece = match self.board.get_top_piece(&from) {
                Some(piece) => *piece,
                None => continue,
            };

            if top_piece.color == self.turn {
                for to in top_piece.get_legal_moves(
                    &mut self.board,
                    &from,
                    Some(top_piece.piece_type),
                    &self.history,
                )? {
                    actions.push(LegalAction::Move { from, to });
                }
            }

            for to in top_piece.get_pillbug_special_moves(
                &mut self.board,
                &from,
                &self.history,
                self.turn,
            )? {
                actions.push(LegalAction::PillbugSpecial {
                    piece_from: from,
                    to,
                });
            }
        }

        Ok(actions)
    }

    fn queen_must_be_placed_before_non_place_actions(&self) -> bool {
        let inventory = if self.turn == Color::White {
            &self.white_inventory
        } else {
            &self.black_inventory
        };
        self.move_num >= 4 && inventory.Queen > 0
    }

    fn piece_types_in_hand(inventory: &Inventory) -> Vec<PieceType> {
        const ALL: [PieceType; 8] = [
            PieceType::Queen,
            PieceType::Ant,
            PieceType::Beetle,
            PieceType::Grasshopper,
            PieceType::Spider,
            PieceType::Mosquito,
            PieceType::Ladybug,
            PieceType::Pillbug,
        ];
        ALL.into_iter()
            .filter(|&piece_type| inventory.count(piece_type) > 0)
            .collect()
    }

    fn legal_placement_actions(&mut self) -> Result<Vec<LegalAction>, HiveError> {
        let inventory = if self.turn == Color::White {
            &self.white_inventory
        } else {
            &self.black_inventory
        };
        let queen_must_place = self.queen_must_be_placed_before_non_place_actions();
        let mut actions = Vec::new();

        let mut push_placements = |piece_types: &[PieceType], positions: &[Position]| {
            for &piece in piece_types {
                if queen_must_place && piece != PieceType::Queen {
                    continue;
                }
                for &at in positions {
                    actions.push(LegalAction::Place { piece, at });
                }
            }
        };

        if self.move_num == 1 && self.turn == Color::White {
            let origin = Position::new(0, 0, 0).unwrap();
            push_placements(&Self::piece_types_in_hand(inventory), &[origin]);
            return Ok(actions);
        }

        if self.move_num == 1 && self.turn == Color::Black {
            let positions = Position::new(0, 0, 0).unwrap().get_neighbours();
            push_placements(&Self::piece_types_in_hand(inventory), &positions);
            return Ok(actions);
        }

        let positions = self.board.get_all_allowed_placement_positions(self.turn);
        push_placements(&Self::piece_types_in_hand(inventory), &positions);
        Ok(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hive::inventory::Inventory;

    fn pos(q: i8, s: i8, r: i8) -> Position {
        Position::new(q, s, r).unwrap()
    }

    fn place_on_board(board: &mut Board, q: i8, s: i8, r: i8, color: Color, piece_type: PieceType) {
        board
            .pieces
            .insert(pos(q, s, r), vec![Piece::new(color, piece_type)]);
    }

    fn stack_on_board(board: &mut Board, q: i8, s: i8, r: i8, pieces: &[(Color, PieceType)]) {
        board.pieces.insert(
            pos(q, s, r),
            pieces
                .iter()
                .map(|&(color, piece_type)| Piece::new(color, piece_type))
                .collect(),
        );
    }

    fn game_from_board(
        board: Board,
        turn: Color,
        move_num: u16,
        white_inventory: Inventory,
        black_inventory: Inventory,
    ) -> Game {
        Game {
            board,
            move_num,
            turn,
            white_inventory,
            black_inventory,
            history: History::new(),
        }
    }

    /// Build a game from a fixed board, turn, move number, and hand inventories.
    fn game_from_state(
        pieces: &[(i8, i8, i8, Color, PieceType)],
        turn: Color,
        move_num: u16,
        white_inventory: Inventory,
        black_inventory: Inventory,
    ) -> Game {
        let mut board = Board::new();
        for &(q, s, r, color, piece_type) in pieces {
            place_on_board(&mut board, q, s, r, color, piece_type);
        }
        game_from_board(board, turn, move_num, white_inventory, black_inventory)
    }

    fn base_inventory() -> Inventory {
        Inventory::new(false, false, false)
    }

    fn inventory_with_queen_in_hand() -> Inventory {
        let mut inv = base_inventory();
        inv.Queen = 1;
        inv
    }

    fn inventory_queen_on_board() -> Inventory {
        let mut inv = base_inventory();
        inv.Queen = 0;
        inv
    }

    fn inventory_empty_hands() -> Inventory {
        let mut inv = base_inventory();
        inv.Grasshopper = 0;
        inv.Beetle = 0;
        inv.Spider = 0;
        inv.Ant = 0;
        inv.Queen = 0;
        inv
    }

    fn inventory_expansions_on_board() -> Inventory {
        let mut inv = Inventory::new(true, true, true);
        inv.Queen = 0;
        inv.Mosquito = 0;
        inv.Ladybug = 0;
        inv.Pillbug = 0;
        inv
    }

    /// White pillbug can relocate the adjacent black ant (from piece.rs layout).
    fn pillbug_can_relocate_adjacent_ant() -> Game {
        game_from_state(
            &[
                (0, 0, 0, Color::White, PieceType::Pillbug),
                (1, -1, 0, Color::Black, PieceType::Ant),
                (-1, 1, 0, Color::White, PieceType::Queen),
            ],
            Color::White,
            5,
            inventory_expansions_on_board(),
            inventory_queen_on_board(),
        )
    }

    /// Mosquito copies pillbug special on a neighbour of the pillbug.
    fn mosquito_initiates_pillbug_special() -> Game {
        game_from_state(
            &[
                (0, 0, 0, Color::White, PieceType::Mosquito),
                (1, -1, 0, Color::White, PieceType::Pillbug),
                (2, -2, 0, Color::Black, PieceType::Ant),
            ],
            Color::White,
            5,
            inventory_expansions_on_board(),
            inventory_queen_on_board(),
        )
    }

    fn move_four_white_pillbug_on_board() -> Game {
        game_from_state(
            &[
                (0, 0, 0, Color::White, PieceType::Pillbug),
                (1, -1, 0, Color::Black, PieceType::Ant),
            ],
            Color::White,
            4,
            inventory_with_queen_in_hand(),
            inventory_queen_on_board(),
        )
    }

    fn ring_black_queen(pieces: &mut Vec<(i8, i8, i8, Color, PieceType)>) {
        pieces.push((3, -3, 0, Color::Black, PieceType::Queen));
        for (q, s, r) in [
            (2, -2, 0),
            (4, -4, 0),
            (2, -3, 1),
            (4, -3, -1),
            (3, -2, -1),
            (3, -4, 1),
        ] {
            pieces.push((q, s, r, Color::White, PieceType::Beetle));
        }
    }

    /// White ant can move around a small hive; black's only piece is a surrounded queen.
    fn white_to_move_black_has_no_legal_actions() -> Game {
        let mut pieces = vec![
            (0, 0, 0, Color::White, PieceType::Ant),
            (1, -1, 0, Color::White, PieceType::Queen),
        ];
        ring_black_queen(&mut pieces);
        game_from_state(
            &pieces,
            Color::White,
            5,
            inventory_empty_hands(),
            inventory_empty_hands(),
        )
    }

    fn last_cannot_move_action(history: &History) -> Option<&Action> {
        history
            .actions
            .iter()
            .rev()
            .find(|a| a.action_type == ActionType::CannotMove)
    }

    /// Small hive: white queen at origin, black ant to the east — black to move, move 2.
    fn early_opening_after_two_placements() -> Game {
        game_from_state(
            &[
                (0, 0, 0, Color::White, PieceType::Queen),
                (1, -1, 0, Color::Black, PieceType::Ant),
            ],
            Color::White,
            2,
            inventory_queen_on_board(),
            inventory_with_queen_in_hand(),
        )
    }

    /// Mid-game: both queens on board; white queen can slide to an empty neighbour.
    fn midgame_both_queens_placed() -> Game {
        game_from_state(
            &[
                (0, 0, 0, Color::White, PieceType::Queen),
                (1, -1, 0, Color::Black, PieceType::Queen),
                (2, -2, 0, Color::Black, PieceType::Ant),
            ],
            Color::White,
            5,
            inventory_queen_on_board(),
            inventory_queen_on_board(),
        )
    }

    /// Move 4, white still has queen in hand — non-queen placements and moves are blocked.
    fn move_four_white_queen_still_in_hand() -> Game {
        game_from_state(
            &[
                (0, 0, 0, Color::White, PieceType::Ant),
                (1, -1, 0, Color::Black, PieceType::Queen),
            ],
            Color::White,
            4,
            inventory_with_queen_in_hand(),
            inventory_queen_on_board(),
        )
    }

    /// White queen ringed by six black pieces; black queen still free on the hive.
    fn white_queen_surrounded_black_queen_safe() -> Game {
        let mut pieces = vec![
            (0, 0, 0, Color::White, PieceType::Queen),
            (3, -3, 0, Color::Black, PieceType::Queen),
        ];
        for (q, s, r) in [
            (-1, 1, 0),
            (1, -1, 0),
            (-1, 0, 1),
            (1, 0, -1),
            (0, 1, -1),
            (0, -1, 1),
        ] {
            pieces.push((q, s, r, Color::Black, PieceType::Beetle));
        }
        game_from_state(
            &pieces,
            Color::White,
            5,
            inventory_queen_on_board(),
            inventory_queen_on_board(),
        )
    }

    #[test]
    fn new_game_starts_empty_with_white_to_move() {
        let game = Game::new(true, true, true);
        assert!(game.board.pieces.is_empty());
        assert_eq!(game.turn(), Color::White);
        assert_eq!(game.move_num, 1);
        assert_eq!(game.get_status().unwrap(), GameStatus::InProgress);
    }

    #[test]
    fn white_opening_always_places_at_origin() {
        let mut game = Game::new(true, true, true);
        game.place_piece_with_checks(PieceType::Ant, pos(5, 5, -10))
            .unwrap();
        assert!(game.board.pieces.contains_key(&pos(0, 0, 0)));
        assert_eq!(
            game.board.get_top_piece(&pos(0, 0, 0)).unwrap().piece_type,
            PieceType::Ant
        );
        assert_eq!(game.turn(), Color::Black);
    }

    #[test]
    fn black_second_placement_must_touch_origin() {
        let mut game = Game::new(true, true, true);
        game.place_piece_with_checks(PieceType::Queen, pos(0, 0, 0))
            .unwrap();
        game.place_piece_with_checks(PieceType::Ant, pos(1, -1, 0))
            .unwrap();
        assert_eq!(game.turn(), Color::White);
        assert_eq!(game.move_num, 2);
    }

    #[test]
    fn black_second_placement_rejects_non_adjacent_cell() {
        let mut game = Game::new(true, true, true);
        game.place_piece_with_checks(PieceType::Queen, pos(0, 0, 0))
            .unwrap();
        let err = game
            .place_piece_with_checks(PieceType::Ant, pos(2, -2, 0))
            .unwrap_err();
        assert_eq!(err, HiveError::IllegalPlacementPosition);
    }

    #[test]
    fn queen_must_be_placed_before_other_pieces_from_move_four() {
        let mut game = move_four_white_queen_still_in_hand();
        let err = game
            .place_piece_with_checks(PieceType::Ant, pos(0, 1, -1))
            .unwrap_err();
        assert_eq!(
            err,
            HiveError::QueenMustBePlaced(QueenPlacementContext::Place)
        );
    }

    #[test]
    fn queen_must_be_placed_before_moving_from_move_four() {
        let mut game = move_four_white_queen_still_in_hand();
        let err = game
            .move_piece_with_checks(pos(0, 0, 0), pos(0, -1, 1))
            .unwrap_err();
        assert_eq!(
            err,
            HiveError::QueenMustBePlaced(QueenPlacementContext::Move)
        );
    }

    #[test]
    fn move_piece_rejects_wrong_turn_and_same_cell() {
        let mut game = midgame_both_queens_placed();
        let err = game
            .move_piece_with_checks(pos(1, -1, 0), pos(0, -1, 1))
            .unwrap_err();
        assert_eq!(err, HiveError::WrongTurn);

        let err = game
            .move_piece_with_checks(pos(0, 0, 0), pos(0, 0, 0))
            .unwrap_err();
        assert_eq!(err, HiveError::SameStartAndEnd);
    }

    #[test]
    fn move_piece_rejects_illegal_destination() {
        let mut game = midgame_both_queens_placed();
        let err = game
            .move_piece_with_checks(pos(0, 0, 0), pos(3, -3, 0))
            .unwrap_err();
        assert_eq!(err, HiveError::IllegalMoveDestination);
    }

    #[test]
    fn legal_queen_slide_updates_board_and_turn() {
        let mut game = midgame_both_queens_placed();
        game.move_piece_with_checks(pos(0, 0, 0), pos(0, -1, 1))
            .unwrap();
        assert!(!game.board.pieces.contains_key(&pos(0, 0, 0)));
        assert_eq!(
            game.board.get_top_piece(&pos(0, -1, 1)).unwrap().piece_type,
            PieceType::Queen
        );
        assert_eq!(game.turn(), Color::Black);
    }

    #[test]
    fn get_legal_moves_delegates_to_piece_rules() {
        let mut game = midgame_both_queens_placed();
        let moves = game.get_legal_moves(pos(0, 0, 0)).unwrap();
        assert!(moves.contains(&pos(0, -1, 1)));
        assert!(!moves.contains(&pos(3, -3, 0)));
    }

    #[test]
    fn get_legal_placement_positions_for_current_player() {
        let mut game = midgame_both_queens_placed();
        let placements = game.get_legal_placement_positions();
        assert!(!placements.is_empty());
        assert!(!placements.contains(&pos(0, 0, 0)));
    }

    #[test]
    fn has_legal_actions_true_when_placement_or_move_exists() {
        let mut game = midgame_both_queens_placed();
        assert!(game.has_legal_actions().unwrap());
    }

    #[test]
    fn has_legal_actions_false_when_queen_is_pinned() {
        let mut game = white_queen_surrounded_black_queen_safe();
        game.turn = Color::White;
        assert!(!game.has_legal_actions().unwrap());
    }

    #[test]
    fn black_has_no_legal_actions_when_only_queen_is_pinned() {
        let mut game = white_to_move_black_has_no_legal_actions();
        game.turn = Color::Black;
        assert!(!game.has_legal_actions().unwrap());
    }

    #[test]
    fn white_still_has_legal_actions_before_skipping_pinned_black() {
        let mut game = white_to_move_black_has_no_legal_actions();
        assert!(game.has_legal_actions().unwrap());
        assert!(!game.get_legal_moves(pos(0, 0, 0)).unwrap().is_empty());
    }

    #[test]
    fn move_piece_records_cannot_move_and_skips_pinned_opponent() {
        let mut game = white_to_move_black_has_no_legal_actions();
        let destination = pos(0, -1, 1);
        assert!(
            game.get_legal_moves(pos(0, 0, 0))
                .unwrap()
                .contains(&destination)
        );
        game.move_piece_with_checks(pos(0, 0, 0), destination)
            .unwrap();

        let cannot_move = last_cannot_move_action(&game.history).expect("CannotMove in history");
        assert_eq!(cannot_move.action_type, ActionType::CannotMove);
        assert_eq!(cannot_move.turn, Color::Black);

        assert_eq!(game.turn(), Color::White);
        assert!(game.history.actions.iter().any(|a| {
            a.action_type == ActionType::MovePiece
                && a.turn == Color::White
                && a.end_position == Some(destination)
        }));
    }

    #[test]
    fn move_piece_does_not_skip_turn_when_opponent_can_still_act() {
        let mut game = midgame_both_queens_placed();
        let history_len_before = game.history.actions.len();
        game.move_piece_with_checks(pos(0, 0, 0), pos(0, -1, 1))
            .unwrap();

        assert_eq!(game.turn(), Color::Black);
        assert_eq!(game.history.actions.len(), history_len_before + 1);
        assert!(last_cannot_move_action(&game.history).is_none());
    }

    #[test]
    fn place_piece_records_cannot_move_when_opponent_is_pinned() {
        let mut game = white_to_move_black_has_no_legal_actions();
        game.white_inventory.Ant = 1;
        let placement = game.get_legal_placement_positions().into_iter().next();
        let placement = placement.expect("white should have a placement option");

        game.place_piece_with_checks(PieceType::Ant, placement)
            .unwrap();

        let cannot_move = last_cannot_move_action(&game.history).expect("CannotMove in history");
        assert_eq!(cannot_move.turn, Color::Black);
        assert_eq!(game.turn(), Color::White);
    }

    #[test]
    fn get_status_in_progress_before_move_five() {
        let game = early_opening_after_two_placements();
        assert_eq!(game.get_status().unwrap(), GameStatus::InProgress);
    }

    #[test]
    fn get_status_black_wins_when_white_queen_surrounded() {
        let game = white_queen_surrounded_black_queen_safe();
        assert_eq!(game.get_status().unwrap(), GameStatus::BlackWon);
    }

    #[test]
    fn get_status_white_wins_when_black_queen_surrounded() {
        let mut ringed = vec![(3, -3, 0, Color::Black, PieceType::Queen)];
        for (q, s, r) in [
            (2, -2, 0),
            (4, -4, 0),
            (2, -3, 1),
            (4, -3, -1),
            (3, -2, -1),
            (3, -4, 1),
        ] {
            ringed.push((q, s, r, Color::White, PieceType::Beetle));
        }
        ringed.push((0, 0, 0, Color::White, PieceType::Queen));
        let game = game_from_state(
            &ringed,
            Color::White,
            5,
            inventory_queen_on_board(),
            inventory_queen_on_board(),
        );
        assert_eq!(game.get_status().unwrap(), GameStatus::WhiteWon);
    }

    #[test]
    fn get_status_draw_when_both_queens_surrounded() {
        let mut pieces = vec![
            (0, 0, 0, Color::White, PieceType::Queen),
            (4, -4, 0, Color::Black, PieceType::Queen),
        ];
        for (q, s, r) in [
            (-1, 1, 0),
            (1, -1, 0),
            (-1, 0, 1),
            (1, 0, -1),
            (0, 1, -1),
            (0, -1, 1),
        ] {
            pieces.push((q, s, r, Color::Black, PieceType::Beetle));
        }
        for (q, s, r) in [
            (3, -3, 0),
            (5, -5, 0),
            (3, -4, 1),
            (5, -4, -1),
            (4, -3, -1),
            (4, -5, 1),
        ] {
            pieces.push((q, s, r, Color::White, PieceType::Ant));
        }
        let game = game_from_state(
            &pieces,
            Color::White,
            5,
            inventory_queen_on_board(),
            inventory_queen_on_board(),
        );
        assert_eq!(game.get_status().unwrap(), GameStatus::Draw);
    }

    #[test]
    fn get_status_errors_when_queen_missing_after_move_four() {
        let game = game_from_state(
            &[(0, 0, 0, Color::White, PieceType::Queen)],
            Color::White,
            5,
            inventory_queen_on_board(),
            inventory_queen_on_board(),
        );
        assert_eq!(
            game.get_status().unwrap_err(),
            HiveError::QueenNotFoundAfter4thMove(Color::Black)
        );
    }

    #[test]
    fn apply_action_replays_place_and_move() {
        let mut game = Game::new(true, true, true);
        game.apply_action(Action {
            action_type: ActionType::PlacePiece,
            piece_type: Some(PieceType::Queen),
            start_position: None,
            end_position: Some(pos(0, 0, 0)),
            turn: Color::White,
        })
        .unwrap();
        game.apply_action(Action {
            action_type: ActionType::PlacePiece,
            piece_type: Some(PieceType::Ant),
            start_position: None,
            end_position: Some(pos(1, -1, 0)),
            turn: Color::Black,
        })
        .unwrap();
        assert_eq!(game.board.pieces.len(), 2);

        let mut mid = midgame_both_queens_placed();
        let before = mid.board.pieces.clone();
        mid.apply_action(Action {
            action_type: ActionType::MovePiece,
            piece_type: Some(PieceType::Queen),
            start_position: Some(pos(0, 0, 0)),
            end_position: Some(pos(0, -1, 1)),
            turn: Color::White,
        })
        .unwrap();
        assert_ne!(mid.board.pieces, before);
    }

    #[test]
    fn apply_action_rejects_cannot_move_type() {
        let mut game = Game::new(true, true, true);
        let err = game
            .apply_action(Action {
                action_type: ActionType::CannotMove,
                piece_type: None,
                start_position: None,
                end_position: None,
                turn: Color::White,
            })
            .unwrap_err();
        assert_eq!(err, HiveError::InvalidMoveType);
    }

    #[test]
    fn legal_actions_includes_opening_placements_at_origin() {
        let mut game = Game::new(true, true, true);
        let actions = game.legal_actions().unwrap();
        assert!(actions.contains(&LegalAction::Place {
            piece: PieceType::Queen,
            at: pos(0, 0, 0),
        }));
        assert!(actions.contains(&LegalAction::Place {
            piece: PieceType::Ant,
            at: pos(0, 0, 0),
        }));
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, LegalAction::Move { .. }))
        );
    }

    #[test]
    fn legal_actions_includes_moves_in_midgame() {
        let mut game = midgame_both_queens_placed();
        let actions = game.legal_actions().unwrap();
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, LegalAction::Move { from, to } if *from == pos(0, 0, 0) && *to == pos(0, -1, 1)))
        );
    }

    #[test]
    fn legal_actions_includes_pillbug_special() {
        let mut game = pillbug_can_relocate_adjacent_ant();
        let actions = game.legal_actions().unwrap();
        assert!(actions.iter().any(|a| {
            matches!(
                a,
                LegalAction::PillbugSpecial {
                    piece_from,
                    to
                } if *piece_from == pos(1, -1, 0) && *to == pos(0, 1, -1)
            )
        }));
    }

    #[test]
    fn legal_actions_queen_only_placements_at_move_four() {
        let mut game = move_four_white_queen_still_in_hand();
        let actions = game.legal_actions().unwrap();
        assert!(actions.iter().all(|a| matches!(
            a,
            LegalAction::Place {
                piece: PieceType::Queen,
                ..
            }
        )));
        assert!(!actions.is_empty());
    }

    #[test]
    fn legal_actions_empty_when_player_is_pinned() {
        let mut game = white_queen_surrounded_black_queen_safe();
        game.turn = Color::White;
        assert!(game.legal_actions().unwrap().is_empty());
    }

    #[test]
    fn legal_action_apply_matches_direct_move() {
        let mut game = midgame_both_queens_placed();
        let action = game
            .legal_actions()
            .unwrap()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    LegalAction::Move {
                        from,
                        to
                    } if *from == pos(0, 0, 0) && *to == pos(0, -1, 1)
                )
            })
            .unwrap();
        action.apply(&mut game).unwrap();
        assert_eq!(
            game.board.get_top_piece(&pos(0, -1, 1)).unwrap().piece_type,
            PieceType::Queen
        );
    }

    #[test]
    fn get_legal_pillbug_special_moves_for_adjacent_ant() {
        let mut game = pillbug_can_relocate_adjacent_ant();
        let moves = game.get_legal_pillbug_special_moves(pos(1, -1, 0)).unwrap();
        assert!(moves.contains(&pos(0, 1, -1)));
        assert!(moves.contains(&pos(-1, 0, 1)));
        assert!(!moves.contains(&pos(0, 0, 0)));
    }

    #[test]
    fn get_legal_pillbug_special_moves_via_mosquito() {
        let mut game = mosquito_initiates_pillbug_special();
        let moves = game.get_legal_pillbug_special_moves(pos(2, -2, 0)).unwrap();
        assert!(!moves.is_empty());
    }

    #[test]
    fn get_legal_pillbug_special_moves_empty_for_stacked_target() {
        let mut board = Board::new();
        place_on_board(&mut board, 0, 0, 0, Color::White, PieceType::Pillbug);
        stack_on_board(
            &mut board,
            1,
            -1,
            0,
            &[
                (Color::Black, PieceType::Ant),
                (Color::White, PieceType::Beetle),
            ],
        );
        let mut game = game_from_board(
            board,
            Color::White,
            5,
            inventory_expansions_on_board(),
            inventory_queen_on_board(),
        );
        let moves = game.get_legal_pillbug_special_moves(pos(1, -1, 0)).unwrap();
        assert!(moves.is_empty());
    }

    #[test]
    fn pillbug_special_move_relocates_piece_and_advances_turn() {
        let mut game = pillbug_can_relocate_adjacent_ant();
        let destination = pos(0, 1, -1);
        game.pillbug_special_move_with_checks(pos(1, -1, 0), destination)
            .unwrap();

        assert!(!game.board.pieces.contains_key(&pos(1, -1, 0)));
        assert_eq!(
            game.board.get_top_piece(&destination).unwrap().piece_type,
            PieceType::Ant
        );
        assert_eq!(game.turn(), Color::Black);
        assert!(game.history.actions.iter().any(|a| {
            a.action_type == ActionType::PillbugSpecialMove
                && a.turn == Color::White
                && a.start_position == Some(pos(1, -1, 0))
                && a.end_position == Some(destination)
        }));
    }

    #[test]
    fn pillbug_special_move_rejects_same_start_and_end() {
        let mut game = pillbug_can_relocate_adjacent_ant();
        let err = game
            .pillbug_special_move_with_checks(pos(1, -1, 0), pos(1, -1, 0))
            .unwrap_err();
        assert_eq!(err, HiveError::SameStartAndEnd);
    }

    #[test]
    fn pillbug_special_move_rejects_illegal_destination() {
        let mut game = pillbug_can_relocate_adjacent_ant();
        let err = game
            .pillbug_special_move_with_checks(pos(1, -1, 0), pos(2, -2, 0))
            .unwrap_err();
        assert_eq!(err, HiveError::IllegalMoveDestination);
    }

    #[test]
    fn pillbug_special_move_rejects_stacked_target() {
        let mut board = Board::new();
        place_on_board(&mut board, 0, 0, 0, Color::White, PieceType::Pillbug);
        stack_on_board(
            &mut board,
            1,
            -1,
            0,
            &[
                (Color::Black, PieceType::Ant),
                (Color::White, PieceType::Beetle),
            ],
        );
        let mut game = game_from_board(
            board,
            Color::White,
            5,
            inventory_expansions_on_board(),
            inventory_queen_on_board(),
        );
        let err = game
            .pillbug_special_move_with_checks(pos(1, -1, 0), pos(0, 1, -1))
            .unwrap_err();
        assert_eq!(err, HiveError::IllegalMoveDestination);
    }

    #[test]
    fn pillbug_special_move_blocked_when_target_just_moved() {
        let mut game = pillbug_can_relocate_adjacent_ant();
        game.history.actions.push(Action {
            action_type: ActionType::MovePiece,
            piece_type: Some(PieceType::Ant),
            start_position: Some(pos(2, -2, 0)),
            end_position: Some(pos(1, -1, 0)),
            turn: Color::Black,
        });
        let moves = game.get_legal_pillbug_special_moves(pos(1, -1, 0)).unwrap();
        assert!(moves.is_empty());
    }

    #[test]
    fn queen_must_be_placed_before_pillbug_special_from_move_four() {
        let mut game = move_four_white_pillbug_on_board();
        let err = game
            .pillbug_special_move_with_checks(pos(1, -1, 0), pos(0, 1, -1))
            .unwrap_err();
        assert_eq!(
            err,
            HiveError::QueenMustBePlaced(QueenPlacementContext::Move)
        );
    }

    #[test]
    fn relocated_piece_cannot_move_immediately_after_pillbug_special() {
        let mut game = pillbug_can_relocate_adjacent_ant();
        let destination = pos(0, 1, -1);
        game.pillbug_special_move_with_checks(pos(1, -1, 0), destination)
            .unwrap();
        assert_eq!(game.turn(), Color::Black);

        let moves = game.get_legal_moves(destination).unwrap();
        assert!(moves.is_empty());
    }

    #[test]
    fn has_legal_actions_counts_pillbug_special_on_enemy_piece() {
        let mut game = pillbug_can_relocate_adjacent_ant();
        assert!(
            !game
                .get_legal_pillbug_special_moves(pos(1, -1, 0))
                .unwrap()
                .is_empty()
        );
        assert!(game.has_legal_actions().unwrap());
    }

    #[test]
    fn apply_action_replays_pillbug_special_move() {
        let mut game = pillbug_can_relocate_adjacent_ant();
        let destination = pos(-1, 0, 1);
        game.apply_action(Action {
            action_type: ActionType::PillbugSpecialMove,
            piece_type: Some(PieceType::Ant),
            start_position: Some(pos(1, -1, 0)),
            end_position: Some(destination),
            turn: Color::White,
        })
        .unwrap();
        assert_eq!(
            game.board.get_top_piece(&destination).unwrap().piece_type,
            PieceType::Ant
        );
        assert_eq!(game.turn(), Color::Black);
    }
}
