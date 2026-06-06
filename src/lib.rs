pub mod telemetry;

use std::{fs::File, path::Path};

use memmap2::Mmap;

pub fn get_nms_mmap<P: AsRef<Path>>(path: P) -> Result<Mmap, std::io::Error> {
    let file = File::open(path)?;

    let mmap = unsafe { Mmap::map(&file)? };

    Ok(mmap)
}

pub fn get_pe_header_offset(map: &[u8]) -> anyhow::Result<u32> {
    // Ensure the map is large enough to contain the e_lfanew field
    if map.len() < 0x40 {
        anyhow::bail!("file too small to contain DOS header");
    }

    // Safety check. Ensure valid DOS header with magic bytes
    if map[0..2] != [b'M', b'Z'] {
        anyhow::bail!("not a valid DOS header; magic bytes 'MZ' missing");
    }

    // Read the offset to the PE header, which is at 0x3c and 4 bytes long
    // Convert little endian bytes into an u32
    let pe_offset = u32::from_le_bytes([map[0x3c], map[0x3d], map[0x3e], map[0x3f]]);

    // Ensure offset fits within the map's bounds
    if pe_offset as usize >= map.len() {
        anyhow::bail!("PE header outside bounds of the file bounds");
    }

    Ok(pe_offset)
}

pub struct Offsets {
    _text_start: i32,
    _text_length: i32,
    _rdata_start: i32,
    _rdata_length: i32,
}
