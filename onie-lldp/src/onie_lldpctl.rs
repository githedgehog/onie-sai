use onie_sai_rpc::onie_sai;
use onie_sai_rpc::onie_sai_ttrpc;
use onie_sai_rpc::onie_sai_ttrpc::OnieSaiClient;
use onie_sai_rpc::SOCK_ADDR;
use ttrpc::context::{self, Context};
use ttrpc::Client;

use std::thread;
use std::time::Duration;

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

    /// shuts down onie-said (equals sending a SIGTERM to the process)
    Shutdown,

    /// Wait on initial discovery to complete. This is meant for init scripts which need to wait until the initial discovery is complete
    /// before they can consider the service started up.
    /// NOTE: For onie-lldpd and onie-lldpctl these are no-ops, and are just here for compatibility with onie-said init scripts.
    WaitOnInitialDiscovery,

    /// Information about LLDP on a given device.
    LLDP(LLDPArgs),

    /// Retrieves the network configuration for a given device as received over LLDP over the interface.
    /// NOTE: This command is specific to the Hedgehog Fabric implementation of LLDP packets that are sent from SONiC switches and Hedgehog Fabric control nodes.
    LLDPNetworkConfig(LLDPNetworkConfigArgs),
}

#[derive(Args)]
struct AutoDiscoveryArgs {
    enable: Option<bool>,
}

#[derive(Args)]
struct LLDPArgs {
    device: String,
}

#[derive(Args)]
struct LLDPNetworkConfigArgs {
    device: String,

    #[arg(long, short)]
    wait_secs: Option<u32>,
}

pub fn main() -> onie_sai_common::App {
    onie_sai_common::App(app())
}

fn connect(addr: &str) -> anyhow::Result<OnieSaiClient> {
    log::info!("connecting to onie-said at: {}...", &addr);
    let c =
        Client::connect(&addr).context(format!("failed to connect to onie-said at {}", &addr))?;
    Ok(onie_sai_ttrpc::OnieSaiClient::new(c))
}

fn app() -> anyhow::Result<()> {
    // parse flags and initialize logger
    let cli = Cli::parse();
    env_logger::builder()
        .filter_level(LevelFilter::from(cli.log_level))
        .init();

    match cli.command {
        Commands::Version => {
            let osc = connect(&cli.address)?;
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
        Commands::Shutdown => {
            let osc = connect(&cli.address)?;
            let req = onie_sai::ShutdownRequest::new();
            log::info!("making request to onie-said: {:?}...", req);
            let resp = osc
                .shutdown(default_ctx(), &req)
                .context("request to onie-said failed")?;
            log::info!("response from onie-said: {:?}", resp);
        }
        Commands::WaitOnInitialDiscovery => {
            // we'll give this command up to 60 seconds to connect to the ttrpc server
            // because this is executed right after we start onie-said from init scripts
            // we need to wait until the ttrpc server is up and running
            let mut i = 0u32;
            let osc = loop {
                match connect(&cli.address) {
                    Ok(osc) => break osc,
                    Err(e) => {
                        log::debug!("failed to connect to onie-said (connect count {i}): {e:?}");
                    }
                }
                i += 1;
                if i == 60 {
                    return Err(anyhow::anyhow!("failed to connect to onie-said after 60 connection attempts with 1 second delay in between"));
                }
                thread::sleep(Duration::from_millis(1000));
            };
            // once we are connected, we can poll until the initial discovery is finished
            // this is deterministic, so we know it will complete
            // if it does not, then this is a bug in onie-said
            loop {
                let req = onie_sai::IsInitialDiscoveryFinishedRequest::new();
                log::info!("making request to onie-said: {:?}...", req);
                let resp = osc
                    .is_initial_discovery_finished(default_ctx(), &req)
                    .context("request to onie-said failed")?;
                log::info!("response from onie-said: {:?}", resp);
                if resp.is_finished {
                    log::info!("initial discovery finished");
                    break;
                }
                thread::sleep(Duration::from_millis(1000));
            }
        }
        Commands::LLDP(args) => {
            let osc = connect(&cli.address)?;
            let req = onie_sai::LLDPStatusRequest {
                device: args.device,
                ..Default::default()
            };
            log::info!("making request to onie-said: {:?}...", req);
            let resp = osc
                .lldp_status(default_ctx(), &req)
                .context("request to onie-said failed")?;
            log::info!("response from onie-said: {:?}", resp);
            println!("LLDP packet received: {}", resp.packet_received);
            for tlv in resp.tlvs {
                println!("{}", tlv);
            }
        }
        Commands::LLDPNetworkConfig(args) => {
            let osc = connect(&cli.address)?;
            let mut wait_secs = args.wait_secs.unwrap_or(1);
            while wait_secs > 0 {
                let req = onie_sai::LLDPNetworkConfigRequest {
                    device: args.device.clone(),
                    ..Default::default()
                };
                log::info!("making request to onie-said: {:?}...", req);
                let resp = osc
                    .lldp_network_config(default_ctx(), &req)
                    .context("request to onie-said failed")?;
                log::info!("response from onie-said: {:?}", resp);
                if let Some(network_config) = resp.network_config.into_option() {
                    // we need to replace the "-" in the device name with "_" because shells
                    // don't like dashes in variable names
                    let dev = args.device.replace("-", "_");
                    println!("onie_lldp_{}_ip=\"{}\"", dev, network_config.ip);
                    for (i, route) in network_config.routes.iter().enumerate() {
                        println!(
                            "onie_lldp_{}_route_{}_gateway=\"{}\"",
                            dev, i, route.gateway
                        );
                        println!(
                            "onie_lldp_{}_route_{}_dests=\"{}\"",
                            dev,
                            i,
                            route.destinations.join(" ")
                        );
                    }
                    println!("onie_lldp_{}_is_hh=\"{}\"", dev, network_config.is_hh);
                    break;
                }
                wait_secs -= 1;
                if wait_secs == 0 {
                    break;
                }
                thread::sleep(Duration::from_millis(1000));
            }
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
