use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PieceType {
    Queen,
    SoldierAnt,
    Beetle,
    Grasshopper,
    Spider,
}

#[derive(Debug, Clone, Copy)]
struct Piece {
    color: Color,
    piece_type: PieceType,
}

impl Piece {
    fn new(color: Color, piece_type: PieceType) -> Self {
        Self { color, piece_type }
    }

    fn get_legal_moves(&self, board: & mut Board, position: &Position) -> Vec<Position> {
        let one_hive_rule_result = one_hive_rule(board, position);
        if one_hive_rule_result.is_err() || !one_hive_rule_result.unwrap() {
            return vec![];
        }

        match self.piece_type {
            PieceType::Queen => {
                let neighbours = position.get_neighbours();
                let mut legal_moves = vec![];
                let neighbours_with_piece = board.get_neighbours_with_piece(position);


                for neighbour in neighbours {
                    if neighbours_with_piece.contains(&neighbour) {
                        continue;
                    }
                    let freedom_to_move_rule_result = freedom_to_move_rule(board, position, &neighbour);
                    if freedom_to_move_rule_result.is_err() || !freedom_to_move_rule_result.unwrap() {
                        continue;
                    }
                    let min_distance = get_min_distance_of_position_from_positions(&position, &neighbours_with_piece);
                    if min_distance > 1 {
                        continue;
                    }
                    legal_moves.push(neighbour);
                }
                return legal_moves;
            }
            PieceType::SoldierAnt => return vec![],
            PieceType::Beetle => return vec![],
            PieceType::Grasshopper => return vec![],
            PieceType::Spider => return vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    q: i8,
    s: i8,
    r: i8,
}

impl Position {
    fn new(q: i8, s: i8, r: i8) -> Result<Self, String> {
        if q + s + r != 0 {
            return Err("Invalid position".to_string());
        }

        Ok(Self { q, s, r })
    }

    fn get_neighbours(&self) -> Vec<Position> {
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

    fn get_distance(&self, other: &Position) -> u8 {
        return (((self.q - other.q).abs() + (self.s - other.s).abs() + (self.r - other.r).abs())
            as u8)
            / 2;
    }

    fn diff(&self, other: &Position) -> Position {
        return Position::new(other.q - self.q, other.s - self.s, other.r - self.r).unwrap();
    }

    fn unit_vec(&self, other: &Position) -> Position {
        let diff = self.diff(other);
        return Position::new(diff.q.signum(), diff.s.signum(), diff.r.signum()).unwrap();
    }
}

#[derive(Debug, Clone)]
struct Board {
    pieces: HashMap<Position, Vec<Piece>>,
}

impl Board {
    fn new() -> Self {
        Self {
            pieces: HashMap::new(),
        }
    }

    fn get_pieces_copy(&self, position: &Position) -> Vec<Piece> {
        self.pieces.get(position).unwrap_or(&vec![]).clone()
    }

    fn get_top_piece(&self, position: &Position) -> Option<&Piece> {
        self.pieces.get(position).and_then(|pieces| pieces.last())
    }

    fn get_bottom_piece(&self, position: &Position) -> Option<&Piece> {
        self.pieces.get(position).and_then(|pieces| pieces.first())
    }

    fn get_neighbours_with_piece(&self, position: &Position) -> Vec<Position> {
        let mut neighbours: Vec<Position> = position
            .get_neighbours()
            .iter()
            .filter(|neighbour| self.pieces.contains_key(neighbour))
            .map(|neighbour| neighbour.clone())
            .collect();

        if self.pieces.get(position).unwrap().len() > 1 {
            neighbours.push(position.clone());
        }
        return neighbours;
    }

    fn get_all_allowed_placement_positions(&self, color: Color) -> Vec<Position> {
        let other_color = if color == Color::White {
            Color::Black
        } else {
            Color::White
        };

        let all_color_empty_neighbours: HashSet<Position> = self
            .pieces
            .iter()
            .filter(|(_, pieces)| pieces.last().unwrap().color == color)
            .map(|(position, _)| position.get_neighbours())
            .flatten()
            .filter(|position| !self.pieces.contains_key(position))
            .collect();

        let all_other_color_empty_neighbours: HashSet<Position> = self
            .pieces
            .iter()
            .filter(|(_, pieces)| pieces.last().unwrap().color == other_color)
            .map(|(position, _)| position.get_neighbours())
            .flatten()
            .filter(|position| !self.pieces.contains_key(position))
            .collect();

        return all_color_empty_neighbours
            .difference(&all_other_color_empty_neighbours)
            .copied()
            .collect();
    }
}

struct Inventory {
    Grasshopper: u8,
    Beetle: u8,
    Spider: u8,
    SoldierAnt: u8,
    Queen: u8,
}

impl Inventory {
    fn new() -> Self {
        Self {
            Grasshopper: 3,
            Beetle: 2,
            Spider: 2,
            SoldierAnt: 3,
            Queen: 1,
        }
    }

    fn place_piece(&mut self, piece_type: PieceType) -> Result<(), String> {
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

#[derive(Debug, Clone, Copy)]
enum MoveType {
    PlacePiece,
    MovePiece,
}

#[derive(Debug, Clone, Copy)]
struct Move {
    move_type: MoveType,
    piece_type: PieceType,
    start_position: Option<Position>,
    end_position: Position,
}

struct Game {
    board: Board,
    move_num: u16,
    turn: Color,
    white_inventory: Inventory,
    black_inventory: Inventory,
    history: Vec<Move>,
}

impl Game {
    fn new() -> Self {
        Self {
            board: Board::new(),
            move_num: 1,
            turn: Color::White,
            white_inventory: Inventory::new(),
            black_inventory: Inventory::new(),
            history: Vec::new(),
        }
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
    ) -> Result<(), String> {
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
        self.history.push(Move {
            move_type: MoveType::PlacePiece,
            piece_type: piece_type,
            start_position: None,
            end_position: position,
        });
        return Ok(());
    }

    fn place_piece_with_checks(
        &mut self,
        piece_type: PieceType,
        position: Position,
    ) -> Result<(), String> {
        if (self.move_num == 1) && (self.turn == Color::White) {
            self.place_piece(piece_type, Position::new(0, 0, 0).unwrap(), self.turn)?;
            return Ok(());
        }
        let player_inventory = if self.turn == Color::White {
            &mut self.white_inventory
        } else {
            &mut self.black_inventory
        };

        if self.move_num == 4 && player_inventory.Queen > 0 && piece_type != PieceType::Queen {
            return Err("The queen has to be placed until the 4th move".to_string());
        }

        if !self
            .board
            .get_all_allowed_placement_positions(self.turn)
            .contains(&position)
        {
            return Err("Invalid position".to_string());
        }

        self.place_piece(piece_type, position, self.turn)?;
        return Ok(());
    }

    fn move_piece_with_checks(
        &mut self,
        start_position: Position,
        end_position: Position,
    ) -> Result<(), String> {
        let piece = self
            .board
            .get_top_piece(&start_position)
            .ok_or("Piece not found".to_string())?.clone();
        if piece.color != self.turn {
            return Err("Cannot move opponent's piece".to_string());
        }
        if start_position == end_position {
            return Err("Cannot move to the same position".to_string());
        }
        let legal_moves = piece.get_legal_moves(&mut self.board, &start_position);

        if !legal_moves.contains(&end_position) {
            return Err("Invalid position".to_string());
        }

        let pieces_start = self
            .board
            .pieces
            .get_mut(&start_position)
            .ok_or("Piece not found".to_string())?;
        let piece = pieces_start.pop().ok_or("Piece not found".to_string())?;
        if pieces_start.is_empty() {
            self.board.pieces.remove(&start_position);
        }
        let mut pieces_end = self.board.get_pieces_copy(&end_position);
        pieces_end.push(piece);
        self.board.pieces.insert(end_position, pieces_end);
        self.history.push(Move {
            move_type: MoveType::MovePiece,
            piece_type: piece.piece_type,
            start_position: Some(start_position),
            end_position: end_position,
        });
        self.update_turn();
        return Ok(());
    }
}

fn get_min_distance_of_position_from_positions(
    position: &Position,
    position_vec: &Vec<Position>,
) -> u8 {
    return position_vec
        .iter()
        .map(|pos| pos.get_distance(position))
        .min()
        .unwrap_or(u8::MAX);
}

fn one_hive_rule(board: &mut Board, position: &Position) -> Result<bool, String> {
    let neighbours = board.get_neighbours_with_piece(position);
    let mut pieces = board.get_pieces_copy(position);

    if pieces.len() > 1 {
        return Ok(true);
    }

    let start_position = neighbours.first();
    if start_position.is_none() {
        return Ok(true);
    }

    let piece = pieces
        .pop()
        .ok_or("No piece found in position".to_string())?;
    board.pieces.remove(position);

    let mut visited: HashSet<Position> = HashSet::new();

    fn dfs(position: &Position, visited: &mut HashSet<Position>, board: &Board) {
        if visited.contains(position) {
            return;
        }
        visited.insert(position.clone());
        for neighbour in board.get_neighbours_with_piece(position) {
            dfs(&neighbour, visited, board);
        }
    }
    dfs(start_position.unwrap(), &mut visited, board);

    let board_size = board.pieces.len();

    board.pieces.insert(position.clone(), vec![piece]);

    return Ok(visited.len() == board_size);
}

fn freedom_to_move_rule(
    board: &Board,
    position: &Position,
    adjacent_position: &Position,
) -> Result<bool, String> {
    board
        .get_top_piece(position)
        .ok_or("No piece found in position".to_string())?;
    let position_neighbours = position.get_neighbours();
    let adjacent_position_neighbours = adjacent_position.get_neighbours();

    let common_neighbours = position_neighbours
        .iter()
        .filter(|neighbour| adjacent_position_neighbours.contains(neighbour))
        .count();

    return Ok(common_neighbours < 2);
}

fn main() {
    println!("Hello, world!");
    let mut game = Game::new();
    let piece = Piece::new(Color::White, PieceType::Queen);
    game.place_piece_with_checks(PieceType::Queen, Position::new(0, 0, 0).unwrap())
        .unwrap();
    println!(
        "{:?}",
        game.board
            .pieces
            .get(&Position::new(0, 0, 0).unwrap())
            .unwrap()
    );

    println!("{:?}", game.history);
}
