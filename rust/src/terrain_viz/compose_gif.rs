use anyhow::Result;
use gif::{Encoder, Frame, Repeat};
use rayon::prelude::*;
use std::{fs::File, path::Path};

/// Exports a list of images to a gif. 
///
/// Given a vector of images along with their respective width and height, frames are composed into a gif and saved to a file.
/// 
/// # Example
/// ```rs
/// use image::{ImageBuffer, open};
/// let img1: ImageBuffer = open("path/to/some1.png").unwrap().into_rgba8();
/// let img2: ImageBuffer = open("path/to/some2.png").unwrap().into_rgba8();
/// let img3: ImageBuffer = open("path/to/some3.png").unwrap().into_rgba8();
/// 
/// let images = vec![
///     (img1, 128, 128),
///     (img2, 128, 128),
///     (img3, 128, 128),
/// ];
/// 
/// compose_gif(images, "target.gif").unwrap();
/// ```
/// 
pub fn compose_gif(images: &mut [(Vec<u8>, u16, u16)], output_path: &Path) -> Result<()> {
    let (frames, dim): (Vec<Frame>, Vec<(u16, u16)>) = images.par_iter_mut().map(|(pixels, width, height)|{
        (Frame::from_rgba_speed(*width, *height, pixels, 10), (*width, *height))
    }).collect::<Vec<_>>().into_iter().unzip();

    let (widths, heights): (Vec<u16>, Vec<u16>)=dim.into_iter().unzip();
    let max_width =widths.iter().max().unwrap_or(&widths[0]);
    let max_height = heights.iter().max().unwrap_or(&heights[0]);
    

    let mut image_file = File::create(output_path)?;
    let mut encoder = Encoder::new(&mut image_file, *max_width, *max_height, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for frame in frames {
        encoder.write_frame(&frame)?;
    }

    Ok(())
}
