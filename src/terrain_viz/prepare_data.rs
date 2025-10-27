use super::{AltitudeData, ImageData};
use crate::{swissalti3d, swissimage, utils::bounding_box::BoundingBox};

use anyhow::{Error, Result};
use log::debug;
use std::cmp;
use std::sync::Arc;
use tokio::sync::Semaphore;

const SWISSALTI3D_DATAPOINT_PER_METER: f64 = 0.5;
const SWISSIMAGE_DATAPOINT_PER_METER: f64 = 0.5;

static SQLITE_SWISSIMAGE_PERMIT: Semaphore = Semaphore::const_new(1);
static SQLITE_SWISSALTI3D_PERMIT: Semaphore = Semaphore::const_new(1);

pub async fn prepare_data(
    peak_coordinates: (u32, u32),
    radius: u32,
    width: usize,
    offline: bool,
    download_threads: usize,
) -> Result<(AltitudeData, ImageData)> {
    let peak_north = cmp::min(peak_coordinates.0, peak_coordinates.1) as i32;
    let peak_east = cmp::max(peak_coordinates.0, peak_coordinates.1) as i32;

    let bounding_box = BoundingBox::from_ranges(
        (peak_east - radius as i32, peak_east + radius as i32),
        (peak_north - radius as i32, peak_north + radius as i32),
    );
    let total_width = 2 * radius;
    let swissalti3d_datapoint_percent =
        width as f64 / (total_width as f64 * SWISSALTI3D_DATAPOINT_PER_METER) * 100f64;
    let swissimage_datapoint_percent =
        width as f64 / (total_width as f64 * SWISSIMAGE_DATAPOINT_PER_METER) * 100f64;

    if !offline {
        let fetching_semaphore = Arc::new(Semaphore::new(download_threads));
        let mut handles: std::vec::Vec<tokio::task::JoinHandle<Result<_>>> = vec![];

        // cache altitude points
        let swissalti3d_url_list = swissalti3d::fetch::get_url_list(&bounding_box)?;
        debug!("prefetching {} height files", swissalti3d_url_list.len());
        for url in swissalti3d_url_list {
            let fetching_permit = fetching_semaphore.clone().acquire_owned().await?;
            let url = url.clone();
            handles.push(tokio::spawn(async {
                let _permit = fetching_permit;
                if let Some((data, reference)) = swissalti3d::fetch::prefetch(url).await? {
                    let _writing_permit = SQLITE_SWISSALTI3D_PERMIT.acquire().await?;
                    swissalti3d::cache::write_to_cache(data, &reference).await?;
                }
                Ok(())
            }));
        }

        // cache colors
        let swissimage_url_list = swissimage::fetch::get_url_list(&bounding_box)?;
        debug!("prefetching {} color files", swissimage_url_list.len());
        for url in swissimage_url_list {
            let fetching_permit = fetching_semaphore.clone().acquire_owned().await.unwrap();
            let url = url.clone();
            handles.push(tokio::spawn(async {
                let _permit = fetching_permit;
                if let Some((data, reference)) = swissimage::fetch::prefetch(url).await? {
                    let _writing_permit = SQLITE_SWISSIMAGE_PERMIT.acquire().await?;
                    swissimage::cache::write_to_cache(&data, &reference).await?;
                }
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

    debug!(
        "expecting {} altitude points and {} color points",
        width.pow(2),
        width.pow(2)
    );

    debug!(
        "collecting altitude points from cache with {}%",
        swissalti3d_datapoint_percent
    );
    let complete_altitude_data =
        swissalti3d::cache::get_from_cache(swissalti3d_datapoint_percent, &bounding_box).await?;
    let altitude_data = complete_altitude_data
        .iter()
        .filter(|(x, y, _z)| {
            // make the data a circle around the peak
            (x - peak_east).pow(2) + (y - peak_north).pow(2) < radius.pow(2) as i32
        })
        .cloned()
        .collect::<AltitudeData>();

    debug!(
        "collecting colors from cache with {}%",
        swissimage_datapoint_percent
    );
    let image_data =
        swissimage::cache::get_from_cache(swissimage_datapoint_percent, &bounding_box).await?;

    debug!(
        "working with {} altitude points and {} color points",
        altitude_data.len(),
        image_data.len()
    );
    Ok((altitude_data, image_data))
}
