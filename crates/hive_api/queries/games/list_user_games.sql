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
WHERE creator_user_id = $1
    OR white_user_id = $1
    OR black_user_id = $1
ORDER BY
    CASE current_status::text
        WHEN 'in_progress' THEN 0
        WHEN 'waiting_for_opponent' THEN 1
        ELSE 2
    END,
    created_at DESC,
    id DESC
