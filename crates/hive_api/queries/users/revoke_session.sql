UPDATE sessions
SET revoked_at = CURRENT_TIMESTAMP
WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
