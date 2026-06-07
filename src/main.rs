use std::path::PathBuf;

use anomaly_vector::{get_nms_mmap, get_pe_header_offset, read_pe_header, telemetry};
use tracing::{debug, error, info};

fn main() -> anyhow::Result<()> {
    //Configure log levels to channels
    // let stdout_sink = std::io::stdout.with_min_level(tracing::Level::INFO);
    // let stderr_sink = std::io::stderr.with_max_level(tracing::Level::WARN);

    telemetry::init_subscriber(
        "anomaly_vector".to_owned(),
        "info".to_owned(),
        std::io::stderr,
        None,
    );

    ready_go()
}

#[tracing::instrument]
fn ready_go() -> anyhow::Result<()> {
    println!("Hello, Traveller!");
    let path =
        PathBuf::from(format!("{}/.steam/steam/steamapps/common/No Man's Sky/Binaries/NMS.exe", std::env::var("HOME").unwrap()));
    if path.exists() {
        info!("Found NMS.exe");
    } else {
        error!("Can't find NMS.exe");
    }

    info!("Preparing memory map...");

    let nms_mmap = get_nms_mmap(path)?;

    info!("Memory map retrieved.");
    let pe_offset = get_pe_header_offset(&nms_mmap)?;
    info!("PE header offset: {}", &pe_offset);

    info!("Getting offsets for text and data sections...");
    let pe_header_info = read_pe_header(&nms_mmap, pe_offset);

    info!("Got PE Header Info");
    debug!("PEHeaderInfo: {:#?}", &pe_header_info);

    Ok(())
}
