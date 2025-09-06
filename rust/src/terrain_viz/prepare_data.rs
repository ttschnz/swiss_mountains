use super::{AltitudeData, ImageData};
use crate::{swissalti3d, swissimage, utils::bounding_box::BoundingBox};

use anyhow::Result;
use log::debug;
use std::{cmp, error::Error};

pub fn prepare_data(
    peak_coordinates: (u32, u32),
    radius: u32,
    width: u16,
    offline: bool,
) -> Result<(AltitudeData, ImageData), Box<dyn Error>> {
    let peak_north = cmp::min(peak_coordinates.0, peak_coordinates.1) as i32;
    let peak_east = cmp::max(peak_coordinates.0, peak_coordinates.1) as i32;

    let bounding_box = BoundingBox::from_ranges(
        (peak_east - radius as i32, peak_east + radius as i32),
        (peak_north - radius as i32, peak_north + radius as i32),
    );
    let total_width = 2 * radius;
    let step = total_width as usize / width as usize;

    if !offline {
        // cache altitude points
        let swissalti3d_url_list = swissalti3d::fetch::get_url_list(&bounding_box)?;
        debug!("prefetching {} height files", swissalti3d_url_list.len());
        for url in swissalti3d_url_list {
            swissalti3d::fetch::prefetch(url)?;
        }

        // cache colors
        let swissimage_url_list = swissimage::fetch::get_url_list(&bounding_box)?;
        debug!("prefetching {} color files", swissimage_url_list.len());
        for url in swissimage_url_list {
            swissimage::fetch::prefetch(url)?;
        }
    }

    debug!("collecting altitude points from cache");
    let complete_altitude_data = swissalti3d::cache::get_from_cache(step, &bounding_box)?;
    let altitude_data = complete_altitude_data
        .iter()
        .filter(|(x, y, _z)| {
            // make the data a circle around the peak
            (x - peak_east).pow(2) + (y - peak_north).pow(2) < radius.pow(2) as i32
        })
        .cloned()
        .collect::<AltitudeData>();

    debug!("collecting colors from cache");
    let image_data = swissimage::cache::get_from_cache(step, &bounding_box)?;

    Ok((altitude_data, image_data))
}
