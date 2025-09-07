use super::{AltitudeData, ImageData};
use crate::{swissalti3d, swissimage, utils::bounding_box::BoundingBox};

use anyhow::{Error, Result};
use log::debug;
use std::cmp;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub async fn prepare_data(
    peak_coordinates: (u32, u32),
    radius: u32,
    width: u16,
    offline: bool,
) -> Result<(AltitudeData, ImageData)> {
    let peak_north = cmp::min(peak_coordinates.0, peak_coordinates.1) as i32;
    let peak_east = cmp::max(peak_coordinates.0, peak_coordinates.1) as i32;

    let bounding_box = BoundingBox::from_ranges(
        (peak_east - radius as i32, peak_east + radius as i32),
        (peak_north - radius as i32, peak_north + radius as i32),
    );
    let total_width = 2 * radius;
    let step = total_width as usize / width as usize;

    if !offline {
        let semaphore = Arc::new(Semaphore::new(10));
        let mut handles: std::vec::Vec<tokio::task::JoinHandle<Result<_>>> = vec![];

        // cache altitude points
        let swissalti3d_url_list = swissalti3d::fetch::get_url_list(&bounding_box)?;
        debug!("prefetching {} height files", swissalti3d_url_list.len());
        for url in swissalti3d_url_list {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let url = url.clone();
            handles.push(tokio::spawn(async {
                let _permit = permit;
                swissalti3d::fetch::prefetch(url).await?;
                Ok(())
            }));
        }

        // cache colors
        let swissimage_url_list = swissimage::fetch::get_url_list(&bounding_box)?;
        debug!("prefetching {} color files", swissimage_url_list.len());
        for url in swissimage_url_list {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let url = url.clone();
            handles.push(tokio::spawn(async {
                let _permit = permit;
                swissimage::fetch::prefetch(url).await?;
                Ok(())
            }));
        }

        // wait for each download to complete, then handle errors
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => return Err(e),
                Err(join_err) => return Err(Error::from(join_err)),
            }
        }
    }

    debug!("collecting altitude points from cache");
    let complete_altitude_data = swissalti3d::cache::get_from_cache(step, &bounding_box).await?;
    let altitude_data = complete_altitude_data
        .iter()
        .filter(|(x, y, _z)| {
            // make the data a circle around the peak
            (x - peak_east).pow(2) + (y - peak_north).pow(2) < radius.pow(2) as i32
        })
        .cloned()
        .collect::<AltitudeData>();

    debug!("collecting colors from cache");
    let image_data = swissimage::cache::get_from_cache(step, &bounding_box).await?;

    Ok((altitude_data, image_data))
}
