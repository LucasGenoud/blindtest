use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db() -> Result<DbPool> {
    std::fs::create_dir_all("data").ok();
    let conn = Connection::open("data/blindtest.db")?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    let schema = include_str!("../migrations/001_init.sql");
    let migration2 = include_str!("../migrations/002_add_s3_key.sql");
    conn.execute_batch(schema)?;
    // Apply migration2 safely: execute each statement individually and ignore "duplicate column" errors
let statements: Vec<&str> = migration2.split(';').filter(|s| !s.trim().is_empty()).collect();
for stmt in statements {
    let stmt = stmt.trim();
    if stmt.is_empty() { continue; }
    // Execute and ignore error if column already exists
    if let Err(e) = conn.execute(stmt, []) {
        // SQLite error code for duplicate column is "SQLITE_ERROR" with message containing "duplicate column"
        if e.to_string().contains("duplicate column") {
            // Log and continue
            log::debug!("Migration ignored duplicate column: {}", stmt);
        } else {
            // Propagate other errors
            return Err(e.into());
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
