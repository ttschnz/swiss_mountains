use pyo3::{exceptions::PyValueError, prelude::*};
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::Path;

use crate::utils::bounding_box::BoundingBox;

const SWISSIMAGE_CACHE_FILE: &str = "cache/swissimage.sqlite3";

pub fn initialize_cache() -> Result<Connection> {
    let db_path = Path::new(SWISSIMAGE_CACHE_FILE);
    // create directories recursively if not exist
    if let Some(parent_dir) = db_path.parent() {
        let _ = fs::create_dir_all(parent_dir);
    }
    let conn = Connection::open(SWISSIMAGE_CACHE_FILE)?;
    conn.execute_batch(
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
    )?;
    Ok(conn)
}

pub fn get_from_cache(
    step: usize,
    bounding_box: &BoundingBox,
) -> Result<Vec<(i32, i32, u8, u8, u8)>> {
    let conn = initialize_cache()?;
    let mut stmt = conn.prepare(
        "
        SELECT x_min, y_min, r, g, b
        FROM swissimage_data
        WHERE
            x_min BETWEEN ?2 AND ?3 AND -- bounding box x
            y_min BETWEEN ?4 AND ?5 AND -- bounding box y
            -- sampling
            (CAST(x_min AS INTEGER) % ?1 = 0) AND
            (CAST(y_min AS INTEGER) % ?1 = 0)
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

    let mut parsed_rows: Vec<(i32, i32, u8, u8, u8)> = vec![];
    while let Some(row) = rows.next()? {
        let x: f64 = row.get("x_min")?;
        let y: f64 = row.get("y_min")?;
        let r: u8 = row.get("r")?;
        let g: u8 = row.get("g")?;
        let b: u8 = row.get("b")?;
        parsed_rows.push((x as i32, y as i32, r, g, b));
    }
    Ok(parsed_rows)
}

pub fn write_to_cache(data: &[(u32, u32, u8, u8, u8)], reference: &str) -> Result<()> {
    let mut conn = initialize_cache()?;
    conn.execute(
        "INSERT OR REPLACE INTO swissimage_references (name) VALUES (?1)",
        [reference],
    )?;
    let ref_id = conn.last_insert_rowid();
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO swissimage_data (x_min, x_max, y_min, y_max, r, g, b, reference_id) VALUES (?1, ?1, ?2, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for (x, y, r, g, b) in data {
            stmt.execute(params![x, y, r, g, b, ref_id])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn check_cache(reference: &str) -> Result<bool> {
    let conn = initialize_cache()?;
    let count_found: isize = conn.query_row(
        "SELECT COUNT(*) FROM swissimage_references WHERE name IS ?1",
        [reference],
        |row| row.get(0),
    )?;

    Ok(count_found > 0)
}

#[pyfunction(name = "initialize_cache")]
pub fn initialize_cache_python_wrapper() -> PyResult<()> {
    let _conn = initialize_cache().map_err(|db_err| PyValueError::new_err(db_err.to_string()))?;
    Ok(())
}

#[pyfunction(name = "get_from_cache")]
pub fn get_from_cache_python_wrapper(
    step: usize,
    x_range: (i32, i32),
    y_range: (i32, i32),
) -> PyResult<Vec<(i32, i32, u8, u8, u8)>> {
    let bounding_box = BoundingBox::from_ranges(x_range, y_range);
    get_from_cache(step, &bounding_box).map_err(|db_err| PyValueError::new_err(db_err.to_string()))
}

#[pymodule(name = "swissimage_cache")]
pub fn cache_module(_py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    parent.add_function(wrap_pyfunction!(initialize_cache_python_wrapper, parent)?)?;
    parent.add_function(wrap_pyfunction!(get_from_cache_python_wrapper, parent)?)?;
    Ok(())
}
