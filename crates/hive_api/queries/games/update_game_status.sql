UPDATE games
SET
    current_status = $2::game_status_type,
    ended_at = CASE
        WHEN $2::game_status_type = 'in_progress' THEN ended_at
        ELSE COALESCE(ended_at, CURRENT_TIMESTAMP)
    END
WHERE id = $1
RETURNING
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
