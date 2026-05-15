use std::collections::{HashMap, HashSet};
use std::ptr;
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

    fn get_legal_moves(
        &self,
        board: &mut Board,
        position: &Position,
    ) -> Result<Vec<Position>, String> {
        if !one_hive_rule(board, position)? {
            return Ok(vec![]);
        }

        let top_piece_of_position = board
            .get_top_piece(position)
            .ok_or("No piece found in position".to_string())?;

        if !ptr::eq(top_piece_of_position, self) {
            return Ok(vec![]);
        }
        let neighbours = position.get_neighbours();
        let mut legal_moves = vec![];
        let neighbours_with_piece = board.get_neighbours_with_piece(position);

        match self.piece_type {
            PieceType::Queen => {
                for neighbour in neighbours {
                    if neighbours_with_piece.contains(&neighbour) {
                        continue;
                    }
                    if !freedom_to_move_rule(board, position, &neighbour)? {
                        continue;
                    }
                    if position.get_min_distance_from_positions(&neighbours_with_piece) > 1 {
                        continue;
                    }
                    legal_moves.push(neighbour);
                }
                return Ok(legal_moves);
            }
            PieceType::SoldierAnt => {
                let piece = board
                    .pieces
                    .get_mut(position)
                    .ok_or("No piece found in position".to_string())?
                    .pop()
                    .ok_or("No piece found in position".to_string())?
                    .clone();

                let mut visited: HashSet<Position> = HashSet::new();

                fn dfs(
                    position: &Position,
                    visited: &mut HashSet<Position>,
                    board: &mut Board,
                ) -> Result<(), String> {
                    visited.insert(position.clone());
                    let neighbours_with_piece = board.get_neighbours_with_piece(position);
                    for neighbour in position.get_neighbours() {
                        if !neighbours_with_piece.contains(&neighbour)
                            && !visited.contains(&neighbour)
                            && freedom_to_move_rule(board, position, &neighbour)?
                            && neighbour.get_min_distance_from_positions(&neighbours_with_piece)
                                == 1
                        {
                            dfs(&neighbour, visited, board)?;
                        }
                    }
                    Ok(())
                }

                dfs(position, &mut visited, board)?;
                visited.remove(position);
                board.pieces.insert(*position, vec![piece]);
                legal_moves.extend(visited.iter().map(|position| position.clone()));
                return Ok(legal_moves);
            }
            PieceType::Beetle => {
                for neighbour in neighbours {
                    if position.get_min_distance_from_positions(&neighbours_with_piece) <= 1 {
                        legal_moves.push(neighbour);
                    }
                }
                return Ok(legal_moves);
            }
            PieceType::Grasshopper => {
                for neighbour in neighbours {
                    if !neighbours_with_piece.contains(&neighbour) {
                        continue;
                    }
                    let unit_vec = position.unit_vec(&neighbour);
                    let mut current_position = neighbour.add(&unit_vec);
                    while board.has_piece(&current_position) {
                        current_position = current_position.add(&unit_vec);
                    }
                    legal_moves.push(current_position);
                }

                return Ok(legal_moves);
            }
            PieceType::Spider => {
                let piece = board
                    .pieces
                    .get_mut(position)
                    .ok_or("No piece found in position".to_string())?
                    .pop()
                    .ok_or("No piece found in position".to_string())?
                    .clone();

                let mut visited: HashSet<Position> = HashSet::new();
                let mut legal_moves_set: HashSet<Position> = HashSet::new();

                fn dfs(
                    position: &Position,
                    visited: &mut HashSet<Position>,
                    board: &mut Board,
                    move_num: u8,
                    legal_moves_set: &mut HashSet<Position>,
                ) -> Result<(), String> {
                    if move_num > 3 {
                        legal_moves_set.insert(position.clone());
                        return Ok(());
                    }
                    visited.insert(position.clone());
                    let neighbours_with_piece = board.get_neighbours_with_piece(position);
                    for neighbour in position.get_neighbours() {
                        if !neighbours_with_piece.contains(&neighbour)
                            && !visited.contains(&neighbour)
                            && freedom_to_move_rule(board, position, &neighbour)?
                            && neighbour.get_min_distance_from_positions(&neighbours_with_piece)
                                == 1
                        {
                            dfs(&neighbour, visited, board, move_num + 1, legal_moves_set)?;
                        }
                    }
                    Ok(())
                }

                dfs(position, &mut visited, board, 1, &mut legal_moves_set)?;
                board.pieces.insert(*position, vec![piece]);
                legal_moves.extend(legal_moves_set.iter().map(|position| position.clone()));
                return Ok(legal_moves);
            }
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

    fn get_min_distance_from_positions(&self, positions: &Vec<Position>) -> u8 {
        return positions
            .iter()
            .map(|pos| pos.get_distance(self))
            .min()
            .unwrap_or(u8::MAX);
    }

    fn get_distance(&self, other: &Position) -> u8 {
        return (((self.q - other.q).abs() + (self.s - other.s).abs() + (self.r - other.r).abs())
            as u8)
            / 2;
    }

    fn diff(&self, other: &Position) -> Position {
        return Position::new(other.q - self.q, other.s - self.s, other.r - self.r).unwrap();
    }

    fn add(&self, other: &Position) -> Position {
        return Position::new(self.q + other.q, self.s + other.s, self.r + other.r).unwrap();
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

    fn has_piece(&self, position: &Position) -> bool {
        self.pieces.contains_key(position) && !self.pieces.get(position).unwrap().is_empty()
    }

    fn get_neighbours_with_piece(&self, position: &Position) -> Vec<Position> {
        let mut neighbours: Vec<Position> = position
            .get_neighbours()
            .iter()
            .filter(|neighbour| self.has_piece(neighbour))
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
            .filter(|position| !self.has_piece(position))
            .collect();

        let all_other_color_empty_neighbours: HashSet<Position> = self
            .pieces
            .iter()
            .filter(|(_, pieces)| pieces.last().unwrap().color == other_color)
            .map(|(position, _)| position.get_neighbours())
            .flatten()
            .filter(|position| !self.has_piece(position))
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

trait HistoryExporter {
    fn export(&self, history: &History);
}

struct JsonHistoryExporter {
    file_path: String,
}

impl HistoryExporter for JsonHistoryExporter {
    fn export(&self, history: &History) {
    }
}

struct History {
    moves: Vec<Move>,
    exporter: Option<Box< dyn HistoryExporter>>,
}

impl History {
    fn new(exporter: Option<Box< dyn HistoryExporter>>) -> Self {
        Self { moves: Vec::new(), exporter: exporter }
    }
    fn export(&self) {
        if let Some(exporter) = &self.exporter {
            exporter.export(&self);
        }
        else {
            println!("{:?}", self.moves);
        }
    }
}

struct Game {
    board: Board,
    move_num: u16,
    turn: Color,
    white_inventory: Inventory,
    black_inventory: Inventory,
    history: History,
}

impl Game {
    fn new(history_exporter: Option<Box< dyn HistoryExporter>>) -> Self {
        Self {
            board: Board::new(),
            move_num: 1,
            turn: Color::White,
            white_inventory: Inventory::new(),
            black_inventory: Inventory::new(),
            history: History::new(history_exporter),
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
        self.history.moves.push(Move {
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
            .ok_or("Piece not found".to_string())?
            .clone();
        if piece.color != self.turn {
            return Err("Cannot move opponent's piece".to_string());
        }
        if start_position == end_position {
            return Err("Cannot move to the same position".to_string());
        }

        let player_inventory = if self.turn == Color::White {
            &self.white_inventory
        } else {
            &self.black_inventory
        };

        if self.move_num == 4 && player_inventory.Queen > 0 {
            return Err("Cannot Move. The queen has to be placed until the 4th move".to_string());
        }

        let legal_moves = piece.get_legal_moves(&mut self.board, &start_position)?;

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
        self.history.moves.push(Move {
            move_type: MoveType::MovePiece,
            piece_type: piece.piece_type,
            start_position: Some(start_position),
            end_position: end_position,
        });
        self.update_turn();
        return Ok(());
    }

    fn get_winner(&self) -> Option<Color> {
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
    let start_position = start_position.unwrap();

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
    dfs(start_position, &mut visited, board);

    let board_size = board.pieces.len();

    board.pieces.insert(position.clone(), vec![piece]);

    return Ok(visited.len() == board_size);
}

fn freedom_to_move_rule(
    board: &Board,
    position: &Position,
    adjacent_position: &Position,
) -> Result<bool, String> {
    let position_neighbours = position.get_neighbours();
    let adjacent_position_neighbours = adjacent_position.get_neighbours();

    let common_neighbours = position_neighbours
        .iter()
        .filter(|neighbour| {
            adjacent_position_neighbours.contains(neighbour) && board.has_piece(neighbour)
        })
        .count();

    return Ok(common_neighbours < 2);
}

fn main() {
    println!("Hello, world!");
    let mut game = Game::new(None);
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

    game.history.export();
}
