use once_cell::sync::OnceCell;
use sqlx::SqlitePool;

static DB_POOL: OnceCell<SqlitePool> = OnceCell::new();

/// Set the global DB pool once (usually in `main`)
pub fn set(pool: SqlitePool) {
    DB_POOL.set(pool).expect("DB_POOL already initialized");
}

/// Get a reference to the shared global DB pool
pub fn get() -> &'static SqlitePool {
    DB_POOL.get().expect("DB_POOL is not initialized")
}
