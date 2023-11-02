pub(crate) mod discovery;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

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

use serde::{Deserialize, Serialize};

use sai::hostif::HostIf;
use sai::port::BreakoutModeType;
use sai::port::Port;

use crate::oniesai::netlink;

use super::PlatformContextHolder;

#[derive(Debug, Error)]
pub(crate) enum PortError {
    #[error("SAI command failed: {0}")]
    SAIError(sai::Error),

    #[error("transceiver platform library call failed: {0}")]
    XcvrError(xcvr::Error),

    #[error("the number of port config entries does not match the number of ports ({0} != {1})")]
    PortConfigLengthMismatch(usize, usize),

    #[error("Invalid port config file. No matching port found for lane mapping: {0:?}")]
    PortConfigInvalid(Vec<u32>),

    #[error("port config unavailable")]
    PortConfigUnavailable,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PhysicalPortConfigSpeed {
    #[serde(rename = "FourLanes")]
    four_lanes: Option<u32>,
    #[serde(rename = "TwoLanes")]
    two_lanes: Option<u32>,
    #[serde(rename = "OneLane")]
    one_lane: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PhysicalPortConfig {
    pub(crate) lanes: Vec<u32>,
    pub(crate) speed: PhysicalPortConfigSpeed,
}

impl PhysicalPortConfig {
    #[allow(dead_code)]
    pub(crate) fn get_default_speed(&self) -> u32 {
        match self.lanes.len() {
            1 => match self.speed.one_lane {
                Some(v) => v,
                None => {
                    log::error!(
                        "Physical Port Config: missing OneLane speed configuration. Returning 10000.",
                    );
                    10000
                }
            },
            2 => match self.speed.two_lanes {
                Some(v) => v,
                None => {
                    log::error!(
                        "Physical Port Config: missing TwoLanes speed configuration. Returning 50000.",
                    );
                    50000
                }
            },
            4 => match self.speed.four_lanes {
                Some(v) => v,
                None => {
                    log::error!(
                        "Physical Port Config: missing FourLanes speed configuration. Returning 100000.",
                    );
                    100000
                }
            },
            _ => {
                log::error!(
                    "Physical Port Config: invalid number of lanes: {}. Returning 0.",
                    self.lanes.len()
                );
                0
            }
        }
    }

    pub(crate) fn get_speed_for_breakout_mode_type(
        &self,
        breakout_mode_type: BreakoutModeType,
    ) -> u32 {
        match breakout_mode_type {
            BreakoutModeType::OneLane => match self.speed.one_lane {
                Some(v) => v,
                None => {
                    log::error!(
                        "Physical Port Config: missing OneLane speed configuration. Returning 10000.",
                    );
                    10000
                }
            },
            BreakoutModeType::TwoLanes => match self.speed.two_lanes {
                Some(v) => v,
                None => {
                    log::error!(
                        "Physical Port Config: missing TwoLanes speed configuration. Returning 50000.",
                    );
                    50000
                }
            },
            BreakoutModeType::FourLanes => match self.speed.four_lanes {
                Some(v) => v,
                None => {
                    log::error!(
                        "Physical Port Config: missing FourLanes speed configuration. Returning 100000.",
                    );
                    100000
                }
            },
            _ => {
                log::error!(
                    "Physical Port Config: invalid breakout mode type: {:?}. Returning 10000.",
                    breakout_mode_type
                );
                10000
            }
        }
    }

    // when the supported breakout mode type SAI property is not available
    // we will use this function to guess the available breakout modes
    // NOTE: on Broadcom SAI, this can only work if all possible breakouts
    // have actually been defined as inactive ports in the config.bcm
    pub(crate) fn get_supported_breakout_mode_types(&self) -> Vec<BreakoutModeType> {
        match self.lanes.len() {
            1 => {
                vec![BreakoutModeType::OneLane]
            }
            2 => {
                vec![BreakoutModeType::OneLane, BreakoutModeType::TwoLanes]
            }
            4 => {
                vec![
                    BreakoutModeType::FourLanes,
                    BreakoutModeType::FourLanes,
                    BreakoutModeType::FourLanes,
                ]
            }
            _ => {
                log::error!("Physical Port Config: invalid number of lanes: {}. Returning no breakout types.", self.lanes.len());
                vec![]
            }
        }
    }

    pub(crate) fn get_current_breakout_mode_type(&self) -> BreakoutModeType {
        Self::get_breakout_type_from_lanes(self.lanes.clone())
    }

    pub(crate) fn get_breakout_type_from_lanes(lanes: Vec<u32>) -> BreakoutModeType {
        match lanes.len() {
            1 => BreakoutModeType::OneLane,
            2 => BreakoutModeType::TwoLanes,
            4 => BreakoutModeType::FourLanes,
            _ => {
                log::error!("Physical Port Config: invalid number of lanes: {}. Returning unknown breakout type", lanes.len());
                BreakoutModeType::Unknown(lanes.len() as i32)
            }
        }
    }

    pub(crate) fn from_file(path: &PathBuf) -> Option<Vec<PhysicalPortConfig>> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                log::error!("failed to open physical port config file: {:?}", e);
                return None;
            }
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(e) => {
                log::error!("failed to read physical port config file: {:?}", e);
                return None;
            }
        };
        let config: Vec<PhysicalPortConfig> = match serde_json::from_str(&contents) {
            Ok(config) => config,
            Err(e) => {
                log::error!("failed to parse physical port config file: {:?}", e);
                return None;
            }
        };
        Some(config)
    }
}

pub(crate) trait SortPortsByLanes<'a> {
    fn sort_ports_by_lanes(&self, ports: &Vec<Port<'a>>) -> Result<Vec<Port<'a>>, PortError>;
}

impl<'a> SortPortsByLanes<'a> for Vec<PhysicalPortConfig> {
    fn sort_ports_by_lanes(&self, ports: &Vec<Port<'a>>) -> Result<Vec<Port<'a>>, PortError> {
        if self.len() != ports.len() {
            return Err(PortError::PortConfigLengthMismatch(self.len(), ports.len()));
        }
        let mut ret = Vec::with_capacity(ports.len());
        for pc in self {
            let mut found = false;
            for port in ports.iter() {
                let lanes = port.get_hw_lanes()?;
                if lanes == pc.lanes {
                    found = true;
                    ret.push(port.clone());
                }
            }
            if !found {
                return Err(PortError::PortConfigInvalid(pc.lanes.clone()));
            }
        }
        Ok(ret)
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
    pub(crate) oper_status: bool,
    pub(crate) port_config: Option<PhysicalPortConfig>,
}

// just a convenience conversion method for our RPC
impl From<&PhysicalPort<'_, '_>> for onie_sai_rpc::onie_sai::Port {
    fn from(port: &PhysicalPort) -> Self {
        let mut ret = onie_sai_rpc::onie_sai::Port::new();
        ret.id = port.idx as u32;
        ret.hw_lanes = port.lanes.clone();
        ret.oper_status = port.oper_status;
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
            ret_p.auto_neg = p.auto_negotiation;
            ret_p.host_intf = wrap_message_field(hif);
            ports.push(ret_p);
        }
        ret.ports = ports;
        ret
    }
}

impl<'a, 'b> PhysicalPort<'a, 'b> {
    pub(crate) fn from_port(
        xcvr_api: PlatformContextHolder<'b>,
        switch: Switch<'a>,
        router: VirtualRouter<'a>,
        mac_address: sai_mac_t,
        physical_port_index: usize,
        port: Port<'a>,
        port_config: Option<PhysicalPortConfig>,
    ) -> Result<PhysicalPort<'a, 'b>, PortError> {
        // get the transceiver state first
        let xcvr_present = xcvr_api.obj.get_presence(physical_port_index as u16)?;
        let xcvr_oper_status = if xcvr_present {
            Some(xcvr_api.obj.get_oper_status(physical_port_index as u16)?)
        } else {
            None
        };
        let xcvr_inserted_type = if xcvr_present {
            Some(
                xcvr_api
                    .obj
                    .get_inserted_port_type(physical_port_index as u16)?,
            )
        } else {
            None
        };
        let xcvr_supported_types = xcvr_api
            .obj
            .get_supported_port_types(physical_port_index as u16)?;

        // get the port attributes that we need for initialization
        // let oper_status = port.get_oper_status()?;
        let hw_lanes = port.get_hw_lanes()?;
        let current_breakout_mode = match port.get_current_breakout_mode() {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Physical Port {}: failed to get current breakout mode: {:?}. Guessing breakout mode from port config", physical_port_index, e);
                match &port_config {
                    Some(pc) => pc.get_current_breakout_mode_type(),
                    None => {
                        log::error!("Physical Port {}: no port config available. Port config must be available if it cannot be queried through SAI", physical_port_index);
                        return Err(PortError::PortConfigUnavailable);
                    }
                }
            }
        };
        let supported_breakout_modes = match port.get_supported_breakout_modes() {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Physical Port {}: failed to get supported breakout modes: {:?}. Guessing supported breakout modes from port config", physical_port_index, e);
                match &port_config {
                    Some(pc) => pc.get_supported_breakout_mode_types(),
                    None => {
                        log::error!("Physical Port {}: no port config available. Port config must be available if it cannot be queried through SAI", physical_port_index);
                        return Err(PortError::PortConfigUnavailable);
                    }
                }
            }
        };

        Ok(PhysicalPort {
            xcvr_api: xcvr_api.clone(),
            switch: switch.clone(),
            router: router.clone(),
            xcvr_present: xcvr_present,
            xcvr_inserted_type: xcvr_inserted_type,
            xcvr_oper_status: xcvr_oper_status,
            xcvr_supported_types: xcvr_supported_types,
            idx: physical_port_index,
            auto_discovery: false,
            auto_discovery_with_breakout: false,
            auto_discovery_counter: 0,
            sm: None,
            oper_status: false,
            lanes: hw_lanes.clone(),
            mac_address,
            current_breakout_mode: current_breakout_mode,
            supported_breakout_modes: supported_breakout_modes,
            port_config: port_config,
            ports: vec![LogicalPort::new(
                switch.clone(),
                router.clone(),
                hw_lanes,
                mac_address,
                port,
            )?],
        })
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

    pub(crate) fn create_port(&mut self, breakout_mode: BreakoutModeType, hw_lanes: Vec<u32>) {
        // try to get the speed from the port config
        // it's awkwared if there is no port config, so we'll just assume 10G for that
        let speed = self
            .port_config
            .as_ref()
            .map(|pc| pc.get_speed_for_breakout_mode_type(breakout_mode))
            .unwrap_or(10000);

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
        log::debug!("Physical Port {}: port created: {}", self.idx, port.to_id());

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
        log::debug!("Physical Port {}: removing all logical ports...", self.idx);
        let ports = std::mem::take(&mut self.ports);
        for port in ports.into_iter() {
            port.remove();
        }
    }

    fn initialize_state_machines(&mut self) {
        if self.sm.is_none() {
            self.sm = Some(discovery::physicalport::DiscoveryStateMachine::new(
                self.idx,
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
            self.create_hifs_and_rifs();
        }
    }

    pub(crate) fn disable_auto_discovery(&mut self) {
        self.auto_discovery = false;
        if self.sm.is_some() {
            log::info!("Physical Port {}: disabling auto discovery. Stopping and removing auto discovery state machine (port breakout discovery: {})", self.idx, self.auto_discovery_with_breakout);
        }
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
                    if sm.is_done_and_success() {
                        // this means that we have successfully brought up at least one logical port
                        // for this port. However, we should continue to try to bring up the other logical ports
                        // regardless, we consider this port operating
                        for port in self.ports.iter_mut() {
                            if let Some(sm) = &mut port.sm {
                                if sm.is_done() && !sm.is_done_and_success() {
                                    *sm = discovery::logicalport::DiscoveryStateMachine::new(
                                        &port.port,
                                        port.supported_speeds.clone(),
                                        port.speed,
                                        port.auto_negotiation,
                                    );
                                }
                                if !sm.is_done() {
                                    if sm.can_step() {
                                        *sm = sm.clone().step(
                                            &port.port,
                                            discovery::logicalport::Event::NoChange,
                                        );
                                        port.reconcile_state();
                                    }
                                }
                            }
                        }
                    } else {
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
                    log::debug!(
                        "Port {}: successfully created host interface {} ({})",
                        self.port,
                        name,
                        &hif
                    );
                    let idx = match netlink::get_interface_index(name.as_str()) {
                        Ok(idx) => idx,
                        Err(e) => {
                            log::error!(
                                "Port {}: failed to get interface index for {}: {:?}",
                                self.port,
                                name,
                                e
                            );
                            0
                        }
                    };
                    self.hif = Some(HostInterface {
                        intf: hif,
                        name: name,
                        idx: idx,
                        oper_status: false,
                    })
                }
                Err(e) => {
                    log::error!(
                        "Port {}: failed to create host interface {}: {:?}",
                        self.port,
                        name,
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
                Ok(rif) => {
                    log::debug!(
                        "Port {}: successfully created router interface {}",
                        self.port,
                        &rif
                    );
                    self.rif = Some(rif);
                }
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
                Ok(_) => {
                    log::debug!(
                        "Port {}: successfully removed host interface {}",
                        self.port,
                        hif.name
                    );
                }
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
                Ok(_) => {
                    log::debug!("Port {}: successfully removed router interface", self.port);
                }
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
            Ok(_) => {
                log::debug!("Port {}: successfully removed", port_id);
            }
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
    pub(crate) idx: u32,
    pub(crate) oper_status: bool,
}

impl<'a> HostInterface<'a> {
    pub(crate) fn set_netlink_oper_status(&self, oper_status: bool) {
        match netlink::set_link_status(self.idx, oper_status) {
            Ok(_) => {
                log::debug!(
                    "Host Interface {}: successfully set netlink oper status to {}",
                    self,
                    oper_status
                );
            }
            Err(e) => {
                log::error!(
                    "Host Interface {}: failed to set netlink oper status to {}: {:?}",
                    self,
                    oper_status,
                    e
                );
            }
        }
    }
}

impl<'a> std::fmt::Display for HostInterface<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.intf)
    }
}
