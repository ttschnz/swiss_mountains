use anyhow::{Error, Result};
use gif::{Encoder, Frame, Repeat};
use rayon::prelude::*;
use std::fs::File;

pub fn compose_gif(paths: &[String], output_path: &str, size: (u16, u16)) -> Result<()> {

    let frames = paths.par_iter().map(|path| {
        let img = image::open(&path)?.to_rgba8();
        let mut raw_pixels = img.into_raw();
        Ok::<Frame<'_>, Error>(Frame::from_rgba_speed(size.0, size.1, &mut raw_pixels, 10))
    }).collect::<Vec<_>>();

    let mut image_file = File::create(output_path)?;
    let mut encoder = Encoder::new(&mut image_file, size.0, size.1, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for frame in frames{
        encoder.write_frame(&frame?)?;
    }

    Ok(())
}
