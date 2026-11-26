//! Pictures: resized for the page, and rid of what cameras write into them.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail, ensure};
use image::imageops::FilterType;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageReader, RgbaImage};
use moxcms::{ColorProfile, DataColorSpace, Layout, TransformOptions};

use crate::output::{Cache, read, read_text};

pub struct Photo {
    pub width: u32,
    pub height: u32,
    /// Its widths and files, from the smallest.
    pub prints: Vec<(u32, PathBuf)>,
}

const WIDTHS: [u32; 4] = [480, 800, 1300, 1900];

pub fn is(path: &Path, kinds: &[&str]) -> bool {
    path.extension().is_some_and(|extension| kinds.iter().any(|kind| extension.eq_ignore_ascii_case(kind)))
}

pub fn is_photo(path: &Path) -> bool {
    is(path, &["jpg", "jpeg", "png"])
}

pub fn webp(rgba: &[u8], width: u32, height: u32, quality: f32) -> Result<Vec<u8>> {
    let config = webp::WebPConfig::new().map_err(|()| anyhow!("libwebp did not start"))?;
    let config = webp::WebPConfig { quality, method: 5, alpha_quality: 90, use_sharp_yuv: 1, ..config };
    let encoded = webp::Encoder::from_rgba(rgba, width, height).encode_advanced(&config);
    Ok(encoded.map_err(|error| anyhow!("cannot encode: {error:?}"))?.to_vec())
}

fn decoder(bytes: &[u8]) -> Result<impl ImageDecoder> {
    Ok(ImageReader::new(Cursor::new(bytes)).with_guessed_format()?.into_decoder()?)
}

/// The right way up, in sRGB.
fn develop(bytes: &[u8]) -> Result<RgbaImage> {
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

fn widths(full: u32) -> impl DoubleEndedIterator<Item = u32> {
    let largest = full.min(WIDTHS[WIDTHS.len() - 1]);
    WIDTHS.into_iter().filter(move |width| width * 5 < largest * 4).chain([largest])
}

pub fn print(cache: &Cache, path: &Path) -> Result<Photo> {
    let bytes = read(path)?;
    let folder = cache.entry(&[include_bytes!("photo.rs"), &bytes], |folder| {
        let mut image = develop(&bytes).with_context(|| format!("cannot decode {}", path.display()))?;
        let (full_width, full_height) = image.dimensions();
        let quality = if bytes.starts_with(PNG) { 92.0 } else { 80.0 };
        for width in widths(full_width).rev() {
            let height = (f64::from(full_height) * f64::from(width) / f64::from(full_width)).round().max(1.0) as u32;
            image = image::imageops::resize(&image, width, height, FilterType::Lanczos3);
            fs::write(folder.join(format!("{width}.webp")), webp(&image, width, height, quality)?)?;
        }
        Ok(fs::write(folder.join("size"), format!("{full_width} {full_height}"))?)
    })?;
    let size = read_text(&folder.join("size"))?;
    let (width, height) = size.split_once(' ').context("the cache is damaged")?;
    let (width, height) = (width.parse()?, height.parse()?);
    Ok(Photo { width, height, prints: widths(width).map(|w| (w, folder.join(format!("{w}.webp")))).collect() })
}

const JPEG: &[u8] = &[0xFF, 0xD8];
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";

/// The same pixels, without the place, the time or the camera.
pub fn scrub(bytes: &[u8]) -> Result<Vec<u8>> {
    let scrubbed = if bytes.starts_with(JPEG) {
        scrub_jpeg(bytes, decoder(bytes)?.orientation()?)
    } else if bytes.starts_with(PNG) {
        scrub_png(bytes)
    } else {
        bail!("neither a JPEG nor a PNG");
    };
    scrubbed.context("the file is cut short")
}

/// Fails for a picture or a video that cannot be scrubbed, and has Exif or a location in it.
pub fn discreet(path: &Path) -> Result<()> {
    const SIGNS: [&[u8]; 7] = [
        b"MM\0*\0\0\0\x08",
        b"II*\0\x08\0\0\0",
        b"GPSLatitude",
        b"\xA9xyz\0",
        b"loci\0\0\0\0",
        b"ISO6709",
        b"LOCATION",
    ];
    if !is(path, &["avif", "heic", "heif", "m4v", "mov", "mp4", "webm", "webp"]) {
        return Ok(());
    }
    let (bytes, how) = (read(path)?, "`exiftool -all=` or `ffmpeg -map_metadata -1`");
    let clean = SIGNS.iter().all(|sign| memchr::memmem::find(&bytes, sign).is_none());
    ensure!(clean, "{} may say where it was made. Clean it with {how}", path.display());
    Ok(())
}

fn scrub_jpeg(bytes: &[u8], orientation: Orientation) -> Option<Vec<u8>> {
    let mut out = JPEG.to_vec();
    let mut turned = orientation != Orientation::NoTransforms;
    let mut at = JPEG.len();
    loop {
        match *bytes.get(at..at + 2)? {
            [0xFF, 0xFF] => at += 1,
            // Phones hide more pictures past the end.
            [0xFF, 0xD9] => break,
            [0xFF, marker] => {
                let length = u16::from_be_bytes(bytes.get(at + 2..at + 4)?.try_into().ok()?);
                let mut end = at + 2 + usize::from(length);
                let body = bytes.get(at + 4..end)?;
                let keep = match marker {
                    0xE0 => body.starts_with(b"JFIF\0"),
                    0xE2 => body.starts_with(b"ICC_PROFILE\0"),
                    0xEE => body.starts_with(b"Adobe"),
                    // All that is left of Exif is which way is up.
                    0xE1 if std::mem::take(&mut turned) => {
                        out.extend(b"\xFF\xE1\0\x22Exif\0\0MM\0\x2A\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01\0");
                        out.extend([orientation.to_exif(), 0, 0, 0, 0, 0, 0]);
                        false
                    }
                    0xE1..=0xEF | 0xFE => false,
                    // A scan runs up to the next marker.
                    0xDA => {
                        end +=
                            bytes[end..].windows(2).position(|w| w[0] == 0xFF && !matches!(w[1], 0 | 0xD0..=0xD7))?;
                        true
                    }
                    _ => true,
                };
                if keep {
                    out.extend(&bytes[at..end]);
                }
                at = end;
            }
            _ => return None,
        }
    }
    out.extend([0xFF, 0xD9]);
    Some(out)
}

fn scrub_png(bytes: &[u8]) -> Option<Vec<u8>> {
    const APPEARANCE: [&[u8]; 12] =
        [b"tRNS", b"gAMA", b"cHRM", b"sRGB", b"iCCP", b"cICP", b"sBIT", b"bKGD", b"pHYs", b"acTL", b"fcTL", b"fdAT"];
    let (mut out, mut rest) = (PNG.to_vec(), &bytes[PNG.len()..]);
    loop {
        let length = u32::from_be_bytes(rest.get(..4)?.try_into().ok()?) as usize;
        let chunk;
        (chunk, rest) = rest.split_at_checked(length + 12)?;
        let kind = &chunk[4..8];
        if kind[0].is_ascii_uppercase() || APPEARANCE.contains(&kind) {
            out.extend(chunk);
        }
        if kind == b"IEND" {
            return Some(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use image::{ImageEncoder, ImageFormat};

    use super::*;

    const EXIF: &[u8] = b"MM\0\x2A\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01\0\x06\0\0\0\0\0\0somewhere";

    fn has_secret(bytes: &[u8]) -> bool {
        bytes.windows(9).any(|w| w == b"somewhere")
    }

    #[test]
    fn jpegs_keep_only_their_orientation() {
        let mut jpeg = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut jpeg);
        encoder.set_exif_metadata(EXIF.to_vec()).unwrap();
        encoder.write_image(&[200; 4 * 2 * 3], 4, 2, image::ExtendedColorType::Rgb8).unwrap();
        jpeg.extend(b"somewhere");
        let clean = scrub(&jpeg).unwrap();
        assert!(has_secret(&jpeg) && !has_secret(&clean));
        assert_eq!(develop(&clean).unwrap(), develop(&jpeg).unwrap());
        assert_eq!(develop(&clean).unwrap().dimensions(), (2, 4));
        assert_eq!(scrub(&clean).unwrap(), clean);
    }

    #[test]
    fn pngs_lose_their_text() {
        let mut png = Cursor::new(Vec::new());
        RgbaImage::new(2, 2).write_to(&mut png, ImageFormat::Png).unwrap();
        let plain = png.into_inner();
        let (head, tail) = plain.split_at(33);
        let png = [head, b"\0\0\0\x09tEXtsomewhere\0\0\0\0", tail].concat();
        assert_eq!(scrub(&png).unwrap(), plain);
    }

    #[test]
    fn nothing_is_enlarged() {
        assert!(widths(4032).eq([480, 800, 1300, 1900]));
        assert!(widths(900).eq([480, 900]));
        assert!(widths(300).eq([300]));
    }
}
