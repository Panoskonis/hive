from hive.engine import Hive, Position, Color, Queen, Grasshopper, SoldierAnt, Spider, Beetle
from hive.visualize import visualize_hive



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
    main()
