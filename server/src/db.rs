use rusqlite::{Connection, OpenFlags, Result};
use std::sync::{Arc, Mutex, MutexGuard};

pub const DB_PATH: &str = "data/blindtest.db";

pub type DbPool = Arc<Mutex<Connection>>;

/// A separate read-only connection for long scans.
///
/// The whole app shares one connection behind a mutex, so a query that takes a
/// second (the canvas is a million rows) stalls every other request for that long.
/// WAL lets readers run concurrently with the writer, so scans that only read are
/// better off on their own connection.
pub fn open_read_only() -> Result<Connection> {
    Connection::open_with_flags(
        DB_PATH,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI,
    )
}

/// Lock the shared connection, recovering from a poisoned mutex.
///
/// A panic anywhere while the guard is held poisons the mutex, after which every
/// `lock().unwrap()` in the process panics too — one bad request would take down
/// every database route until the server is restarted. The connection itself stays
/// usable, so recovering is strictly better than propagating the poison.
pub fn lock_db(pool: &DbPool) -> MutexGuard<'_, Connection> {
    pool.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Unwrap a `rusqlite` result inside a request handler, answering 500 instead of
/// panicking. Panicking while the connection guard is held poisons the mutex and
/// aborts the connection, so a single malformed query used to be able to damage
/// unrelated requests. Only usable in handlers returning `HttpResponse`.
#[macro_export]
macro_rules! db_try {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(e) => {
                log::error!("database error at {}:{}: {}", file!(), line!(), e);
                return actix_web::HttpResponse::InternalServerError().json("Database error");
            }
        }
    };
}

pub fn init_db() -> Result<DbPool> {
    std::fs::create_dir_all("data").ok();
    let conn = Connection::open(DB_PATH)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    let schema = include_str!("../migrations/001_init.sql");
    let migration2 = include_str!("../migrations/002_add_s3_key.sql");
    let migration3 = include_str!("../migrations/003_remove_ratings.sql");
    let migration4 = include_str!("../migrations/004_remove_chat.sql");
    let migration5 = include_str!("../migrations/005_flag_source_and_cleanup.sql");
    let migration6 = include_str!("../migrations/006_blindtest_agent.sql");
    conn.execute_batch(schema)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            name TEXT PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;
    for (name, migration) in [
        ("002_add_s3_key", migration2),
        ("003_remove_ratings", migration3),
        ("004_remove_chat", migration4),
        ("005_flag_source_and_cleanup", migration5),
        ("006_blindtest_agent", migration6),
    ] {
        let applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE name = ?1)",
            [name],
            |row| row.get(0),
        )?;
        if applied {
            continue;
        }
        let statements: Vec<&str> = migration
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .collect();
        for stmt in statements {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            if let Err(e) = conn.execute(stmt, []) {
                let msg = e.to_string();
                if msg.contains("duplicate column")
                    || msg.contains("no such column")
                    || msg.contains("no such table")
                {
                    log::debug!("Migration ignored: {}", msg);
                } else {
                    return Err(e);
                }
            }
        }
        conn.execute(
            "INSERT INTO schema_migrations (name, applied_at) VALUES (?1, ?2)",
            rusqlite::params![name, chrono::Utc::now().to_rfc3339()],
        )?;
    }

    // Initialize canvas if empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM canvas_pixels", [], |row| row.get(0))?;
    if count == 0 {
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt =
                tx.prepare("INSERT INTO canvas_pixels (x, y, color) VALUES (?1, ?2, 'ffffff')")?;
            for y in 0..1000 {
                for x in 0..1000 {
                    stmt.execute([x, y])?;
                }
            }
        }
        tx.commit()?;
        log::info!("Initialized 1,000,000 canvas pixels");
    }

    Ok(Arc::new(Mutex::new(conn)))
}
