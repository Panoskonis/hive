use crate::hive::board::{Board, Piece};
use crate::hive::error::{HiveError, QueenPlacementContext};
use crate::hive::history::{History, HistoryExporter, Move, MoveType};
use crate::hive::inventory::Inventory;
use crate::hive::position::Position;
use crate::hive::types::{Color, PieceType};

pub struct Game {
    pub board: Board,
    pub(crate) move_num: u16,
    pub(crate) turn: Color,
    white_inventory: Inventory,
    black_inventory: Inventory,
    pub(crate) history: History,
}

impl Game {
    pub fn new(history_exporter: Option<Box<dyn HistoryExporter>>) -> Self {
        Self {
            board: Board::new(),
            move_num: 1,
            turn: Color::White,
            white_inventory: Inventory::new(),
            black_inventory: Inventory::new(),
            history: History::new(history_exporter),
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
        self.update_turn();
        self.history.moves.push(Move {
            move_type: MoveType::PlacePiece,
            piece_type: piece_type,
            start_position: None,
            end_position: position,
        });
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

        if self.move_num == 4 && player_inventory.Queen > 0 && piece_type != PieceType::Queen {
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

        if self.move_num == 4 && player_inventory.Queen > 0 {
            return Err(HiveError::QueenMustBePlaced(QueenPlacementContext::Move));
        }

        let legal_moves = piece.get_legal_moves(&mut self.board, &start_position)?;

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
        self.history.moves.push(Move {
            move_type: MoveType::MovePiece,
            piece_type: piece.piece_type,
            start_position: Some(start_position),
            end_position: end_position,
        });
        self.update_turn();
        return Ok(());
    }

    pub fn get_winner(&self) -> Option<Color> {
        if self.move_num < 5 {
            return None;
        }

        let white_queen_position = self
            .board
            .pieces
            .iter()
            .filter(|(pos, _)| self.board.has_piece(pos))
            .find(|(_, pieces)| {
                let last_piece = pieces.last().unwrap();
                last_piece.color == Color::White && last_piece.piece_type == PieceType::Queen
            });
        if white_queen_position.is_none() {
            panic!("No white queen found");
        }

        let black_queen_position = self
            .board
            .pieces
            .iter()
            .filter(|(pos, _)| self.board.has_piece(pos))
            .find(|(_, pieces)| {
                let last_piece = pieces.last().unwrap();
                last_piece.color == Color::Black && last_piece.piece_type == PieceType::Queen
            });
        if black_queen_position.is_none() {
            panic!("No black queen found");
        }

        let white_queen_position = white_queen_position.unwrap().0;
        let black_queen_position = black_queen_position.unwrap().0;
        let white_queen_neighbours = white_queen_position
            .get_neighbours()
            .iter()
            .filter(|pos| self.board.has_piece(pos))
            .count();
        if white_queen_neighbours == 6 {
            return Some(Color::Black);
        }
        let black_queen_neighbours = black_queen_position
            .get_neighbours()
            .iter()
            .filter(|pos| self.board.has_piece(pos))
            .count();
        if black_queen_neighbours == 6 {
            return Some(Color::White);
        }
        return None;
    }
}
