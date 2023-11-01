pub(crate) mod port;

use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
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

use crate::oniesai::port::SortPortsByLanes;

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
    AutoDiscoveryPoll,
    AutoDiscoveryStatus(
        (
            onie_sai::AutoDiscoveryRequest,
            Sender<Result<onie_sai::AutoDiscoveryResponse, ProcessError>>,
        ),
    ),
    LogicalPortStateChange((PortID, OperStatus)),
}

pub(crate) struct Processor<'a, 'b> {
    auto_discovery: bool,
    auto_discovery_with_breakout: bool,
    switch: Switch<'a>,
    virtual_router: VirtualRouter<'a>,
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

        // remove default vlan members
        let default_vlan = switch
            .get_default_vlan()
            .context("failed to get default VLAN")?;
        log::info!("default VLAN of switch {} is: {:?}", switch, default_vlan);
        let members = default_vlan.get_members().context(format!(
            "failed to get VLAN members for default VLAN {}",
            default_vlan
        ))?;
        for member in members {
            log::info!("Removing VLAN member {}...", member);
            member.remove().context("failed to remove VLAN member")?;
        }

        // remove default bridge ports
        let default_bridge = switch
            .get_default_bridge()
            .context("failed to get dfeault bridge")?;
        log::info!(
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

            log::info!("removing bridge port {}...", bridge_port);
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

        let _default_route_entry = default_virtual_router
            .create_route_entry(
                IpNet::from_str("0.0.0.0/0").unwrap(),
                vec![RouteEntryAttribute::PacketAction(PacketAction::Drop)],
            )
            .context(format!(
                "failed to create default route entry for virtual router {}",
                default_virtual_router
            ))?;

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
                port.enable_auto_discovery(auto_discovery_with_breakout)
            }
        }

        Ok(Processor {
            auto_discovery: auto_discovery,
            auto_discovery_with_breakout: auto_discovery_with_breakout,
            switch: switch,
            virtual_router: default_virtual_router,
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
                        log::error!(
                            "processor: failed to send version response to rpc server: {:?}",
                            e
                        );
                    };
                }
                ProcessRequest::Shell((r, resp_tx)) => {
                    let resp = p.process_shell_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!(
                            "processor: failed to send shell response to rpc server: {:?}",
                            e
                        );
                    };
                }
                ProcessRequest::PortList((r, resp_tx)) => {
                    let resp = p.process_port_list_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!(
                            "processor: failed to send port list response to rpc server: {:?}",
                            e
                        );
                    };
                }
                ProcessRequest::AutoDiscoveryStatus((r, resp_tx)) => {
                    let resp = p.process_auto_discovery_status_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!(
                            "processor: failed to send auto discovery status response to rpc server: {:?}",
                            e
                        );
                    };
                }

                // internal events
                ProcessRequest::AutoDiscoveryPoll => p.process_auto_discovery_poll(),
                ProcessRequest::LogicalPortStateChange((port_id, port_state)) => {
                    p.process_logical_port_state_change(port_id, port_state)
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
        log::warn!("processor: shell requested, this blocks the processor thread!");
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
        log::warn!("processor: shell finished, processor thread unblocked!");
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

    fn process_auto_discovery_poll(&mut self) {
        log::debug!("processor: auto discovery poll");
        for phy_port in self.ports.iter_mut() {
            phy_port.auto_discovery_poll();
        }
    }

    fn process_logical_port_state_change(&mut self, port_id: PortID, port_state: OperStatus) {
        let mut found = false;
        for phy_port in self.ports.iter_mut() {
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
                                log::debug!(
                                    "processor: port {} has no discovery state machine",
                                    port_id
                                );
                            }
                        }
                    }

                    // reconcile state
                    log_port.reconcile_state();

                    // update the associated host interface
                    match log_port.hif.as_mut() {
                        Some(hif) => {
                            match hif.intf.set_oper_status(oper_status) {
                                Ok(_) => {
                                    log::info!("processor: set host interface {} ({}) operational status to {} for port {}", hif.name, hif.intf, oper_status, port_id);
                                    hif.oper_status = oper_status;
                                }
                                Err(e) => log::error!("processor: failed to set host interface {} ({}) operational status to {} for port {}: {:?}", hif.name, hif.intf, oper_status, port_id, e),
                            }
                        }
                        None => log::warn!(
                            "processor: port {} has no associated host interface created yet",
                            port_id
                        ),
                    }
                }
            }
        }
        if !found {
            log::warn!(
                "processor: port {} not found during port state change event",
                port_id
            );
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
            Ok(_) => log::info!("processor: removed CPU host interface {}", cpu_hostif_id),
            Err(e) => log::error!(
                "processor: failed to remove CPU host interface {}: {:?}",
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
                        Ok(_) => log::info!(
                            "processor: removed host interface {} for port {}",
                            hif_id,
                            port_id
                        ),
                        Err(e) => log::error!(
                            "processor: failed to remove host interface {} for port {}: {:?}",
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

// let _myip_route_entry = match default_virtual_router.create_route_entry(
//     IpNet::from_str("10.10.10.1/32").unwrap(),
//     vec![
//         RouteEntryAttribute::PacketAction(PacketAction::Forward),
//         RouteEntryAttribute::NextHopID(cpu_port_id.into()),
//     ],
// ) {
//     Ok(v) => v,
//     Err(e) => {
//         log::error!(
//             "failed to create route entry for ourselves for virtual router {}: {:?}",
//             default_virtual_router,
//             e
//         );
//         return ExitCode::FAILURE;
//     }
// };

// let mut hostifs: Vec<HostIf> = Vec::with_capacity(ports.len());
// for (i, port) in ports.into_iter().enumerate() {
//     let port_id = port.to_id();
//     // create host interface
//     let hostif = match switch.create_hostif(vec![
//         HostIfAttribute::Type(HostIfType::Netdev),
//         HostIfAttribute::ObjectID(port_id.into()),
//         HostIfAttribute::Name(format!("Ethernet{}", i)),
//     ]) {
//         Ok(v) => v,
//         Err(e) => {
//             log::error!(
//                 "failed to create host interface for port {} on switch {}: {:?}",
//                 port,
//                 switch,
//                 e
//             );
//             return ExitCode::FAILURE;
//         }
//     };
//     hostifs.push(hostif.clone());

//     // check supported speeds, and set port to 10G if possible
//     match port.get_supported_speeds() {
//         Err(e) => {
//             log::error!(
//                 "failed to query port {} for supported speeds: {:?}",
//                 port,
//                 e
//             );
//         }
//         Ok(speeds) => {
//             if !speeds.contains(&10000) {
//                 log::warn!("port {} does not support 10G, only {:?}", port, speeds)
//             } else {
//                 match port.set_speed(10000) {
//                     Ok(_) => {
//                         log::info!("set port speed to 10G for port {}", port);
//                     }
//                     Err(e) => {
//                         log::error!(
//                             "failed to set port speed to 10G for port {}: {:?}",
//                             port,
//                             e
//                         );
//                     }
//                 }
//             }
//         }
//     }

//     // set port up
//     match port.set_admin_state(true) {
//         Ok(_) => {
//             log::info!("set admin state to true for port {}", port);
//         }
//         Err(e) => {
//             log::error!(
//                 "failed to set admin state to true for port {}: {:?}",
//                 port,
//                 e
//             );
//         }
//     }

//     // allow vlan tags on host interfaces
//     match hostif.set_vlan_tag(VlanTag::Original) {
//         Ok(_) => {
//             log::info!(
//                 "set vlan tag to keep original for host interface {}",
//                 hostif
//             );
//         }
//         Err(e) => {
//             log::error!(
//                 "failed to set vlan tag to keep original for host interface {}: {:?}",
//                 hostif,
//                 e
//             );
//         }
//     }

//     // bring host interface up
//     match hostif.set_oper_status(true) {
//         Ok(_) => {
//             log::info!("set oper status to true for host interface {}", hostif);
//         }
//         Err(e) => {
//             log::error!(
//                 "failed to set oper status to true for host interface {}: {:?}",
//                 hostif,
//                 e
//             );
//         }
//     }

//     // create router interface
//     match default_virtual_router.create_router_interface(vec![
//         RouterInterfaceAttribute::SrcMacAddress(mac_address),
//         RouterInterfaceAttribute::Type(RouterInterfaceType::Port),
//         RouterInterfaceAttribute::PortID(port.into()),
//         RouterInterfaceAttribute::MTU(9100),
//         RouterInterfaceAttribute::NATZoneID(0),
//     ]) {
//         Ok(v) => {
//             log::info!("successfully created router interface {}", v);
//         }
//         Err(e) => {
//             log::error!("failed create router interface: {:?}", e);
//         }
//     }
// }

// match switch.enable_shell() {
//     Ok(_) => {}
//     Err(e) => {
//         log::error!("failed to enter switch shell: {:?}", e);
//     }
// }

// shutdown: remove things again
// hostifs.push(cpu_intf);
// for hostif in hostifs {
//     let id = hostif.to_id();
//     match hostif.remove() {
//         Ok(_) => {
//             log::info!("successfully removed host interface {}", id);
//         }
//         Err(e) => {
//             log::error!("failed to remove host interface {}: {:?}", id, e);
//         }
//     }
// }

// match switch.remove() {
//     Ok(_) => {
//         log::info!("successfully removed switch");
//     }
//     Err(e) => {
//         log::error!("failed to remove switch: {:?}", e);
//     }
// }
