pub(crate) mod discovery;

use onie_sai_rpc::wrap_message_field;
use sai::hostif::HostIfAttribute;
use sai::hostif::HostIfType;
use sai::hostif::VlanTag;
use sai::router_interface::RouterInterface;
use sai::router_interface::RouterInterfaceAttribute;
use sai::router_interface::RouterInterfaceType;
use sai::sai_mac_t;
use sai::switch::Switch;
use sai::virtual_router::VirtualRouter;
use sai::ObjectID;
use thiserror::Error;

use sai::hostif::HostIf;
use sai::port::BreakoutModeType;
use sai::port::Port;

use super::PlatformContextHolder;

#[derive(Debug, Error)]
pub(crate) enum PortError {
    #[error("SAI command failed")]
    SAIError(sai::Error),

    #[error("transceiver platform library call failed")]
    XcvrError(xcvr::Error),
    // #[error("port validation failed")]
    // Validation,
}

impl From<sai::Error> for PortError {
    fn from(value: sai::Error) -> Self {
        PortError::SAIError(value)
    }
}

impl From<xcvr::Error> for PortError {
    fn from(value: xcvr::Error) -> Self {
        PortError::XcvrError(value)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PhysicalPort<'a, 'b> {
    xcvr_api: PlatformContextHolder<'b>,
    switch: Switch<'a>,
    router: VirtualRouter<'a>,
    pub(crate) idx: usize,
    pub(crate) ports: Vec<LogicalPort<'a>>,
    pub(crate) lanes: Vec<u32>,
    pub(crate) mac_address: sai_mac_t,
    pub(crate) auto_discovery: bool,
    pub(crate) auto_discovery_with_breakout: bool,
    pub(crate) auto_discovery_counter: u64,
    pub(crate) current_breakout_mode: BreakoutModeType,
    pub(crate) supported_breakout_modes: Vec<BreakoutModeType>,
    pub(crate) xcvr_present: bool,
    pub(crate) xcvr_oper_status: Option<bool>,
    pub(crate) xcvr_inserted_type: Option<xcvr::PortType>,
    pub(crate) xcvr_supported_types: Vec<xcvr::PortType>,
    pub(crate) sm: Option<discovery::physicalport::DiscoveryStateMachine>,
}

// just a convenience conversion method for our RPC
impl From<&PhysicalPort<'_, '_>> for onie_sai_rpc::onie_sai::Port {
    fn from(port: &PhysicalPort) -> Self {
        let mut ret = onie_sai_rpc::onie_sai::Port::new();
        ret.id = port.idx as u32;
        ret.hw_lanes = port.lanes.clone();
        ret.xcvr_present = port.xcvr_present;
        ret.xcvr_oper_status = port.xcvr_oper_status;
        ret.xcvr_inserted_type = port.xcvr_inserted_type.map(|t| format!("{:?}", t));
        ret.xcvr_supported_types = port
            .xcvr_supported_types
            .iter()
            .map(|t| format!("{:?}", t))
            .collect();
        let mut ports: Vec<onie_sai_rpc::onie_sai::LogicalPort> =
            Vec::with_capacity(port.ports.len());
        for p in &port.ports {
            let mut ret_p = onie_sai_rpc::onie_sai::LogicalPort::new();
            let hif: Option<onie_sai_rpc::onie_sai::HostInterface> = p.hif.as_ref().map(|hif| {
                let mut ret_hif = onie_sai_rpc::onie_sai::HostInterface::new();
                ret_hif.name = hif.name.clone();
                ret_hif.oper_status = hif.oper_status;
                ret_hif
            });
            ret_p.oid = p.port.to_string();
            ret_p.hw_lanes = p.lanes.clone();
            ret_p.oper_status = p.oper_status;
            ret_p.admin_state = p.admin_state;
            ret_p.speed = p.speed;
            ret_p.oper_speed = p.oper_speed;
            ret_p.supported_speeds = p.supported_speeds.clone();
            ret_p.host_intf = wrap_message_field(hif);
            ports.push(ret_p);
        }
        ret
    }
}

impl<'a, 'b> PhysicalPort<'a, 'b> {
    pub(crate) fn from_ports(
        xcvr_api: PlatformContextHolder<'b>,
        switch: Switch<'a>,
        router: VirtualRouter<'a>,
        mac_address: sai_mac_t,
        ports: Vec<Port<'a>>,
    ) -> Result<Vec<PhysicalPort<'a, 'b>>, PortError> {
        let mut ret: Vec<PhysicalPort<'a, 'b>> = Vec::with_capacity(ports.len());
        for (i, port) in ports.into_iter().enumerate() {
            // get the transceiver state first
            let xcvr_present = xcvr_api.obj.get_presence(i as u16)?;
            let xcvr_oper_status = if xcvr_present {
                Some(xcvr_api.obj.get_oper_status(i as u16)?)
            } else {
                None
            };
            let xcvr_inserted_type = if xcvr_present {
                Some(xcvr_api.obj.get_inserted_port_type(i as u16)?)
            } else {
                None
            };
            let xcvr_supported_types = xcvr_api.obj.get_supported_port_types(i as u16)?;

            // get the port attributes that we need for initialization
            // let oper_status = port.get_oper_status()?;
            let hw_lanes = port.get_hw_lanes()?;
            let current_breakout_mode = port.get_current_breakout_mode()?;
            let supported_breakout_modes = port.get_supported_breakout_modes()?;

            ret.push(PhysicalPort {
                xcvr_api: xcvr_api.clone(),
                switch: switch.clone(),
                router: router.clone(),
                xcvr_present: xcvr_present,
                xcvr_inserted_type: xcvr_inserted_type,
                xcvr_oper_status: xcvr_oper_status,
                xcvr_supported_types: xcvr_supported_types,
                idx: i,
                auto_discovery: false,
                auto_discovery_with_breakout: false,
                auto_discovery_counter: 0,
                sm: None,
                lanes: hw_lanes.clone(),
                mac_address,
                current_breakout_mode: current_breakout_mode,
                supported_breakout_modes: supported_breakout_modes,
                ports: vec![LogicalPort::new(
                    switch.clone(),
                    router.clone(),
                    hw_lanes,
                    mac_address,
                    port,
                )?],
            })
        }
        Ok(ret)
    }

    fn xcvr_reconcile_state(&mut self) {
        let xcvr_present = match self.xcvr_api.obj.get_presence(self.idx as u16) {
            Ok(v) => v,
            Err(e) => {
                log::error!(
                    "Physical Port {}: transceiver presence check failed: {:?}",
                    self.idx,
                    e
                );
                return;
            }
        };

        if xcvr_present {
            match self.xcvr_api.obj.get_oper_status(self.idx as u16) {
                Ok(v) => {
                    self.xcvr_oper_status = Some(v);
                }
                Err(e) => {
                    log::error!(
                        "Physical Port {}: transceiver oper status check failed: {:?}",
                        self.idx,
                        e
                    );
                }
            };
            match self.xcvr_api.obj.get_inserted_port_type(self.idx as u16) {
                Ok(v) => {
                    self.xcvr_inserted_type = Some(v);
                }
                Err(e) => {
                    log::error!(
                        "Physical Port {}: transceiver inserted type check failed: {:?}",
                        self.idx,
                        e
                    );
                }
            };
        }

        // NOTE: supported types don't change for physical ports ever, so no need to recheck again
    }

    pub(crate) fn create_hifs_and_rifs(&mut self) {
        for (i, port) in self.ports.iter_mut().enumerate() {
            let name = format!("Ethernet{}-{}", self.idx, i);
            port.create_hif_and_rif(name);
        }
    }

    pub(crate) fn remove_hifs_and_rifs(&mut self) {
        for port in self.ports.iter_mut() {
            port.remove_hif_and_rif();
        }
    }

    pub(crate) fn create_port(&mut self, hw_lanes: Vec<u32>) {
        // TODO: we gotta deal with the speed somehow
        let speed = 10000;

        // create the port with SAI
        let port = match self.switch.create_port(hw_lanes.clone(), speed) {
            Ok(port) => port,
            Err(e) => {
                log::error!(
                    "Physical Port {}: port creation failed for lanes {:?} and speed {}: {:?}",
                    self.idx,
                    hw_lanes,
                    speed,
                    e
                );
                return;
            }
        };

        // create our "logical port" which will query certain basics
        let port = match LogicalPort::new(
            self.switch.clone(),
            self.router.clone(),
            hw_lanes.clone(),
            self.mac_address,
            port,
        ) {
            Ok(port) => port,
            Err(e) => {
                log::error!(
                    "Physical Port {}: logical port creation failed for lanes {:?}: {:?}",
                    self.idx,
                    hw_lanes,
                    e
                );
                return;
            }
        };

        // add the port to our list of logical ports
        self.ports.push(port);
    }

    pub(crate) fn remove_ports(&mut self) {
        let ports = std::mem::take(&mut self.ports);
        for port in ports.into_iter() {
            port.remove();
        }
    }

    fn initialize_state_machines(&mut self) {
        if self.sm.is_none() {
            self.sm = Some(discovery::physicalport::DiscoveryStateMachine::new(
                self.auto_discovery_with_breakout,
                self.current_breakout_mode.clone(),
                self.supported_breakout_modes.clone(),
            ));
        }
        for port in self.ports.iter_mut() {
            if port.sm.is_none() {
                port.sm = Some(discovery::logicalport::DiscoveryStateMachine::new(
                    &port.port,
                    port.supported_speeds.clone(),
                    port.speed,
                    port.auto_negotiation,
                ));
            }
        }
    }

    fn destroy_state_machines(&mut self) {
        if self.sm.is_some() {
            self.sm = None;
        }
        for port in self.ports.iter_mut() {
            if port.sm.is_some() {
                port.sm = None;
            }
        }
    }

    pub(crate) fn enable_auto_discovery(&mut self, auto_discovery_with_breakout: bool) {
        self.auto_discovery = true;
        self.auto_discovery_with_breakout = auto_discovery_with_breakout;
        if self.xcvr_present {
            if self.sm.is_none() {
                log::info!("Physical Port {}: transceiver presence detected. Initializing auto discovery state machine (port breakout discovery: {})", self.idx, self.auto_discovery_with_breakout);
            }
            self.initialize_state_machines();
        }
    }

    pub(crate) fn disable_auto_discovery(&mut self) {
        self.auto_discovery = false;
        self.destroy_state_machines();
    }

    pub(crate) fn auto_discovery_poll(&mut self) {
        if self.auto_discovery {
            // poll on xcvr state first
            self.xcvr_reconcile_state();

            // create and/or remove host interfaces and router interfaces
            // depending on if a transceiver is present or not
            // and initialize state machines for polling
            // NOTE: these functions are safe to be called in cases
            // when HIFs and RIFs are already created and/or removed
            if self.xcvr_present {
                if self.sm.is_none() {
                    log::info!("Physical Port {}: transceiver presence detected. Initializing auto discovery state machine (port breakout discovery: {})", self.idx, self.auto_discovery_with_breakout);
                }
                self.initialize_state_machines();
                self.create_hifs_and_rifs();
            } else {
                if self.sm.is_some() {
                    log::info!("Physical Port {}: transceiver presence lost. Stopping and removing auto discovery state machine (port breakout discovery: {})", self.idx, self.auto_discovery_with_breakout);
                }
                self.destroy_state_machines();
                self.auto_discovery_counter = 0;
                self.remove_hifs_and_rifs();
            }

            if let Some(sm) = &self.sm {
                if sm.is_done() {
                    if !sm.is_done_and_success() {
                        // if the state machine was not successful, then we simply start over
                        // and log that
                        self.auto_discovery_counter += 1;
                        log::warn!(
                            "Physical Port {}: auto discovery did not complete successfully. Restarting it... (counter: {})",
                            self.idx,
                            self.auto_discovery_counter
                        );
                        self.destroy_state_machines();
                        self.initialize_state_machines();
                    }
                } else {
                    // if the state machine is not done yet, we'll call step if we can
                    // first on the logical ports
                    for port in self.ports.iter_mut() {
                        if let Some(sm) = &mut port.sm {
                            if !sm.is_done() {
                                if sm.can_step() {
                                    *sm = sm
                                        .clone()
                                        .step(&port.port, discovery::logicalport::Event::NoChange);
                                    port.reconcile_state();
                                }
                            }
                        }
                    }

                    // and then on the physical port state machine
                    if sm.can_step(self) {
                        self.sm = Some(sm.clone().step(self));
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LogicalPort<'a> {
    switch: Switch<'a>,
    router: VirtualRouter<'a>,
    pub(crate) port: Port<'a>,
    pub(crate) hif: Option<HostInterface<'a>>,
    pub(crate) rif: Option<RouterInterface<'a>>,
    pub(crate) lanes: Vec<u32>,
    pub(crate) mac_address: sai_mac_t,
    pub(crate) oper_status: bool,
    pub(crate) admin_state: bool,
    pub(crate) auto_negotiation: bool,
    pub(crate) speed: u32,
    pub(crate) oper_speed: u32,
    pub(crate) supported_speeds: Vec<u32>,
    pub(crate) sm: Option<discovery::logicalport::DiscoveryStateMachine>,
}

impl<'a> LogicalPort<'a> {
    pub(crate) fn new(
        switch: Switch<'a>,
        router: VirtualRouter<'a>,
        hw_lanes: Vec<u32>,
        mac_address: sai_mac_t,
        port: Port<'a>,
    ) -> Result<Self, PortError> {
        let oper_status: bool = port.get_oper_status()?.into();
        let admin_state = port.get_admin_state()?;
        let auto_negotiation = port.get_auto_neg_mode()?;
        let speed = port.get_speed()?;
        let oper_speed = port.get_oper_speed()?;
        let supported_speeds = port.get_supported_speeds()?;
        Ok(Self {
            switch: switch,
            router: router,
            port: port,
            hif: None,
            rif: None,
            lanes: hw_lanes,
            mac_address: mac_address,
            oper_status: oper_status,
            admin_state: admin_state,
            auto_negotiation: auto_negotiation,
            speed: speed,
            oper_speed: oper_speed,
            supported_speeds: supported_speeds,
            sm: None,
        })
    }

    pub(crate) fn reconcile_state(&mut self) {
        let _ = self
            .port
            .get_oper_status()
            .map(|v| self.oper_status = v.into())
            .map_err(|e| log_port_error(&self.port, e));
        let _ = self
            .port
            .get_admin_state()
            .map(|v| self.admin_state = v)
            .map_err(|e| log_port_error(&self.port, e));
        let _ = self
            .port
            .get_auto_neg_mode()
            .map(|v| self.auto_negotiation = v)
            .map_err(|e| log_port_error(&self.port, e));
        let _ = self
            .port
            .get_speed()
            .map(|v| self.speed = v)
            .map_err(|e| log_port_error(&self.port, e));
        let _ = self
            .port
            .get_oper_speed()
            .map(|v| self.oper_speed = v)
            .map_err(|e| log_port_error(&self.port, e));
    }

    pub(crate) fn create_hif_and_rif(&mut self, name: String) {
        if self.hif.is_none() {
            match self.switch.create_hostif(vec![
                HostIfAttribute::Name(name.clone()),
                HostIfAttribute::Type(HostIfType::Netdev),
                HostIfAttribute::ObjectID(self.port.to_id().into()),
                HostIfAttribute::VlanTag(VlanTag::Original),
                HostIfAttribute::OperStatus(false),
            ]) {
                Ok(hif) => {
                    self.hif = Some(HostInterface {
                        intf: hif,
                        name: name,
                        oper_status: false,
                    })
                }
                Err(e) => {
                    log::error!(
                        "Port {}: failed to create host interface {}: {:?}",
                        name,
                        self.port,
                        e
                    );
                }
            }
        }
        if self.rif.is_none() {
            match self.router.create_router_interface(vec![
                RouterInterfaceAttribute::SrcMacAddress(self.mac_address),
                RouterInterfaceAttribute::Type(RouterInterfaceType::Port),
                RouterInterfaceAttribute::PortID(self.port.to_id().into()),
                RouterInterfaceAttribute::MTU(9100),
                RouterInterfaceAttribute::NATZoneID(0),
            ]) {
                Ok(rif) => self.rif = Some(rif),
                Err(e) => {
                    log::error!(
                        "Port {}: failed to create router interface: {:?}",
                        self.port,
                        e
                    );
                }
            }
        }
    }

    pub(crate) fn remove_hif_and_rif(&mut self) {
        if let Some(hif) = self.hif.take() {
            match hif.intf.remove() {
                Ok(_) => {}
                Err(e) => {
                    log::error!(
                        "Port {}: failed to remove host interface {}: {:?}",
                        self.port,
                        hif.name,
                        e
                    );
                }
            }
        }
        if let Some(rif) = self.rif.take() {
            match rif.remove() {
                Ok(_) => {}
                Err(e) => {
                    log::error!(
                        "Port {}: failed to remove router interface: {:?}",
                        self.port,
                        e
                    );
                }
            }
        }
    }

    pub(crate) fn remove(self) {
        let mut s = self;
        s.remove_hif_and_rif();
        let port_id = s.port.to_id();
        match s.port.remove() {
            Ok(_) => {}
            Err(e) => {
                log::error!("Port {}: failed to remove port: {:?}", port_id, e);
            }
        }
    }
}

fn log_port_error(port: &Port<'_>, e: sai::Error) {
    log::error!("Port {}: SAI command failed: {:?}", port, e);
}

#[derive(Debug, Clone)]
pub(crate) struct HostInterface<'a> {
    pub(crate) intf: HostIf<'a>,
    pub(crate) name: String,
    pub(crate) oper_status: bool,
}

impl<'a> std::fmt::Display for HostInterface<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.intf)
    }
}
