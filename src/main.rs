mod anki;
mod swissalti3d;
mod swissimage;
mod swissnames;
mod terrain_viz;
mod utils;

use std::env::temp_dir;

use crate::anki::create_deck;
use crate::swissnames::get_peaks;
use crate::terrain_viz::{compose_gif, create_mesh, prepare_data, render_mesh};

use anyhow::Result;
use env_logger::{fmt::TimestampPrecision, Builder};
use log::info;
use three_d::*;

#[tokio::main]
async fn main() -> Result<()> {
    // basic parameters (will be configurable via command line arguments)
    let render_size = (300u16, 300u16);
    let center = (2609898, 1172630); // easting, northing
    let radius = 10000f64; // m
    let sampling_size = 500; // total nodes per dimension
    let anki_file_name = "mountains.apkg";

    // --- Initialise logging
    let mut builder = Builder::new();
    builder.format_timestamp(Some(TimestampPrecision::Millis));
    builder.filter_level(log::LevelFilter::Debug);
    builder.init();

    // --- headless GL context: singleton, cannot be created multiple times per program ---
    let context = HeadlessContext::new()?;
    let all_peaks = get_peaks();

    let filtered_peaks = all_peaks
        .iter()
        .filter(|named_place| named_place.distance_to(center.0, center.1) < radius)
        .collect::<Vec<_>>();

    let temp_dir = temp_dir();
    let mut file_list = vec![];
    let mut peak_names = vec![];
    info!(
        "{} peaks in range: {}",
        filtered_peaks.len(),
        filtered_peaks
            .iter()
            .map(|peak| peak.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    for peak in filtered_peaks {
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
        let gif_path = temp_dir.join(format!("{}.gif", peak.name.to_lowercase()));
        compose_gif(&mut raw_data, gif_path.as_path())?;
        file_list.push(gif_path);
        peak_names.push(peak.name.as_str());
    }
    info!("creating anki deck");

    let anki_file_name = std::env::current_dir()?.join(anki_file_name);
    create_deck(&file_list, &peak_names, &anki_file_name)?;

    info!("done");
    showfile::show_path_in_file_manager(anki_file_name.into_os_string());

    Ok(())
}
