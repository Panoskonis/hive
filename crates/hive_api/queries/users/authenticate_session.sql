UPDATE sessions
SET last_seen_at = CURRENT_TIMESTAMP
FROM users
WHERE sessions.token_hash = $1
    AND sessions.revoked_at IS NULL
    AND sessions.expires_at > CURRENT_TIMESTAMP
    AND users.id = sessions.user_id
    AND users.deleted_at IS NULL
RETURNING
    users.id,
    users.username,
    users.email,
    users.user_role::text AS role,
    sessions.id AS session_id
