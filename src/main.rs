mod anki;
mod swissalti3d;
mod swissboundaries;
mod swissimage;
mod swissnames;
mod terrain_viz;
mod utils;

use std::env::temp_dir;
use std::fmt::format;

use crate::anki::create_deck;
use crate::swissnames::get_peaks;
use crate::terrain_viz::{compose_gif, create_mesh, prepare_data, render_mesh};

use anyhow::Result;
use env_logger::{fmt::TimestampPrecision, Builder};
use log::info;
use swissboundaries::{get_region, RegionType};
use three_d::*;

#[tokio::main]
async fn main() -> Result<()> {
    // basic parameters (will be configurable via command line arguments)
    let region = ("Bern".to_string(), RegionType::KANTONSGEBIET);
    let render_size = (1000u16, 1000u16);

    let sampling_size = 500; // total nodes per dimension
    let anki_file_name = format!("mountains_{}.apkg", region.0);
    let deck_name = format!("Mountains::{}", region.0);
    // --- Initialise logging
    let mut builder = Builder::new();
    builder.format_timestamp(Some(TimestampPrecision::Millis));
    builder.filter_level(log::LevelFilter::Debug);
    builder.init();

    // --- headless GL context: singleton, cannot be created multiple times per program ---
    let context = HeadlessContext::new()?;
    let all_peaks = get_peaks();
    let regiondb_conn =
        swissboundaries::open_connection(swissboundaries::dump_db_to_tempfile()?).await?;

    let mut filtered_peaks = Vec::new();
    for peak in all_peaks {
        if get_region((peak.easting, peak.northing), region.1, &regiondb_conn)
            .await?
            .contains(&region.0)
        {
            filtered_peaks.push(peak);
        }
    }

    let temp_dir = temp_dir();
    let mut file_list = vec![];
    let mut peak_names = vec![];
    info!(
        "{} peaks in region {}.\n{}",
        filtered_peaks.len(),
        region.0,
        filtered_peaks
            .iter()
            .map(|peak| peak.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    for peak in filtered_peaks {
        let gif_path = temp_dir.join(format!(
            "{}_{}_{}_{}_{}.gif",
            peak.name.to_lowercase(),
            peak.easting,
            peak.northing,
            render_size.0,
            render_size.1
        ));

        if !gif_path.exists() {
            info!("preparing data for {}", peak.name);
            let (altitude_data, image_data) = prepare_data(
                (peak.easting, peak.northing),
                (peak.altitude * 2) as u32,
                sampling_size,
                false,
            )
            .await?;

            info!("creating mesh");
            let mesh = create_mesh(altitude_data, image_data)?;

            let mut raw_data = Vec::new();
            for phi in 0..360 {
                info!("rendering mesh ({phi}/360)");
                let image = render_mesh(&mesh, 10.0, phi as f64, render_size, &context)?;
                let dim = (image.width() as u16, image.height() as u16);
                raw_data.push((image.into_raw(), dim.0, dim.1));
            }

            info!("composing gif");
            compose_gif(&mut raw_data, gif_path.as_path())?;
        }else{
            info!("gif for {} already exists ({:?})", peak.name, gif_path);
        }
        file_list.push(gif_path);
        peak_names.push(peak.name);
    }
    info!("creating anki deck");

    let anki_file_name = std::env::current_dir()?.join(anki_file_name);
    create_deck(&file_list, &peak_names, &anki_file_name, &deck_name)?;

    info!("done");
    showfile::show_path_in_file_manager(anki_file_name.into_os_string());

    Ok(())
}
