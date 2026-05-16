//! Structured errors for game rules and parsing.

use std::fmt;

use crate::hive::types::PieceType;

/// High-level failure modes for Hive engine operations and `TryFrom` parsers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HiveError {
    /// Cube coordinates do not satisfy `q + s + r == 0`.
    InvalidPositionConstraint,
    /// Position string does not look like `"q,s,r"`.
    InvalidPositionFormat,
    /// A coordinate in a position string is not a valid `i8`.
    InvalidCoordinate(String),
    /// Unknown piece name in CLI / parsing.
    InvalidPieceType,
    /// Unknown move type in CLI / parsing.
    InvalidMoveType,
    /// Expected a stack or cell that is missing or empty.
    PieceNotFound,
    /// Tried to move a piece that belongs to the other player.
    WrongTurn,
    /// Start and end of a move are identical.
    SameStartAndEnd,
    /// Destination is not in the piece's legal move set.
    IllegalMoveDestination,
    /// Cell is not allowed for placement (e.g. breaks adjacent-color rules).
    IllegalPlacementPosition,
    /// Queen must be on the board before this action on or after move 4.
    QueenMustBePlaced(QueenPlacementContext),
    /// No copies of this piece type remain in hand.
    NoPiecesLeft(PieceType),
}

/// Whether the queen rule blocked a **place** or a **move**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueenPlacementContext {
    Place,
    Move,
}

impl fmt::Display for HiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HiveError::InvalidPositionConstraint => {
                write!(f, "Invalid position creation")
            }
            HiveError::InvalidPositionFormat => {
                write!(f, "Invalid position. A position is like '0,0,0'.")
            }
            HiveError::InvalidCoordinate(msg) => write!(f, "{msg}"),
            HiveError::InvalidPieceType => write!(f, "Invalid piece type"),
            HiveError::InvalidMoveType => write!(f, "Invalid move type"),
            HiveError::PieceNotFound => write!(f, "Piece not found"),
            HiveError::WrongTurn => write!(f, "Cannot move opponent's piece"),
            HiveError::SameStartAndEnd => write!(f, "Cannot move to the same position"),
            HiveError::IllegalMoveDestination => write!(f, "Invalid move position"),
            HiveError::IllegalPlacementPosition => write!(f, "Invalid placement position"),
            HiveError::QueenMustBePlaced(QueenPlacementContext::Place) => {
                write!(f, "The queen has to be placed until the 4th move")
            }
            HiveError::QueenMustBePlaced(QueenPlacementContext::Move) => write!(
                f,
                "Cannot Move. The queen has to be placed until the 4th move"
            ),
            HiveError::NoPiecesLeft(PieceType::Grasshopper) => {
                write!(f, "No Grasshopper left")
            }
            HiveError::NoPiecesLeft(PieceType::Beetle) => write!(f, "No Beetle left"),
            HiveError::NoPiecesLeft(PieceType::Spider) => write!(f, "No Spider left"),
            HiveError::NoPiecesLeft(PieceType::SoldierAnt) => {
                write!(f, "No SoldierAnt left")
            }
            HiveError::NoPiecesLeft(PieceType::Queen) => write!(f, "No Queen left"),
            HiveError::NoPiecesLeft(PieceType::Mosquito) => write!(f, "No Mosquito left"),
            HiveError::NoPiecesLeft(PieceType::Ladybug) => write!(f, "No Ladybug left"),
            HiveError::NoPiecesLeft(PieceType::Pillbug) => write!(f, "No Pillbug left"),
        }
    }
}

impl std::error::Error for HiveError {}
