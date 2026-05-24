use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use hive_engine::{
    Action, ActionType, Color, Game, GameStatus, HiveError, Inventory, LegalAction, PieceType,
    Position,
};
use rand::{Rng, distributions::Alphanumeric, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool, Postgres, Transaction, postgres::PgDatabaseError};

use crate::{auth::AuthUser, error::ApiError, queries::games, state::AppState};

const INVITE_CODE_LEN: usize = 10;
const INVITE_CODE_RETRIES: usize = 5;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_games).post(create_game))
        .route("/{id}", get(get_game))
        .route("/{id}/state", get(get_game_state))
        .route("/{id}/actions", post(submit_action))
        .route("/invites/{invite_code}", get(preview_invite))
        .route("/join", post(join_game))
}

#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    pub creator_color: PlayerColor,
    #[serde(default)]
    pub mosquito_enabled: bool,
    #[serde(default)]
    pub ladybug_enabled: bool,
    #[serde(default)]
    pub pillbug_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct JoinGameRequest {
    pub invite_code: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlayerColor {
    White,
    Black,
}

#[derive(Debug, Serialize)]
pub struct GameResponse {
    pub id: i32,
    pub creator_user_id: i32,
    pub white_user_id: Option<i32>,
    pub black_user_id: Option<i32>,
    pub invite_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub current_status: String,
    pub mosquito_enabled: bool,
    pub ladybug_enabled: bool,
    pub pillbug_enabled: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct PositionResponse {
    pub q: i8,
    pub s: i8,
    pub r: i8,
}

#[derive(Debug, Serialize)]
pub struct PieceResponse {
    pub color: PlayerColor,
    pub piece_type: PieceTypeResponse,
}

#[derive(Debug, Serialize)]
pub struct BoardCellResponse {
    pub q: i8,
    pub s: i8,
    pub r: i8,
    pub pieces: Vec<PieceResponse>,
}

#[derive(Debug, Serialize)]
pub struct InventoryResponse {
    pub queen: u8,
    pub ant: u8,
    pub beetle: u8,
    pub grasshopper: u8,
    pub spider: u8,
    pub mosquito: u8,
    pub ladybug: u8,
    pub pillbug: u8,
}

#[derive(Debug, Serialize)]
pub struct InventoriesResponse {
    pub white: InventoryResponse,
    pub black: InventoryResponse,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PieceTypeResponse {
    Queen,
    Ant,
    Beetle,
    Grasshopper,
    Spider,
    Mosquito,
    Ladybug,
    Pillbug,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionRequest {
    Place {
        piece_type: PieceTypeResponse,
        to: PositionResponse,
    },
    Move {
        from: PositionResponse,
        to: PositionResponse,
    },
    PillbugSpecial {
        from: PositionResponse,
        to: PositionResponse,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionResponse {
    Place {
        id: Option<i32>,
        move_number: u16,
        turn: PlayerColor,
        piece_type: PieceTypeResponse,
        to: PositionResponse,
    },
    Move {
        id: Option<i32>,
        move_number: u16,
        turn: PlayerColor,
        from: PositionResponse,
        to: PositionResponse,
    },
    PillbugSpecial {
        id: Option<i32>,
        move_number: u16,
        turn: PlayerColor,
        from: PositionResponse,
        to: PositionResponse,
    },
    CannotMove {
        id: Option<i32>,
        move_number: u16,
        turn: PlayerColor,
    },
}

#[derive(Debug, Serialize)]
pub struct GameStateResponse {
    #[serde(flatten)]
    pub game: GameResponse,
    pub viewer_color: Option<PlayerColor>,
    pub current_turn: PlayerColor,
    pub move_number: u16,
    pub board: Vec<BoardCellResponse>,
    pub inventories: InventoriesResponse,
    pub actions: Vec<ActionResponse>,
    pub legal_actions: Vec<ActionResponse>,
}

#[derive(Debug, sqlx::FromRow)]
struct GameRow {
    id: i32,
    creator_user_id: i32,
    white_user_id: Option<i32>,
    black_user_id: Option<i32>,
    invite_code: Option<String>,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    ended_at: Option<DateTime<Utc>>,
    current_status: String,
    mosquito_enabled: bool,
    ladybug_enabled: bool,
    pillbug_enabled: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct ActionRow {
    id: i32,
    move_number: i32,
    action_type: String,
    start_q: Option<i16>,
    start_s: Option<i16>,
    start_r: Option<i16>,
    end_q: Option<i16>,
    end_s: Option<i16>,
    end_r: Option<i16>,
    piece_type: Option<String>,
    turn: String,
}

async fn list_games(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<GameResponse>>, ApiError> {
    let games = sqlx::query_as::<_, GameRow>(games::LIST_USER_GAMES)
        .bind(user.id)
        .fetch_all(&state.pool)
        .await?
        .into_iter()
        .map(GameRow::into_response)
        .collect();

    Ok(Json(games))
}

async fn create_game(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateGameRequest>,
) -> Result<(StatusCode, Json<GameResponse>), ApiError> {
    let (white_user_id, black_user_id) = match payload.creator_color {
        PlayerColor::White => (Some(user.id), None),
        PlayerColor::Black => (None, Some(user.id)),
    };

    for _ in 0..INVITE_CODE_RETRIES {
        let invite_code = generate_invite_code();
        let result = sqlx::query_as::<_, GameRow>(games::INSERT_WAITING_GAME)
            .bind(user.id)
            .bind(white_user_id)
            .bind(black_user_id)
            .bind(invite_code)
            .bind(payload.mosquito_enabled)
            .bind(payload.ladybug_enabled)
            .bind(payload.pillbug_enabled)
            .fetch_one(&state.pool)
            .await;

        match result {
            Ok(game) => return Ok((StatusCode::CREATED, Json(game.into_response()))),
            Err(error) if is_unique_violation(&error) => continue,
            Err(error) => return Err(error.into()),
        }
    }

    Err(ApiError::Database(sqlx::Error::Protocol(
        "failed to generate a unique invite code".to_owned(),
    )))
}

async fn preview_invite(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(invite_code): Path<String>,
) -> Result<Json<GameResponse>, ApiError> {
    let game = find_waiting_game_by_invite(&state, &invite_code).await?;
    Ok(Json(game.into_response()))
}

async fn join_game(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<JoinGameRequest>,
) -> Result<Json<GameResponse>, ApiError> {
    let invite_code = normalize_invite_code(&payload.invite_code)?;
    let game = find_game_by_invite(&state, &invite_code).await?;

    if game.creator_user_id == user.id {
        return Err(ApiError::CannotJoinOwnGame);
    }

    if game.current_status != "waiting_for_opponent" {
        return Err(ApiError::GameAlreadyStarted);
    }

    let game = sqlx::query_as::<_, GameRow>(games::JOIN_WAITING_GAME)
        .bind(invite_code)
        .bind(user.id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::GameAlreadyStarted)?;

    Ok(Json(game.into_response()))
}

async fn get_game(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<GameResponse>, ApiError> {
    let game = sqlx::query_as::<_, GameRow>(games::FIND_GAME_BY_ID)
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::GameNotFound)?;

    if !game.can_view(user.id) {
        return Err(ApiError::Forbidden);
    }

    Ok(Json(game.into_response()))
}

async fn get_game_state(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<GameStateResponse>, ApiError> {
    let game = sqlx::query_as::<_, GameRow>(games::FIND_GAME_BY_ID)
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::GameNotFound)?;

    if !game.can_view(user.id) {
        return Err(ApiError::Forbidden);
    }

    let actions = fetch_actions(&state.pool, game.id).await?;
    let state = build_game_state(game, user.id, actions)?;
    Ok(Json(state))
}

async fn submit_action(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<ActionRequest>,
) -> Result<Json<GameStateResponse>, ApiError> {
    let mut tx = state.pool.begin().await?;
    let mut game_row = sqlx::query_as::<_, GameRow>(games::FIND_GAME_BY_ID_FOR_UPDATE)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError::GameNotFound)?;

    if !game_row.can_view(user.id) {
        return Err(ApiError::Forbidden);
    }

    match game_row.current_status.as_str() {
        "waiting_for_opponent" => return Err(ApiError::GameNotStarted),
        "in_progress" => {}
        _ => return Err(ApiError::GameAlreadyFinished),
    }

    let actions = fetch_actions_conn(&mut *tx, game_row.id).await?;
    let mut game = rebuild_engine_game(&game_row, &actions)?;
    let viewer_color = game_row.viewer_color(user.id).ok_or(ApiError::Forbidden)?;
    if viewer_color != PlayerColor::from(game.turn()) {
        return Err(ApiError::WrongTurn);
    }

    let mut persisted_move_number = game.move_num;
    let engine_action = payload.into_engine_action(game.turn())?;
    let previous_action_count = game.history.actions.len();
    game.apply_action(engine_action).map_err(map_hive_error)?;
    let new_actions: Vec<Action> = game.history.actions[previous_action_count..].to_vec();

    for action in &new_actions {
        insert_action(&mut tx, game_row.id, persisted_move_number, *action).await?;
        if action.turn == Color::Black {
            persisted_move_number = persisted_move_number.saturating_add(1);
        }
    }

    let status = game.get_status().map_err(map_hive_error)?;
    let status = game_status_string(status);
    if status != game_row.current_status {
        game_row = sqlx::query_as::<_, GameRow>(games::UPDATE_GAME_STATUS)
            .bind(game_row.id)
            .bind(status)
            .fetch_one(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    let actions = fetch_actions(&state.pool, game_row.id).await?;
    let state = build_game_state(game_row, user.id, actions)?;
    Ok(Json(state))
}

async fn find_waiting_game_by_invite(
    state: &AppState,
    invite_code: &str,
) -> Result<GameRow, ApiError> {
    let invite_code = normalize_invite_code(invite_code)?;

    sqlx::query_as::<_, GameRow>(games::FIND_WAITING_GAME_BY_INVITE)
        .bind(invite_code)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::InviteNotFound)
}

async fn find_game_by_invite(state: &AppState, invite_code: &str) -> Result<GameRow, ApiError> {
    let invite_code = normalize_invite_code(invite_code)?;

    sqlx::query_as::<_, GameRow>(games::FIND_GAME_BY_INVITE)
        .bind(invite_code)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::InviteNotFound)
}

fn normalize_invite_code(invite_code: &str) -> Result<String, ApiError> {
    let invite_code = invite_code.trim().to_uppercase();
    if invite_code.is_empty() {
        return Err(ApiError::InvalidGameRequest);
    }
    Ok(invite_code)
}

fn generate_invite_code() -> String {
    OsRng
        .sample_iter(&Alphanumeric)
        .take(INVITE_CODE_LEN)
        .map(char::from)
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(database_error) = error {
        return database_error
            .try_downcast_ref::<PgDatabaseError>()
            .is_some_and(|postgres_error| postgres_error.code() == "23505");
    }
    false
}

async fn fetch_actions(pool: &PgPool, game_id: i32) -> Result<Vec<ActionRow>, ApiError> {
    sqlx::query_as::<_, ActionRow>(games::LIST_ACTIONS_FOR_GAME)
        .bind(game_id)
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)
}

async fn fetch_actions_conn(
    conn: &mut PgConnection,
    game_id: i32,
) -> Result<Vec<ActionRow>, ApiError> {
    sqlx::query_as::<_, ActionRow>(games::LIST_ACTIONS_FOR_GAME)
        .bind(game_id)
        .fetch_all(conn)
        .await
        .map_err(ApiError::from)
}

async fn insert_action(
    tx: &mut Transaction<'_, Postgres>,
    game_id: i32,
    move_number: u16,
    action: Action,
) -> Result<(), ApiError> {
    let start = action.start_position.map(PositionResponse::from);
    let end = action.end_position.map(PositionResponse::from);

    sqlx::query(games::INSERT_ACTION)
        .bind(game_id)
        .bind(i32::from(move_number))
        .bind(action_type_string(action.action_type))
        .bind(start.as_ref().map(|pos| i16::from(pos.q)))
        .bind(start.as_ref().map(|pos| i16::from(pos.s)))
        .bind(start.as_ref().map(|pos| i16::from(pos.r)))
        .bind(end.as_ref().map(|pos| i16::from(pos.q)))
        .bind(end.as_ref().map(|pos| i16::from(pos.s)))
        .bind(end.as_ref().map(|pos| i16::from(pos.r)))
        .bind(action.piece_type.map(piece_type_string))
        .bind(color_string(action.turn))
        .execute(&mut **tx)
        .await?;

    Ok(())
}

fn build_game_state(
    game_row: GameRow,
    viewer_user_id: i32,
    persisted_actions: Vec<ActionRow>,
) -> Result<GameStateResponse, ApiError> {
    let mut game = rebuild_engine_game(&game_row, &persisted_actions)?;
    let legal_actions = if game_row.current_status == "in_progress" {
        game.legal_actions()
            .map_err(map_hive_error)?
            .into_iter()
            .map(|action| ActionResponse::from_legal(action, game.move_num, game.turn()))
            .collect()
    } else {
        Vec::new()
    };

    let mut board: Vec<BoardCellResponse> = game
        .board
        .pieces
        .iter()
        .map(|(position, pieces)| BoardCellResponse {
            q: position.q,
            s: position.s,
            r: position.r,
            pieces: pieces.iter().map(PieceResponse::from).collect(),
        })
        .collect();
    board.sort_by_key(|cell| (cell.q, cell.s, cell.r));

    let actions = persisted_actions
        .iter()
        .map(ActionResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let viewer_color = game_row.viewer_color(viewer_user_id);
    let current_turn = PlayerColor::from(game.turn());
    let move_number = game.move_num;
    let response = game_row.into_response();

    Ok(GameStateResponse {
        game: response,
        viewer_color,
        current_turn,
        move_number,
        board,
        inventories: InventoriesResponse {
            white: InventoryResponse::from(&game.white_inventory),
            black: InventoryResponse::from(&game.black_inventory),
        },
        actions,
        legal_actions,
    })
}

fn rebuild_engine_game(game: &GameRow, actions: &[ActionRow]) -> Result<Game, ApiError> {
    let mut engine_game = Game::new(
        game.mosquito_enabled,
        game.ladybug_enabled,
        game.pillbug_enabled,
    );

    for row in actions {
        if row.action_type == "cannot_move" {
            continue;
        }
        engine_game
            .apply_action(row.to_engine_action()?)
            .map_err(map_hive_error)?;
    }

    Ok(engine_game)
}

fn map_hive_error(error: HiveError) -> ApiError {
    match error {
        HiveError::InvalidMoveType
        | HiveError::InvalidPieceType
        | HiveError::InvalidPositionConstraint
        | HiveError::InvalidPositionFormat
        | HiveError::InvalidCoordinate(_) => ApiError::InvalidAction,
        HiveError::WrongTurn => ApiError::WrongTurn,
        other => ApiError::RuleViolation(other.to_string()),
    }
}

fn action_type_string(action_type: ActionType) -> &'static str {
    match action_type {
        ActionType::PlacePiece => "place_piece",
        ActionType::MovePiece => "move_piece",
        ActionType::PillbugSpecialMove => "pillbug_special_move",
        ActionType::CannotMove => "cannot_move",
    }
}

fn piece_type_string(piece_type: PieceType) -> &'static str {
    match piece_type {
        PieceType::Queen => "queen",
        PieceType::Ant => "ant",
        PieceType::Beetle => "beetle",
        PieceType::Grasshopper => "grasshopper",
        PieceType::Spider => "spider",
        PieceType::Mosquito => "mosquito",
        PieceType::Ladybug => "ladybug",
        PieceType::Pillbug => "pillbug",
    }
}

fn color_string(color: Color) -> &'static str {
    match color {
        Color::White => "white",
        Color::Black => "black",
    }
}

fn game_status_string(status: GameStatus) -> &'static str {
    match status {
        GameStatus::InProgress => "in_progress",
        GameStatus::WhiteWon => "white_win",
        GameStatus::BlackWon => "black_win",
        GameStatus::Draw => "draw",
    }
}

impl GameRow {
    fn can_view(&self, user_id: i32) -> bool {
        self.creator_user_id == user_id
            || self.white_user_id == Some(user_id)
            || self.black_user_id == Some(user_id)
    }

    fn viewer_color(&self, user_id: i32) -> Option<PlayerColor> {
        if self.white_user_id == Some(user_id) {
            Some(PlayerColor::White)
        } else if self.black_user_id == Some(user_id) {
            Some(PlayerColor::Black)
        } else {
            None
        }
    }

    fn into_response(self) -> GameResponse {
        let invite_code = if self.current_status == "waiting_for_opponent" {
            self.invite_code
        } else {
            None
        };

        GameResponse {
            id: self.id,
            creator_user_id: self.creator_user_id,
            white_user_id: self.white_user_id,
            black_user_id: self.black_user_id,
            invite_code,
            created_at: self.created_at,
            started_at: self.started_at,
            ended_at: self.ended_at,
            current_status: self.current_status,
            mosquito_enabled: self.mosquito_enabled,
            ladybug_enabled: self.ladybug_enabled,
            pillbug_enabled: self.pillbug_enabled,
        }
    }
}

impl ActionRow {
    fn to_engine_action(&self) -> Result<Action, ApiError> {
        Ok(Action {
            action_type: match self.action_type.as_str() {
                "place_piece" => ActionType::PlacePiece,
                "move_piece" => ActionType::MovePiece,
                "pillbug_special_move" => ActionType::PillbugSpecialMove,
                "cannot_move" => ActionType::CannotMove,
                _ => return Err(ApiError::InvalidAction),
            },
            piece_type: self
                .piece_type
                .as_deref()
                .map(PieceType::try_from)
                .transpose()
                .map_err(map_hive_error)?,
            start_position: maybe_position(self.start_q, self.start_s, self.start_r)?,
            end_position: maybe_position(self.end_q, self.end_s, self.end_r)?,
            turn: match self.turn.as_str() {
                "white" => Color::White,
                "black" => Color::Black,
                _ => return Err(ApiError::InvalidAction),
            },
        })
    }
}

impl ActionRequest {
    fn into_engine_action(self, turn: Color) -> Result<Action, ApiError> {
        match self {
            Self::Place { piece_type, to } => Ok(Action {
                action_type: ActionType::PlacePiece,
                piece_type: Some(PieceType::from(piece_type)),
                start_position: None,
                end_position: Some(Position::try_from(to)?),
                turn,
            }),
            Self::Move { from, to } => Ok(Action {
                action_type: ActionType::MovePiece,
                piece_type: None,
                start_position: Some(Position::try_from(from)?),
                end_position: Some(Position::try_from(to)?),
                turn,
            }),
            Self::PillbugSpecial { from, to } => Ok(Action {
                action_type: ActionType::PillbugSpecialMove,
                piece_type: None,
                start_position: Some(Position::try_from(from)?),
                end_position: Some(Position::try_from(to)?),
                turn,
            }),
        }
    }
}

impl TryFrom<&ActionRow> for ActionResponse {
    type Error = ApiError;

    fn try_from(row: &ActionRow) -> Result<Self, Self::Error> {
        let move_number = u16::try_from(row.move_number).map_err(|_| ApiError::InvalidAction)?;
        let turn = PlayerColor::try_from(row.turn.as_str())?;
        match row.action_type.as_str() {
            "place_piece" => Ok(Self::Place {
                id: Some(row.id),
                move_number,
                turn,
                piece_type: row
                    .piece_type
                    .as_deref()
                    .ok_or(ApiError::InvalidAction)?
                    .try_into()?,
                to: maybe_position(row.end_q, row.end_s, row.end_r)?
                    .ok_or(ApiError::InvalidAction)?
                    .into(),
            }),
            "move_piece" => Ok(Self::Move {
                id: Some(row.id),
                move_number,
                turn,
                from: maybe_position(row.start_q, row.start_s, row.start_r)?
                    .ok_or(ApiError::InvalidAction)?
                    .into(),
                to: maybe_position(row.end_q, row.end_s, row.end_r)?
                    .ok_or(ApiError::InvalidAction)?
                    .into(),
            }),
            "pillbug_special_move" => Ok(Self::PillbugSpecial {
                id: Some(row.id),
                move_number,
                turn,
                from: maybe_position(row.start_q, row.start_s, row.start_r)?
                    .ok_or(ApiError::InvalidAction)?
                    .into(),
                to: maybe_position(row.end_q, row.end_s, row.end_r)?
                    .ok_or(ApiError::InvalidAction)?
                    .into(),
            }),
            "cannot_move" => Ok(Self::CannotMove {
                id: Some(row.id),
                move_number,
                turn,
            }),
            _ => Err(ApiError::InvalidAction),
        }
    }
}

impl ActionResponse {
    fn from_legal(action: LegalAction, move_number: u16, turn: Color) -> Self {
        let turn = PlayerColor::from(turn);
        match action {
            LegalAction::Place { piece, at } => Self::Place {
                id: None,
                move_number,
                turn,
                piece_type: PieceTypeResponse::from(piece),
                to: at.into(),
            },
            LegalAction::Move { from, to } => Self::Move {
                id: None,
                move_number,
                turn,
                from: from.into(),
                to: to.into(),
            },
            LegalAction::PillbugSpecial { piece_from, to } => Self::PillbugSpecial {
                id: None,
                move_number,
                turn,
                from: piece_from.into(),
                to: to.into(),
            },
        }
    }
}

impl TryFrom<PositionResponse> for Position {
    type Error = ApiError;

    fn try_from(position: PositionResponse) -> Result<Self, Self::Error> {
        Position::new(position.q, position.s, position.r).map_err(map_hive_error)
    }
}

impl From<Position> for PositionResponse {
    fn from(position: Position) -> Self {
        Self {
            q: position.q,
            s: position.s,
            r: position.r,
        }
    }
}

impl From<&hive_engine::Piece> for PieceResponse {
    fn from(piece: &hive_engine::Piece) -> Self {
        Self {
            color: PlayerColor::from(piece.color),
            piece_type: PieceTypeResponse::from(piece.piece_type),
        }
    }
}

impl From<Color> for PlayerColor {
    fn from(color: Color) -> Self {
        match color {
            Color::White => Self::White,
            Color::Black => Self::Black,
        }
    }
}

impl TryFrom<&str> for PlayerColor {
    type Error = ApiError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "white" => Ok(Self::White),
            "black" => Ok(Self::Black),
            _ => Err(ApiError::InvalidAction),
        }
    }
}

impl From<PieceTypeResponse> for PieceType {
    fn from(piece_type: PieceTypeResponse) -> Self {
        match piece_type {
            PieceTypeResponse::Queen => Self::Queen,
            PieceTypeResponse::Ant => Self::Ant,
            PieceTypeResponse::Beetle => Self::Beetle,
            PieceTypeResponse::Grasshopper => Self::Grasshopper,
            PieceTypeResponse::Spider => Self::Spider,
            PieceTypeResponse::Mosquito => Self::Mosquito,
            PieceTypeResponse::Ladybug => Self::Ladybug,
            PieceTypeResponse::Pillbug => Self::Pillbug,
        }
    }
}

impl From<PieceType> for PieceTypeResponse {
    fn from(piece_type: PieceType) -> Self {
        match piece_type {
            PieceType::Queen => Self::Queen,
            PieceType::Ant => Self::Ant,
            PieceType::Beetle => Self::Beetle,
            PieceType::Grasshopper => Self::Grasshopper,
            PieceType::Spider => Self::Spider,
            PieceType::Mosquito => Self::Mosquito,
            PieceType::Ladybug => Self::Ladybug,
            PieceType::Pillbug => Self::Pillbug,
        }
    }
}

impl TryFrom<&str> for PieceTypeResponse {
    type Error = ApiError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        PieceType::try_from(value)
            .map(Self::from)
            .map_err(map_hive_error)
    }
}

impl From<&Inventory> for InventoryResponse {
    fn from(inventory: &Inventory) -> Self {
        Self {
            queen: inventory.Queen,
            ant: inventory.Ant,
            beetle: inventory.Beetle,
            grasshopper: inventory.Grasshopper,
            spider: inventory.Spider,
            mosquito: inventory.Mosquito,
            ladybug: inventory.Ladybug,
            pillbug: inventory.Pillbug,
        }
    }
}

fn maybe_position(
    q: Option<i16>,
    s: Option<i16>,
    r: Option<i16>,
) -> Result<Option<Position>, ApiError> {
    match (q, s, r) {
        (Some(q), Some(s), Some(r)) => {
            let q = i8::try_from(q).map_err(|_| ApiError::InvalidAction)?;
            let s = i8::try_from(s).map_err(|_| ApiError::InvalidAction)?;
            let r = i8::try_from(r).map_err(|_| ApiError::InvalidAction)?;
            Position::new(q, s, r).map(Some).map_err(map_hive_error)
        }
        (None, None, None) => Ok(None),
        _ => Err(ApiError::InvalidAction),
    }
}
