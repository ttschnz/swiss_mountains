use crate::{
    swissalti3d::cache,
    utils::{bounding_box::BoundingBox, url_to_ref},
};
use pyo3::{exceptions::PyValueError, prelude::*};
use reqwest::blocking;
use rusqlite::Result;
use std::error::Error;
use std::io::{BufRead, BufReader, Cursor};
use zip::ZipArchive;

// Request download at:
// https://ogd.swisstopo.admin.ch/services/swiseld/services/assets/ch.swisstopo.swissalti3d/search?format=application%2Fx.ascii-xyz%2Bzip&resolution=2.0&srid=2056&state=current&csv=true
const URL_LIST: &str = include_str!("ch.swisstopo.swissalti3d.csv");

pub fn get_url_list(searching_box: &BoundingBox) -> Result<Vec<String>, String> {
    let mut url_list = vec![];
    for url in URL_LIST.lines() {
        let box_covered = BoundingBox::get_box_covered(url)?;
        if searching_box.intersects(&box_covered) {
            url_list.push(url.to_string())
        }
    }
    Ok(url_list)
}

pub fn prefetch(url: String) -> Result<(), Box<dyn Error>> {
    let reference = url_to_ref(&url).ok_or(rusqlite::Error::InvalidQuery)?;

    // check if the url is already cached
    if cache::check_cache(&reference)? {
        return Ok(());
    }

    // download reference
    let mut data = vec![];
    let response = blocking::get(&url)?;
    let bytes = response.bytes()?;
    let buffer = bytes.to_vec();
    let cursor = Cursor::new(buffer);

    let mut archive = ZipArchive::new(cursor).expect("Failed to read zip archive");

    let mut file = archive.by_index(0)?;
    let reader = BufReader::new(&mut file);
    // first row is "X Y Z"
    for line_result in reader.lines().skip(1) {
        let line = line_result?;

        let mut data_line = line.split(' ');

        let x = data_line.next().ok_or("not enough data")?.parse::<u32>()?;
        let y = data_line.next().ok_or("not enough data")?.parse::<u32>()?;
        let z = data_line.next().ok_or("not enough data")?.parse::<f64>()?;

        data.push((x, y, z));
    }
    cache::write_to_cache(data, &reference)?;

    Ok(())
}

#[pyfunction(name = "get_url_list")]
fn get_url_list_python_wrapper(x_range: (i32, i32), y_range: (i32, i32)) -> PyResult<Vec<String>> {
    let searching_box = BoundingBox::from_ranges(x_range, y_range);
    get_url_list(&searching_box).map_err(PyValueError::new_err)
}

#[pyfunction(name = "prefetch")]
fn prefetch_python_wrapper(url: String) -> PyResult<()> {
    // fn prefetch(url: String) -> Result<(), Box<dyn Error>> {
    prefetch(url).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pymodule(name = "swissalti3d_fetch")]
pub fn fetch_module(_py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    parent.add_function(wrap_pyfunction!(get_url_list_python_wrapper, parent)?)?;
    parent.add_function(wrap_pyfunction!(prefetch_python_wrapper, parent)?)?;
    Ok(())
}

// swissalti3d.fetch.prefetch

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_prefetch() {
        prefetch("https://data.geo.admin.ch/ch.swisstopo.swissalti3d/swissalti3d_2019_2617-1166/swissalti3d_2019_2617-1166_2_2056_5728.xyz.zip".to_string()).unwrap();
    }
}
