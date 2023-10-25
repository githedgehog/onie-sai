use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use anyhow::{Context, Result};

use ttrpc::Server;

use onie_sai_rpc::onie_sai;
use onie_sai_rpc::onie_sai_ttrpc;

use crate::oniesai::ProcessError;
use crate::oniesai::ProcessRequest;

struct OnieSaiServer {
    proc_tx: Sender<ProcessRequest>,
}

impl onie_sai_ttrpc::OnieSai for OnieSaiServer {
    fn version(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        req: onie_sai::VersionRequest,
    ) -> ttrpc::Result<onie_sai::VersionResponse> {
        let (tx, rx) = channel();
        self.proc_tx
            .send(ProcessRequest::Version((req, tx)))
            .map_err(map_tx_error)?;
        let resp = rx.recv().map_err(map_rx_error)?;
        let resp = resp.map_err(map_process_error)?;
        Ok(resp)
    }

    fn shell(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        req: onie_sai::ShellRequest,
    ) -> ttrpc::Result<onie_sai::ShellResponse> {
        let (tx, rx) = channel();
        self.proc_tx
            .send(ProcessRequest::Shell((req, tx)))
            .map_err(map_tx_error)?;
        let resp = rx.recv().map_err(map_rx_error)?;
        let resp = resp.map_err(map_process_error)?;
        Ok(resp)
    }

    fn port_list(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        req: onie_sai::PortListRequest,
    ) -> ttrpc::Result<onie_sai::PortListResponse> {
        let (tx, rx) = channel();
        self.proc_tx
            .send(ProcessRequest::PortList((req, tx)))
            .map_err(map_tx_error)?;
        let resp = rx.recv().map_err(map_rx_error)?;
        let resp = resp.map_err(map_process_error)?;
        Ok(resp)
    }

    fn auto_discovery(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        req: onie_sai::AutoDiscoveryRequest,
    ) -> ttrpc::Result<onie_sai::AutoDiscoveryResponse> {
        let (tx, rx) = channel();
        self.proc_tx
            .send(ProcessRequest::AutoDiscoveryStatus((req, tx)))
            .map_err(map_tx_error)?;
        let resp = rx.recv().map_err(map_rx_error)?;
        let resp = resp.map_err(map_process_error)?;
        Ok(resp)
    }
}

fn map_tx_error<T: std::fmt::Debug>(e: T) -> ttrpc::error::Error {
    ttrpc::Error::RpcStatus(ttrpc::get_status(
        ttrpc::Code::INTERNAL,
        format!("failed to submit request to SAI processor: {:?}", e),
    ))
}

fn map_rx_error<T: std::fmt::Debug>(e: T) -> ttrpc::error::Error {
    ttrpc::Error::RpcStatus(ttrpc::get_status(
        ttrpc::Code::INTERNAL,
        format!("failed to receive response from SAI processor: {:?}", e),
    ))
}

fn map_process_error(e: ProcessError) -> ttrpc::error::Error {
    ttrpc::Error::RpcStatus(ttrpc::get_status(
        ttrpc::Code::INTERNAL,
        format!("processor failed to process request: {}", e),
    ))
}

pub(crate) fn start_rpc_server(proc_tx: Sender<ProcessRequest>) -> Result<Server> {
    let service = Box::new(OnieSaiServer { proc_tx: proc_tx })
        as Box<dyn onie_sai_ttrpc::OnieSai + Send + Sync>;
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
