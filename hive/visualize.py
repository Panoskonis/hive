import matplotlib.patches as mpatches
import matplotlib.pyplot as plt
from matplotlib.patches import Circle, RegularPolygon
import math
import os
from pathlib import Path
from .engine import Hive, Position, Color, Queen, Grasshopper, SoldierAnt, Spider, Beetle, Block, Piece



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


def _cube_coord_label_color(block: Block) -> str:
    piece = block.top_piece
    if piece is None:
        return "#2c2c2c"
    if piece.color == Color.BLACK:
        return "#e8e8e8"
    return "#1e1e1e"


def _cube_axis_label_offsets(hex_radius: float) -> tuple[
    tuple[float, float],
    tuple[float, float],
    tuple[float, float],
]:
    """Offsets from hex center toward top, bottom-left, bottom-right vertices (inside cell).

    Vertex 0 sits at ``_HEX_ORIENTATION`` (pointy-top ``RegularPolygon``).
    """
    inset = hex_radius * 0.58
    t0 = _HEX_ORIENTATION
    out: list[tuple[float, float]] = []
    for k in (0, 2, 4):
        ang = t0 + k * math.pi / 3
        out.append((inset * math.cos(ang), inset * math.sin(ang)))
    return out[0], out[1], out[2]


def visualize_hive(hive: Hive, *, title: str = "Hive") -> None:
    """Draw the hive in the hex plane: empty cells vs pieces by color."""
    empty_positions: list[Position] = []
    white_positions: list[Position] = []
    black_positions: list[Position] = []

    for pos, block in hive.blocks.items():
        if block.top_piece is None:
            empty_positions.append(pos)
        elif block.color == Color.WHITE:
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

    dq, ds, dr = _cube_axis_label_offsets(hex_radius)
    for pos, block in hive.blocks.items():
        px, py = position_to_plane_xy(pos)
        color = _cube_coord_label_color(block)
        ax.text(
            px + dq[0],
            py + dq[1],
            str(pos.q),
            ha="center",
            va="center",
            fontsize=5,
            color=color,
            zorder=4,
        )
        ax.text(
            px + dr[0],
            py + dr[1],
            str(pos.r),
            ha="center",
            va="center",
            fontsize=5,
            color=color,
            zorder=4,
        )
        ax.text(
            px + ds[0],
            py + ds[1],
            str(pos.s),
            ha="center",
            va="center",
            fontsize=5,
            color=color,
            zorder=4,
        )

    center_radius = hex_radius * 0.42
    for pos in white_positions + black_positions:
        piece = hive.blocks[pos].top_piece
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