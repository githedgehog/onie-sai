mod oniesai;
mod rpc;

use std::ffi::CString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use log::LevelFilter;

use macaddr::MacAddr6;

use sai::SAI;

use crate::oniesai::Processor;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Changes the log level setting
    #[arg(long, value_enum, default_value_t=LogLevel::Warn)]
    log_level: LogLevel,

    #[arg(long, default_value_t=MacAddr6::new(0xee, 0xba, 0x4a, 0xb9, 0xb1, 0x24))]
    mac_addr: MacAddr6,

    #[arg(long, default_value = "/root/saictl/etc/config.bcm")]
    init_config_file: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum LogLevel {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    env_logger::builder()
        .filter_level(LevelFilter::from(cli.log_level))
        .init();

    // get SAI API version
    if let Ok(version) = SAI::api_version() {
        log::info!("SAI version: {}", version);
    }

    // our profile
    let profile = vec![(
        CString::from_vec_with_nul(sai::SAI_KEY_INIT_CONFIG_FILE.to_vec()).unwrap(),
        CString::new("/root/saictl/etc/config.bcm").unwrap(),
    )];

    // init SAI
    let sai_api = match SAI::new(profile) {
        Ok(sai_api) => sai_api,
        Err(e) => {
            log::error!("failed to initialize SAI: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    log::info!("successfully initialized SAI");

    if let Err(e) = SAI::log_set_all(sai::LogLevel::Info) {
        log::error!("failed to set log level for all APIs: {:?}", e);
    }

    let _proc = match Processor::new(&sai_api, cli.mac_addr.into_array()) {
        Ok(proc) => proc,
        Err(e) => {
            log::error!("failed to initialize ONIE SAI processor: {}", e);
            return ExitCode::FAILURE;
        }
    };

    log::info!("Success");

    return ExitCode::SUCCESS;
}
