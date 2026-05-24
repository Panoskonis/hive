SELECT id, username, email, password, user_role::text AS role
FROM users
WHERE email = $1 AND deleted_at IS NULL
