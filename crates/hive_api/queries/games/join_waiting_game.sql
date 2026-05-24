WITH locked_game AS (
    SELECT *
    FROM games
    WHERE invite_code = $1
    FOR UPDATE
),
updated_game AS (
    UPDATE games
    SET
        white_user_id = CASE
            WHEN locked_game.white_user_id IS NULL THEN $2
            ELSE locked_game.white_user_id
        END,
        black_user_id = CASE
            WHEN locked_game.black_user_id IS NULL THEN $2
            ELSE locked_game.black_user_id
        END,
        current_status = 'in_progress',
        started_at = CURRENT_TIMESTAMP
    FROM locked_game
    WHERE games.id = locked_game.id
        AND locked_game.current_status = 'waiting_for_opponent'
        AND locked_game.creator_user_id <> $2
        AND (
            locked_game.white_user_id IS NULL
            OR locked_game.black_user_id IS NULL
        )
    RETURNING games.*
)
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
FROM updated_game
