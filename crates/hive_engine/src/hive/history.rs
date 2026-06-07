use crate::hive::error::HiveError;
use crate::hive::position::Position;
use crate::hive::types::{Color, PieceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    PlacePiece,
    MovePiece,
    PillbugSpecialMove,
    CannotMove,
}

impl TryFrom<&str> for ActionType {
    type Error = HiveError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "move" => Ok(ActionType::MovePiece),
            "m" => Ok(ActionType::MovePiece),
            "place" => Ok(ActionType::PlacePiece),
            "p" => Ok(ActionType::PlacePiece),
            "pillbug special move" => Ok(ActionType::PillbugSpecialMove),
            "pb" => Ok(ActionType::PillbugSpecialMove),
            _ => Err(HiveError::InvalidMoveType),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Action {
    pub action_type: ActionType,
    pub piece_type: Option<PieceType>,
    pub start_position: Option<Position>,
    pub end_position: Option<Position>,
    pub turn: Color,
}

#[derive(Clone)]
pub struct History {
    pub actions: Vec<Action>,
}

impl History {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }
}
