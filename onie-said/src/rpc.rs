use std::sync::Arc;

use anyhow::{Context, Result};

use ttrpc::Server;

use onie_sai_rpc::onie_sai;
use onie_sai_rpc::onie_sai_ttrpc;

use sai::SAI;

struct OnieSaiServer {}

impl onie_sai_ttrpc::OnieSai for OnieSaiServer {
    fn version(
        &self,
        _ctx: &::ttrpc::TtrpcContext,
        _: onie_sai::VersionRequest,
    ) -> ::ttrpc::Result<onie_sai::VersionResponse> {
        let sai_version = match SAI::api_version() {
            Err(e) => {
                return Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                    ttrpc::Code::INTERNAL,
                    format!("failed to query SAI API version from SAI: {:?}", e),
                )))
            }
            Ok(v) => v.to_string(),
        };
        let resp = onie_sai::VersionResponse {
            onie_said_version: "0.1.0".to_string(),
            sai_version: sai_version,
            ..Default::default()
        };
        Ok(resp)
    }
}

pub(crate) fn start_rpc_server() -> Result<Server> {
    let service = Box::new(OnieSaiServer {}) as Box<dyn onie_sai_ttrpc::OnieSai + Send + Sync>;
    let service = Arc::new(service);
    let onie_sai_service = onie_sai_ttrpc::create_onie_sai(service);

    onie_sai_rpc::remove_sock_addr_if_exist().context(format!(
        "failed to remove socket file {}",
        onie_sai_rpc::SOCK_ADDR,
    ))?;

    let mut rpc_server = Server::new()
        .bind(onie_sai_rpc::SOCK_ADDR)
        .map(|s| s.register_service(onie_sai_service))
        .context(format!(
            "failed to bind to socket file {}",
            onie_sai_rpc::SOCK_ADDR,
        ))?;

    rpc_server.start().context("starting ttrpc server failed")?;

    log::info!("ttrpc server listening now on {}", onie_sai_rpc::SOCK_ADDR);

    Ok(rpc_server)
}
