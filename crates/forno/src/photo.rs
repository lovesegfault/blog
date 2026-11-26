//! Pictures, measured and resized for the page.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use image::imageops::{FilterType, resize};
use image::{DynamicImage, ImageDecoder, ImageReader, Rgba, RgbaImage};
use imagesize::{Compression, ImageSize, ImageType};
use memchr::memmem::find;
use moxcms::{ColorProfile, DataColorSpace, Layout, TransformOptions};
use typst::foundations::Bytes;
use typst::visualize::SvgImage;

use crate::output::{Cache, read, read_text};

const WIDTHS: [u32; 4] = [480, 800, 1300, 1900];

/// The kind and the size of a picture that browsers can show.
pub fn measure(extension: Option<&str>, bytes: &Bytes) -> Option<(&'static str, [f64; 2])> {
    use ImageType::*;
    if extension.is_some_and(|extension| extension.eq_ignore_ascii_case("svg")) && find(bytes, b"<svg").is_some() {
        let kind = if find(bytes, b"currentColor").is_some() { "themed" } else { "lines" };
        return SvgImage::new(bytes.clone()).ok().map(|svg| (kind, [svg.width(), svg.height()]));
    }
    let kind = match imagesize::image_type(bytes).ok()? {
        Jpeg => "jpeg",
        Png if find(bytes, b"\0\0\0\x08acTL").is_none() => "png",
        Bmp | Gif | Heif(Compression::Av1) | Ico | Png | Webp => "pixels",
        _ => return None,
    };
    let ImageSize { width, height } = imagesize::blob_size(bytes).ok()?;
    let turned = kind != "pixels" && decoder(bytes).ok()?.orientation().ok()?.to_exif() > 4;
    Some((kind, if turned { [height, width] } else { [width, height] }.map(|side| side as f64)))
}

pub fn webp(rgba: &[u8], width: u32, height: u32, quality: f32) -> Result<Vec<u8>> {
    let config = webp::WebPConfig::new().map_err(|()| anyhow!("libwebp did not start"))?;
    let config = webp::WebPConfig { quality, method: 5, alpha_quality: 90, use_sharp_yuv: 1, ..config };
    let encoded = webp::Encoder::from_rgba(rgba, width, height).encode_advanced(&config);
    Ok(encoded.map_err(|error| anyhow!("cannot encode: {error:?}"))?.to_vec())
}

pub fn decoder(bytes: &[u8]) -> Result<impl ImageDecoder> {
    Ok(ImageReader::new(Cursor::new(bytes)).with_guessed_format()?.into_decoder()?)
}

/// The right way up, in sRGB.
pub fn develop(bytes: &[u8]) -> Result<RgbaImage> {
    let mut decoder = decoder(bytes)?;
    let (orientation, profile) = (decoder.orientation()?, decoder.icc_profile()?);
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    let mut image = image.into_rgba8();
    let profile = profile.map(|profile| ColorProfile::new_from_slice(&profile)).transpose()?;
    if let Some(profile) = profile.filter(|profile| profile.color_space == DataColorSpace::Rgb) {
        let srgb = ColorProfile::new_srgb();
        let to_srgb = profile.create_transform_8bit(Layout::Rgba, &srgb, Layout::Rgba, TransformOptions::default())?;
        to_srgb.transform(&image.clone(), &mut image)?;
    }
    Ok(image)
}

/// Colours by their opacity, or resizing would bring out those that cannot be seen.
pub fn weigh(mut image: RgbaImage, by: fn(f32) -> f32) -> RgbaImage {
    for Rgba([colour @ .., alpha]) in image.pixels_mut().filter(|pixel| pixel[3] < 255) {
        *colour = colour.map(|channel| (f32::from(channel) * by(f32::from(*alpha) / 255.0)).round() as u8);
    }
    image
}

fn widths(full: u32) -> impl DoubleEndedIterator<Item = u32> {
    let largest = full.min(WIDTHS[WIDTHS.len() - 1]);
    WIDTHS.into_iter().filter(move |width| width * 5 < largest * 4).chain([largest])
}

/// The widths of a photo and their files, from the smallest.
pub fn print(cache: &Cache, path: &Path) -> Result<Vec<(u32, PathBuf)>> {
    let bytes = read(path)?;
    let folder = cache.entry(&[include_bytes!("photo.rs"), &bytes], |folder| {
        let image = develop(&bytes).with_context(|| format!("cannot decode {}", path.display()))?;
        let ((full_width, full_height), mut image) = (image.dimensions(), weigh(image, |alpha| alpha));
        let quality = if bytes[1..].starts_with(b"PNG") { 92.0 } else { 80.0 };
        for width in widths(full_width).rev() {
            let height = (f64::from(full_height) * f64::from(width) / f64::from(full_width)).round().max(1.0) as u32;
            image = resize(&image, width, height, FilterType::Lanczos3);
            let print = webp(&weigh(image.clone(), f32::recip), width, height, quality)?;
            fs::write(folder.join(format!("{width}.webp")), print)?;
        }
        Ok(fs::write(folder.join("width"), full_width.to_string())?)
    })?;
    let width = read_text(&folder.join("width"))?.parse()?;
    Ok(widths(width).map(|width| (width, folder.join(format!("{width}.webp")))).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_enlarged() {
        assert!(widths(4032).eq([480, 800, 1300, 1900]));
        assert!(widths(900).eq([480, 900]));
        assert!(widths(300).eq([300]));
    }
}
