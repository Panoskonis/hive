from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import math
import os
from typing import Protocol, runtime_checkable

import matplotlib

# TkAgg needs a working Tcl/Tk; many WSL/minimal Linux images lack it. When
# MPLBACKEND is unset, use a headless backend and write a PNG from visualize_hive.
if os.environ.get("MPLBACKEND") is None:
    matplotlib.use("Agg")


from pydantic import BaseModel, ConfigDict, Field, model_validator


class GameState(BaseModel):
    move_number: int = 1



class Color(Enum):
    BLACK = "black"
    WHITE = "white"


@runtime_checkable
class Piece(Protocol):
    color: Color
    position: Position | None
    
    def get_legal_moves(self, hive: Hive) -> list[Position]: ...




@dataclass
class Queen:
    color: Color
    position: Position | None = None

    def get_legal_moves(self, hive: Hive) -> list[Position]:
        if hive.blocks[self.position].top_piece is not self:
            return []
        if not hive.one_hive_rule_general(self.position):
            return []
        possible_moves: list[Position] = []
        for adjacent_position in self.position.all_adjacent_positions:
            if (
                hive.get_min_distance_from_neighbours(self.position, adjacent_position) == 1
                and hive.blocks[adjacent_position].top_piece is None
                and hive.freedom_to_move(self.position, adjacent_position)
            ):
                possible_moves.append(adjacent_position)
        return possible_moves



@dataclass
class Move:
    start_position: Position
    end_position: Position
    move_number: int
    piece: Piece
    player: Color
    

@dataclass
class Spider:
    color: Color
    position: Position | None = None

    def get_legal_moves(self, hive: Hive) -> list[Position]:
        if hive.blocks[self.position].top_piece is not self:
            return []
        if not hive.one_hive_rule_general(self.position):
            return []
        hive.blocks[self.position].pieces_placed = []
        visited: set[Position] = {self.position}
        
        legal_moves: set[Position] = set()
        
        def dfs(position: Position, visited: set[Position], move_num: int) -> None:
            if move_num > 3:
                legal_moves.add(position)
                return
            visited.add(position)
            for adjacent_position in position.all_adjacent_positions:
                if (
                    hive.get_min_distance_from_neighbours(position, adjacent_position) == 1
                    and hive.blocks[adjacent_position].top_piece is None
                    and hive.freedom_to_move(position, adjacent_position)
                    and adjacent_position not in visited
                ):
                    dfs(adjacent_position, visited, move_num + 1)
        dfs(self.position, visited, 1)
                    
        hive.blocks[self.position].pieces_placed = [self]
        return list(legal_moves)


@dataclass
class Beetle:
    color: Color
    position: Position | None = None

    def get_legal_moves(self, hive: Hive) -> list[Position]:
        if hive.blocks[self.position].top_piece is not self:
            return []
        if not hive.one_hive_rule_general(self.position):
            return []
        possible_moves: list[Position] = []
        for adjacent_position in self.position.all_adjacent_positions:
            if (
                hive.get_min_distance_from_neighbours(self.position, adjacent_position) <= 1
            ):
                possible_moves.append(adjacent_position)
        return possible_moves


@dataclass
class Grasshopper:
    color: Color
    position: Position | None = None

    def get_legal_moves(self, hive: Hive) -> list[Position]:
        if hive.blocks[self.position].top_piece is not self:
            return []
        if not hive.one_hive_rule_general(self.position):
            return []
        legal_moves = []
        for adjacent_position in self.position.all_adjacent_positions:
            if adjacent_position not in hive.blocks_w_pieces:
                continue
            diff = adjacent_position - self.position
            
            while adjacent_position + diff in hive.blocks_w_pieces:
                adjacent_position += diff
            final_position = adjacent_position + diff
            legal_moves.append(final_position)
        return legal_moves


@dataclass
class SoldierAnt:
    color: Color
    position: Position | None = None

    def get_legal_moves(self, hive: Hive) -> list[Position]:
        if hive.blocks[self.position].top_piece is not self:
            return []
        if not hive.one_hive_rule_general(self.position):
            return []
        hive.blocks[self.position].pieces_placed = []
        visited: set[Position] = {self.position}
        
        def dfs(position: Position, visited: set[Position]) -> None:
            visited.add(position)
            for adjacent_position in position.all_adjacent_positions:
                if (
                    hive.get_min_distance_from_neighbours(position, adjacent_position) == 1
                    and hive.blocks[adjacent_position].top_piece is None
                    and hive.freedom_to_move(position, adjacent_position)
                    and adjacent_position not in visited
                ):
                    dfs(adjacent_position, visited)
        dfs(self.position, visited)
                    
        hive.blocks[self.position].pieces_placed = [self]
        visited.remove(self.position)
        return list(visited)


class PlayerInventory(BaseModel):
    model_config = ConfigDict(arbitrary_types_allowed=True)
    grasshopper_count: int = 3
    spider_count: int = 2
    beetle_count: int = 2
    soldier_ant_count: int = 3
    queen_count: int = 1

    def place_piece(self, piece: Piece) -> None:
        if isinstance(piece, Grasshopper):
            if self.grasshopper_count <= 0:
                raise ValueError("No grasshopper available")
            self.grasshopper_count -= 1
        elif isinstance(piece, Spider):
            if self.spider_count <= 0:
                raise ValueError("No spider available")
            self.spider_count -= 1
        elif isinstance(piece, Beetle):
            if self.beetle_count <= 0:
                raise ValueError("No beetle available")
            self.beetle_count -= 1
        elif isinstance(piece, SoldierAnt):
            if self.soldier_ant_count <= 0:
                raise ValueError("No soldier ant available")
            self.soldier_ant_count -= 1
        elif isinstance(piece, Queen):
            if self.queen_count <= 0:
                raise ValueError("No queen available")
            self.queen_count -= 1


@dataclass(frozen=True)
class Position:
    q: int = Field(..., ge=-28, le=28)
    s: int = Field(..., ge=-28, le=28)
    r: int = Field(..., ge=-28, le=28)

    @model_validator(mode="after")
    def validate_position(self):
        if self.r + self.s + self.q != 0:
            raise ValueError("r + s + q must be 0 based on the cube coordinates")
        return self

    @property
    def all_adjacent_positions(self) -> list[Position]:
        return [
            Position(q=self.q + 1, s=self.s - 1, r=self.r),
            Position(q=self.q - 1, s=self.s + 1, r=self.r),
            Position(q=self.q, s=self.s + 1, r=self.r - 1),
            Position(q=self.q, s=self.s - 1, r=self.r + 1),
            Position(q=self.q - 1, s=self.s, r=self.r + 1),
            Position(q=self.q + 1, s=self.s, r=self.r - 1),
        ]

    def get_adjacent_positions_w_pieces(
        self, blocks_w_pieces: dict[Position, Block]
    ) -> list[Position]:
        return [pos for pos in self.all_adjacent_positions if pos in blocks_w_pieces]
    
    def dist(self, other: Position) -> int:
        return int((abs(self.q - other.q) + abs(self.s - other.s) + abs(self.r - other.r))/2)
    
    def __sub__(self, other: Position) -> int:
        return Position(q=self.q - other.q, s=self.s - other.s, r=self.r - other.r)

    def __add__(self, other: Position) -> Position:
        return Position(q=self.q + other.q, s=self.s + other.s, r=self.r + other.r)

class Block(BaseModel):
    model_config = ConfigDict(arbitrary_types_allowed=True)
    pieces_placed: list[Piece] = Field(default_factory=list)
    position: Position
    
    @property
    def top_piece(self) -> Piece | None:
        return self.pieces_placed[-1] if self.pieces_placed else None
    
    @property
    def bottom_piece(self) -> Piece | None:
        return self.pieces_placed[0] if self.pieces_placed else None

    @property
    def color(self) -> Color | None:
        return self.top_piece.color if self.top_piece else None

    def get_adjacent_positions_w_pieces(
        self, blocks_w_pieces: dict[Position, Block]
    ) -> list[Position]:
        return self.position.get_adjacent_positions_w_pieces(blocks_w_pieces)



# tmp solution
def init_blocks() -> dict[Position, Block]:
    blocks = {}
    for q in range(-10, 11):
        for s in range(-10, 11):
            for r in range(-10, 11):
                if q + s + r != 0:
                    continue
                pos = Position(q=q, s=s, r=r)
                blocks[pos] = Block(position=pos)
    return blocks


class Hive(BaseModel):
    blocks: dict[Position, Block] = Field(default_factory=init_blocks)
    white_inventory: PlayerInventory = Field(default_factory=PlayerInventory)
    black_inventory: PlayerInventory = Field(default_factory=PlayerInventory)
    game_state: GameState = Field(default_factory=GameState)

    @property
    def blocks_w_pieces(self) -> dict[Position, Block]:
        return {
            pos: block
            for pos, block in self.blocks.items()
            if len(block.pieces_placed) > 0
        }

    def get_adjacent_positions_w_pieces(self, position: Position) -> bool:
        return [pos for pos in position.all_adjacent_positions if pos in self.blocks_w_pieces]

    def is_placement_legal(self, piece: Piece, position: Position) -> bool:
        if (
            self.game_state.move_number == 1
            and self.blocks[position].top_piece is None
        ):  # In first move pieces can be placed anywhere
            return True
        inventory = (
            self.white_inventory if piece.color == Color.WHITE else self.black_inventory
        )
        if (
            self.game_state.move_number >= 4
            and isinstance(piece, Queen)
            and inventory.queen_count != 0
        ):
            return False
        if (
            self.blocks[position].top_piece is not None
        ):  # No piece can be placed on top of another piece
            return False

        if all(
            pos not in self.blocks or self.blocks[pos].top_piece is None
            for pos in position.all_adjacent_positions
        ):  # Cannot split the hive in two
            return False

        if any(
            pos in self.blocks
            and self.blocks[pos].top_piece is not None
            and self.blocks[pos].color != piece.color
            for pos in position.all_adjacent_positions
        ):  # Cannot place a piece next to a different color piece
            return False
        return True
            
        

    def get_legal_placement_positions(self, piece: Piece) -> list[Position]:
        return [pos for pos in self.blocks if self.is_placement_legal(piece, pos)]

    def one_hive_rule_general(self, piece_position: Position) -> bool:
        if self.game_state.move_number == 1:
            return True
        blocks_w_pieces = self.blocks_w_pieces.copy()

        starting_position = blocks_w_pieces[
            piece_position
        ].get_adjacent_positions_w_pieces(blocks_w_pieces)[0]
        
        if len(blocks_w_pieces[piece_position].pieces_placed) == 1:
            blocks_w_pieces.pop(piece_position)

        visited: set[Position] = set()

        def dfs(position: Position, searched_positions: set[Position]) -> None:
            searched_positions.add(position)

            for adjacent_position in position.get_adjacent_positions_w_pieces(
                blocks_w_pieces
            ):
                if adjacent_position in searched_positions:
                    continue
                dfs(adjacent_position, searched_positions)

        dfs(starting_position, visited)

        return len(visited) == len(blocks_w_pieces)
        
    
    def freedom_to_move(self, position: Position, next_position: Position) -> bool:
        if next_position not in position.all_adjacent_positions:
            raise ValueError("Freedom to move can only be determined between"
                             " the current position and one adjacent position")
        
        pos_neighbours_pieces = position.get_adjacent_positions_w_pieces(self.blocks_w_pieces)
        next_pos_neighbours_pieces = next_position.get_adjacent_positions_w_pieces(self.blocks_w_pieces)
        
        common_neighbours_w_pieces = set(pos_neighbours_pieces) & set(next_pos_neighbours_pieces)
        length = len(common_neighbours_w_pieces)
        if length > 2:
            raise ValueError("Impossible for 2 neighbouring pieces to have"
                             " more than 2 common neighbours")
        
        return length < 2


    def place_piece(
        self, piece: Piece, position: Position
    ) -> None:
        player = piece.color
        if not self.is_placement_legal(piece, position):
            raise ValueError("Placement is not legal")
        if player == Color.BLACK:
            self.game_state.move_number += 1
        piece.position = position
        inventory = self.black_inventory if player == Color.BLACK else self.white_inventory
        inventory.place_piece(piece)
        self.blocks[position].pieces_placed = [piece]


    def move_piece(self, piece: Piece, new_position: Position) -> None:
        if piece.position is None:
            raise ValueError("Piece is not placed")
        legal_moves = piece.get_legal_moves(self)
        if new_position not in legal_moves:
            raise ValueError("Move is not legal")
        if piece.color == Color.BLACK:
            self.game_state.move_number += 1
        self.blocks[piece.position].pieces_placed.pop()
        piece.position = new_position
        self.blocks[new_position].pieces_placed.append(piece)

    def get_min_distance_from_neighbours(self, position: Position, new_position: Position) -> int:
        neighbours = position.get_adjacent_positions_w_pieces(self.blocks_w_pieces)
        if len(self.blocks[position].pieces_placed) > 1:
            neighbours.append(position)
        return min(new_position.dist(neighbour) for neighbour in neighbours)
    
    def get_winner(self) -> Color | None:
        if self.game_state.move_number < 4:
            return None
        def find_queen(color: Color) -> Position:
            for block in self.blocks_w_pieces:
                if block.bottom_piece.color == color and isinstance(block.bottom_piece, Queen):
                    return block.position
            raise ValueError(f"No queen found for {color.value}.")
        white_queen_pos = find_queen(Color.WHITE)
        black_queen_pos = find_queen(Color.BLACK)
        if len(white_queen_pos.get_adjacent_positions_w_pieces(self.blocks_w_pieces)) == 6:
            return Color.BLACK
        if len(black_queen_pos.get_adjacent_positions_w_pieces(self.blocks_w_pieces)) == 6:
            return Color.BLACK
        return None
        