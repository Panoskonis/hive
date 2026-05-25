INSERT INTO games (
    creator_user_id,
    white_user_id,
    black_user_id,
    invite_code,
    current_status,
    started_at,
    mosquito_enabled,
    ladybug_enabled,
    pillbug_enabled
)
VALUES ($1, $1, $1, NULL, 'in_progress', CURRENT_TIMESTAMP, $2, $3, $4)
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
