//! Application state container shared across Axum route handlers and services.
//!
//! This struct holds shared resources such as the database connection and WebSocket manager.
//! It is typically wrapped in an `Arc` and passed into route handlers via Axum's `State<T>` extractor.

use crate::ws::WebSocketManager;
use sea_orm::DatabaseConnection;

/// Central application state shared across the server.
///
/// This includes:
/// - A cloned, thread-safe database connection for use with SeaORM.
/// - A global `WebSocketManager` for broadcasting and subscribing to topics.
#[derive(Clone)]
pub struct AppState {
    db: DatabaseConnection,
    ws: WebSocketManager,
}

impl AppState {
    /// Creates a new `AppState` with the given database connection and WebSocket manager.
    ///
    /// # Arguments
    ///
    /// * `db` - A SeaORM `DatabaseConnection`, typically cloned from the main pool.
    /// * `ws` - A `WebSocketManager` responsible for managing topic-based communication.
    pub fn new(db: DatabaseConnection, ws: WebSocketManager) -> Self {
        Self { db, ws }
    }

    /// Returns a shared reference to the internal `DatabaseConnection`.
    ///
    /// This is ideal when the caller does not need ownership.
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Returns a shared reference to the internal `WebSocketManager`.
    pub fn ws(&self) -> &WebSocketManager {
        &self.ws
    }
}

impl AppState {
    /// Returns a cloned copy of the database connection.
    ///
    /// Useful for async contexts or spawning tasks that require ownership.
    pub fn db_clone(&self) -> DatabaseConnection {
        self.db.clone()
    }

    /// Returns a cloned instance of the `WebSocketManager`.
    ///
    /// This allows handlers to send or subscribe without holding a reference.
    pub fn ws_clone(&self) -> WebSocketManager {
        self.ws.clone()
    }
}
