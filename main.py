from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import math
import os
from pathlib import Path
from typing import Protocol, runtime_checkable

import matplotlib

# TkAgg needs a working Tcl/Tk; many WSL/minimal Linux images lack it. When
# MPLBACKEND is unset, use a headless backend and write a PNG from visualize_hive.
if os.environ.get("MPLBACKEND") is None:
    matplotlib.use("Agg")

import matplotlib.patches as mpatches
import matplotlib.pyplot as plt
from matplotlib.patches import Circle, RegularPolygon
from pydantic import BaseModel, ConfigDict, Field, model_validator


def position_to_plane_xy(pos: Position) -> tuple[float, float]:
    """Map hive (r, s, q) to 2D plot coordinates.

    Plane X = q - cos(60°) * (r + s), plane Y = sin(60°) * (s - r).
    """
    r, s, q = pos.r, pos.s, pos.q
    t = math.radians(60)
    px = q - math.cos(t) * (r + s)
    py = math.sin(t) * (s - r)
    return px, py


# Cube neighbors step two coords by ±1; under ``position_to_plane_xy``, adjacent
# centers are √3 apart. Pointy-top hexes sharing edges have center separation
# √3·R (circumradius R), so R = 1.
_HEX_CIRCUMRADIUS = 1.0
_HEX_ORIENTATION = math.pi / 2


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
        if not hive.one_hive_rule_general(self.position):
            return []
        possible_moves: list[Position] = []
        for adjacent_position in self.position.all_adjacent_positions:
            if (
                hive.get_min_distance_from_neighbours(self.position, adjacent_position) == 1
                and hive.blocks[adjacent_position].piece_placed is None
                and hive.freedom_to_move(self.position, adjacent_position)
            ):
                possible_moves.append(adjacent_position)
        return possible_moves


@dataclass
class Spider:
    color: Color
    position: Position | None = None

    def get_legal_moves(self, hive: Hive) -> list[Position]:
        if not hive.one_hive_rule_general(self.position):
            return []
        hive.blocks[self.position].piece_placed = None
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
                    and hive.blocks[adjacent_position].piece_placed is None
                    and hive.freedom_to_move(position, adjacent_position)
                    and adjacent_position not in visited
                ):
                    dfs(adjacent_position, visited, move_num + 1)
        dfs(self.position, visited, 1)
                    
        hive.blocks[self.position].piece_placed = self
        return list(legal_moves)


@dataclass
class Beetle:
    color: Color
    position: Position | None = None

    def get_legal_moves(self, hive: Hive) -> list[Position]:
        if not hive.one_hive_rule_general(self.position):
            return []
        possible_moves: list[Position] = []
        for adjacent_position in self.position.all_adjacent_positions:
            if (
                hive.get_min_distance_from_neighbours(self.position, adjacent_position) == 1
            ):
                possible_moves.append(adjacent_position)
        return possible_moves


@dataclass
class Grasshopper:
    color: Color
    position: Position | None = None

    def is_move_legal(self, hive: Hive, new_position: Position) -> bool:
        return True


@dataclass
class SoldierAnt:
    color: Color
    position: Position | None = None

    def get_legal_moves(self, hive: Hive) -> list[Position]:
        if not hive.one_hive_rule_general(self.position):
            return []
        hive.blocks[self.position].piece_placed = None
        visited: set[Position] = {self.position}
        
        def dfs(position: Position, visited: set[Position]) -> None:
            visited.add(position)
            for adjacent_position in position.all_adjacent_positions:
                if (
                    hive.get_min_distance_from_neighbours(position, adjacent_position) == 1
                    and hive.blocks[adjacent_position].piece_placed is None
                    and hive.freedom_to_move(position, adjacent_position)
                    and adjacent_position not in visited
                ):
                    dfs(adjacent_position, visited)
        dfs(self.position, visited)
                    
        hive.blocks[self.position].piece_placed = self
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
    
    def __sub__(self, other: Position) -> int:
        return int((abs(self.q - other.q) + abs(self.s - other.s) + abs(self.r - other.r))/2)


class Block(BaseModel):
    model_config = ConfigDict(arbitrary_types_allowed=True)
    piece_placed: Piece | None = None
    position: Position

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
            if block.piece_placed is not None
        }

    def get_adjacent_positions_w_pieces(self, position: Position) -> bool:
        return [pos for pos in position.all_adjacent_positions if pos in self.blocks_w_pieces]

    def is_placement_legal(self, piece: Piece, position: Position) -> bool:
        if (
            self.game_state.move_number == 1
            and self.blocks[position].piece_placed is None
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
            self.blocks[position].piece_placed is not None
        ):  # No piece can be placed on top of another piece
            return False

        if all(
            pos not in self.blocks or self.blocks[pos].piece_placed is None
            for pos in position.all_adjacent_positions
        ):  # Cannot split the hive in two
            return False

        if any(
            pos in self.blocks
            and self.blocks[pos].piece_placed is not None
            and self.blocks[pos].piece_placed.color != piece.color
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
        turn = piece.color
        if not self.is_placement_legal(piece, position):
            raise ValueError("Placement is not legal")
        if turn == Color.BLACK:
            self.game_state.move_number += 1
        piece.position = position
        inventory = self.black_inventory if turn == Color.BLACK else self.white_inventory
        inventory.place_piece(piece)
        self.blocks[position].piece_placed = piece


    def move_piece(self, piece: Piece, new_position: Position) -> None:
        if piece.position is None:
            raise ValueError("Piece is not placed")
        legal_moves = piece.get_legal_moves(self)
        if new_position not in legal_moves:
            raise ValueError("Move is not legal")
        if piece.color == Color.BLACK:
            self.game_state.move_number += 1
        self.blocks[piece.position].piece_placed = None
        piece.position = new_position
        self.blocks[new_position].piece_placed = piece

    def get_min_distance_from_neighbours(self, position: Position, new_position: Position) -> int:
        return min(new_position - neighbour for neighbour in position.get_adjacent_positions_w_pieces(self.blocks_w_pieces))


# Insect markers inside occupied hexes (player color stays on hex face).
_PIECE_CENTER_COLOR: dict[type, str] = {
    Queen: "#f1c40f",  # yellow
    Grasshopper: "#27ae60",  # green
    SoldierAnt: "#2980b9",  # blue
    Spider: "#8b4513",  # brown
    Beetle: "#8e44ad",  # purple
}


def _piece_insect_facecolor(piece: Piece) -> str:
    for cls, face in _PIECE_CENTER_COLOR.items():
        if isinstance(piece, cls):
            return face
    return "#888888"


def visualize_hive(hive: Hive, *, title: str = "Hive") -> None:
    """Draw the hive in the hex plane: empty cells vs pieces by color."""
    empty_positions: list[Position] = []
    white_positions: list[Position] = []
    black_positions: list[Position] = []

    for pos, block in hive.blocks.items():
        if block.piece_placed is None:
            empty_positions.append(pos)
        elif block.piece_placed.color == Color.WHITE:
            white_positions.append(pos)
        else:
            black_positions.append(pos)

    hex_radius = _HEX_CIRCUMRADIUS
    projected = [position_to_plane_xy(p) for p in hive.blocks]

    fig, ax = plt.subplots(figsize=(8, 7))

    def _draw_hexes(
        positions: list[Position],
        *,
        face: str,
        edge: str,
        alpha: float,
        zorder: int,
    ) -> None:
        for pos in positions:
            px, py = position_to_plane_xy(pos)
            ax.add_patch(
                RegularPolygon(
                    (px, py),
                    numVertices=6,
                    radius=hex_radius,
                    orientation=_HEX_ORIENTATION,
                    facecolor=face,
                    edgecolor=edge,
                    linewidth=1.0,
                    alpha=alpha,
                    zorder=zorder,
                )
            )

    _draw_hexes(
        empty_positions,
        face="#d0d0d0",
        edge="#888888",
        alpha=0.55,
        zorder=1,
    )
    _draw_hexes(
        white_positions,
        face="#f8f4e8",
        edge="#333333",
        alpha=0.95,
        zorder=2,
    )
    _draw_hexes(
        black_positions,
        face="#1a1a1a",
        edge="#666666",
        alpha=0.95,
        zorder=2,
    )

    center_radius = hex_radius * 0.42
    for pos in white_positions + black_positions:
        piece = hive.blocks[pos].piece_placed
        if piece is None:
            continue
        px, py = position_to_plane_xy(pos)
        ax.add_patch(
            Circle(
                (px, py),
                radius=center_radius,
                facecolor=_piece_insect_facecolor(piece),
                edgecolor="#222222" if piece.color == Color.WHITE else "#cccccc",
                linewidth=0.8,
                alpha=0.95,
                zorder=3,
            )
        )

    all_xy = projected
    if all_xy:
        xs, ys = zip(*all_xy, strict=True)
        pad = hex_radius + 0.35
        ax.set_xlim(min(xs) - pad, max(xs) + pad)
        ax.set_ylim(min(ys) - pad, max(ys) + pad)

    ax.set_aspect("equal")
    ax.set_xlabel("Plane x")
    ax.set_ylabel("Plane y")
    ax.set_title(title)
    ax.legend(
        handles=[
            mpatches.Patch(
                facecolor="#d0d0d0", edgecolor="#888888", label="Empty block"
            ),
            mpatches.Patch(
                facecolor="#f8f4e8", edgecolor="#333333", label="White piece"
            ),
            mpatches.Patch(
                facecolor="#1a1a1a", edgecolor="#666666", label="Black piece"
            ),
            mpatches.Patch(
                facecolor=_PIECE_CENTER_COLOR[Queen],
                edgecolor="#333333",
                label="Queen",
            ),
            mpatches.Patch(
                facecolor=_PIECE_CENTER_COLOR[Grasshopper],
                edgecolor="#333333",
                label="Grasshopper",
            ),
            mpatches.Patch(
                facecolor=_PIECE_CENTER_COLOR[SoldierAnt],
                edgecolor="#333333",
                label="Ant",
            ),
            mpatches.Patch(
                facecolor=_PIECE_CENTER_COLOR[Spider],
                edgecolor="#333333",
                label="Spider",
            ),
            mpatches.Patch(
                facecolor=_PIECE_CENTER_COLOR[Beetle],
                edgecolor="#333333",
                label="Beetle",
            ),
        ],
        loc="upper left",
        fontsize=8,
    )
    plt.grid()
    plt.tight_layout()
    if os.environ.get("MPLBACKEND") is None:
        out_path = Path(__file__).resolve().parent / "hive_view.png"
        fig.savefig(out_path, dpi=150, bbox_inches="tight")
        print(f"Hive plot saved to {out_path}")
    else:
        plt.show()
    plt.close(fig)


def main():
    hive = Hive()
    gh1 = Grasshopper(color=Color.WHITE)
    hive.place_piece(gh1, Position(q=-1, s=0, r=1))
    q1 = Queen(color=Color.BLACK)
    q2 = Queen(color=Color.WHITE)
    b1 = Beetle(color=Color.BLACK)
    sa1 = SoldierAnt(color=Color.WHITE)
    sp1 = Spider(color=Color.BLACK)
    hive.place_piece(q1, Position(q=0, s=0, r=0))
    hive.place_piece(Grasshopper(color=Color.WHITE), Position(q=-2, s=1, r=1))
    hive.place_piece(Grasshopper(color=Color.BLACK), Position(q=0, s=1, r=-1))
    hive.place_piece(q2, Position(q=-3, s=2, r=1))
    hive.place_piece(b1, Position(q=1, s=0, r=-1))
    hive.place_piece(sa1, Position(q=-2, s=0, r=2))
    hive.place_piece(sp1, Position(q=-1, s=2, r=-1))
    hive.move_piece(q2, Position(q=-3, s=1, r=2))
    visualize_hive(hive)
    
    print(len(sp1.get_legal_moves(hive)), sp1.get_legal_moves(hive))


if __name__ == "__main__":
    main()
