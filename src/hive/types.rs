use crate::hive::error::HiveError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Queen,
    Ant,
    Beetle,
    Grasshopper,
    Spider,
    Mosquito,
    Ladybug,
    Pillbug,
}

impl TryFrom<&str> for PieceType {
    type Error = HiveError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "queen" => Ok(PieceType::Queen),
            "q" => Ok(PieceType::Queen),
            "Ant" => Ok(PieceType::Ant),
            "a" => Ok(PieceType::Ant),
            "beetle" => Ok(PieceType::Beetle),
            "b" => Ok(PieceType::Beetle),
            "grasshopper" => Ok(PieceType::Grasshopper),
            "g" => Ok(PieceType::Grasshopper),
            "spider" => Ok(PieceType::Spider),
            "s" => Ok(PieceType::Spider),
            "mosquito" => Ok(PieceType::Mosquito),
            "m" => Ok(PieceType::Mosquito),
            "ladybug" => Ok(PieceType::Ladybug),
            "l" => Ok(PieceType::Ladybug),
            "pillbug" => Ok(PieceType::Pillbug),
            "p" => Ok(PieceType::Pillbug),
            _ => Err(HiveError::InvalidPieceType),
        }
    }
}
