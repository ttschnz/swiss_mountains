use std::path::Path;
use std::{fs, time::Duration};
use tokio_rusqlite::{params, Connection, Result};

use crate::utils::bounding_box::BoundingBox;

const SWISSIMAGE_CACHE_FILE: &str = "cache/swissimage.sqlite3";

pub async fn initialize_cache() -> Result<Connection> {
    let db_path = Path::new(SWISSIMAGE_CACHE_FILE);
    // create directories recursively if not exist
    if let Some(parent_dir) = db_path.parent() {
        let _ = fs::create_dir_all(parent_dir);
    }
    let conn = Connection::open(SWISSIMAGE_CACHE_FILE).await?;
    conn.call(|conn| {
        conn.busy_timeout(Duration::from_millis(30000))?;
        Ok(conn.execute_batch(
            "
            BEGIN;
            -- References
            CREATE TABLE IF NOT EXISTS swissimage_references (
                ref_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                modify_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            -- R*Tree table for fast queries
            CREATE VIRTUAL TABLE IF NOT EXISTS swissimage_data
            USING rtree(
                rowid,
                x_min, x_max,
                y_min, y_max,
                +r INTEGER NOT NULL,
                +g INTEGER NOT NULL,
                +b INTEGER NOT NULL,
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
    datapoint_percent: f64,
    bounding_box: &BoundingBox,
) -> Result<Vec<(i32, i32, u8, u8, u8)>> {
    let bounding_box = (*bounding_box).clone();
    let conn = initialize_cache().await?;
    let rows = conn
        .call(move |conn| {
            let mut stmt = conn.prepare(
                "
                SELECT * FROM (
                    SELECT x_min as x, y_min as y, r, g, b, rowid
                        FROM swissimage_data
                        WHERE
                            x_min BETWEEN ?2 AND ?3 AND -- bounding box x
                            y_min BETWEEN ?4 AND ?5     -- bounding box y                
                    )
                WHERE rowid % ?1 == 0
                ORDER BY x,y;                
            ",
            )?;
            let mut rows = stmt.query((
                100f64 / datapoint_percent,
                bounding_box.x_range.0,
                bounding_box.x_range.1,
                bounding_box.y_range.0,
                bounding_box.y_range.1,
            ))?;
            let mut parsed_rows: Vec<(i32, i32, u8, u8, u8)> = vec![];
            while let Some(row) = rows.next()? {
                let x: f64 = row.get("x")?;
                let y: f64 = row.get("y")?;
                let r: u8 = row.get("r")?;
                let g: u8 = row.get("g")?;
                let b: u8 = row.get("b")?;
                parsed_rows.push((x as i32, y as i32, r, g, b));
            }
            Ok(parsed_rows)
        })
        .await?;
    Ok(rows)
}

pub async fn write_to_cache(data: &Vec<(u32, u32, u8, u8, u8)>, reference: &str) -> Result<()> {
    let data = data.clone();
    let reference = reference.to_string();
    let conn = initialize_cache().await?;
    conn.call(move |conn|{
        let tx = conn.transaction()?;
        {
            tx.execute(
                "INSERT OR REPLACE INTO swissimage_references (name) VALUES (?1)",
                [reference],
            )?;
            let ref_id = tx.last_insert_rowid();
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO swissimage_data (x_min, x_max, y_min, y_max, r, g, b, reference_id) VALUES (?1, ?1, ?2, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for (x, y, r, g, b) in data {
                stmt.execute(params![x, y, r, g, b, ref_id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }).await
}

pub async fn check_cache(reference: &str) -> Result<bool> {
    let conn = initialize_cache().await?;
    let reference = reference.to_string();
    conn.call(|conn| {
        let count_found: isize = conn.query_row(
            "SELECT COUNT(*) FROM swissimage_references WHERE name IS ?1",
            [reference],
            |row| row.get(0),
        )?;

        Ok(count_found > 0)
    })
    .await
}
