use crate::utils::bounding_box::BoundingBox;
use std::fs;
use std::path::Path;
use tokio_rusqlite::{params, Connection, Result};

const SWISSALTI3D_CACHE_FILE: &str = "cache/swissalti3d.sqlite3";

pub async fn initialize_cache() -> Result<Connection> {
    let db_path = Path::new(SWISSALTI3D_CACHE_FILE);
    // create directories recursively if not exist
    if let Some(parent_dir) = db_path.parent() {
        let _ = fs::create_dir_all(parent_dir);
    }
    let conn = Connection::open(SWISSALTI3D_CACHE_FILE).await?; // open database file from constant path
    conn.call(move |conn| {
        Ok(conn.execute_batch(
            "
            BEGIN;
            -- References
            CREATE TABLE IF NOT EXISTS swissalti3d_references (
                ref_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                modify_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
    
            -- R*Tree table for fast queries
            CREATE VIRTUAL TABLE IF NOT EXISTS swissalti3d_data
            USING rtree(
                rowid,
                x_min, x_max,
                y_min, y_max,
                +z REAL,
                +reference_id INTEGER
            );
    
            COMMIT;
            ",
        ))
    })
    .await??;
    Ok(conn)
}

pub async fn get_from_cache(
    step: usize,
    bounding_box: &BoundingBox,
) -> Result<Vec<(i32, i32, f32)>> {
    let conn = initialize_cache().await?;
    let bounding_box = (*bounding_box).clone();
    let rows = conn
        .call(move |conn| {
            let mut stmt = conn.prepare(
                "
            SELECT x_min, y_min, z
        FROM swissalti3d_data
        WHERE
            x_min BETWEEN ?2 AND ?3 AND -- bounding box x
            y_min BETWEEN ?4 AND ?5 AND -- bounding box y
            -- sampling
            (CAST(x_min AS INTEGER) % ?1 = 1) AND
            (CAST(y_min AS INTEGER) % ?1 = 1)
        ORDER BY x_min, y_min;
        ",
            )?;
            let mut rows = stmt.query((
                step,
                bounding_box.x_range.0,
                bounding_box.x_range.1,
                bounding_box.y_range.0,
                bounding_box.y_range.1,
            ))?;

            let mut parsed_rows: Vec<(i32, i32, f32)> = vec![];
            while let Some(row) = rows.next()? {
                let x: f64 = row.get("x_min")?;
                let y: f64 = row.get("y_min")?;
                let z: f64 = row.get("z")?;
                parsed_rows.push((x as i32, y as i32, z as f32));
            }
            Ok(parsed_rows)
        })
        .await?;
    Ok(rows)
}

pub async fn write_to_cache(data: Vec<(u32, u32, f64)>, reference: &str) -> Result<()> {
    let reference = reference.to_string();

    let conn = initialize_cache().await?;
    conn.call(|conn|{
        conn.execute(
            "
            INSERT OR REPLACE INTO swissalti3d_references (name) VALUES (?1)
            ",
            [reference],
        )?;
        let ref_id = conn.last_insert_rowid();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO swissalti3d_data (x_min, x_max, y_min, y_max, z, reference_id) VALUES (?1, ?1, ?2, ?2, ?3, ?4)",
            )?;
            for (x, y, z) in data {
                stmt.execute(params![x, y, z, ref_id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }).await
}

pub async fn check_cache(reference: &str) -> Result<bool> {
    let reference = reference.to_string();
    let conn = initialize_cache().await?;
    conn.call(|conn| {
        let count_found: isize = conn.query_row(
            "SELECT COUNT(*) FROM swissalti3d_references WHERE name IS ?1",
            [reference],
            |row| row.get(0),
        )?;

        Ok(count_found > 0)
    })
    .await
}

// TODO: Decimal places? currently, values are represended in f32. floating point shit makes them look funny like 1625.5400390625, while we'd like to stay at 2 decimal places.
// maybe try rust_decimal
