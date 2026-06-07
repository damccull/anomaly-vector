use std::{fs::File, path::Path};

use anyhow::Context;
use memmap2::Mmap;
use tracing::{debug, info};

use crate::error::NmsFileReadError;

pub struct NmsKeyExtractor {
    key_start: usize,
    key_offset: usize,
}

impl NmsKeyExtractor {
    // TODO: Replace this anyhow::Error with a custom lib error
    pub fn initialize<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        info!("Preparing memory map...");

        let nms_mmap = Self::get_nms_mmap(path)?;

        info!("Memory map retrieved.");
        let pe_offset = Self::get_pe_header_offset(&nms_mmap)?;
        info!("PE header offset: {}", &pe_offset);

        info!("Getting offsets for text and rdata sections...");
        let pe_header_info = Self::read_pe_header(&nms_mmap, pe_offset);

        info!("Got PE Header Info");
        debug!("PEHeaderInfo: {:#?}", &pe_header_info);

        Ok(Self {
            key_start: 0,
            key_offset: 0,
        })
    }

    pub fn key(&self) -> &[u8] {
        // Temporary to stop unused variable warnings
        let _s = self.key_start;
        let _o = self.key_offset;
        todo!()
    }
}

// Private fn implementations

impl NmsKeyExtractor {
    pub fn get_nms_mmap<P: AsRef<Path>>(path: P) -> Result<Mmap, std::io::Error> {
        let file = File::open(path)?;

        let mmap = unsafe { Mmap::map(&file)? };

        Ok(mmap)
    }

    pub fn get_pe_header_offset(map: &[u8]) -> Result<usize, NmsFileReadError> {
        // Ensure the map is large enough to contain the e_lfanew field
        if map.len() < 0x40 {
            return Err(NmsFileReadError::InvalidDosHeader(
                "file too short to contain DOS header",
            ));
        }

        // Safety check. Ensure valid DOS header with magic bytes
        if map[0..2] != [b'M', b'Z'] {
            return Err(NmsFileReadError::InvalidDosHeader(
                "missing or malformed magic bytes",
            ));
        }

        // Read the offset to the PE header, which is at 0x3c and 4 bytes long
        // Convert little endian bytes into an u32
        let pe_offset = u32::from_le_bytes([map[0x3c], map[0x3d], map[0x3e], map[0x3f]]) as usize;

        // Ensure offset fits within the map's bounds
        if pe_offset >= map.len() {
            return Err(NmsFileReadError::InvalidPeHeader(
                "file too short to contain PE header",
            ));
        }

        Ok(pe_offset)
    }

    fn read_pe_header(
        map: &[u8],
        pe_header_offset: usize,
    ) -> Result<PEHeaderInfo, NmsFileReadError> {
        // Ensure this is a PE header by checking magic bytes
        let signature_end = pe_header_offset + 4;
        if map.len() < signature_end
            || map[pe_header_offset..signature_end] != [b'P', b'E', 0x00, 0x00]
        {
            return Err(NmsFileReadError::InvalidPeHeader(
                "incorrect or malformed magic bytes",
            ));
        }

        // Get COFF offsets
        let coff_start = signature_end;
        let coff_end = coff_start + 20; // COFF is always exactly 20 bytes long

        if map.len() < coff_end {
            return Err(NmsFileReadError::InvalidCoffsHeader(
                "file too small to contain COFFS header",
            ));
        }

        let coff_slice = &map[coff_start..coff_end];

        // Read the needed sections from the COFF header
        let num_sections = u16::from_le_bytes(
            coff_slice[2..4]
                .try_into()
                .context("unable to convert slice to bytes")?,
        );
        let optional_header_length = u16::from_le_bytes(
            coff_slice[16..18]
                .try_into()
                .context("unable to convert slice to bytes")?,
        );

        // Section table starts right after the optional header. We don't need optional header
        // so we can just skip over it
        let section_table_start = coff_end + optional_header_length as usize;

        // Each section definition is 40 bytes long
        let section_table_size = (num_sections as usize) * 40;

        // Check size to avoid crash
        let section_table_end = section_table_start + section_table_size;
        if map.len() < section_table_end {
            return Err(NmsFileReadError::InvalidCoffsHeader(
                "file too small to contain entire section table",
            ));
        }

        let section_table_slice =
            &map[section_table_start..section_table_start + section_table_size];

        let coff_info = CoffHeaderInfo {
            number_sections: num_sections,
            section_table_start,
            section_table_size,
        };

        // Get the section offsets
        let mut pe_header_info = PEHeaderInfo {
            coff_header_info: coff_info,
            ..Default::default()
        };

        for i in 0..num_sections as usize {
            // Where the header for each section definition starts
            let header_start = i * 40;

            // Get the bytes representing the section name
            let section_name = &section_table_slice[header_start..header_start + 8];

            let secmap = SectionMap {
                disk_offset: u32::from_le_bytes([
                    section_table_slice[header_start + 20],
                    section_table_slice[header_start + 21],
                    section_table_slice[header_start + 22],
                    section_table_slice[header_start + 23],
                ]) as usize,
                length: u32::from_le_bytes([
                    section_table_slice[header_start + 16],
                    section_table_slice[header_start + 17],
                    section_table_slice[header_start + 18],
                    section_table_slice[header_start + 19],
                ]) as usize,
                virtual_address: u32::from_le_bytes([
                    section_table_slice[header_start + 12],
                    section_table_slice[header_start + 13],
                    section_table_slice[header_start + 14],
                    section_table_slice[header_start + 15],
                ]) as usize,
            };

            // Must include extra padding in each match arm because we're comparing against
            // 8 bytes. Otherwise the catch-all gets it and valid sections don't match
            match section_name {
                b".text\0\0\0" => {
                    pe_header_info.text_section = secmap;
                }

                b".rdata\0\0" => {
                    pe_header_info.rdata_section = secmap;
                }
                _ => {
                    debug!(
                        "unsupported section `{}` found; ignoring",
                        std::str::from_utf8(section_name)
                            .unwrap_or("unknown")
                            .trim_end_matches('\0')
                    );
                }
            }
        }

        Ok(pe_header_info)
    }
}

#[allow(unused_variables, dead_code)]
#[derive(Debug, Default)]
struct PEHeaderInfo {
    pub coff_header_info: CoffHeaderInfo,
    pub rdata_section: SectionMap,
    pub text_section: SectionMap,
}

#[allow(unused_variables, dead_code)]
#[derive(Debug, Default)]
struct CoffHeaderInfo {
    pub number_sections: u16,
    pub section_table_start: usize,
    pub section_table_size: usize,
}

#[allow(dead_code)]
enum SectionType {
    Text(SectionMap),
    Rdata(SectionMap),
}

#[allow(unused_variables, dead_code)]
#[derive(Clone, Copy, Debug, Default)]
struct SectionMap {
    pub disk_offset: usize,
    pub length: usize,
    pub virtual_address: usize,
}
