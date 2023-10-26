mod oniesai;
mod rpc;

use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::process::Stdio;
use std::process::Termination;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use log::LevelFilter;

use macaddr::MacAddr6;

use sai::SAI;

use crate::oniesai::PlatformContextHolder;
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

    #[arg(long, default_value = arg_platform())]
    platform: String,

    #[arg(long, default_value = arg_init_config_file())]
    init_config_file: PathBuf,
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
    // parse flags and initialize logger
    // NOTE: we need to do this before we call the wrapper
    // as it will eat help and anything else otherwise
    let cli = Cli::parse();
    env_logger::builder()
        .filter_level(LevelFilter::from(cli.log_level))
        .init();

    // NOTE: wrapper will call app(). See below for details.
    App(wrapper(cli))
}

fn app(cli: Cli, stdin_write: File, stdout_read: File) -> anyhow::Result<()> {
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
    let proc = Processor::new(
        &sai_api,
        cli.mac_addr.into_array(),
        platform_ctx,
        stdin_write,
        stdout_read,
    )
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

/// This is the indicator that we are in the client process.
const CLIENT_PROCESS_INDICATOR: &str =
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const PIPE_STDIN_WRITE: &str = "PIPE_STDIN_WRITE";
const PIPE_STDOUT_READ: &str = "PIPE_STDOUT_READ";

/// The whole purpose of the wrapper is to spawn a child process which runs the actual
/// application. We do this so that we can send and read from the stdin and stdout of the
/// process. And we require to do this so that we are able to forward the vendor shell
/// input and output to the connecting client application through which we want to access
/// it.
/// Essentially we are creating additional pipes for stdin and stdout, so that we can
/// forward the other sides of the pipes to the child process, so that we can effectively
/// pass back the stdin and stdout of the process. The only reason why we need to do this
/// is because Linux does not allow write access to stdin and read access from stdout for
/// its own process.
/// Yes, this is extremely stupid, and a lot of effort for a debugging tool. Nonetheless,
/// we would be lost if we don't have access to this shell. And leaving it within the
/// server application is not acceptable as it would mean that we need to stop and restart
/// the application every time we want to access the shell. And that might destroy the
/// state that we want to debug.
fn wrapper(cli: Cli) -> anyhow::Result<()> {
    // First of all, check if we are in the client process.
    // If we are, we read the environment variables which must be set, and we switch to the
    // actual application then
    if env::var(CLIENT_PROCESS_INDICATOR).is_ok() {
        let stdin_write_fd: i32 = env::var(PIPE_STDIN_WRITE)
            .context("PIPE_STDIN_WRITE not set")?
            .parse()
            .context("PIPE_STDIN_WRITE contains and invalid fd (not a number)")?;
        let stdin_write = unsafe { File::from_raw_fd(stdin_write_fd) };
        let stdout_read_fd: i32 = env::var(PIPE_STDOUT_READ)
            .context("PIPE_STDOUT_READ not set")?
            .parse()
            .context("PIPE_STDOUT_READ is an invalid fd")?;
        let stdout_read = unsafe { File::from_raw_fd(stdout_read_fd) };
        return app(cli, stdin_write, stdout_read);
    }

    // Get the current program's arguments and environment variables
    let args: Vec<String> = env::args().collect();
    let mut env_vars: Vec<(String, String)> = env::vars().collect();

    // Add the indicator environment variable to the environment
    // this is how the child process knows that it needs to start the real application
    env_vars.push((
        CLIENT_PROCESS_INDICATOR.to_string(),
        std::process::id().to_string(),
    ));

    // Create the pipe for stdin now
    // - we push the write end of the pipe to the client process, so that the child can write to its own stdin
    // - we keep the read end of the pipe for ourselves, so that we can write back to the stdin of the child
    let mut fds = vec![0i32; 2];
    let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if ret < 0 {
        return Err(anyhow::anyhow!(
            "wrapper: failed to create dedicate pipe for stdin pump: {}",
            std::io::Error::last_os_error()
        ));
    }
    let ret = unsafe { libc::fcntl(fds[0], libc::F_SETFL, libc::O_NONBLOCK) };
    if ret < 0 {
        return Err(anyhow::anyhow!(
            "wrapper: failed to set dedicated pipe fd[0] for stdin to non-blocking: {}",
            std::io::Error::last_os_error()
        ));
    }
    let ret = unsafe { libc::fcntl(fds[1], libc::F_SETFL, libc::O_NONBLOCK) };
    if ret < 0 {
        return Err(anyhow::anyhow!(
            "wrapper: failed to set dedicated pipe fd[1] for stdin to non-blocking: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut pipe_stdin_read = unsafe { File::from_raw_fd(fds[0]) };
    env_vars.push((PIPE_STDIN_WRITE.to_string(), fds[1].to_string()));

    // Create the pipe for stdout now
    // - we push the read end of the pipe to the client process, so that the child can read from its own stdout
    // - we keep the write end of the pipe for ourselves, so that we can read from the stdout of the child, and write it back over the pipe
    let mut fds = vec![0i32; 2];
    let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if ret < 0 {
        return Err(anyhow::anyhow!(
            "wrapper: failed to create dedicate pipe for stdin pump: {}",
            std::io::Error::last_os_error()
        ));
    }
    let ret = unsafe { libc::fcntl(fds[0], libc::F_SETFL, libc::O_NONBLOCK) };
    if ret < 0 {
        return Err(anyhow::anyhow!(
            "wrapper: failed to set dedicated pipe fd[0] for stdout to non-blocking: {}",
            std::io::Error::last_os_error()
        ));
    }
    let ret = unsafe { libc::fcntl(fds[1], libc::F_SETFL, libc::O_NONBLOCK) };
    if ret < 0 {
        return Err(anyhow::anyhow!(
            "wrapper: failed to set dedicated pipe fd[1] for stdout to non-blocking: {}",
            std::io::Error::last_os_error()
        ));
    }
    env_vars.push((PIPE_STDOUT_READ.to_string(), fds[0].to_string()));
    let mut pipe_stdout_write = unsafe { File::from_raw_fd(fds[1]) };

    // We spawn a new process with the same executable, arguments, and our modified environment variables now
    // NOTE: it is extremely important that we inherit stderr because all of our logging is done through it
    let mut child_process = std::process::Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .envs(env_vars)
        .spawn()
        .context("wrapper: failed to spawn child process")?;

    // get the stdin and stdout handles for the child process
    // we will use them in our data pumping threads below
    let mut child_stdin = child_process
        .stdin
        .take()
        .context("wrapper: failed to open stdin of child process")?;
    let child_stdin_fd = child_stdin.as_raw_fd();
    let ret = unsafe { libc::fcntl(child_stdin_fd, libc::F_SETFL, libc::O_NONBLOCK) };
    if ret < 0 {
        return Err(anyhow::anyhow!(
            "wrapper: failed to set child stdin fd to non-blocking: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut child_stdout = child_process
        .stdout
        .take()
        .context("wrapper: failed to open stdout of child process")?;
    let child_stdout_fd = child_stdout.as_raw_fd();
    let ret = unsafe { libc::fcntl(child_stdout_fd, libc::F_SETFL, libc::O_NONBLOCK) };
    if ret < 0 {
        return Err(anyhow::anyhow!(
            "wrapper: failed to set child stdout fd to non-blocking: {}",
            std::io::Error::last_os_error()
        ));
    }

    // this thread will read from the stdout of the child process and write it back to the child process
    // on the dedicated pipe, where the child can read it from the fd which is in PIPE_STDOUT_READ of its process
    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        let mut send_to_child = false;
        loop {
            let n = match child_stdout.read(&mut buf) {
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // Non-blocking mode, no data available yet
                    // NOTE: I believe child_stdout will always be a blocking read, so this is useless
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    log::error!(
                        "wrapper: failed to read from stdout of child process: {:?}",
                        e
                    );
                    continue;
                }
            };
            if n == 0 {
                // EOF, end thread
                break;
            }

            // we have read something, now check if we need to flip to send to the child
            // because the shell was enabled
            if let Ok(shell_enable) = rx.try_recv() {
                send_to_child = shell_enable;
            }
            if send_to_child {
                if let Err(e) = pipe_stdout_write.write_all(&buf[..n]) {
                    log::error!(
                        "wrapper: failed to write to dedicated pipe for stdout: {:?}",
                        e
                    );
                    continue;
                }
            } else {
                let mut stdout = std::io::stdout().lock();
                let _ = stdout.write_all(&buf[..n]);
                let _ = stdout.flush();
            }
        }
    });

    // this thread will read on the read end of the dedicated pipe for the child process where the child
    // can write to its own stdin and it will write it to the stdin of the child process
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            let n = match pipe_stdin_read.read(&mut buf) {
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // Non-blocking mode, no data available yet
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    log::error!(
                        "wrapper: failed to read from dedicated pipe for stdin: {:?}",
                        e
                    );
                    continue;
                }
            };
            if n == 0 {
                // EOF, end thread
                break;
            }

            // check if we got the shell enable markers
            if &buf[..n] == b"SAI_SHELL_ENABLE".as_slice() {
                // we received the shell enable command, send it to the other thread
                if let Err(e) = tx.send(true) {
                    log::error!(
                        "wrapper: failed to send shell enable command to stdout thread: {:?}",
                        e
                    );
                }
                continue;
            } else if &buf[..n] == b"SAI_SHELL_DISABLE".as_slice() {
                // we received the shell disable command, send it to the other thread
                if let Err(e) = tx.send(false) {
                    log::error!(
                        "wrapper: failed to send shell disable command to other thread: {:?}",
                        e
                    );
                }
                continue;
            }

            // apart from those markers we send everything to the child process
            if let Err(e) = child_stdin.write_all(&buf[..n]) {
                log::error!(
                    "wrapper: failed to write to stdin of child process: {:?}",
                    e
                );
                continue;
            }
        }
    });

    // Check if the process was created successfully
    // Wait for the child process to complete
    let status = child_process
        .wait()
        .context("wrapper: child process did not run")?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "wrapper: child process did not exit successfully: exit status: {:?}",
            status.code()
        ));
    }

    Ok(())
}
