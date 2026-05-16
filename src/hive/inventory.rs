use crate::hive::types::PieceType;

pub(crate) struct Inventory {
    pub(crate) Grasshopper: u8,
    pub(crate) Beetle: u8,
    pub(crate) Spider: u8,
    pub(crate) SoldierAnt: u8,
    pub(crate) Queen: u8,
}

impl Inventory {
    pub(crate) fn new() -> Self {
        Self {
            Grasshopper: 3,
            Beetle: 2,
            Spider: 2,
            SoldierAnt: 3,
            Queen: 1,
        }
    }

    pub(crate) fn place_piece(&mut self, piece_type: PieceType) -> Result<(), String> {
        match piece_type {
            PieceType::Grasshopper => {
                if self.Grasshopper == 0 {
                    Err("No Grasshopper left".to_string())
                } else {
                    self.Grasshopper -= 1;
                    Ok(())
                }
            }
            PieceType::Beetle => {
                if self.Beetle == 0 {
                    Err("No Beetle left".to_string())
                } else {
                    self.Beetle -= 1;
                    Ok(())
                }
            }
            PieceType::Spider => {
                if self.Spider == 0 {
                    Err("No Spider left".to_string())
                } else {
                    self.Spider -= 1;
                    Ok(())
                }
            }
            PieceType::SoldierAnt => {
                if self.SoldierAnt == 0 {
                    Err("No SoldierAnt left".to_string())
                } else {
                    self.SoldierAnt -= 1;
                    Ok(())
                }
            }
            PieceType::Queen => {
                if self.Queen == 0 {
                    Err("No Queen left".to_string())
                } else {
                    self.Queen -= 1;
                    Ok(())
                }
            }
        }
    }
}
