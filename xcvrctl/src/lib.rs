use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use log::LevelFilter;
use xcvr::idx_t;

use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::process::ExitCode;
use std::process::Termination;
use std::rc::Rc;
use std::sync::OnceLock;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Changes the log level setting
    #[arg(long, value_enum, default_value_t=LogLevel::Info)]
    log_level: LogLevel,

    #[arg(long, default_value = arg_platform())]
    platform: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Operate on or get info on port
    Port {
        /// Port number (zero based index)
        index: Option<idx_t>,

        #[command(subcommand)]
        command: PortCommands,
    },

    /// List all ports
    List,
}

#[derive(Subcommand)]
enum PortCommands {
    /// get all port information
    Get,

    /// Reset transceiver
    Reset,

    /// Set low power mode on/off
    SetLowPowerMode {
        /// enable or disable lower power mode of transceiver
        enable: bool,
    },
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

pub fn main() -> onie_sai_common::App {
    onie_sai_common::App(app())
}

#[derive(Clone)]
pub(crate) struct PlatformContextHolder<'a> {
    obj: Rc<dyn xcvr::PlatformContext + 'a>,
}

impl<'a> PlatformContextHolder<'a> {
    pub(crate) fn new<T: xcvr::PlatformContext + 'a>(object: T) -> Self {
        Self {
            obj: Rc::new(object),
        }
    }
}

impl std::fmt::Debug for PlatformContextHolder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PlatformContextHolder")
    }
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
    let platform_ctx: PlatformContextHolder = match platform_lib_ctx {
        Some(l) => PlatformContextHolder::new(l),
        None => {
            log::warn!("platform library: using fallback implementation");
            PlatformContextHolder::new(xcvr::FallbackPlatformLibrary {})
        }
    };

    match cli.command {
        Commands::Port { index, command } => match index {
            Some(index) => match command {
                PortCommands::Get => get_port(&platform_ctx, index),
                PortCommands::Reset => reset_port(platform_ctx, index)?,
                PortCommands::SetLowPowerMode { enable } => {
                    set_low_power_mode(platform_ctx, index, enable)?
                }
            },
            None => {
                log::error!("port: no port index specified");
                return Err(anyhow::anyhow!("port: no port index specified"));
            }
        },
        Commands::List => list_ports(&platform_ctx)?,
    }
    Ok(())
}

fn get_port(platform_ctx: &PlatformContextHolder, index: idx_t) {
    let supported_port_types = match platform_ctx.obj.get_supported_port_types(index) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("port {}: failed to get supported port types: {}", index, e);
            Vec::new()
        }
    };
    let present = match platform_ctx.obj.get_presence(index) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("port {}: failed to detect port presence: {}. Assuming port is present like in fallback implementation.", index, e);
            true
        }
    };

    let oper_status = if present {
        match platform_ctx.obj.get_oper_status(index) {
            Ok(v) => Some(v),
            Err(e) => {
                log::warn!("port {}: failed to get oper status: {}", index, e);
                None
            }
        }
    } else {
        None
    };

    let reset_status = if present {
        match platform_ctx.obj.get_reset_status(index) {
            Ok(v) => Some(v),
            Err(e) => {
                log::warn!("port {}: failed to get reset status: {}", index, e);
                None
            }
        }
    } else {
        None
    };

    let low_power_mode = if present {
        match platform_ctx.obj.get_low_power_mode(index) {
            Ok(v) => Some(v),
            Err(e) => {
                log::warn!("port {}: failed to get low power mode: {}", index, e);
                None
            }
        }
    } else {
        None
    };

    let inserted_port_type = if present {
        match platform_ctx.obj.get_inserted_port_type(index) {
            Ok(t) => Some(t),
            Err(e) => {
                log::warn!("port {}: failed to get inserted port type: {}", index, e);
                None
            }
        }
    } else {
        None
    };

    // simply print it to stdout
    println!(
        "port {}: present: {}, supported port types: {:?}, inserted port type: {:?}, oper status: {:?}, reset status: {:?}, low power mode: {:?}",
        index,
        present,
        supported_port_types,
        inserted_port_type,
        oper_status,
        reset_status,
        low_power_mode,
    );
}

fn list_ports(platform_ctx: &PlatformContextHolder) -> anyhow::Result<()> {
    let num_ports = platform_ctx
        .obj
        .num_physical_ports()
        .context("failed to get number of physical ports")?;

    for idx in 0..num_ports {
        get_port(platform_ctx, idx);
    }
    Ok(())
}

fn reset_port(platform_ctx: PlatformContextHolder, index: idx_t) -> anyhow::Result<()> {
    platform_ctx
        .obj
        .reset(index)
        .context("failed to reset port")?;
    Ok(())
}

fn set_low_power_mode(
    platform_ctx: PlatformContextHolder,
    index: idx_t,
    enable: bool,
) -> anyhow::Result<()> {
    platform_ctx
        .obj
        .set_low_power_mode(index, enable)
        .context("failed to set low power mode")?;
    Ok(())
}
