mod swissalti3d;
mod swissimage;
mod terrain_viz;
mod utils;
use crate::terrain_viz::{compose_gif, create_mesh, prepare_data, render_mesh};
use anyhow::Result;
use env_logger::{fmt::TimestampPrecision, Builder};
use log::info;
use three_d::*;

#[tokio::main]
async fn main() -> Result<()> {
    // --- Initialise logging
    let render_size = (300u16, 300u16);
    let mut builder = Builder::new();
    builder.format_timestamp(Some(TimestampPrecision::Millis));
    builder.filter_level(log::LevelFilter::Debug);
    builder.init();

    // --- headless GL context: singleton, cannot be created multiple times per program ---
    let context = HeadlessContext::new()?;

    info!("preparing data");
    let (altitude_data, image_data) = prepare_data(
        (2616370, 1166137), // coordinates of niesen
        4000,               // radius covered
        500,                // sampling size width
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
    compose_gif(&mut raw_data, "niesen.gif")?;

    info!("done");

    Ok(())
}
