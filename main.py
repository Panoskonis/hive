from hive.engine import Hive, Position, Color, Queen, Grasshopper, SoldierAnt, Spider, Beetle
from hive.visualize import visualize_hive
from enum import Enum

class PieceType(Enum):
    GRASSHOPPER = "grasshopper"
    QUEEN = "queen"
    BEETLE = "beetle"
    SOLDIER_ANT = "soldier_ant"
    SPIDER = "spider"


def get_piece_from_position(hive: Hive, position: tuple[int, int, int]) -> Piece:
    return hive.blocks[Position(q=position[0], s=position[1], r=position[2])].top_piece

def player_move_piece(hive: Hive, color: Color, start_position: tuple[int, int, int], end_position: tuple[int, int, int]) -> None:
    piece = get_piece_from_position(hive, start_position)
    if piece.color != color:
        raise ValueError(f"Piece at {start_position} is not owned by {color.value}.")
    
    hive.move_piece(piece, Position(q=end_position[0], s=end_position[1], r=end_position[2]))

def player_place_piece(hive: Hive, piece_type: PieceType,  color: Color, position: tuple[int, int, int]) -> None:
    piece = None
    if piece_type == PieceType.GRASSHOPPER.value:
        piece = Grasshopper(color=color)
    elif piece_type == PieceType.QUEEN.value:
        piece = Queen(color=color)
    elif piece_type == PieceType.BEETLE.value:
        piece = Beetle(color=color)
    elif piece_type == PieceType.SOLDIER_ANT.value:
        piece = SoldierAnt(color=color)
    elif piece_type == PieceType.SPIDER.value:
        piece = Spider(color=color)
    else:
        raise ValueError(f"Invalid piece type: {piece_type}")
    hive.place_piece(piece, Position(q=position[0], s=position[1], r=position[2]))

def parse_position(position: str) -> tuple[int, int, int]:
    return tuple(int(x) for x in position.split(","))

def play_game()->None:
    hive = Hive()
    color = Color.WHITE
    visualize_hive(hive)

    
    while hive.get_winner() is None:
        print("Enter your move:")
        move = input()
        try:
            if move == "move":
                start_position = input("Enter the start position:")
                end_position = input("Enter the end position:")
                player_move_piece(hive, color, parse_position(start_position), parse_position(end_position))
            elif move == "place":
                piece_type = input("Enter the piece type:")
                position = input("Enter the position:")
                player_place_piece(hive, piece_type, color, parse_position(position))
            else:
                print("Invalid move")
            visualize_hive(hive)
            color = Color.BLACK if color == Color.WHITE else Color.WHITE
        except Exception as e:
            print(e)
    print(Hive.get_winner())

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
    hive.place_piece(gh1, Position(q=-2, s=1, r=1))
    hive.place_piece(Grasshopper(color=Color.BLACK), Position(q=0, s=1, r=-1))
    hive.place_piece(q2, Position(q=-3, s=2, r=1))
    hive.place_piece(b1, Position(q=1, s=0, r=-1))
    hive.place_piece(sa1, Position(q=-2, s=0, r=2))
    hive.place_piece(sp1, Position(q=-1, s=2, r=-1))
    hive.move_piece(q2, Position(q=-3, s=1, r=2))
    hive.move_piece(b1, Position(q=0, s=0, r=0))
    # hive.move_piece(q2, Position(q=-3, s=0, r=3))
    # hive.move_piece(b1, Position(q=-1, s=1, r=0))
    visualize_hive(hive)
    print(len(sp1.get_legal_moves(hive)), sp1.get_legal_moves(hive))


if __name__ == "__main__":
    play_game()
