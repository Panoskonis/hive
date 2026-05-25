CREATE TYPE user_role_type AS ENUM ('user', 'admin');
CREATE TYPE game_status_type AS ENUM ('draw', 'white_win', 'black_win', 'waiting_for_opponent', 'in_progress', 'cancelled');
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
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
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
    creator_user_id INT NOT NULL REFERENCES users(id),
    white_user_id INT REFERENCES users(id),
    black_user_id INT REFERENCES users(id),
    invite_code TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMPTZ DEFAULT NULL,
    ended_at TIMESTAMPTZ DEFAULT NULL,
    current_status game_status_type NOT NULL DEFAULT 'waiting_for_opponent',
    mosquito_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ladybug_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    pillbug_enabled BOOLEAN NOT NULL DEFAULT FALSE,

CONSTRAINT games_creator_is_player CHECK (
    creator_user_id = white_user_id
    OR creator_user_id = black_user_id
),
CONSTRAINT games_waiting_has_invite CHECK (
    current_status != 'waiting_for_opponent'
    OR invite_code IS NOT NULL
),
CONSTRAINT games_started_has_both_players CHECK (
    current_status IN ('waiting_for_opponent', 'cancelled')
    OR (white_user_id IS NOT NULL AND black_user_id IS NOT NULL)
)
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
CREATE UNIQUE INDEX games_invite_code_idx ON games (invite_code)
    WHERE invite_code IS NOT NULL;


CREATE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at_column
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sessions_updated_at_column
BEFORE UPDATE ON sessions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
