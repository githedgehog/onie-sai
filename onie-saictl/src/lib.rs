use onie_sai_rpc::onie_sai;
use onie_sai_rpc::onie_sai_ttrpc;
use onie_sai_rpc::onie_sai_ttrpc::OnieSaiClient;
use onie_sai_rpc::SOCK_ADDR;
use ttrpc::context::{self, Context};
use ttrpc::Client;

use std::io::stdin;
use std::io::stdout;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
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

    /// lists all ports and their details
    Ports,

    /// lists all routes that are installed directed to the CPU
    Routes,

    /// gets auto-discovery status of onie-said
    /// or you can enable/disable it within onie-said
    AutoDiscovery(AutoDiscoveryArgs),

    /// runs the SAI vendor shell
    Shell,

    /// shuts down onie-said (equals sending a SIGTERM to the process)
    Shutdown,

    /// Wait on initial discovery to complete. This is meant for init scripts which need to wait until the initial discovery is complete
    /// before they can consider the service started up.
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
        Commands::Routes => {
            let req = onie_sai::RouteListRequest::new();
            log::info!("making request to onie-said: {:?}...", req);
            let resp = osc
                .route_list(default_ctx(), &req)
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
            shell_command(osc)?;
        }
        Commands::Shutdown => {
            let req = onie_sai::ShutdownRequest::new();
            log::info!("making request to onie-said: {:?}...", req);
            let resp = osc
                .shutdown(default_ctx(), &req)
                .context("request to onie-said failed")?;
            log::info!("response from onie-said: {:?}", resp);
        }
        Commands::WaitOnInitialDiscovery => loop {
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
        },
        Commands::LLDP(args) => {
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

const SHELL_PROMPT: &str = "sai-shell> ";
const SHELL_SOCKET: &str = "/run/onie-saictl-shell.socket";

fn shell_command(osc: OnieSaiClient) -> anyhow::Result<()> {
    // start listener that listens on SHELL_SOCKET that onie-said will connect to
    let _ = std::fs::remove_file(SHELL_SOCKET);
    let listener = UnixListener::bind(SHELL_SOCKET).context(format!(
        "failed to bind to shell socket at {}",
        SHELL_SOCKET
    ))?;

    // now send request to onie-said to start shell
    let rpc_thread = thread::spawn(move || {
        let req = onie_sai::ShellRequest {
            socket: SHELL_SOCKET.to_string(),
            ..Default::default()
        };
        log::info!("making request to onie-said: {:?}...", req);
        let resp = osc
            .shell(default_ctx(), &req)
            .expect("request to onie-said failed");
        log::info!("response from onie-said: {:?}", resp);
    });

    // wait for onie-said to connect
    let (mut conn, _) = listener
        .accept()
        .context("failed to accept incoming connection")?;

    // set the connection to nonblocking
    conn.set_nonblocking(true)
        .context("failed to set socket to nonblocking")?;

    // Clone the connection to handle input and output separately
    let mut conn_reader = conn.try_clone().context("failed to clone socket")?;

    // Spawn a separate thread to handle input from the socket and send it to stdout
    let (stdout_thread_tx, stdout_thread_rx) = mpsc::channel::<()>();
    let stdout_thread = thread::spawn(move || {
        let mut buffer = [0; 1024];
        let mut need_to_write_prompt = true;
        let mut need_to_exit_thread = false;
        loop {
            if let Ok(_) = stdout_thread_rx.try_recv() {
                need_to_exit_thread = true;
            }
            match conn_reader.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        // EOF
                        break;
                    }
                    // Write received data to stdout - ignore errors here
                    {
                        let mut stdout = stdout().lock();
                        let _ = stdout.write_all(&buffer[0..n]);
                        let _ = stdout.flush();
                    }
                    need_to_write_prompt = true;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Non-blocking mode, no data available yet
                    if need_to_exit_thread {
                        break;
                    }
                    if need_to_write_prompt {
                        write_prompt();
                        need_to_write_prompt = false;
                    }
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    log::error!(
                        "Error reading from socket, closing stdout writer thread: {:?}",
                        e
                    );
                    break;
                }
            }
        }
        log::debug!("stdout thread exiting");
    });

    // now read input from stdin and send it to the socket
    // NOTE: mheese: don't ask me about the voodoo below.
    // This is the only way I could get this to work with the initialization prompt already displayed
    // It is somewhat similar to what bcmshell.py is doin in SONiC, but not identical. They only need
    // to echo once, we need to echo twice. Also, the thread sleep in between the echoes seems to be essential.
    let stdin = stdin().lock();
    for _ in 1..=2 {
        conn.write_all("echo\n".as_bytes())
            .context("failed to write to socket")?;
        conn.flush().context("failed to flush write to socket")?;
        thread::sleep(Duration::from_millis(100));
    }

    for line in stdin.lines() {
        let mut line = line.context("failed to read from stdin")?;
        line.push('\n');
        conn.write_all(line.as_bytes())
            .context("failed to write to socket")?;
        if line == "quit\n" {
            break;
        }
        conn.flush().context("failed to flush write to socket")?;
    }
    log::debug!("shell command finished");

    // sync with threads before returning
    rpc_thread
        .join()
        .map_err(|e| anyhow::anyhow!("RPC thread paniced: {:?}", e))?;
    log::debug!("RPC thread exited");

    // this thread might be dead, so the channel might be closed already
    // the error can be ignored
    let _ = stdout_thread_tx.send(());
    stdout_thread
        .join()
        .map_err(|e| anyhow::anyhow!("stdout thread paniced: {:?}", e))?;
    Ok(())
}

fn write_prompt() {
    let mut stdout = stdout().lock();
    let _ = write!(stdout, "{}", SHELL_PROMPT);
    let _ = stdout.flush();
}
