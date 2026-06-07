use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use hive_engine::Game;
use sqlx::PgPool;
use tokio::sync::Mutex;

const GAME_CACHE_CAPACITY: usize = 100;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub game_cache: GameCache,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            game_cache: GameCache::new(GAME_CACHE_CAPACITY),
        }
    }
}

#[derive(Clone)]
pub struct GameCache {
    inner: Arc<Mutex<GameCacheInner>>,
}

struct GameCacheInner {
    capacity: usize,
    games: HashMap<i32, CachedGame>,
    recently_used: VecDeque<i32>,
}

#[derive(Clone)]
pub struct CachedGame {
    pub game: Game,
    pub action_count: usize,
    pub last_action_id: Option<i32>,
    pub current_status: String,
}

impl GameCache {
    fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(GameCacheInner {
                capacity,
                games: HashMap::new(),
                recently_used: VecDeque::new(),
            })),
        }
    }

    pub async fn get(
        &self,
        game_id: i32,
        action_count: usize,
        last_action_id: Option<i32>,
        current_status: &str,
    ) -> Option<Game> {
        let mut inner = self.inner.lock().await;
        let cached = inner.games.get(&game_id)?;
        if cached.action_count != action_count
            || cached.last_action_id != last_action_id
            || cached.current_status != current_status
        {
            return None;
        }

        let game = cached.game.clone();
        inner.touch(game_id);
        Some(game)
    }

    pub async fn put(&self, game_id: i32, cached: CachedGame) {
        let mut inner = self.inner.lock().await;
        inner.games.insert(game_id, cached);
        inner.touch(game_id);
        inner.evict_oldest();
    }
}

impl GameCacheInner {
    fn touch(&mut self, game_id: i32) {
        self.recently_used.retain(|cached_id| *cached_id != game_id);
        self.recently_used.push_back(game_id);
    }

    fn evict_oldest(&mut self) {
        while self.games.len() > self.capacity {
            if let Some(game_id) = self.recently_used.pop_front() {
                self.games.remove(&game_id);
            } else {
                break;
            }
        }
    }
}
