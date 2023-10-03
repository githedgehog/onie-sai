use onie_sai_rpc::onie_sai;
use onie_sai_rpc::onie_sai_ttrpc;
use onie_sai_rpc::SOCK_ADDR;
use ttrpc::context::{self, Context};
use ttrpc::Client;

fn main() {
    // initialize logger, and log at info level if RUST_LOG is not set
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    log::info!("connecting to onie-said at: {}...", SOCK_ADDR);
    let c = Client::connect(SOCK_ADDR).unwrap();
    let osc = onie_sai_ttrpc::OnieSaiClient::new(c);

    let req = onie_sai::VersionRequest::new();
    log::info!("making request to onie-said: {:?}...", req);
    let resp = match osc.version(default_ctx(), &req) {
        Err(e) => {
            log::error!("request to onie-said failed: {:?}", e);
            return;
        }
        Ok(x) => x,
    };
    log::info!("response from onie-said: {:?}", resp);

    log::info!("Success");
}

fn default_ctx() -> Context {
    let mut ctx = context::with_timeout(0);
    ctx.add("key-1".to_string(), "value-1-1".to_string());
    ctx.add("key-1".to_string(), "value-1-2".to_string());
    ctx.set("key-2".to_string(), vec!["value-2".to_string()]);

    ctx
}
