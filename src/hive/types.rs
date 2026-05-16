#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Queen,
    SoldierAnt,
    Beetle,
    Grasshopper,
    Spider,
}

impl TryFrom<&str> for PieceType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "queen" => Ok(PieceType::Queen),
            "q" => Ok(PieceType::Queen),
            "soldierant" => Ok(PieceType::SoldierAnt),
            "a" => Ok(PieceType::SoldierAnt),
            "beetle" => Ok(PieceType::Beetle),
            "b" => Ok(PieceType::Beetle),
            "grasshopper" => Ok(PieceType::Grasshopper),
            "g" => Ok(PieceType::Grasshopper),
            "spider" => Ok(PieceType::Spider),
            "s" => Ok(PieceType::Spider),
            _ => Err("Invalid piece type".to_string()),
        }
    }
}
