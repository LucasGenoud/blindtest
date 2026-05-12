use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db() -> Result<DbPool> {
    std::fs::create_dir_all("data").ok();
    let conn = Connection::open("data/blindtest.db")?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    let schema = include_str!("../migrations/001_init.sql");
    let migration2 = include_str!("../migrations/002_add_s3_key.sql");
    let migration3 = include_str!("../migrations/003_remove_ratings.sql");
    let migration4 = include_str!("../migrations/004_remove_chat.sql");
    conn.execute_batch(schema)?;
    // Apply migrations safely: execute each statement individually and ignore known benign errors
    for migration in &[migration2, migration3, migration4] {
        let statements: Vec<&str> = migration.split(';').filter(|s| !s.trim().is_empty()).collect();
        for stmt in statements {
            let stmt = stmt.trim();
            if stmt.is_empty() { continue; }
            if let Err(e) = conn.execute(stmt, []) {
                let msg = e.to_string();
                if msg.contains("duplicate column") || msg.contains("no such column") || msg.contains("no such table") {
                    log::debug!("Migration ignored: {}", msg);
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    // Initialize canvas if empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM canvas_pixels", [], |row| row.get(0))?;
    if count == 0 {
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare("INSERT INTO canvas_pixels (x, y, color) VALUES (?1, ?2, 'ffffff')")?;
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
