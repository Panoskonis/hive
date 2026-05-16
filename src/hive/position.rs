use crate::hive::error::HiveError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub(crate) q: i8,
    pub(crate) s: i8,
    pub(crate) r: i8,
}

impl Position {
    pub fn new(q: i8, s: i8, r: i8) -> Result<Self, HiveError> {
        if q + s + r != 0 {
            return Err(HiveError::InvalidPositionConstraint);
        }

        Ok(Self { q, s, r })
    }

    pub fn get_neighbours(&self) -> Vec<Position> {
        // Safe to unwrap because we know the original
        // position is valid so the neighbours are also valid
        return vec![
            Position::new(self.q - 1, self.s + 1, self.r).unwrap(),
            Position::new(self.q + 1, self.s - 1, self.r).unwrap(),
            Position::new(self.q - 1, self.s, self.r + 1).unwrap(),
            Position::new(self.q + 1, self.s, self.r - 1).unwrap(),
            Position::new(self.q, self.s + 1, self.r - 1).unwrap(),
            Position::new(self.q, self.s - 1, self.r + 1).unwrap(),
        ];
    }

    pub fn get_min_distance_from_positions(&self, positions: &Vec<Position>) -> u8 {
        return positions
            .iter()
            .map(|pos| pos.get_distance(self))
            .min()
            .unwrap_or(u8::MAX);
    }

    pub fn get_distance(&self, other: &Position) -> u8 {
        return (((self.q - other.q).abs() + (self.s - other.s).abs() + (self.r - other.r).abs())
            as u8)
            / 2;
    }

    pub fn diff(&self, other: &Position) -> Position {
        return Position::new(other.q - self.q, other.s - self.s, other.r - self.r).unwrap();
    }

    pub fn add(&self, other: &Position) -> Position {
        return Position::new(self.q + other.q, self.s + other.s, self.r + other.r).unwrap();
    }

    pub fn unit_vec(&self, other: &Position) -> Position {
        let diff = self.diff(other);
        return Position::new(diff.q.signum(), diff.s.signum(), diff.r.signum()).unwrap();
    }
}

impl TryFrom<&str> for Position {
    type Error = HiveError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts = value.split(',').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(HiveError::InvalidPositionFormat);
        }
        let q = parts[0]
            .parse::<i8>()
            .map_err(|e| HiveError::InvalidCoordinate(e.to_string()))?;
        let s = parts[1]
            .parse::<i8>()
            .map_err(|e| HiveError::InvalidCoordinate(e.to_string()))?;
        let r = parts[2]
            .parse::<i8>()
            .map_err(|e| HiveError::InvalidCoordinate(e.to_string()))?;
        return Position::new(q, s, r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let position = Position::new(0, 0, 0).unwrap();
        assert_eq!(position.q, 0);
        assert_eq!(position.s, 0);
        assert_eq!(position.r, 0);
    }

    #[test]
    fn test_invalid_position_new() {
        let position = Position::new(1, 0, 0);
        assert!(position.is_err());
    }

    #[test]
    fn test_get_neighbours() {
        let position = Position::new(-1, 1, 0).unwrap();
        let neighbours = position.get_neighbours();
        assert_eq!(neighbours.len(), 6);
        assert!(neighbours.contains(&Position::new(-2, 2, 0).unwrap()));
        assert!(neighbours.contains(&Position::new(0, 0, 0).unwrap()));
        assert!(neighbours.contains(&Position::new(-1, 2, -1).unwrap()));
        assert!(neighbours.contains(&Position::new(-1, 0, 1).unwrap()));
        assert!(neighbours.contains(&Position::new(-2, 1, 1).unwrap()));
        assert!(neighbours.contains(&Position::new(0, 1, -1).unwrap()));
    }

    #[test]
    fn test_from_str() {
        let position = Position::try_from("0,0,0").unwrap();
        assert_eq!(position, Position::new(0, 0, 0).unwrap());
    }
    #[test]
    fn test_from_str_invalid() {
        let position = Position::try_from("00");
        assert!(position.is_err());
    }

    #[test]
    fn test_get_min_distance_from_positions() {
        let position = Position::new(0, 0, 0).unwrap();
        let positions_1 = vec![
            Position::new(1, 0, -1).unwrap(),
            Position::new(0, 1, -1).unwrap(),
            Position::new(2, -2, 0).unwrap(),
        ];
        assert_eq!(position.get_min_distance_from_positions(&positions_1), 1);
        let positions_2 = vec![
            Position::new(3, 0, -3).unwrap(),
            Position::new(0, 2, -2).unwrap(),
            Position::new(2, -2, 0).unwrap(),
        ];
        assert_eq!(position.get_min_distance_from_positions(&positions_2), 2);
    }

    #[test]
    fn test_get_distance() {
        let position = Position::new(0, 0, 0).unwrap();
        let other_position = Position::new(1, 0, -1).unwrap();
        assert_eq!(position.get_distance(&other_position), 1);
    }

    #[test]
    fn test_diff() {
        let position = Position::new(0, 0, 0).unwrap();
        let other_position = Position::new(1, 0, -1).unwrap();
    }

    #[test]
    fn test_add() {
        let position = Position::new(0, 0, 0).unwrap();
        let other_position = Position::new(1, 0, -1).unwrap();
        assert_eq!(
            position.add(&other_position),
            Position::new(1, 0, -1).unwrap()
        );
    }

    #[test]
    fn test_unit_vec() {
        let position = Position::new(0, 0, 0).unwrap();
        let other_position = Position::new(1, 0, -1).unwrap();
        assert_eq!(
            position.unit_vec(&other_position),
            Position::new(1, 0, -1).unwrap()
        );
    }
}
