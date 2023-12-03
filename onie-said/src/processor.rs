pub(crate) mod netlink;
pub(crate) mod port;

use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use ipnet::IpNet;
use onie_sai_rpc::onie_sai;
use onie_sai_rpc::wrap_message_field;
use sai::bridge;
use sai::hostif::table_entry::ChannelType;
use sai::hostif::table_entry::TableEntryAttribute;
use sai::hostif::table_entry::TableEntryType;
use sai::hostif::trap::TrapAttribute;
use sai::hostif::trap::TrapType;
use sai::hostif::HostIf;
use sai::hostif::HostIfAttribute;
use sai::hostif::HostIfType;
use sai::port::OperStatus;
use sai::port::PortID;
use sai::route::RouteEntry;
use sai::route::RouteEntryAttribute;
use sai::router_interface::RouterInterfaceAttribute;
use sai::router_interface::RouterInterfaceType;
use sai::switch::Switch;
use sai::switch::SwitchAttribute;
use sai::ObjectID;
use sai::PacketAction;
use sai::SAI;

use anyhow::Context;
use sai::sai_mac_t;
use sai::virtual_router::VirtualRouter;

use thiserror::Error;

use crate::lldp::LLDPTLVs;
use crate::lldp::NetworkConfig;
use crate::processor::port::SortPortsByLanes;

use self::port::discovery::logicalport::Event::PortUp;
use self::port::PhysicalPort;
use self::port::PhysicalPortConfig;

#[derive(Clone)]
pub(crate) struct PlatformContextHolder<'a> {
    obj: Rc<dyn xcvr::PlatformContext + 'a>,
}

impl<'a> PlatformContextHolder<'a> {
    pub(crate) fn new<T: xcvr::PlatformContext + 'a>(object: T) -> Self {
        Self {
            obj: Rc::new(object),
        }
    }
}

impl std::fmt::Debug for PlatformContextHolder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PlatformContextHolder")
    }
}

#[derive(Error, Debug)]
pub(crate) enum ProcessError {
    #[error("SAI status returned unsuccessful")]
    SAIStatus(#[from] sai::Status),

    #[error("SAI command failed")]
    SAIError(#[from] sai::Error),

    #[error("Shell Command IO Error")]
    ShellIOError(anyhow::Error),

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
    Shell(
        (
            onie_sai::ShellRequest,
            Sender<Result<onie_sai::ShellResponse, ProcessError>>,
        ),
    ),
    PortList(
        (
            onie_sai::PortListRequest,
            Sender<Result<onie_sai::PortListResponse, ProcessError>>,
        ),
    ),
    RouteList(
        (
            onie_sai::RouteListRequest,
            Sender<Result<onie_sai::RouteListResponse, ProcessError>>,
        ),
    ),
    AutoDiscoveryPoll,
    AutoDiscoveryStatus(
        (
            onie_sai::AutoDiscoveryRequest,
            Sender<Result<onie_sai::AutoDiscoveryResponse, ProcessError>>,
        ),
    ),
    IsInitialDiscoveryFinished(
        (
            onie_sai::IsInitialDiscoveryFinishedRequest,
            Sender<Result<onie_sai::IsInitialDiscoveryFinishedResponse, ProcessError>>,
        ),
    ),
    LogicalPortStateChange((PortID, OperStatus)),
    NetlinkAddrAdded((u32, IpAddr)),
    NetlinkAddrRemoved((u32, IpAddr)),
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

pub(crate) struct Processor<'a, 'b> {
    auto_discovery: bool,
    auto_discovery_with_breakout: bool,
    switch: Switch<'a>,
    virtual_router: VirtualRouter<'a>,
    routes: Vec<RouteEntry<'a>>,
    cpu_port_id: PortID,
    cpu_hostif: HostIf<'a>,
    ports: Vec<PhysicalPort<'a, 'b>>,
    rx: Receiver<ProcessRequest>,
    tx: Sender<ProcessRequest>,
    stdin_write: File,
    stdout_read: File,
}

impl<'a, 'b> Processor<'a, 'b> {
    pub(crate) fn new(
        sai_api: &'a SAI,
        mac_address: sai_mac_t,
        ports_config: Option<Vec<PhysicalPortConfig>>,
        auto_discovery: bool,
        auto_discovery_with_breakout: bool,
        platform_ctx: PlatformContextHolder<'b>,
        stdin_write: File,
        stdout_read: File,
    ) -> anyhow::Result<Self> {
        // now create switch
        let switch: Switch<'a> = sai_api
            .switch_create(vec![
                SwitchAttribute::InitSwitch(true),
                SwitchAttribute::SrcMacAddress(mac_address),
            ])
            .context("failed to create switch")?;
        log::info!("successfully created switch: {:?}", switch);

        // the processor channel
        let (tx, rx) = channel();
        let psc_cb_tx = tx.clone();

        // port state change callback
        switch
            .set_port_state_change_callback(Box::new(move |notifications| {
                for notification in notifications {
                    let port_id = PortID::from(notification);
                    let port_state = OperStatus::from(notification);
                    log::info!(
                        "Port State Change Event: port_id = {:?}, port_state = {:?}",
                        port_id,
                        port_state
                    );
                    if let Err(e) = psc_cb_tx.send(ProcessRequest::LogicalPortStateChange((port_id, port_state))) {
                        log::error!("Port State Change Event: failed to submit port state change event to processor (port_id = {:?}, port_state = {:?}): {}", port_id, port_state, e);
                    }
                }
            }))
            .context("failed to set port state change callback")?;

        // remove default bridge ports
        let default_bridge = switch
            .get_default_bridge()
            .context("failed to get dfeault bridge")?;
        log::debug!(
            "default bridge of switch {} is: {:?}",
            switch,
            default_bridge
        );
        let bridge_ports = default_bridge.get_ports().context(format!(
            "failed to get bridge ports for default bridge {}",
            default_bridge
        ))?;
        for bridge_port in bridge_ports {
            match bridge_port.get_type() {
                // we only go ahead when this is a real port
                Ok(bridge::port::Type::Port) => {}
                Ok(v) => {
                    log::info!("not removing bridge port {} of type: {:?}", bridge_port, v);
                    continue;
                }
                Err(e) => {
                    return Err(anyhow!(
                        "failed to get bridge port type of bridge port {}: {:?}",
                        bridge_port,
                        e
                    ));
                }
            }

            log::debug!("removing bridge port {}...", bridge_port);
            bridge_port
                .remove()
                .context("failed to remove bridge port")?;
        }

        // program traps
        let default_trap_group = switch
            .get_default_hostif_trap_group()
            .context("failed to get default host interface trap group")?;
        let default_trap_group_id = default_trap_group.to_id();
        // we can perfectly survive without this trap, so no need to fail or alarm anybody
        match switch.create_hostif_trap(vec![
            TrapAttribute::TrapType(TrapType::TTLError),
            TrapAttribute::PacketAction(PacketAction::Trap),
            TrapAttribute::TrapGroup(default_trap_group_id),
        ]) {
            Ok(_) => log::debug!("traps: added TTL error trap (action: trap)"),
            Err(e) => log::debug!(
                "traps: failed to create TTL error trap (action: trap): {}",
                e
            ),
        }
        // critical, we must fail if this does not work
        let _ip2me_trap = switch
            .create_hostif_trap(vec![
                TrapAttribute::TrapType(TrapType::IP2ME),
                TrapAttribute::PacketAction(PacketAction::Trap),
                TrapAttribute::TrapGroup(default_trap_group_id),
            ])
            .context("failed to create IP2ME trap")?;
        log::debug!("traps: added IP2ME trap (action: trap)");
        // ARP request/response are critical for IPv4, we must fail if this does not work
        let _arp_req_trap = switch
            .create_hostif_trap(vec![
                TrapAttribute::TrapType(TrapType::ARPRequest),
                TrapAttribute::PacketAction(PacketAction::Copy),
                TrapAttribute::TrapGroup(default_trap_group_id),
            ])
            .context("failed to create ARP request trap")?;
        log::debug!("traps: added ARP request trap (action: copy)");
        let _arp_resp_trap = switch
            .create_hostif_trap(vec![
                TrapAttribute::TrapType(TrapType::ARPResponse),
                TrapAttribute::PacketAction(PacketAction::Copy),
                TrapAttribute::TrapGroup(default_trap_group_id),
            ])
            .context("failed to create ARP response trap")?;
        log::debug!("traps: added ARP response trap (action: copy)");
        // IPv6 Neighbor Discovery is critical for what we need at Hedgehog
        // so we must fail if this does not work
        let _neigh_disc_trap = switch
            .create_hostif_trap(vec![
                TrapAttribute::TrapType(TrapType::IPv6NeighborDiscovery),
                TrapAttribute::PacketAction(PacketAction::Copy),
                TrapAttribute::TrapGroup(default_trap_group_id),
            ])
            .context("failed to create IPv6 Neighbor Discovery trap")?;
        log::debug!("traps: added IPv6 Neighbor Discovery trap (action: copy)");
        // IPv6 Neighbor Discovery is probably already enough, so we don't want to fail
        // if the next two are failing (and they are not implemented on Broadcom SAI for example)
        match switch.create_hostif_trap(vec![
            TrapAttribute::TrapType(TrapType::IPv6NeighborSolicitation),
            TrapAttribute::PacketAction(PacketAction::Copy),
            TrapAttribute::TrapGroup(default_trap_group_id),
        ]) {
            Ok(_) => log::debug!("traps: added IPv6 Neighbor Solicitation trap (action: copy)"),
            Err(e) => log::debug!(
                "traps: failed to create IPv6 Neighbor Solicitation trap (action: copy): {}",
                e
            ),
        }
        match switch.create_hostif_trap(vec![
            TrapAttribute::TrapType(TrapType::IPv6NeighborAdvertisement),
            TrapAttribute::PacketAction(PacketAction::Copy),
            TrapAttribute::TrapGroup(default_trap_group_id),
        ]) {
            Ok(_) => log::debug!("traps: added IPv6 Neighbor Advertisement trap (action: copy)"),
            Err(e) => log::debug!(
                "traps: failed to create IPv6 Neighbor Advertisement trap (action: copy): {}",
                e
            ),
        }
        // TODO: probably not necessary? they are not in SONiC
        // and I don't understand yet what they would do as compared to the "normal" L3 ones below?!
        // - SAI_HOSTIF_TRAP_TYPE_DHCP_L2
        // - SAI_HOSTIF_TRAP_TYPE_DHCPV6_L2
        // as IPv6 link-local waterfall is enough for us (Hedgehog) in ONIE, we can actually even
        // let the DHCP traps fail technically
        match switch.create_hostif_trap(vec![
            TrapAttribute::TrapType(TrapType::DHCP),
            TrapAttribute::PacketAction(PacketAction::Copy),
            TrapAttribute::TrapGroup(default_trap_group_id),
        ]) {
            Ok(_) => log::debug!("traps: added DHCP trap (action: copy)"),
            Err(e) => log::debug!("traps: failed to create DHCP trap (action: copy): {}", e),
        }
        match switch.create_hostif_trap(vec![
            TrapAttribute::TrapType(TrapType::DHCPv6),
            TrapAttribute::PacketAction(PacketAction::Copy),
            TrapAttribute::TrapGroup(default_trap_group_id),
        ]) {
            Ok(_) => log::debug!("traps: added DHCPv6 trap (action: copy)"),
            Err(e) => log::debug!("traps: failed to create DHCPv6 trap (action: copy): {}", e),
        }
        // TODO: LLDP/UDLD not necessary, but if actioned upon might improve debuggability from outside of a box (would need an implementation though)
        // as they are not critical, no need to fail if they are not created
        match switch.create_hostif_trap(vec![
            TrapAttribute::TrapType(TrapType::LLDP),
            TrapAttribute::PacketAction(PacketAction::Trap),
            TrapAttribute::TrapGroup(default_trap_group_id),
        ]) {
            Ok(_) => log::debug!("traps: added LLDP trap (action: trap)"),
            Err(e) => log::debug!("traps: failed to create LLDP trap (action: trap): {}", e),
        }
        match switch.create_hostif_trap(vec![
            TrapAttribute::TrapType(TrapType::UDLD),
            TrapAttribute::PacketAction(PacketAction::Trap),
            TrapAttribute::TrapGroup(default_trap_group_id),
        ]) {
            Ok(_) => log::debug!("traps: added UDLD trap (action: trap)"),
            Err(e) => log::debug!("traps: failed to create UDLD trap (action: trap): {}", e),
        }

        // by default we want to create a table entry which matches all created traps on all interfaces
        // and receives them over the Linux netdev interfaces (thanks)
        let _default_table_entry = switch
            .create_hostif_table_entry(vec![
                TableEntryAttribute::Type(TableEntryType::Wildcard),
                TableEntryAttribute::ChannelType(ChannelType::NetdevPhysicalPort),
            ])
            .context("failed to create default host interface table entry")?;
        log::debug!("host interface table entry: added default entry: type=Wildcard Interface, wildcard trap id, channel=Receive packets via Linux netdev type port");

        // get CPU port
        let cpu_port = switch.get_cpu_port().context("failed to get CPU port")?;
        let cpu_port_id = PortID::from(cpu_port);

        // create host interface for it
        let cpu_intf: HostIf<'a> = switch
            .create_hostif(vec![
                HostIfAttribute::Name("CPU".to_string()),
                HostIfAttribute::Type(HostIfType::Netdev),
                HostIfAttribute::ObjectID(cpu_port_id.into()),
                HostIfAttribute::OperStatus(true),
            ])
            .context(format!(
                "failed to create host interface for CPU port {}",
                cpu_port_id
            ))?;

        // get the default virtual router
        let default_virtual_router: VirtualRouter<'a> = switch
            .get_default_virtual_router()
            .context("failed to get default virtual router")?;

        // prep the router: create loopback interface
        // create initial routes
        let _lo_rif = default_virtual_router
            .create_router_interface(vec![
                RouterInterfaceAttribute::Type(RouterInterfaceType::Loopback),
                RouterInterfaceAttribute::MTU(9100),
            ])
            .context(format!(
                "failed to get create loopback interface for virtual router {}",
                default_virtual_router
            ))?;

        let _default_ipv4_route_entry = default_virtual_router
            .create_route_entry(
                IpNet::from_str("0.0.0.0/0").unwrap(),
                vec![RouteEntryAttribute::PacketAction(PacketAction::Drop)],
            )
            .context(format!(
                "failed to create default route entry for virtual router {}",
                default_virtual_router
            ))?;
        log::info!(
            "added default route entry for IPv4 for virtual router {} (action: drop)",
            default_virtual_router
        );

        let _default_ipv6_route_entry = default_virtual_router
            .create_route_entry(
                IpNet::from_str("0000::/0").unwrap(),
                vec![RouteEntryAttribute::PacketAction(PacketAction::Drop)],
            )
            .context(format!(
                "failed to create default IPv6 route entry for virtual router {}",
                default_virtual_router
            ))?;
        log::info!(
            "added default route entry for IPv6 for virtual router {} (action: drop)",
            default_virtual_router
        );

        // get ports now
        let ports = switch
            .get_ports()
            .context(format!("failed to get port list from switch {}", switch))?;

        let mut ports = match ports_config {
            None => {
                // create the ports without port config
                let mut err = Ok(());
                let ret = ports
                    .into_iter()
                    .enumerate()
                    .map(|(i, port)| {
                        PhysicalPort::from_port(
                            platform_ctx.clone(),
                            switch.clone(),
                            default_virtual_router.clone(),
                            mac_address,
                            i,
                            port,
                            None,
                        )
                    })
                    .scan(&mut err, until_err)
                    .collect::<Vec<PhysicalPort>>();
                err?;
                ret
            }
            Some(ports_config) => {
                log::info!("Initializing ports from ports config, sorting ports according to their lane mappings...");

                // we have a ports configuration
                // so we will sort the ports according to our ports config file, and then we create the physical port from
                // the port config as well as the SAI port by zipping both vectors together
                let ports = ports_config.sort_ports_by_lanes(&ports)?;
                let mut err = Ok(());
                let ret = ports
                    .into_iter()
                    .zip(ports_config.into_iter())
                    .enumerate()
                    .map(|(i, (port, port_config))| {
                        PhysicalPort::from_port(
                            platform_ctx.clone(),
                            switch.clone(),
                            default_virtual_router.clone(),
                            mac_address,
                            i,
                            port,
                            Some(port_config),
                        )
                    })
                    .scan(&mut err, until_err)
                    .collect();
                err?;
                ret
            }
        };

        // if auto-discovery is enabled on startup (the default), we are going to start it now
        if auto_discovery {
            for port in ports.iter_mut() {
                port.enable_auto_discovery(auto_discovery_with_breakout);

                // we also need to deal with the initial discovery process wait thingy
                // we enable it for all the ports where a transceiver seems to be present
                // as we want to do this only once, we keep this outside of the enable_auto_discovery function
                // yes, this is all a bit ugly
                if port.xcvr_present {
                    port.initial_port_discovery = Some(());
                }
            }
        }

        Ok(Processor {
            auto_discovery: auto_discovery,
            auto_discovery_with_breakout: auto_discovery_with_breakout,
            switch: switch,
            virtual_router: default_virtual_router,
            routes: Vec::new(),
            cpu_port_id: cpu_port_id,
            cpu_hostif: cpu_intf,
            ports: ports,
            rx: rx,
            tx: tx,
            stdin_write: stdin_write,
            stdout_read: stdout_read,
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
                ProcessRequest::Shell((r, resp_tx)) => {
                    let resp = p.process_shell_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!("failed to send shell response to rpc server: {:?}", e);
                    };
                }
                ProcessRequest::PortList((r, resp_tx)) => {
                    let resp = p.process_port_list_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!("failed to send port list response to rpc server: {:?}", e);
                    };
                }
                ProcessRequest::RouteList((r, resp_tx)) => {
                    let resp = p.process_route_list_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!("failed to send route list response to rpc server: {:?}", e);
                    };
                }
                ProcessRequest::AutoDiscoveryStatus((r, resp_tx)) => {
                    let resp = p.process_auto_discovery_status_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!(
                            "failed to send auto discovery status response to rpc server: {:?}",
                            e
                        );
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
                ProcessRequest::AutoDiscoveryPoll => p.process_auto_discovery_poll(),
                ProcessRequest::LogicalPortStateChange((port_id, port_state)) => {
                    p.process_logical_port_state_change(port_id, port_state)
                }
                ProcessRequest::NetlinkAddrAdded((if_idx, ip)) => {
                    p.process_netlink_addr_added(if_idx, ip)
                }
                ProcessRequest::NetlinkAddrRemoved((if_idx, ip)) => {
                    p.process_netlink_addr_removed(if_idx, ip)
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
        match SAI::api_version() {
            Err(e) => Err(ProcessError::SAIStatus(e)),
            Ok(v) => Ok(onie_sai::VersionResponse {
                onie_said_version: "0.1.0".to_string(),
                sai_version: v.to_string(),
                ..Default::default()
            }),
        }
    }

    fn process_shell_request(
        &self,
        req: onie_sai::ShellRequest,
    ) -> Result<onie_sai::ShellResponse, ProcessError> {
        let mut conn = UnixStream::connect(&req.socket.as_str())
            .context(format!(
                "failed to connect to socket at {}",
                &req.socket.as_str()
            ))
            .map_err(|e| ProcessError::ShellIOError(e))?;
        conn.set_nonblocking(true)
            .context("failed to set non-blocking mode")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        let mut conn_writer = conn
            .try_clone()
            .context("failed to clone stream")
            .map_err(|e| ProcessError::ShellIOError(e))?;

        // enable shell IO
        thread::sleep(Duration::from_millis(10));
        let mut stdin_write_enabler = self
            .stdin_write
            .try_clone()
            .context("failed to clone stdin")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        stdin_write_enabler
            .write(b"SAI_SHELL_ENABLE")
            .context("failed to write shell enable marker to stdin")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        stdin_write_enabler
            .flush()
            .context("failed to flush shell enable marker to stdin")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        thread::sleep(Duration::from_millis(10));
        log::debug!("shell: SAI_SHELL_ENABLE sent");

        // this thread reads from the connection and writes to the stdin data pump
        let mut stdin_write = self
            .stdin_write
            .try_clone()
            .context("failed to clone stdin")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        let (stdin_thread_tx, stdin_thread_rx) = mpsc::channel::<()>();
        let stdin_thread = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut need_to_exit_thread = false;
            loop {
                if let Ok(_) = stdin_thread_rx.try_recv() {
                    need_to_exit_thread = true;
                }
                match conn.read(&mut buf) {
                    Ok(n) => {
                        if n == 0 {
                            // EOF
                            break;
                        }
                        if let Err(e) = stdin_write.write_all(buf[..n].as_ref()) {
                            log::error!("shell: failed to write to stdin: {:?}", e);
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        // Non-blocking mode, no data available yet
                        if need_to_exit_thread {
                            break;
                        }
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(e) => {
                        log::error!("shell: failed to read from socket: {:?}", e);
                        break;
                    }
                }
            }
            log::debug!("shell: stdin thread exiting");
        });

        // this thread reads from the stdout data pump and writes to the connection
        let mut stdout_read = self
            .stdout_read
            .try_clone()
            .context("failed to clone stdout")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        let (stdout_thread_tx, stdout_thread_rx) = mpsc::channel::<()>();
        let stdout_thread = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut need_to_exit_thread = false;
            loop {
                if let Ok(_) = stdout_thread_rx.try_recv() {
                    need_to_exit_thread = true;
                }
                match stdout_read.read(&mut buf) {
                    Ok(n) => {
                        if n == 0 {
                            // EOF
                            break;
                        }
                        if let Err(e) = conn_writer.write_all(&buf[..n]) {
                            log::error!("shell: failed to write to socket: {:?}", e);
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        // Non-blocking mode, no data available yet
                        if need_to_exit_thread {
                            break;
                        }
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(e) => {
                        log::error!("shell: failed to read from stdout: {:?}", e);
                        break;
                    }
                }
            }
            log::debug!("shell: stdout thread exiting");
        });

        // this warning is great because even with default warning level logs
        // it gives a hint that the processor thread is blocked
        log::warn!("shell requested, this blocks the processor thread!");
        self.switch
            .enable_shell()
            .map_err(|e| ProcessError::SAIError(e))?;

        // wait for all other threads to exit
        stdout_thread_tx
            .send(())
            .context("failed to send exit to stdout thread")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        stdout_thread
            .join()
            .map_err(|e| anyhow::anyhow!("stdout thread paniced: {:?}", e))
            .map_err(|e| ProcessError::ShellIOError(e))?;
        stdin_thread_tx
            .send(())
            .context("failed to send exit to stdin thread")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        stdin_thread
            .join()
            .map_err(|e| anyhow::anyhow!("stdin thread paniced: {:?}", e))
            .map_err(|e| ProcessError::ShellIOError(e))?;

        // disable shell IO again
        thread::sleep(Duration::from_millis(10));
        stdin_write_enabler
            .write(b"SAI_SHELL_DISABLE")
            .context("failed to write shell disable marker to stdin")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        stdin_write_enabler
            .flush()
            .context("failed to flush shell disable marker to stdin")
            .map_err(|e| ProcessError::ShellIOError(e))?;
        log::debug!("shell: SAI_SHELL_DISABLE sent");

        // this warning matches the one above, tell the users that we are unblocked again
        log::warn!("shell finished, processor thread unblocked!");
        Ok(onie_sai::ShellResponse {
            ..Default::default()
        })
    }

    fn process_port_list_request(
        &self,
        _: onie_sai::PortListRequest,
    ) -> Result<onie_sai::PortListResponse, ProcessError> {
        let mut ports = Vec::with_capacity(self.ports.len());
        for phy_port in self.ports.iter() {
            ports.push(phy_port.into());
        }
        Ok(onie_sai::PortListResponse {
            port_list: ports,
            ..Default::default()
        })
    }

    fn process_route_list_request(
        &self,
        _: onie_sai::RouteListRequest,
    ) -> Result<onie_sai::RouteListResponse, ProcessError> {
        let mut routes = Vec::with_capacity(self.routes.len());
        for route in self.routes.iter() {
            let ret: IpNet = route.into();
            routes.push(ret.to_string());
        }
        Ok(onie_sai::RouteListResponse {
            route_list: routes,
            ..Default::default()
        })
    }

    fn process_auto_discovery_status_request(
        &mut self,
        req: onie_sai::AutoDiscoveryRequest,
    ) -> Result<onie_sai::AutoDiscoveryResponse, ProcessError> {
        match req.enable {
            None => Ok(onie_sai::AutoDiscoveryResponse {
                enabled: self.auto_discovery,
                ..Default::default()
            }),
            Some(enable) => {
                self.auto_discovery = enable;
                let enable_with_breakout = match req.enable_with_breakout {
                    None => self.auto_discovery_with_breakout,
                    Some(v) => v,
                };
                if enable {
                    for port in self.ports.iter_mut() {
                        port.enable_auto_discovery(enable_with_breakout)
                    }
                } else {
                    for port in self.ports.iter_mut() {
                        port.disable_auto_discovery()
                    }
                }
                Ok(onie_sai::AutoDiscoveryResponse {
                    enabled: self.auto_discovery,
                    ..Default::default()
                })
            }
        }
    }

    fn process_is_initial_discovery_finished_request(
        &self,
        _: onie_sai::IsInitialDiscoveryFinishedRequest,
    ) -> Result<onie_sai::IsInitialDiscoveryFinishedResponse, ProcessError> {
        // if the initial_port_discovery is still Some(), then we are still
        // waiting for some state machine to complete
        // if they are all none, then we are done
        let finished = self
            .ports
            .iter()
            .find(|port| port.initial_port_discovery.is_some())
            .map(|_| false)
            .unwrap_or(true);
        Ok(onie_sai::IsInitialDiscoveryFinishedResponse {
            is_finished: finished,
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

        for phy_port in self.ports.iter() {
            for log_port in phy_port.ports.iter() {
                if let Some(ref hif) = log_port.hif {
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

        for phy_port in self.ports.iter() {
            for log_port in phy_port.ports.iter() {
                if let Some(ref hif) = log_port.hif {
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
            }
        }

        log::debug!("LLDPNetworkConfigRequest: interface not found or no LLDP network config found for interface {if_idx}");
        Ok(onie_sai::LLDPNetworkConfigResponse {
            network_config: wrap_message_field(None),
            ..Default::default()
        })
    }

    fn process_auto_discovery_poll(&mut self) {
        log::debug!("auto discovery poll");
        for phy_port in self.ports.iter_mut() {
            phy_port.auto_discovery_poll();
        }
    }

    fn process_logical_port_state_change(&mut self, port_id: PortID, port_state: OperStatus) {
        let mut found = false;
        let sender = self.get_sender();
        'outer: for phy_port in self.ports.iter_mut() {
            for log_port in phy_port.ports.iter_mut() {
                if log_port.port == port_id {
                    found = true;

                    // step port discovery state machine if it is a port up event
                    let oper_status: bool = port_state.into();
                    if oper_status {
                        match log_port.sm.as_mut() {
                            Some(sm) => {
                                *sm = sm.clone().step(&log_port.port, PortUp);
                            }
                            None => {
                                log::debug!("port {} has no discovery state machine", port_id);
                            }
                        }
                    }

                    // reconcile state
                    log_port.reconcile_state();

                    // update the associated host interface
                    match log_port.hif.as_mut() {
                        Some(hif) => {
                            // that sets the host interface operational status (SAI internal I guess?)
                            // TODO: should just be on one function on HostInterface
                            match hif.set_oper_status(oper_status, sender) {
                                Ok(_) => log::info!("set host interface {} ({}) operational status to {} for port {}", hif.name, hif.intf, oper_status, port_id),
                                Err(e) => log::error!("failed to set host interface {} ({}) operational status to {} for port {}: {:?}", hif.name, hif.intf, oper_status, port_id, e),
                            }
                        }
                        None => log::warn!(
                            "port {} has no associated host interface created yet",
                            port_id
                        ),
                    }
                    break 'outer;
                }
            }
        }
        if !found {
            log::warn!("port {} not found during port state change event", port_id);
        }
    }

    fn process_netlink_addr_added(&mut self, if_idx: u32, ip: IpAddr) {
        let if_name = netlink::get_interface_name(if_idx).unwrap_or("unknown".to_string());
        // find the host interface
        // we actually don't need to do anything with it
        // however, we need to check that the interface belongs to us
        let mut found = false;
        for phy_port in self.ports.iter() {
            for log_port in phy_port.ports.iter() {
                for hif in log_port.hif.iter() {
                    if hif.idx == if_idx {
                        found = true;
                    }
                }
            }
        }
        if found {
            // try to add the route, the function will handle if it is in there already
            self.add_route(ip.into());
        } else {
            log::warn!("host interface {if_name} ({if_idx}) not found during netlink address add event. Route not added.");
        }
    }

    fn process_netlink_addr_removed(&mut self, if_idx: u32, ip: IpAddr) {
        let if_name = netlink::get_interface_name(if_idx).unwrap_or("unknown".to_string());
        // find the host interface
        // we actually don't need to do anything with it
        // however, we need to check that the interface belongs to us
        let mut found = false;
        for phy_port in self.ports.iter() {
            for log_port in phy_port.ports.iter() {
                for hif in log_port.hif.iter() {
                    if hif.idx == if_idx {
                        found = true;
                    }
                }
            }
        }
        if found {
            // try to remove the route, the function will handle if it is even there or not
            self.remove_route(ip.into());
        } else {
            log::warn!("host interface {if_name} ({if_idx}) not found during netlink address remove event. Route was not removed.");
        }
    }

    fn process_lldp_tlvs_received(&mut self, if_idx: u32, lldp_tlvs: LLDPTLVs) {
        let if_name = netlink::get_interface_name(if_idx).unwrap_or("unknown".to_string());
        // find the host interface
        // and update the TLVs in there
        let mut found = false;
        'outer: for phy_port in self.ports.iter_mut() {
            for log_port in phy_port.ports.iter_mut() {
                for hif in log_port.hif.iter_mut() {
                    if hif.idx == if_idx {
                        found = true;
                        hif.lldp_tlvs = Some(lldp_tlvs);
                        break 'outer;
                    }
                }
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
        'outer: for phy_port in self.ports.iter_mut() {
            for log_port in phy_port.ports.iter_mut() {
                for hif in log_port.hif.iter_mut() {
                    if hif.idx == if_idx {
                        found = true;
                        hif.lldp_network_config = Some(config);
                        break 'outer;
                    }
                }
            }
        }
        if !found {
            log::warn!("host interface {if_name} ({if_idx}) not found during LLDP network config event. Discovered LLDP Network Config was not stored.");
        }
    }

    pub(crate) fn add_route(&mut self, route: IpNet) {
        if self
            .routes
            .iter()
            .find(|route_entry| **route_entry == route)
            .is_some()
        {
            log::debug!("route entry already exists: {route}. Not adding anymore.");
            return;
        }

        // if not, we program it in our router
        match self.virtual_router.create_route_entry(
            route,
            vec![
                RouteEntryAttribute::PacketAction(PacketAction::Forward),
                RouteEntryAttribute::NextHopID(self.cpu_port_id.into()),
            ],
        ) {
            Ok(route_entry) => {
                // if programming is successful, we add the route to our list
                log::info!(
                    "added route entry {:?} for ourselves on virtual router {}",
                    route_entry,
                    self.virtual_router
                );
                self.routes.push(route_entry);
            }
            Err(e) => {
                log::error!(
                    "failed to create route entry for ourselves on virtual router {}: {e:?}",
                    self.virtual_router
                );
            }
        };
    }

    pub(crate) fn remove_route(&mut self, route: IpNet) {
        if let Some(idx) = self
            .routes
            .iter()
            .position(|route_entry| *route_entry == route)
        {
            let route_entry = self.routes.remove(idx);
            match route_entry.remove() {
                Ok(_) => {
                    log::info!(
                        "removed route entry {:?} for ourselves from virtual router {}",
                        route,
                        self.virtual_router
                    );
                }
                Err(e) => {
                    // NOTE: there is a problem here that cannot really be solved.
                    // If we cannot remove the route entry, our state is screwed up
                    // we keep it out of the list because we don't really know what
                    // the error means anyways.
                    log::error!(
                        "failed to remove route entry from virtual router {}: {e:?}",
                        self.virtual_router
                    );
                }
            }
        }
    }
}

impl<'a, 'b> Drop for Processor<'a, 'b> {
    fn drop(&mut self) {
        // TODO: the `clone()`s here are ugly, but there is no real good other solution (that I know of)
        log::info!("Shutting down ONIE SAI processor...");

        // removing CPU host interface
        let cpu_hostif_id = self.cpu_hostif.to_id();
        match self.cpu_hostif.clone().remove() {
            Ok(_) => log::info!("removed CPU host interface {}", cpu_hostif_id),
            Err(e) => log::error!(
                "failed to remove CPU host interface {}: {:?}",
                cpu_hostif_id,
                e
            ),
        };

        // removing host interfaces for all ports
        for phy_port in self.ports.clone() {
            for port in phy_port.ports {
                let port_id = port.port.to_id();
                if let Some(hif) = port.hif {
                    let hif_id = hif.intf.to_id();
                    match hif.intf.remove() {
                        Ok(_) => {
                            log::info!("removed host interface {} for port {}", hif_id, port_id)
                        }
                        Err(e) => log::error!(
                            "failed to remove host interface {} for port {}: {:?}",
                            hif_id,
                            port_id,
                            e
                        ),
                    }
                }
            }
        }
    }
}

fn until_err<T, E>(err: &mut &mut Result<(), E>, item: Result<T, E>) -> Option<T> {
    match item {
        Ok(item) => Some(item),
        Err(e) => {
            **err = Err(e);
            None
        }
    }
}
