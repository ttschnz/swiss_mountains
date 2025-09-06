use anyhow::Result;
use gif::{Encoder, Frame, Repeat};
use image::GenericImageView;
use std::fs::File;

pub fn compose_gif(paths: &[String], output_path: &str, size: (u16, u16)) -> Result<()> {
    let mut image_file = File::create(output_path)?;
    let mut encoder = Encoder::new(&mut image_file, size.0, size.1, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for path in paths {
        let img = image::open(&path)?.to_rgba8();
        let mut raw_pixels = img.into_raw();
        let frame = Frame::from_rgba_speed(size.0, size.1, &mut raw_pixels, 10);
        encoder.write_frame(&frame)?;
    }

    Ok(())
}
