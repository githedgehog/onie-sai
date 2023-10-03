mod oniesai;
mod rpc;

use std::ffi::CString;
use std::path::PathBuf;
use std::process::ExitCode;
use std::process::Termination;
use std::thread;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use log::LevelFilter;

use macaddr::MacAddr6;

use sai::SAI;

use crate::oniesai::Processor;

use ctrlc;
use std::sync::mpsc::channel;

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

impl Cli {
    fn sai_profile(&self) -> anyhow::Result<Vec<(CString, CString)>> {
        let init_config_file =
            self.init_config_file
                .as_os_str()
                .to_str()
                .ok_or(anyhow::anyhow!(
                    "init config file is not a valid unicode string"
                ))?;

        Ok(vec![(
            CString::from_vec_with_nul(sai::SAI_KEY_INIT_CONFIG_FILE.to_vec())?,
            CString::new(init_config_file)?,
        )])
    }
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

    // initialize signal handling
    let (ctrlc_tx, ctrlc_rx) = channel();
    ctrlc::set_handler(move || {
        ctrlc_tx
            .send(())
            .expect("could not send signal on termination channel.")
    })
    .context("failed to set signal handler for SIGINT, SIGTERM and SIGHUP")?;

    // get SAI API version
    if let Ok(version) = SAI::api_version() {
        log::info!("SAI version: {}", version);
    }

    // construct our profile from the CLI arguments and initialize SAI
    let profile = cli.sai_profile()?;
    let sai_api = SAI::new(profile).context("failed to initialize SAI")?;
    log::info!("successfully initialized SAI");

    if let Err(e) = SAI::log_set_all(sai::LogLevel::Info) {
        log::error!("failed to set log level for all APIs: {:?}", e);
    }

    // this initializes the switch, and prepares the system for receiving processing requests either from RPC, or the other threads
    let proc = Processor::new(&sai_api, cli.mac_addr.into_array())
        .context("failed to initialize ONIE SAI processor")?;

    // move the signal handling to its own thread
    // send a shutdown request to the processor when we receive it
    let ctrlc_proc_tx = proc.get_sender();
    thread::spawn(move || {
        log::info!("ONIE SAI daemon started. Waiting for termination signal...");
        ctrlc_rx
            .recv()
            .expect("could not receive from termination channel.");
        if let Err(e) = ctrlc_proc_tx.send(oniesai::ProcessRequest::Shutdown) {
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

    // this blocks until processing is all done
    // process consumes the processor, so it will be dropped immediately after
    // which will trigger the cleanup
    proc.process();

    // stop ttrpc server as well
    rpc_server.shutdown();

    log::info!("Success");
    Ok(())
}
