use std::path::PathBuf;

use anomaly_vector::{NmsKeyExtractor, telemetry};
use tracing::{error, info};

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

    initialize_variance()
}

#[tracing::instrument]
fn initialize_variance() -> anyhow::Result<()> {
    println!("Hello, Traveller!");
    let path = PathBuf::from(format!(
        "{}/.steam/steam/steamapps/common/No Man's Sky/Binaries/NMS.exe",
        std::env::var("HOME").unwrap()
    ));
    if path.exists() {
        info!("Found NMS.exe");
    } else {
        error!("Can't find NMS.exe");
    }

    let _extractor = NmsKeyExtractor::initialize(path)?;
    _extractor.key();

    Ok(())
}
