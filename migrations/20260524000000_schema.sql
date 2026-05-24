CREATE TYPE user_role_type AS ENUM ('user', 'admin');
CREATE TYPE game_status_type AS ENUM ('draw', 'white_win', 'black_win', 'in_progress', 'cancelled');
CREATE TYPE action_type AS ENUM ('place_piece', 'move_piece', 'pillbug_special_move', 'cannot_move');
CREATE TYPE piece_type AS ENUM ('queen', 'ant', 'beetle', 'grasshopper', 'spider', 'mosquito', 'ladybug', 'pillbug');
CREATE TYPE color_type AS ENUM ('white', 'black');
CREATE TYPE hive_position_type AS (
    q SMALLINT,
    s SMALLINT,
    r SMALLINT
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ DEFAULT NULL,
    last_login_at TIMESTAMPTZ DEFAULT NULL,
    user_role user_role_type NOT NULL DEFAULT 'user'
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked_at TIMESTAMPTZ,
    user_agent TEXT,
    ip_address INET
);

CREATE UNIQUE INDEX sessions_token_hash_active_idx ON sessions (token_hash)
    WHERE revoked_at IS NULL;
CREATE INDEX sessions_user_id_idx ON sessions (user_id);
CREATE INDEX sessions_expires_at_idx ON sessions (expires_at)
    WHERE revoked_at IS NULL;

CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    white_user_id INT NOT NULL REFERENCES users(id),
    black_user_id INT NOT NULL REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMPTZ DEFAULT NULL,
    current_status game_status_type NOT NULL DEFAULT 'in_progress',
    mosquito_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ladybug_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    pillbug_enabled BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE actions (
    id SERIAL PRIMARY KEY,
    game_id INT NOT NULL REFERENCES games(id),
    move_number INT NOT NULL,
    action_type action_type NOT NULL,
    starting_position hive_position_type,
    ending_position hive_position_type,
    piece_type piece_type,
    turn color_type NOT NULL,

CONSTRAINT starting_position_cube CHECK (
    starting_position IS NULL
    OR (starting_position).q + (starting_position).s + (starting_position).r = 0
),
CONSTRAINT ending_position_cube CHECK (
    ending_position IS NULL
    OR (ending_position).q + (ending_position).s + (ending_position).r = 0
),
UNIQUE (game_id, move_number, turn)

);


CREATE INDEX actions_game_id_idx ON actions (game_id);
CREATE INDEX games_white_user_id_idx ON games (white_user_id);
CREATE INDEX games_black_user_id_idx ON games (black_user_id);
CREATE INDEX games_in_progress_idx ON games (current_status) WHERE current_status = 'in_progress';