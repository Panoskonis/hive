SELECT
    COUNT(*)::bigint AS action_count,
    MAX(id) AS last_action_id
FROM actions
WHERE game_id = $1
