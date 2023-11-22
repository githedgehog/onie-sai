pub(crate) mod netlink;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use onie_sai_rpc::onie_sai;
use onie_sai_rpc::wrap_message_field;

use thiserror::Error;

use crate::onie_lldpd::lldp::LLDPTLVs;
use crate::onie_lldpd::lldp::NetworkConfig;

use super::lldp::LLDPSocket;

#[derive(Error, Debug)]
pub(crate) enum ProcessError {
    #[error("failed to get interface: {0}")]
    GetInterfaceError(std::io::Error),

    #[error("failed to get interface index for '{0}'")]
    NoSuchInterfaceError(String),
}

pub(crate) enum ProcessRequest {
    Shutdown,
    Version(
        (
            onie_sai::VersionRequest,
            Sender<Result<onie_sai::VersionResponse, ProcessError>>,
        ),
    ),
    IsInitialDiscoveryFinished(
        (
            onie_sai::IsInitialDiscoveryFinishedRequest,
            Sender<Result<onie_sai::IsInitialDiscoveryFinishedResponse, ProcessError>>,
        ),
    ),
    NetlinkLinkChanged((u32, bool)),
    LLDPTLVsReceived((u32, LLDPTLVs)),
    LLDPNetworkConfigReceived((u32, NetworkConfig)),
    LLDPStatus(
        (
            onie_sai::LLDPStatusRequest,
            Sender<Result<onie_sai::LLDPStatusResponse, ProcessError>>,
        ),
    ),
    LLDPNetworkConfig(
        (
            onie_sai::LLDPNetworkConfigRequest,
            Sender<Result<onie_sai::LLDPNetworkConfigResponse, ProcessError>>,
        ),
    ),
}

pub(crate) struct Processor {
    rx: Receiver<ProcessRequest>,
    tx: Sender<ProcessRequest>,
    hifs: Vec<HostInterface>,
}

impl Processor {
    pub(crate) fn new() -> anyhow::Result<Self> {
        // the processor channel
        let (tx, rx) = channel();

        let mut hifs = Vec::new();
        for (idx, ifname) in netlink::get_interfaces()? {
            let mut hif = HostInterface {
                name: ifname,
                idx: idx,
                lldp_socket: None,
                lldp_tlvs: None,
                lldp_network_config: None,
            };
            hif.start_lldp_recv_thread(tx.clone());
            hifs.push(hif);
        }

        Ok(Processor {
            rx: rx,
            tx: tx,
            hifs: hifs,
        })
    }

    pub(crate) fn get_sender(&self) -> Sender<ProcessRequest> {
        self.tx.clone()
    }

    pub(crate) fn process(self) {
        let mut p = self;
        while let Ok(req) = p.rx.recv() {
            match req {
                // shut down processor
                ProcessRequest::Shutdown => return,

                // all RPC request handling
                ProcessRequest::Version((r, resp_tx)) => {
                    let resp = p.process_version_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!("failed to send version response to rpc server: {:?}", e);
                    };
                }
                ProcessRequest::IsInitialDiscoveryFinished((r, resp_tx)) => {
                    let resp = p.process_is_initial_discovery_finished_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!(
                            "failed to send is initial discovery finished response to rpc server: {:?}",
                            e
                        );
                    };
                }
                ProcessRequest::LLDPStatus((r, resp_tx)) => {
                    let resp = p.process_lldp_status_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!("failed to send lldp status response to rpc server: {e:?}");
                    };
                }
                ProcessRequest::LLDPNetworkConfig((r, resp_tx)) => {
                    let resp = p.process_lldp_network_config_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!(
                            "failed to send lldp network config response to rpc server: {e:?}"
                        );
                    };
                }

                // internal events
                ProcessRequest::NetlinkLinkChanged((if_idx, is_up)) => {
                    p.process_netlink_link_changed(if_idx, is_up)
                }
                ProcessRequest::LLDPTLVsReceived((if_idx, tlvs)) => {
                    p.process_lldp_tlvs_received(if_idx, tlvs)
                }
                ProcessRequest::LLDPNetworkConfigReceived((if_idx, config)) => {
                    p.process_lldp_network_config_received(if_idx, config)
                }
            }
        }
    }

    fn process_version_request(
        &self,
        _: onie_sai::VersionRequest,
    ) -> Result<onie_sai::VersionResponse, ProcessError> {
        Ok(onie_sai::VersionResponse {
            onie_said_version: "0.1.0".to_string(),
            ..Default::default()
        })
    }

    fn process_is_initial_discovery_finished_request(
        &self,
        _: onie_sai::IsInitialDiscoveryFinishedRequest,
    ) -> Result<onie_sai::IsInitialDiscoveryFinishedResponse, ProcessError> {
        // NOTE: just a fake result in onie-lldpd
        Ok(onie_sai::IsInitialDiscoveryFinishedResponse {
            is_finished: true,
            ..Default::default()
        })
    }

    fn process_lldp_status_request(
        &self,
        req: onie_sai::LLDPStatusRequest,
    ) -> Result<onie_sai::LLDPStatusResponse, ProcessError> {
        let if_idx = netlink::get_interface_index(req.device.as_str())
            .map_err(|e| ProcessError::GetInterfaceError(e))?;

        // this should not be necessary actually, but we'll keep this just in case
        if if_idx == 0 {
            return Err(ProcessError::NoSuchInterfaceError(req.device.clone()));
        }
        log::debug!("LLDPStatusRequest: looking for LLDP TLVs for interface {if_idx}");

        for hif in self.hifs.iter() {
            if hif.idx == if_idx {
                if let Some(ref lldp_tlvs) = hif.lldp_tlvs {
                    return Ok(onie_sai::LLDPStatusResponse {
                        tlvs: lldp_tlvs.to_strings(),
                        packet_received: true,
                        ..Default::default()
                    });
                }
                log::debug!("LLDPStatusRequest: interface found, but no LLDP TLVs found for interface {if_idx}");
            }
        }

        log::debug!(
            "LLDPStatusRequest: interface not found or no LLDP TLVs found for interface {if_idx}"
        );
        Ok(onie_sai::LLDPStatusResponse {
            packet_received: false,
            ..Default::default()
        })
    }

    fn process_lldp_network_config_request(
        &self,
        req: onie_sai::LLDPNetworkConfigRequest,
    ) -> Result<onie_sai::LLDPNetworkConfigResponse, ProcessError> {
        let if_idx = netlink::get_interface_index(req.device.as_str())
            .map_err(|e| ProcessError::GetInterfaceError(e))?;

        // this should not be necessary actually, but we'll keep this just in case
        if if_idx == 0 {
            return Err(ProcessError::NoSuchInterfaceError(req.device.clone()));
        }
        log::debug!(
            "LLDPNetworkConfigRequest: looking for LLDP network config for interface {if_idx}"
        );

        for hif in self.hifs.iter() {
            if hif.idx == if_idx {
                if let Some(ref config) = hif.lldp_network_config {
                    return Ok(onie_sai::LLDPNetworkConfigResponse {
                        network_config: wrap_message_field(Some(config.clone().into())),
                        ..Default::default()
                    });
                }
                log::debug!("LLDPNetworkConfigRequest: interface found, but no LLDP network config found for interface {if_idx}");
            }
        }

        log::debug!("LLDPNetworkConfigRequest: interface not found or no LLDP network config found for interface {if_idx}");
        Ok(onie_sai::LLDPNetworkConfigResponse {
            network_config: wrap_message_field(None),
            ..Default::default()
        })
    }

    fn process_netlink_link_changed(&mut self, if_idx: u32, is_up: bool) {
        let if_name = netlink::get_interface_name(if_idx).unwrap_or("unknown".to_string());
        // find the host interface
        // and update the link status in there
        let mut found = false;

        for hif in self.hifs.iter_mut() {
            if hif.idx == if_idx {
                found = true;
                if is_up {
                    hif.stop_lldp_recv_thread();
                    hif.start_lldp_recv_thread(self.tx.clone());
                } else {
                    hif.stop_lldp_recv_thread();
                }
                break;
            }
        }

        if !found {
            log::warn!("host interface {if_name} ({if_idx}) not found during netlink link changed event. LLDP receive thread was not started/stopped.");
        }
    }

    fn process_lldp_tlvs_received(&mut self, if_idx: u32, lldp_tlvs: LLDPTLVs) {
        let if_name = netlink::get_interface_name(if_idx).unwrap_or("unknown".to_string());
        // find the host interface
        // and update the TLVs in there
        let mut found = false;

        for hif in self.hifs.iter_mut() {
            if hif.idx == if_idx {
                found = true;
                hif.lldp_tlvs = Some(lldp_tlvs);
                break;
            }
        }

        if !found {
            log::warn!("host interface {if_name} ({if_idx}) not found during LLDP TLVs event. Discovered LLDP TLVs were not stored.");
        }
    }

    fn process_lldp_network_config_received(&mut self, if_idx: u32, config: NetworkConfig) {
        let if_name = netlink::get_interface_name(if_idx).unwrap_or("unknown".to_string());
        // find the host interface
        // and update the network config in there
        let mut found = false;

        for hif in self.hifs.iter_mut() {
            if hif.idx == if_idx {
                found = true;
                hif.lldp_network_config = Some(config);
                break;
            }
        }

        if !found {
            log::warn!("host interface {if_name} ({if_idx}) not found during LLDP network config event. Discovered LLDP Network Config was not stored.");
        }
    }
}

impl Drop for Processor {
    fn drop(&mut self) {
        for hif in self.hifs.iter_mut() {
            hif.stop_lldp_recv_thread();
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HostInterface {
    pub(crate) name: String,
    pub(crate) idx: u32,
    pub(crate) lldp_socket: Option<Arc<LLDPSocket>>,
    pub(crate) lldp_tlvs: Option<LLDPTLVs>,
    pub(crate) lldp_network_config: Option<NetworkConfig>,
}

impl HostInterface {
    fn start_lldp_recv_thread(&mut self, processor_sender: Sender<ProcessRequest>) {
        if self.lldp_socket.is_none() {
            match LLDPSocket::new(self.idx as i32) {
                Ok(socket) => {
                    let socket = Arc::new(socket);
                    self.lldp_socket = Some(socket.clone());
                    let hif_idx = self.idx;
                    let hif_name = self.name.clone();
                    thread::spawn(move || {
                        log::debug!("Host Interface {hif_name}: LLDP receive thread started");
                        loop {
                            // this blocks until we receive a new packet on the socket
                            match socket.recv_packet() {
                                Ok(pkt) => {
                                    log::debug!("Host Interface {hif_name}: received LLDP packet");
                                    // parse the LLDP TLVs from the packet
                                    let lldp_tlvs: LLDPTLVs = match pkt.as_slice().try_into() {
                                        Ok(v) => v,
                                        Err(err) => {
                                            log::error!("Host Interface {}: failed to read and/or parse LLDP packet: {:?}", hif_name, err);
                                            continue;
                                        }
                                    };

                                    // send the LLDP TLVs to the processor thread
                                    // this is for general LLDP information that we received on this host interface
                                    // which can be queried through onie-saictl
                                    if let Err(e) =
                                        processor_sender.send(ProcessRequest::LLDPTLVsReceived((
                                            hif_idx,
                                            lldp_tlvs.clone(),
                                        )))
                                    {
                                        log::error!(
                                            "Host Interface {}: failed to send LLDP TLVs to processor thread: {:?}",
                                            hif_name,
                                            e
                                        );
                                    }

                                    // and try to get the network config from the LLDP TLVs
                                    if let Some(network_config) = lldp_tlvs.get_hh_network_config()
                                    {
                                        // store network config for this interface by sending it to the processor thread
                                        if let Err(e) = processor_sender.send(
                                            ProcessRequest::LLDPNetworkConfigReceived((
                                                hif_idx,
                                                network_config.clone(),
                                            )),
                                        ) {
                                            log::error!(
                                                "Host Interface {}: failed to send LLDP network config to processor thread: {:?}",
                                                hif_name,
                                                e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    // an error receiving most likely means that the socket was closed
                                    // we simply abort the thread for now until we know exactly what error to watch for
                                    log::error!(
                                        "Host Interface {}: failed to receive LLDP packet: {:?}",
                                        hif_name,
                                        e
                                    );
                                    break;
                                }
                            }
                        }
                        log::debug!("Host Interface {}: LLDP receive thread stopped", hif_name);
                    });
                }
                Err(e) => {
                    log::error!(
                        "Host Interface {}: failed to create LLDP socket: {:?}",
                        self.name,
                        e
                    );
                }
            }
        }
    }

    fn stop_lldp_recv_thread(&mut self) {
        if let Some(socket) = self.lldp_socket.take() {
            socket.ref_close();
        }
    }
}

impl std::fmt::Display for HostInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.idx)
    }
}
