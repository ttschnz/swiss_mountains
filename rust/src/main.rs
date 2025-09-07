mod swissalti3d;
mod swissimage;
mod terrain_viz;
mod utils;
use crate::terrain_viz::{compose_gif, create_mesh, prepare_data, render_mesh};
use env_logger::{fmt::TimestampPrecision, Builder};
use log::info;
use three_d::*;

#[tokio::main]
async fn main() {
    // --- Initialise logging
    let render_size = (1024u16, 1024u16);
    let mut builder = Builder::new();
    builder.format_timestamp(Some(TimestampPrecision::Millis));
    builder.filter_level(log::LevelFilter::Debug);
    builder.init();

    // --- headless GL context: singleton, cannot be created multiple times per program ---
    let context = HeadlessContext::new().unwrap();

    info!("preparing data");
    let (altitude_data, image_data) = prepare_data(
        (2616370, 1166137), // Niesen
        //(2664198, 1171605),
        4000,
        render_size.0.max(render_size.1),
        false,
    )
    .await
    .unwrap();

    info!("creating mesh");
    let mesh = create_mesh(altitude_data, image_data).unwrap();

    info!("rendering mesh");
    let mut pngs = Vec::new();
    for phi in 0..360 {
        let path = format!("frames/frame_{}.png", phi);
        render_mesh(&mesh, 10.0, phi as f64, &path, render_size, &context).unwrap();
        pngs.push(path);
    }

    info!("composing gif");
    compose_gif(&pngs, "niesen.gif", render_size).unwrap();

    info!("done");
}
