use crate::hive::position::Position;
use crate::hive::types::PieceType;

#[derive(Debug, Clone, Copy)]
pub enum MoveType {
    PlacePiece,
    MovePiece,
}

impl TryFrom<&str> for MoveType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "move" => Ok(MoveType::MovePiece),
            "m" => Ok(MoveType::MovePiece),
            "place" => Ok(MoveType::PlacePiece),
            "p" => Ok(MoveType::PlacePiece),
            _ => Err("Invalid move type".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub move_type: MoveType,
    pub piece_type: PieceType,
    pub start_position: Option<Position>,
    pub end_position: Position,
}

pub trait HistoryExporter {
    fn export(&self, history: &History);
}

pub struct JsonHistoryExporter {
    pub file_path: String,
}

impl HistoryExporter for JsonHistoryExporter {
    fn export(&self, history: &History) {}
}

pub struct History {
    pub moves: Vec<Move>,
    pub exporter: Option<Box<dyn HistoryExporter>>,
}

impl History {
    pub fn new(exporter: Option<Box<dyn HistoryExporter>>) -> Self {
        Self {
            moves: Vec::new(),
            exporter: exporter,
        }
    }
    pub fn export(&self) {
        if let Some(exporter) = &self.exporter {
            exporter.export(&self);
        } else {
            println!("{:?}", self.moves);
        }
    }
}
