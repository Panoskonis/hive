INSERT INTO actions (
    game_id,
    move_number,
    action_type,
    starting_position,
    ending_position,
    piece_type,
    turn
)
VALUES (
    $1,
    $2,
    $3::action_type,
    CASE
        WHEN $4::smallint IS NULL THEN NULL
        ELSE ROW($4, $5, $6)::hive_position_type
    END,
    CASE
        WHEN $7::smallint IS NULL THEN NULL
        ELSE ROW($7, $8, $9)::hive_position_type
    END,
    $10::piece_type,
    $11::color_type
)
