mod swissalti3d;
mod swissimage;
mod terrain_viz;
mod utils;
use crate::terrain_viz::generate_img::{create_mesh, prepare_data, render_mesh};
use env_logger::{fmt::TimestampPrecision, Builder};
use log::info;
use three_d::*;

fn main() {
    let mut builder = Builder::new();
    builder.format_timestamp(Some(TimestampPrecision::Millis));
    builder.filter_level(log::LevelFilter::Debug);
    builder.init();

    // --- headless GL context ---
    let context = HeadlessContext::new().unwrap();

    info!("preparing data");
    let (altitude_data, image_data) = prepare_data((2616370, 1166137), 4000, 100, false).unwrap();

    info!("creating mesh");
    let mesh = create_mesh(altitude_data, image_data).unwrap();

    info!("rendering mesh");
    for phi in 0..360 {
        render_mesh(
            &mesh,
            10.0,
            phi as f64,
            &format!("frame_{}.png", phi),
            &context,
        )
        .unwrap();
    }

    info!("done");
}
