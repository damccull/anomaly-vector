use std::path::PathBuf;

use anomaly_vector::{get_nms_mmap, get_pe_header_offset, get_section_offsets};

fn main() -> anyhow::Result<()> {
    println!("Hello, Traveller!");
    let path =
        PathBuf::from(format!("{}/.steam/steam/steamapps/common/No Man's Sky/Binaries/NMS.exe", std::env::var("HOME").unwrap()));
    if path.exists() {
        println!("Found NMS.exe");
    } else {
        eprintln!("Can't find NMS.exe");
    }

    println!("Preparing memory map...");

    let nms_mmap = get_nms_mmap(path)?;

    println!("Memory map retrieved.");
    let pe_offset = get_pe_header_offset(&nms_mmap)?;
    println!("PE header offset: {}", &pe_offset);

    Ok(())
}
