use crate::hive::error::HiveError;
use crate::hive::types::PieceType;

pub(crate) struct Inventory {
    pub(crate) Grasshopper: u8,
    pub(crate) Beetle: u8,
    pub(crate) Spider: u8,
    pub(crate) Ant: u8,
    pub(crate) Queen: u8,
    pub(crate) Mosquito: u8,
    pub(crate) Ladybug: u8,
    pub(crate) Pillbug: u8,
}

impl Inventory {
    pub(crate) fn new(m: bool, l: bool, p: bool) -> Self {
        Self {
            Grasshopper: 3,
            Beetle: 2,
            Spider: 2,
            Ant: 3,
            Queen: 1,
            Mosquito: m as u8,
            Ladybug: l as u8,
            Pillbug: p as u8,
        }
    }

    pub(crate) fn place_piece(&mut self, piece_type: PieceType) -> Result<(), HiveError> {
        let count = match piece_type {
            PieceType::Grasshopper => &mut self.Grasshopper,
            PieceType::Beetle => &mut self.Beetle,
            PieceType::Spider => &mut self.Spider,
            PieceType::Ant => &mut self.Ant,
            PieceType::Queen => &mut self.Queen,
            PieceType::Mosquito => &mut self.Mosquito,
            PieceType::Ladybug => &mut self.Ladybug,
            PieceType::Pillbug => &mut self.Pillbug,
        };
        if *count == 0 {
            Err(HiveError::NoPiecesLeft(piece_type))
        } else {
            *count -= 1;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hive::types::PieceType;

    #[test]
    fn new_without_expansions_has_base_counts_only() {
        let inv = Inventory::new(false, false, false);
        assert_eq!(inv.Grasshopper, 3);
        assert_eq!(inv.Beetle, 2);
        assert_eq!(inv.Spider, 2);
        assert_eq!(inv.Ant, 3);
        assert_eq!(inv.Queen, 1);
        assert_eq!(inv.Mosquito, 0);
        assert_eq!(inv.Ladybug, 0);
        assert_eq!(inv.Pillbug, 0);
    }

    #[test]
    fn new_with_expansions_enables_one_of_each() {
        let inv = Inventory::new(true, true, true);
        assert_eq!(inv.Mosquito, 1);
        assert_eq!(inv.Ladybug, 1);
        assert_eq!(inv.Pillbug, 1);
    }

    #[test]
    fn new_expansion_flags_are_independent() {
        let inv = Inventory::new(true, false, true);
        assert_eq!(inv.Mosquito, 1);
        assert_eq!(inv.Ladybug, 0);
        assert_eq!(inv.Pillbug, 1);
    }

    #[test]
    fn place_piece_decrements_count() {
        let mut inv = Inventory::new(false, false, false);
        inv.place_piece(PieceType::Ant).unwrap();
        assert_eq!(inv.Ant, 2);
    }

    #[test]
    fn place_piece_can_empty_a_stack() {
        let mut inv = Inventory::new(false, false, false);
        inv.place_piece(PieceType::Queen).unwrap();
        assert_eq!(inv.Queen, 0);
    }

    #[test]
    fn place_piece_errors_when_none_left() {
        let mut inv = Inventory::new(false, false, false);
        inv.place_piece(PieceType::Queen).unwrap();
        let err = inv.place_piece(PieceType::Queen).unwrap_err();
        assert_eq!(err, HiveError::NoPiecesLeft(PieceType::Queen));
        assert_eq!(inv.Queen, 0);
    }

    #[test]
    fn place_piece_errors_for_disabled_expansion_piece() {
        let mut inv = Inventory::new(false, false, false);
        let err = inv.place_piece(PieceType::Mosquito).unwrap_err();
        assert_eq!(err, HiveError::NoPiecesLeft(PieceType::Mosquito));
    }

    #[test]
    fn place_piece_works_for_each_expansion_when_enabled() {
        let mut inv = Inventory::new(true, true, true);
        inv.place_piece(PieceType::Mosquito).unwrap();
        inv.place_piece(PieceType::Ladybug).unwrap();
        inv.place_piece(PieceType::Pillbug).unwrap();
        assert_eq!(inv.Mosquito, 0);
        assert_eq!(inv.Ladybug, 0);
        assert_eq!(inv.Pillbug, 0);
    }

    #[test]
    fn place_piece_exhausts_grasshopper_stack() {
        let mut inv = Inventory::new(false, false, false);
        for _ in 0..3 {
            inv.place_piece(PieceType::Grasshopper).unwrap();
        }
        assert_eq!(inv.Grasshopper, 0);
        assert_eq!(
            inv.place_piece(PieceType::Grasshopper).unwrap_err(),
            HiveError::NoPiecesLeft(PieceType::Grasshopper)
        );
    }
}
