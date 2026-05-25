use axum::body::Body;
use axum::http::{Request, StatusCode};
use hive_api::router;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test(migrations = "../../migrations")]
async fn create_game_requires_auth(pool: PgPool) -> sqlx::Result<()> {
    let (status, body) = post_json(
        pool,
        "/games",
        json!({
            "creator_color": "white",
            "mosquito_enabled": false,
            "ladybug_enabled": false,
            "pillbug_enabled": false
        }),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "authentication required");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_waiting_game_as_white(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;

    let (status, game) = post_json(
        pool,
        "/games",
        json!({
            "creator_color": "white",
            "mosquito_enabled": true,
            "ladybug_enabled": false,
            "pillbug_enabled": true
        }),
        Some(alice.token()),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(game["creator_user_id"], alice.id());
    assert_eq!(game["white_user_id"], alice.id());
    assert!(game["black_user_id"].is_null());
    assert!(game["invite_code"].is_string());
    assert_eq!(game["current_status"], "waiting_for_opponent");
    assert_eq!(game["mosquito_enabled"], true);
    assert_eq!(game["ladybug_enabled"], false);
    assert_eq!(game["pillbug_enabled"], true);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_waiting_game_as_black(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;

    let (status, game) = post_json(
        pool,
        "/games",
        json!({
            "creator_color": "black"
        }),
        Some(alice.token()),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(game["white_user_id"].is_null());
    assert_eq!(game["black_user_id"], alice.id());
    assert_eq!(game["current_status"], "waiting_for_opponent");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_solo_game_assigns_both_colors_and_starts(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;

    let (status, game) = post_json(
        pool,
        "/games",
        json!({
            "creator_color": "white",
            "self_play": true,
            "mosquito_enabled": true,
            "ladybug_enabled": false,
            "pillbug_enabled": true
        }),
        Some(alice.token()),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(game["creator_user_id"], alice.id());
    assert_eq!(game["white_user_id"], alice.id());
    assert_eq!(game["black_user_id"], alice.id());
    assert!(game["invite_code"].is_null());
    assert_eq!(game["current_status"], "in_progress");
    assert!(game["started_at"].is_string());
    assert_eq!(game["mosquito_enabled"], true);
    assert_eq!(game["ladybug_enabled"], false);
    assert_eq!(game["pillbug_enabled"], true);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn solo_game_creator_can_play_both_turns(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let (status, game) = post_json(
        pool.clone(),
        "/games",
        json!({
            "creator_color": "white",
            "self_play": true
        }),
        Some(alice.token()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let game_id = game["id"].as_i64().unwrap();

    let (white_status, white_state) = post_json(
        pool.clone(),
        &format!("/games/{game_id}/actions"),
        json!({ "type": "place", "piece_type": "queen", "to": { "q": 0, "s": 0, "r": 0 } }),
        Some(alice.token()),
    )
    .await;
    let (black_status, black_state) = post_json(
        pool,
        &format!("/games/{game_id}/actions"),
        json!({ "type": "place", "piece_type": "queen", "to": { "q": 1, "s": -1, "r": 0 } }),
        Some(alice.token()),
    )
    .await;

    assert_eq!(white_status, StatusCode::OK);
    assert_eq!(white_state["current_turn"], "black");
    assert_eq!(black_status, StatusCode::OK);
    assert_eq!(black_state["current_turn"], "white");
    assert_eq!(black_state["move_number"], 2);
    assert_eq!(black_state["board"].as_array().unwrap().len(), 2);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn invite_preview_works_for_authenticated_user(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let invite_code = game["invite_code"].as_str().unwrap();

    let (status, preview) = get_json(
        pool,
        &format!("/games/invites/{invite_code}"),
        Some(bob.token()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(preview["id"], game["id"]);
    assert_eq!(preview["invite_code"], invite_code);
    assert_eq!(preview["current_status"], "waiting_for_opponent");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_invite_fills_opposite_color_and_starts_game(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let game = create_game(pool.clone(), alice.token(), "black").await;
    let invite_code = game["invite_code"].as_str().unwrap();

    let (status, joined) = post_json(
        pool,
        "/games/join",
        json!({ "invite_code": invite_code }),
        Some(bob.token()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(joined["white_user_id"], bob.id());
    assert_eq!(joined["black_user_id"], alice.id());
    assert!(joined["invite_code"].is_null());
    assert!(joined["started_at"].is_string());
    assert_eq!(joined["current_status"], "in_progress");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn creator_cannot_join_own_invite(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let invite_code = game["invite_code"].as_str().unwrap();

    let (status, body) = post_json(
        pool,
        "/games/join",
        json!({ "invite_code": invite_code }),
        Some(alice.token()),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "cannot join your own game");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn second_join_attempt_returns_409_after_invite_is_consumed(
    pool: PgPool,
) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let carol = register(pool.clone(), "carol").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let invite_code = game["invite_code"].as_str().unwrap();

    let (first_status, _) = post_json(
        pool.clone(),
        "/games/join",
        json!({ "invite_code": invite_code }),
        Some(bob.token()),
    )
    .await;
    assert_eq!(first_status, StatusCode::OK);

    let (second_status, body) = post_json(
        pool,
        "/games/join",
        json!({ "invite_code": invite_code }),
        Some(carol.token()),
    )
    .await;

    assert_eq!(second_status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "game already started");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_creator_cannot_fetch_waiting_game_by_id(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let game_id = game["id"].as_i64().unwrap();

    let (status, body) = get_json(pool, &format!("/games/{game_id}"), Some(bob.token())).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "access forbidden");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn participants_can_fetch_started_game_by_id(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let game_id = game["id"].as_i64().unwrap();
    let invite_code = game["invite_code"].as_str().unwrap();

    post_json(
        pool.clone(),
        "/games/join",
        json!({ "invite_code": invite_code }),
        Some(bob.token()),
    )
    .await;

    let (alice_status, alice_game) = get_json(
        pool.clone(),
        &format!("/games/{game_id}"),
        Some(alice.token()),
    )
    .await;
    let (bob_status, bob_game) =
        get_json(pool, &format!("/games/{game_id}"), Some(bob.token())).await;

    assert_eq!(alice_status, StatusCode::OK);
    assert_eq!(bob_status, StatusCode::OK);
    assert_eq!(alice_game["current_status"], "in_progress");
    assert_eq!(bob_game["current_status"], "in_progress");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn user_can_list_waiting_and_active_games(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let waiting = create_game(pool.clone(), alice.token(), "white").await;
    let active = create_game(pool.clone(), alice.token(), "black").await;

    post_json(
        pool.clone(),
        "/games/join",
        json!({ "invite_code": active["invite_code"].as_str().unwrap() }),
        Some(bob.token()),
    )
    .await;

    let (status, games) = get_json(pool, "/games", Some(alice.token())).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(games.as_array().unwrap().len(), 2);
    assert_eq!(games[0]["id"], active["id"]);
    assert_eq!(games[0]["current_status"], "in_progress");
    assert_eq!(games[1]["id"], waiting["id"]);
    assert_eq!(games[1]["current_status"], "waiting_for_opponent");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_participant_cannot_fetch_state_or_submit_actions(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let carol = register(pool.clone(), "carol").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let game_id = game["id"].as_i64().unwrap();

    post_json(
        pool.clone(),
        "/games/join",
        json!({ "invite_code": game["invite_code"].as_str().unwrap() }),
        Some(bob.token()),
    )
    .await;

    let (state_status, state_body) = get_json(
        pool.clone(),
        &format!("/games/{game_id}/state"),
        Some(carol.token()),
    )
    .await;
    let (action_status, action_body) = post_json(
        pool,
        &format!("/games/{game_id}/actions"),
        json!({ "type": "place", "piece_type": "queen", "to": { "q": 0, "s": 0, "r": 0 } }),
        Some(carol.token()),
    )
    .await;

    assert_eq!(state_status, StatusCode::FORBIDDEN);
    assert_eq!(state_body["error"], "access forbidden");
    assert_eq!(action_status, StatusCode::FORBIDDEN);
    assert_eq!(action_body["error"], "access forbidden");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn waiting_game_state_is_readable_but_rejects_actions(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let game_id = game["id"].as_i64().unwrap();

    let (state_status, state) = get_json(
        pool.clone(),
        &format!("/games/{game_id}/state"),
        Some(alice.token()),
    )
    .await;
    let (action_status, body) = post_json(
        pool,
        &format!("/games/{game_id}/actions"),
        json!({ "type": "place", "piece_type": "queen", "to": { "q": 0, "s": 0, "r": 0 } }),
        Some(alice.token()),
    )
    .await;

    assert_eq!(state_status, StatusCode::OK);
    assert_eq!(state["current_status"], "waiting_for_opponent");
    assert!(state["legal_actions"].as_array().unwrap().is_empty());
    assert_eq!(action_status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "game not started");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn white_places_first_piece_and_black_cannot_move_out_of_turn(
    pool: PgPool,
) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let game_id = game["id"].as_i64().unwrap();

    post_json(
        pool.clone(),
        "/games/join",
        json!({ "invite_code": game["invite_code"].as_str().unwrap() }),
        Some(bob.token()),
    )
    .await;

    let (black_status, black_body) = post_json(
        pool.clone(),
        &format!("/games/{game_id}/actions"),
        json!({ "type": "place", "piece_type": "queen", "to": { "q": 0, "s": 0, "r": 0 } }),
        Some(bob.token()),
    )
    .await;
    let (white_status, state) = post_json(
        pool.clone(),
        &format!("/games/{game_id}/actions"),
        json!({ "type": "place", "piece_type": "queen", "to": { "q": 0, "s": 0, "r": 0 } }),
        Some(alice.token()),
    )
    .await;
    let (fresh_status, fresh_state) = get_json(
        pool,
        &format!("/games/{game_id}/state"),
        Some(alice.token()),
    )
    .await;

    assert_eq!(black_status, StatusCode::CONFLICT);
    assert_eq!(black_body["error"], "wrong turn");
    assert_eq!(white_status, StatusCode::OK);
    assert_eq!(state["current_turn"], "black");
    assert!(state.get("actions").is_none());
    assert_eq!(state["board"].as_array().unwrap().len(), 1);
    assert_eq!(state["board"][0]["pieces"][0]["piece_type"], "queen");
    assert_eq!(fresh_status, StatusCode::OK);
    assert_eq!(fresh_state["board"], state["board"]);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn game_actions_returns_persisted_history(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let game_id = game["id"].as_i64().unwrap();

    post_json(
        pool.clone(),
        "/games/join",
        json!({ "invite_code": game["invite_code"].as_str().unwrap() }),
        Some(bob.token()),
    )
    .await;
    post_json(
        pool.clone(),
        &format!("/games/{game_id}/actions"),
        json!({ "type": "place", "piece_type": "queen", "to": { "q": 0, "s": 0, "r": 0 } }),
        Some(alice.token()),
    )
    .await;

    let (status, history) = get_json(
        pool,
        &format!("/games/{game_id}/actions"),
        Some(alice.token()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(history.as_array().unwrap().len(), 1);
    assert_eq!(history[0]["type"], "place");
    assert_eq!(history[0]["move_number"], 1);
    assert_eq!(history[0]["turn"], "white");
    assert_eq!(history[0]["piece_type"], "queen");
    assert!(history[0]["id"].as_i64().is_some());
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn invalid_action_returns_rule_error(pool: PgPool) -> sqlx::Result<()> {
    let alice = register(pool.clone(), "alice").await;
    let bob = register(pool.clone(), "bob").await;
    let game = create_game(pool.clone(), alice.token(), "white").await;
    let game_id = game["id"].as_i64().unwrap();

    post_json(
        pool.clone(),
        "/games/join",
        json!({ "invite_code": game["invite_code"].as_str().unwrap() }),
        Some(bob.token()),
    )
    .await;
    post_json(
        pool.clone(),
        &format!("/games/{game_id}/actions"),
        json!({ "type": "place", "piece_type": "queen", "to": { "q": 0, "s": 0, "r": 0 } }),
        Some(alice.token()),
    )
    .await;

    let (status, body) = post_json(
        pool,
        &format!("/games/{game_id}/actions"),
        json!({ "type": "place", "piece_type": "queen", "to": { "q": 3, "s": -3, "r": 0 } }),
        Some(bob.token()),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Invalid placement position");
    Ok(())
}

#[derive(Debug)]
struct RegisteredUser {
    body: Value,
}

impl RegisteredUser {
    fn token(&self) -> &str {
        self.body["token"].as_str().unwrap()
    }

    fn id(&self) -> Value {
        self.body["user"]["id"].clone()
    }
}

async fn register(pool: PgPool, username: &str) -> RegisteredUser {
    let (status, body) = post_json(
        pool,
        "/auth/register",
        json!({
            "username": username,
            "email": format!("{username}@example.com"),
            "password": "secret-password"
        }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    RegisteredUser { body }
}

async fn create_game(pool: PgPool, token: &str, creator_color: &str) -> Value {
    let (status, game) = post_json(
        pool,
        "/games",
        json!({
            "creator_color": creator_color,
            "mosquito_enabled": false,
            "ladybug_enabled": false,
            "pillbug_enabled": false
        }),
        Some(token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    game
}

async fn post_json(
    pool: PgPool,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    request_json(pool, "POST", uri, Some(body), bearer).await
}

async fn get_json(pool: PgPool, uri: &str, bearer: Option<&str>) -> (StatusCode, Value) {
    request_json(pool, "GET", uri, None, bearer).await
}

async fn request_json(
    pool: PgPool,
    method: &str,
    uri: &str,
    body: Option<Value>,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let app = router(pool);

    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }

    let request_body = match body {
        Some(value) => Body::from(value.to_string()),
        None => Body::empty(),
    };

    let response = app
        .oneshot(builder.body(request_body).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();

    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            panic!(
                "response is not JSON (status {status}): {error}; body={}",
                String::from_utf8_lossy(&bytes)
            );
        })
    };

    (status, json)
}
