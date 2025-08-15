use sea_orm::{ConnectionTrait, DbErr, Statement};

pub async fn clean_db() -> Result<(), DbErr> {
    let db = crate::get_connection().await;

    // 1) get user table names (exclude sqlite internal tables)
    let rows = db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';".to_owned(),
        ))
        .await?;

    let table_names: Vec<String> = rows.into_iter()
        .map(|r| r.try_get("", "name").unwrap())
        .collect();

    if table_names.is_empty() {
        return Ok(());
    }

    // Precompute safely quoted identifiers and single-quoted list for sqlite_sequence
    let quoted_ident: Vec<String> = table_names
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect();

    let seq_list: String = table_names
        .iter()
        .map(|t| format!("'{}'", t.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");

    // Build a single SQL batch: disable foreign keys, begin, delete all tables, reset sequences, commit, re-enable fkeys
    // This reduces roundtrips to one execute in the fast path.
    let mut batch_sql = String::with_capacity(1024);
    batch_sql.push_str("PRAGMA foreign_keys = OFF;\nBEGIN TRANSACTION;\n");
    for q in &quoted_ident {
        batch_sql.push_str(&format!("DELETE FROM {};\n", q));
    }
    // Reset only the tables we touched. If sqlite_sequence doesn't exist this statement may error,
    // we'll catch that in the match below and fall back.
    batch_sql.push_str(&format!("DELETE FROM sqlite_sequence WHERE name IN ({});\n", seq_list));
    batch_sql.push_str("COMMIT;\nPRAGMA foreign_keys = ON;\n");

    // Try single-execute fast path
    let exec_res = db
        .execute(Statement::from_string(db.get_database_backend(), batch_sql.clone()))
        .await;

    match exec_res {
        Ok(_) => return Ok(()),
        Err(e) => {
            // Fast-path failed (some drivers disallow multiple statements). Fall back to a minimal set of operations.
            // We'll:
            //  - ensure foreign keys are OFF on this connection,
            //  - begin, delete per-table (N execs) but still reset sqlite_sequence with a single statement,
            //  - commit and re-enable foreign keys.
            // This is more roundtrips but still safe.
            tracing::debug!("clean_db fast path failed, falling back to safe path: {:?}", e);
        }
    }

    // Fallback (safe) path
    db.execute(Statement::from_string(db.get_database_backend(), "PRAGMA foreign_keys = OFF;".to_owned())).await?;
    db.execute(Statement::from_string(db.get_database_backend(), "BEGIN TRANSACTION;".to_owned())).await?;

    for q in &quoted_ident {
        let sql = format!("DELETE FROM {};", q);
        db.execute(Statement::from_string(db.get_database_backend(), sql)).await?;
    }

    // Try reset sqlite_sequence for touched tables (ignore error if sqlite_sequence doesn't exist)
    let seq_sql = format!("DELETE FROM sqlite_sequence WHERE name IN ({});", seq_list);
    let _ = db.execute(Statement::from_string(db.get_database_backend(), seq_sql)).await;

    db.execute(Statement::from_string(db.get_database_backend(), "COMMIT;".to_owned())).await?;
    db.execute(Statement::from_string(db.get_database_backend(), "PRAGMA foreign_keys = ON;".to_owned())).await?;

    Ok(())
}