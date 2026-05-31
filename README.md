# Hive

This project is a web version of the board game Hive. The core of the app is
the Rust engine in `crates/hive_engine`, which models the board, pieces, turns,
move history, and game rules.

The rules implemented by the engine are based on this page: [Hive rules PDF](https://hivegame.com/download/rules.pdf).

The live game page is available at [my-hive.duckdns.org](https://my-hive.duckdns.org).

## Stack

- Rust workspace for the backend and game engine.
- `hive_engine` for the core game logic.
- `hive_api` for authentication, game creation, joining games, and move history.
- PostgreSQL with SQLx migrations and checked SQL queries.
- Svelte/SvelteKit frontend for the browser UI.
- Docker Compose and nginx configuration for deployment.

## How It Works

Hive is implemented as an invite-based multiplayer game. A player can create a
new game, share the invite link with another player, and play through the web
board once the opponent joins.

The app supports user registration and login, a dashboard for active games, game
history, turn-based play, and a visual board with the Hive piece set including
the expansion pieces used by the project.
