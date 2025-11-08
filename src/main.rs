mod anki;
mod swissalti3d;
mod swissboundaries;
mod swissimage;
mod swissnames;
mod terrain_viz;
mod utils;

use std::env::current_dir;
use std::fs;

use crate::anki::create_deck;
use crate::swissnames::get_peaks;
use crate::terrain_viz::{compose_gif, create_mesh, prepare_data, render_mesh};

use anyhow::Result;
use env_logger::{fmt::TimestampPrecision, Builder};
use log::info;
use swissboundaries::{get_region, RegionType};
use three_d::*;

use clap::Parser;
use clap_num::si_number;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short='x', long="width", default_value_t=1000, value_parser=si_number::<u16>, help="Width of the exported gif in pixels")]
    render_width: u16,
    #[arg(short='y', long="height", default_value_t=1000, value_parser=si_number::<u16>,help="Height of the exported gif in pixels")]
    render_height: u16,
    #[arg(
        short = 'r',
        long = "region",
        help = "Region name that should be processed",
        conflicts_with_all = ["peak_x", "peak_y"]
    )]
    region_name: Option<String>,
    #[arg(short='t', long="region-type", default_value_t=RegionType::Kantonsgebiet, help="Type of region", conflicts_with_all = ["peak_x", "peak_y"])]
    region_type: RegionType,
    #[arg(
        long = "peak_x",
        help = "X-coordinate of a single peak to render",
        conflicts_with_all = ["region_name", "region_type"],
        requires="peak_x"
    )]
    peak_x: Option<u32>,
    #[arg(
        long = "peak_y",
        help = "Y-coordinate of a single peak to render",
        conflicts_with_all = ["region_name", "region_type"],
        requires="peak_x"
    )]
    peak_y: Option<u32>,
    #[arg(
        short = 's',
        long = "sampling",
        default_value_t = 500,
        help = "Amount of points per dimension (mesh density)"
    )]
    sampling_size: usize,
    #[arg(
        long = "download_thread_count",
        default_value_t = 10,
        help = "Amount of threads allowed to download content at the same time"
    )]
    download_threads: usize,
    #[arg(
        short = 'n',
        long = "batch_size",
        help = "Amount of mountains that should be processed",
        requires = "batch_index"
    )]
    batch_size: Option<usize>,
    #[arg(
        short = 'i',
        long = "batch_index",
        help = "Index of batch that should be processed (use with -n)",
        requires = "batch_size"
    )]
    batch_index: Option<usize>,
    #[arg(short='l', long="log_level", help="Verbosity", default_value_t=log::LevelFilter::Info)]
    log_level: log::LevelFilter,
    #[arg(
        short = 'o',
        long = "offline",
        help = "Prevent prefetch: this skips prefetch and works with the data we have",
        default_value_t = false
    )]
    offline: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut region_name = String::new();
    let mut region_type = RegionType::Kantonsgebiet;

    // --- Initialise logging
    let mut builder = Builder::new();
    builder.format_timestamp(Some(TimestampPrecision::Millis));
    builder.filter_level(cli.log_level);
    builder.init();

    // --- headless GL context: singleton, cannot be created multiple times per program ---
    let context = HeadlessContext::new()?;

    let mut all_peaks = get_peaks();
    let mut peaks_to_process = None;

    let regiondb_conn =
        swissboundaries::open_connection(swissboundaries::dump_db_to_tempfile()?).await?;

    if let Some(peak_coords) = cli.peak_x.zip(cli.peak_y) {
        region_name = get_region(peak_coords, RegionType::Kantonsgebiet, &regiondb_conn)
            .await?
            .get(0)
            .ok_or(anyhow::Error::msg("Peak not in any region"))?
            .to_owned();
        region_type = RegionType::Kantonsgebiet;
        all_peaks.sort_by_key(|peak| {
            peak.distance_to(peak_coords.0.into(), peak_coords.1.into())
                .round() as i64
        });
        let closest_peak = all_peaks
            .first()
            .ok_or(anyhow::Error::msg("No peak found at coordinates"))?
            .to_owned();

        peaks_to_process = Some(vec![closest_peak]);
    } else if let Some(cli_region_name) = cli.region_name {
        region_name = cli_region_name;
        region_type = cli.region_type;

        let mut filtered_peaks = Vec::new();
        for peak in all_peaks {
            if get_region(
                (peak.easting, peak.northing),
                cli.region_type,
                &regiondb_conn,
            )
            .await?
            .contains(&region_name)
            {
                filtered_peaks.push(peak);
            }
        }

        peaks_to_process = match (cli.batch_index, cli.batch_size) {
            (Some(batch_index), Some(batch_size)) => filtered_peaks
                .chunks(batch_size)
                .nth(batch_index)
                .map(Vec::from),
            _ => Some(filtered_peaks),
        };
    }

    let anki_file_name = format!("mountains_{}.apkg", region_name);
    let deck_name = format!("Mountains::{}", region_name);

    if let Some(peaks_to_process) = peaks_to_process {
        let gif_dir = current_dir()?.join("gifs").join(&region_name);
        let mut file_list = vec![];
        let mut peak_names = vec![];
        info!(
            "{} selected peaks in region {}.\n{}",
            peaks_to_process.len(),
            region_name,
            peaks_to_process
                .iter()
                .map(|peak| peak.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        for peak in peaks_to_process {
            let gif_path = gif_dir.join(format!(
                "{}_{}_{}_{}_{}.gif",
                peak.name.to_lowercase(),
                peak.easting,
                peak.northing,
                region_name,
                region_type
            ));

            if let Some(parent_dir) = gif_path.parent() {
                let _ = fs::create_dir_all(parent_dir);
            }
            if !gif_path.exists() {
                info!("preparing data for {}", peak.name);
                let (altitude_data, image_data) = prepare_data(
                    (peak.easting, peak.northing),
                    (peak.altitude * 2) as u32,
                    cli.sampling_size,
                    cli.offline,
                    cli.download_threads,
                )
                .await?;

                info!("creating mesh");
                let mesh = create_mesh(altitude_data, image_data)?;

                let mut raw_data = Vec::new();
                for phi in 0..360 {
                    info!("rendering mesh ({phi}/360)");
                    let image = render_mesh(
                        &mesh,
                        10.0,
                        phi as f64,
                        (cli.render_width, cli.render_height),
                        &context,
                    )?;
                    let dim = (image.width() as u16, image.height() as u16);
                    raw_data.push((image.into_raw(), dim.0, dim.1));
                }

                info!("composing gif");
                compose_gif(&mut raw_data, gif_path.as_path())?;
            } else {
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
    } else {
        info!("No peaks in batch.");
    }

    Ok(())
}
