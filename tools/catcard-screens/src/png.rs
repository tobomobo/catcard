//! The smallest PNG writer that will do: 8-bit RGB, no filtering, one IDAT.
//!
//! Compressed with `compcol`, the deflate library the firmware already links, so this tool
//! brings in no image crate.

use anyhow::{Result, anyhow};

/// `rgb` is `w * h` pixels of three bytes, row-major.
pub fn encode(w: usize, h: usize, rgb: &[u8]) -> Result<Vec<u8>> {
    assert_eq!(rgb.len(), w * h * 3);
    // Each scanline is preceded by its filter type: 0, none.
    let mut raw = Vec::with_capacity(h * (1 + w * 3));
    for row in rgb.chunks_exact(w * 3) {
        raw.push(0);
        raw.extend_from_slice(row);
    }

    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    // Bit depth 8, colour type 2 (RGB), deflate, adaptive filtering, no interlace.
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &zlib(&raw)?);
    chunk(&mut out, b"IEND", &[]);
    Ok(out)
}

fn zlib(data: &[u8]) -> Result<Vec<u8>> {
    let mut table = vec![0u16; 1 << 15];
    // Stored blocks bound the worst case: five bytes per 64 KiB block, plus the wrapper.
    let mut out = vec![0u8; data.len() + data.len() / 16 + 64];
    let n = compcol::embed::flate::zlib(
        data,
        &mut table,
        compcol::embed::flate::Buffer::new(&mut out),
    )
    .map_err(|e| anyhow!("compressing: {e:?}"))?;
    out.truncate(n as usize);
    Ok(out)
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(&[kind, data]).to_be_bytes());
}

/// CRC-32 (ISO-HDLC, reflected 0xEDB88320) over `parts` in order, as PNG specifies.
fn crc32(parts: &[&[u8]]) -> u32 {
    let mut crc = !0u32;
    for b in parts.iter().flat_map(|p| p.iter()) {
        crc ^= *b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_of_iend_is_the_well_known_value() {
        // Every PNG ends with this chunk, so its CRC is in every file: AE 42 60 82.
        assert_eq!(crc32(&[b"IEND", &[]]), 0xAE42_6082);
    }

    #[test]
    fn image_data_inflates_back_to_the_scanlines() {
        let rgb: Vec<u8> = (0..4 * 3 * 3).map(|i| i as u8).collect();
        let png = encode(4, 3, &rgb).unwrap();
        // Signature, then IHDR (8 + 13 + 4), then IDAT's length and type.
        let at = 8 + 25;
        let len = u32::from_be_bytes(png[at..at + 4].try_into().unwrap()) as usize;
        assert_eq!(&png[at + 4..at + 8], b"IDAT");
        let mut back = vec![0u8; 64];
        let n = compcol::embed::flate::unzlib(
            &png[at + 8..at + 8 + len],
            compcol::embed::flate::Buffer::new(&mut back),
        )
        .unwrap();
        let back = &back[..n as usize];
        assert_eq!(back.len(), 3 * (1 + 4 * 3));
        assert_eq!(&back[1..13], &rgb[..12]);
    }
}
