use crate::{
    swissimage::cache,
    utils::{bounding_box::BoundingBox, url_to_ref},
};
use anyhow::{Error, Result};
use pyo3::{exceptions::PyValueError, prelude::*};
use reqwest::blocking;
use std::io::BufReader;
use std::io::Cursor;
use tiff::decoder::{Decoder, DecodingResult};

// Request download at:
// https://ogd.swisstopo.admin.ch/services/swiseld/services/assets/ch.swisstopo.swissimage-dop10/search?format=image%2Ftiff%3B%20application%3Dgeotiff%3B%20profile%3Dcloud-optimized&resolution=2.0&srid=2056&state=current&csv=true
const URL_LIST: &str = include_str!("ch.swisstopo.swissimage.csv");

pub fn get_url_list(searching_box: &BoundingBox) -> Result<Vec<String>> {
    let mut url_list = vec![];
    for url in URL_LIST.lines() {
        let box_covered = BoundingBox::get_box_covered(url)?;
        if searching_box.intersects(&box_covered) {
            url_list.push(url.to_string())
        }
    }
    Ok(url_list)
}
// input: Y, Cb, Cr in [0, 255]
// output: R, G, B in [0, 255]
fn ycbcr_to_rgb(y: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    // ITU-R BT.601
    let y = y as f32;
    let cb = cb as f32 - 128.0;
    let cr = cr as f32 - 128.0;

    let r = y + 1.402 * cr;
    let g = y - 0.344136 * cb - 0.714136 * cr;
    let b = y + 1.772 * cb;

    (
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    )
}

pub fn prefetch(url: String) -> Result<()> {
    let reference = url_to_ref(&url).ok_or(rusqlite::Error::InvalidQuery)?;

    // check if the url is already cached
    if cache::check_cache(&reference)? {
        return Ok(());
    }
    let bounding_box = BoundingBox::get_box_covered(&url)?;

    // download reference
    let response = blocking::get(&url)?;
    let bytes = response.bytes()?;
    // data in buffer is an image (.tif)
    let buffer = bytes.to_vec();
    let cursor = Cursor::new(buffer);

    let mut decoder = Decoder::new(BufReader::new(cursor))?;

    let dim = decoder.dimensions()?;
    let width = dim.0;
    let height = dim.1;

    let ycbcr_data: Vec<u8> = match decoder.read_image()? {
        DecodingResult::U8(buf) => buf,
        _ => panic!("Expected U8 image data"),
    };

    // 3 times u8 = 3 * 1 byte = 3 bytes
    // 2 times u32 = 2 * 4 bytes = 8 bytes
    // Total bytes = 3 + 8 = 11 bytes.
    let mut rgb_pixels = Vec::with_capacity((width * height * 11) as usize);

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 3) as usize;

            let y_val = ycbcr_data[idx];
            let cb_val = ycbcr_data[idx + 1];
            let cr_val = ycbcr_data[idx + 2];

            let (r, g, b) = ycbcr_to_rgb(y_val, cb_val, cr_val);

            rgb_pixels.push((
                bounding_box.x_range.0 as u32 + x * 2,
                bounding_box.y_range.1 as u32 - y * 2,
                r,
                g,
                b,
            ));
        }
    }

    cache::write_to_cache(&rgb_pixels, &reference)?;

    Ok(())
}

#[pyfunction(name = "get_url_list")]
fn get_url_list_python_wrapper(x_range: (i32, i32), y_range: (i32, i32)) -> PyResult<Vec<String>> {
    let searching_box = BoundingBox::from_ranges(x_range, y_range);
    get_url_list(&searching_box).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction(name = "prefetch")]
fn prefetch_python_wrapper(url: String) -> PyResult<()> {
    // fn prefetch(url: String) -> Result<(), Box<dyn Error>> {
    prefetch(url).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pymodule(name = "swissimage_fetch")]
pub fn fetch_module(_py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    parent.add_function(wrap_pyfunction!(get_url_list_python_wrapper, parent)?)?;
    parent.add_function(wrap_pyfunction!(prefetch_python_wrapper, parent)?)?;
    Ok(())
}
