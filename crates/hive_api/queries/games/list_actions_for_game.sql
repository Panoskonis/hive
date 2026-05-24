SELECT
    id,
    move_number,
    action_type::text AS action_type,
    (starting_position).q::smallint AS start_q,
    (starting_position).s::smallint AS start_s,
    (starting_position).r::smallint AS start_r,
    (ending_position).q::smallint AS end_q,
    (ending_position).s::smallint AS end_s,
    (ending_position).r::smallint AS end_r,
    piece_type::text AS piece_type,
    turn::text AS turn
FROM actions
WHERE game_id = $1
ORDER BY id ASC
