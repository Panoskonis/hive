SELECT
    id,
    creator_user_id,
    white_user_id,
    black_user_id,
    invite_code,
    created_at,
    started_at,
    ended_at,
    current_status::text AS current_status,
    mosquito_enabled,
    ladybug_enabled,
    pillbug_enabled
FROM games
WHERE invite_code = $1
