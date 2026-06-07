use std::path::PathBuf;

use anomaly_vector::{get_nms_mmap, get_pe_header_offset, get_section_offsets, telemetry};
use tracing_subscriber::fmt::writer::MakeWriterExt;

fn main() -> anyhow::Result<()> {
    //Configure log levels to channels
    let stdout_sink = std::io::stdout.with_max_level(tracing::Level::INFO);
    let stderr_sink = std::io::stderr.with_min_level(tracing::Level::WARN);

    telemetry::init_subscriber(
        "anomaly_vector".to_owned(),
        "info".to_owned(),
        stdout_sink.and(stderr_sink),
        None,
    );
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
