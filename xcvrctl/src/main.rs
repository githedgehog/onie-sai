use anyhow::Context;
use clap::{Parser, ValueEnum};
use log::LevelFilter;

use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::process::ExitCode;
use std::process::Termination;
use std::sync::OnceLock;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Changes the log level setting
    #[arg(long, value_enum, default_value_t=LogLevel::Debug)]
    log_level: LogLevel,

    #[arg(long, default_value = arg_platform())]
    platform: String,
}

static PLATFORM: OnceLock<String> = OnceLock::new();

fn arg_platform() -> String {
    PLATFORM
        .get_or_init(|| {
            // check if the environment variable is set first
            if let Ok(v) = env::var("onie_platform") {
                return v;
            }

            // if not, then we are going to parse /etc/machine.conf
            if let Ok(file) = File::open("/etc/machine.conf") {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if let Some((k, v)) = line.split_once("=") {
                            if k.trim() == "onie_platform" {
                                return v.trim().to_string();
                            }
                        }
                    }
                }
            }

            // if we are here, then we could not determine the platform
            String::new()
        })
        .to_string()
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

struct App(anyhow::Result<()>);

impl Termination for App {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                log::error!("Unrecoverable application error: {:?}. Exiting...", e);
                ExitCode::FAILURE
            }
        }
    }
}

fn main() -> App {
    App(app())
}

fn app() -> anyhow::Result<()> {
    // parse flags and initialize logger
    let cli = Cli::parse();
    env_logger::builder()
        .filter_level(LevelFilter::from(cli.log_level))
        .init();

    // validation of some of the arguments
    if cli.platform.is_empty() {
        return Err(anyhow::anyhow!("no platform detected"));
    }

    // load our platform specific library
    let platform_lib_path = format!("/usr/lib/platform/{}.so", cli.platform);
    let platform_lib_loader = xcvr::LibraryLoader::new(Path::new(platform_lib_path.as_str()))
        .map_err(|e| {
            log::error!(
                "platform library {}: failed to load: {}",
                platform_lib_path,
                e
            )
        })
        .ok();
    let platform_lib_lib = match platform_lib_loader.as_ref() {
        Some(l) => l
            .lib()
            .map_err(|e| {
                log::error!(
                    "platform library {}: failed to initialize: {}",
                    platform_lib_path,
                    e
                )
            })
            .ok(),
        None => None,
    };
    let platform_lib_ctx = match platform_lib_lib.as_ref() {
        Some(l) => l
            .platform_lib(&cli.platform)
            .map_err(|e| {
                log::error!(
                    "platform library {}: failed to return context: {}",
                    platform_lib_path,
                    e
                )
            })
            .ok(),
        None => None,
    };

    // now we are either using the platform specific library or the fallback
    let platform_ctx: Box<dyn xcvr::PlatformContext> = match platform_lib_ctx {
        Some(l) => Box::new(l),
        None => {
            log::warn!("platform library: using fallback implementation");
            Box::new(xcvr::FallbackPlatformLibrary {})
        }
    };

    let num_ports = platform_ctx
        .num_physical_ports()
        .context("failed to get number of physical ports")?;

    for idx in 0..num_ports {
        let supported_port_types = platform_ctx
            .get_supported_port_types(idx)
            .context("failed to get supported port types")?;
        let present = platform_ctx
            .get_presence(idx)
            .context("failed to get port presence")?;

        let oper_status = if present {
            Some(
                platform_ctx
                    .get_oper_status(idx)
                    .context("failed to get port operational status")?,
            )
        } else {
            None
        };

        let reset_status = if present {
            Some(
                platform_ctx
                    .get_reset_status(idx)
                    .context("failed to get port reset status")?,
            )
        } else {
            None
        };

        let low_power_mode = if present {
            Some(
                platform_ctx
                    .get_low_power_mode(idx)
                    .context("failed to get port low power mode")?,
            )
        } else {
            None
        };

        // simply log it
        log::info!(
            "port {}: present: {}, supported port types: {:?}, oper status: {:?}, reset status: {:?}, low power mode: {:?}",
            idx,
            present,
            supported_port_types,
            oper_status,
            reset_status,
            low_power_mode,
        );
    }
    Ok(())
}
