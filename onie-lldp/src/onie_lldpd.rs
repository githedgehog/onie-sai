mod lldp;
mod processor;
mod rpc;

use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use log::LevelFilter;

use macaddr::MacAddr6;

use crate::onie_lldpd::processor::netlink;
use crate::onie_lldpd::processor::Processor;

use ctrlc;
use std::sync::mpsc::channel;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Changes the log level setting
    #[arg(long, value_enum, default_value_t=LogLevel::Warn)]
    log_level: LogLevel,

    /// The default MAC address to use in the switch. Pass the correct MAC address from your ONIE syseeprom here.
    #[arg(long, default_value_t=MacAddr6::new(0xee, 0xba, 0x4a, 0xb9, 0xb1, 0x24))]
    mac_addr: MacAddr6,

    /// Whether to enable port auto discovery
    #[arg(long, default_value = "true", default_missing_value = "true")]
    auto_discovery: Option<Option<bool>>,

    /// When port auto discovery is enabled, also try to break out ports during the discovery
    #[arg(long, default_value = "false", default_missing_value = "true")]
    auto_discovery_with_breakout: Option<Option<bool>>,

    /// The platform to use: this should always be auto-detected.
    #[arg(long, default_value = arg_platform())]
    platform: String,

    #[arg(long, default_value = arg_init_config_file())]
    init_config_file: PathBuf,

    #[arg(long, default_value = arg_port_config_file())]
    port_config_file: PathBuf,
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

fn arg_init_config_file() -> String {
    format!("/etc/platform/{}/config.bcm", arg_platform())
}

fn arg_port_config_file() -> String {
    format!("/etc/platform/{}/port_config.json", arg_platform())
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

pub fn main() -> onie_sai_common::App {
    // parse flags and initialize logger
    // NOTE: we need to do this before we call the wrapper
    // as it will eat help and anything else otherwise
    let cli = Cli::parse();
    env_logger::builder()
        .filter_level(LevelFilter::from(cli.log_level))
        .init();

    // NOTE: wrapper will call app(). See below for details.
    onie_sai_common::App(app(cli))
}

fn app(cli: Cli) -> anyhow::Result<()> {
    // validation of some of the arguments
    if cli.platform.is_empty() {
        return Err(anyhow::anyhow!("no platform detected"));
    }

    // initialize signal handling
    let (ctrlc_tx, ctrlc_rx) = channel();
    ctrlc::set_handler(move || {
        ctrlc_tx
            .send(())
            .expect("could not send signal on termination channel.")
    })
    .context("failed to set signal handler for SIGINT, SIGTERM and SIGHUP")?;

    // this initializes the switch, and prepares the system for receiving processing requests either from RPC, or the other threads
    let proc = Processor::new().context("failed to initialize ONIE SAI processor")?;

    // move the signal handling to its own thread
    // send a shutdown request to the processor when we receive it
    let ctrlc_proc_tx = proc.get_sender();
    thread::spawn(move || {
        log::info!("ONIE SAI daemon started. Waiting for termination signal...");
        ctrlc_rx
            .recv()
            .expect("could not receive from termination channel.");
        if let Err(e) = ctrlc_proc_tx.send(processor::ProcessRequest::Shutdown) {
            log::warn!(
                "failed to send shutdown request from termination thread: {:?}",
                e
            );
        } else {
            log::info!("sent shutdown signal to ONIE SAI processor...");
        }
    });

    // initialize the ttrpc server
    let rpc_server = rpc::start_rpc_server(proc.get_sender())?;

    // initialize netlink link monitor
    let _nl_monitor = netlink::netlink_link_monitor(proc.get_sender())?;

    // initialize auto discovery poll loop
    let poll_proc_tx = proc.get_sender();
    thread::spawn(move || loop {
        // We are going to poll every second
        // NOTE: this might be too aggressive, we need to look at this again
        thread::sleep(Duration::from_secs(1));
        if let Err(e) = poll_proc_tx.send(processor::ProcessRequest::LinkPoll) {
            log::error!(
                "failed to send link poll request: {:?}. Aborting link poll thread.",
                e
            );
            return;
        }
    });

    // this blocks until processing is all done
    // process consumes the processor, so it will be dropped immediately after
    // which will trigger the cleanup
    proc.process();

    // stop ttrpc server as well
    rpc_server.shutdown();

    log::info!("Success");
    Ok(())
}
