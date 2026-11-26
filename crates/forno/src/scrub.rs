//! Keeps files from telling where they were made, and with what.

use std::mem::take;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use base64::prelude::*;
use image::ImageDecoder;
use image::metadata::Orientation;
use memchr::memmem::{find, find_iter};

use crate::links::decode;
use crate::output::{File, read};
use crate::photo::decoder;

const JPEG: &[u8] = b"\xFF\xD8\xFF";
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";

/// A JPEG or a PNG loses the place, the time and the camera. Anything else has to come without them.
pub fn scrub(path: &Path) -> Result<File> {
    let bytes = read(path)?;
    if bytes.starts_with(JPEG) || bytes.starts_with(PNG) {
        return photo(&bytes).map(File::Made).with_context(|| path.display().to_string());
    }
    let how = "`exiftool -overwrite_original -all= -CommonIFD0=` or `ffmpeg -map_metadata -1`";
    ensure!(!tells(&bytes), "{} may say where it was made. Clean it, or what is in it, with {how}", path.display());
    Ok(File::Copied(path.into()))
}

fn tells(bytes: &[u8]) -> bool {
    const PACKED: &[u8] = b"brobExif|brobxml |XtRaw profile type|adobe.xmp\0\x01";
    const XMP: &[u8] = b"GPSLatitude|tiff:Model|photoshop:City|:Location";
    const FILM: &[u8] =
        b"\xA9xyz\0|loci\0\0|ISO6709|LOCATION|\xA9mod\0|.model|.MODEL|camm\0\0|djmd\0\0|gpmd\0\0|rtmd\0\0";
    let has = |bytes, signs: &[u8]| signs.split(|byte| *byte == b'|').any(|sign| find(bytes, sign).is_some());
    let from = |sign: &'static [u8]| find_iter(bytes, sign).map(|at| &bytes[at..]);
    let boxed = bytes.get(4..8).is_some_and(|first| has(first, b"free|ftyp|mdat|moov|pnot|skip|styp|wide"));
    has(bytes, PACKED)
        || has(bytes, b"<rdf:RDF") && has(bytes, XMP)
        || has(bytes, b"8BIM\x04\x04") && has(bytes, b"\x1C\x02\x5A\0|\x1C\x02\x5C\0")
        || (boxed && bytes.contains(&0) || bytes.starts_with(b"\x1A\x45\xDF\xA3")) && has(bytes, FILM)
        || from(b"MM\0*").chain(from(b"II*\0")).any(|tiff| exif(tiff) == Some(true))
        || from(b";base64,").any(|address| tells(&embedded(address)))
}

/// Whether it has a place or a camera, which that of an editor does not.
fn exif(tiff: &[u8]) -> Option<bool> {
    let number = |at: usize, size| {
        let (bytes, add) = (tiff.get(at..at + size)?.iter(), |sum, byte: &u8| sum << 8 | usize::from(*byte));
        Some(if tiff[0] == b'M' { bytes.fold(0, add) } else { bytes.rfold(0, add) })
    };
    let entries = |pointer| {
        let at = number(pointer, 4)?;
        Some((0..number(at, 2)?).map(move |entry| at + 2 + 12 * entry).take_while(|at| number(at + 2, 2) < Some(256)))
    };
    let more = entries(4)?.filter(|at| number(*at, 2) == Some(0x8769)).flat_map(|at| entries(at + 8)).flatten();
    let mut tags = entries(4)?.chain(more).filter_map(|at| number(at, 2));
    Some(tags.any(|tag| {
        matches!(tag, 0x10F | 0x110 | 0x13C | 0x83BB | 0x8825 | 0x9003..=0x9012 | 0x927C | 0x9286 | 0xA420..=0xA435)
    }))
}

fn embedded(data: &[u8]) -> Vec<u8> {
    let text = decode(&String::from_utf8_lossy(data.split(|byte| b"\"'<)".contains(byte)).next().unwrap()));
    let parts = text.split('&').map(|part| part.split_once(';').map_or(part, |(_entity, rest)| rest));
    let digits: Vec<u8> =
        parts.flat_map(str::bytes).skip(7).filter(|b| b.is_ascii_alphanumeric() || b"+/".contains(b)).collect();
    BASE64_STANDARD.decode(&digits[..digits.len() & !3]).unwrap()
}

fn photo(bytes: &[u8]) -> Result<Vec<u8>> {
    let turn = decoder(bytes)?.orientation()?;
    // All that is left of Exif is which way is up.
    let exif = [&b"MM\0*\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01\0"[..], &[turn.to_exif()], &[0; 6]].concat();
    let exif = if turn == Orientation::NoTransforms { &[] } else { &exif[..] };
    if bytes.starts_with(PNG) { png(bytes, exif) } else { jpeg(bytes, exif) }.context("the file is cut short")
}

fn jpeg(bytes: &[u8], mut exif: &[u8]) -> Option<Vec<u8>> {
    let (mut out, mut at) = (bytes[..2].to_vec(), 2);
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
                    0xE1 if !exif.is_empty() => {
                        out.extend(b"\xFF\xE1\0\x22Exif\0\0".iter().chain(take(&mut exif)));
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

fn png(bytes: &[u8], mut exif: &[u8]) -> Option<Vec<u8>> {
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
        } else if kind == b"eXIf" && !exif.is_empty() {
            let named = [kind, take(&mut exif)].concat();
            out.extend([b"\0\0\0\x1A", &named[..], &crc32fast::hash(&named).to_be_bytes()].concat());
        }
        if kind == b"IEND" {
            return Some(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use image::codecs::{jpeg::JpegEncoder, png::PngEncoder};
    use image::{ExtendedColorType, ImageEncoder};

    use super::*;
    use crate::photo::develop;

    const TURNED: &[u8] = b"MM\0*\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01\0\x06\0\0\0\0\0\0somewhere";
    const PLACED: &[u8] = b"..II*\0\x0A\0\0\0..\x01\0\x25\x88\x04\0\x01\0\0\0\x1A\0\0\0";

    fn shoot(mut camera: impl ImageEncoder) {
        camera.set_exif_metadata(TURNED.to_vec()).unwrap();
        camera.write_image(&[200; 4 * 2 * 3], 4, 2, ExtendedColorType::Rgb8).unwrap();
    }

    #[test]
    fn photos_keep_only_their_orientation() {
        let (mut jpeg, mut png) = (Vec::new(), Vec::new());
        shoot(JpegEncoder::new(&mut jpeg));
        shoot(PngEncoder::new(&mut png));
        png.splice(33..33, *b"\0\0\0\x09tEXtsomewhere\0\0\0\0");
        for shot in [jpeg, png].map(|shot| [&shot[..], b"somewhere"].concat()) {
            let clean = photo(&shot).unwrap();
            assert_eq!(find(&clean, b"somewhere"), None);
            assert_eq!(develop(&clean).unwrap(), develop(&shot).unwrap());
            assert_eq!(develop(&clean).unwrap().dimensions(), (2, 4));
            assert_eq!(photo(&clean).unwrap(), clean);
        }
    }

    #[test]
    fn places_are_found_wherever_they_are() {
        let (start, end) = (BASE64_STANDARD.encode(&PLACED[..9]), BASE64_STANDARD.encode(&PLACED[9..]));
        let drawing = format!("<image href='data:image/jpeg;base64,{start}&#10;%0A{end}'/>");
        assert!([PLACED, drawing.as_bytes(), b"\0\0\0\x08free\xA9xyz\0"].into_iter().all(tells));
        assert!(![TURNED, b"The wide $LOCATION %[GPSLatitude]", b"II*\0", b";base64,a"].into_iter().any(tells));
    }
}
