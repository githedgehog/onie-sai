use onie_sai_rpc::onie_sai;
use onie_sai_rpc::onie_sai_ttrpc;
use onie_sai_rpc::SOCK_ADDR;
use ttrpc::context::{self, Context};
use ttrpc::Client;

use std::process::ExitCode;
use std::process::Termination;

use anyhow::Context as AnyhowContext;
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::LevelFilter;

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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Changes the log level setting
    #[arg(long, value_enum, default_value_t=LogLevel::Warn)]
    log_level: LogLevel,

    /// Unix socket address to onie-said
    #[arg(long, short, default_value = SOCK_ADDR)]
    address: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// version information about onie-said, SAI, etc.pp.
    Version,

    /// lists all ports and their details
    Ports,

    /// gets auto-discovery status of onie-said
    /// or you can enable/disable it within onie-said
    AutoDiscovery(AutoDiscoveryArgs),

    /// runs the SAI vendor shell
    Shell,
}

#[derive(Args)]
struct AutoDiscoveryArgs {
    enable: Option<bool>,
}

struct App(anyhow::Result<()>);

impl Termination for App {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                log::error!("Unrecoverable application error. Exiting... ERROR: {:?}", e);
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

    // connect to onie-said
    // if this fails, abort immediately
    log::info!("connecting to onie-said at: {}...", &cli.address);
    let c = Client::connect(&cli.address).context(format!(
        "failed to connect to onie-said at {}",
        &cli.address
    ))?;
    let osc = onie_sai_ttrpc::OnieSaiClient::new(c);

    match cli.command {
        Commands::Version => {
            let req = onie_sai::VersionRequest::new();
            log::info!("making request to onie-said: {:?}...", req);
            let resp = osc
                .version(default_ctx(), &req)
                .context("request to onie-said failed")?;
            log::info!("response from onie-said: {:?}", resp);
            println!("onie-saictl version: {}", env!("CARGO_PKG_VERSION"));
            println!("onie-said version: {}", resp.onie_said_version);
            println!("SAI version: {}", resp.sai_version);
        }
        Commands::Ports => {
            let req = onie_sai::PortListRequest::new();
            log::info!("making request to onie-said: {:?}...", req);
            let resp = osc
                .port_list(default_ctx(), &req)
                .context("request to onie-said failed")?;
            log::info!("response from onie-said: {:?}", resp);
        }
        Commands::AutoDiscovery(v) => {
            log::info!("auto discovery args: {:?}", v.enable);
            let req = onie_sai::AutoDiscoveryRequest {
                enable: v.enable,
                ..Default::default()
            };
            log::info!("making request to onie-said: {:?}...", req);
            let resp = osc
                .auto_discovery(default_ctx(), &req)
                .context("request to onie-said failed")?;
            log::info!("response from onie-said: {:?}", resp);
            println!(
                "onie-said: auto-discovery is {}",
                if resp.enabled { "on" } else { "off" }
            );
        }
        Commands::Shell => {
            let req = onie_sai::ShellRequest::new();
            log::info!("making request to onie-said: {:?}...", req);
            let resp = osc
                .shell(default_ctx(), &req)
                .context("request to onie-said failed")?;
            log::info!("response from onie-said: {:?}", resp);
        }
    }

    log::info!("Success");
    Ok(())
}

fn default_ctx() -> Context {
    let mut ctx = context::with_timeout(0);
    ctx.add(
        "user-agent".to_string(),
        format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
    );

    ctx
}
