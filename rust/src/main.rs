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
    let render_size = (3000u16, 3000u16);
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

    info!("rendering mesh");
    let mut pngs = Vec::new();
    for phi in 0..360 {
        let path = format!("frames/frame_{}.png", phi);
        render_mesh(&mesh, 10.0, phi as f64, &path, render_size, &context)?;
        pngs.push(path);
    }

    info!("composing gif");
    compose_gif(&pngs, "niesen.gif", render_size)?;

    info!("done");

    Ok(())
}
