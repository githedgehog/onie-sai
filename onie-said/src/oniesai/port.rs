pub(crate) mod discovery;

use onie_sai_rpc::wrap_message_field;
use thiserror::Error;

use sai::hostif::HostIf;
use sai::port::BreakoutModeType;
use sai::port::Port;
use sai::router_interface::RouterInterface;

use super::PlatformContextHolder;

#[derive(Debug, Error)]
pub(super) enum PortError {
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
pub(super) struct PhysicalPort<'a, 'b> {
    xcvr_api: PlatformContextHolder<'b>,
    pub(super) idx: usize,
    pub(super) ports: Vec<LogicalPort<'a>>,
    pub(super) lanes: Vec<u32>,
    pub(super) current_breakout_mode: BreakoutModeType,
    pub(super) supported_breakout_modes: Vec<BreakoutModeType>,
    pub(super) xcvr_present: bool,
    pub(super) xcvr_oper_status: Option<bool>,
    pub(super) xcvr_inserted_type: Option<xcvr::PortType>,
    pub(super) xcvr_supported_types: Vec<xcvr::PortType>,
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
    pub(super) fn from_ports(
        xcvr_api: PlatformContextHolder<'b>,
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
                xcvr_present: xcvr_present,
                xcvr_inserted_type: xcvr_inserted_type,
                xcvr_oper_status: xcvr_oper_status,
                xcvr_supported_types: xcvr_supported_types,
                idx: i,
                lanes: hw_lanes.clone(),
                current_breakout_mode: current_breakout_mode,
                supported_breakout_modes: supported_breakout_modes,
                ports: vec![LogicalPort::new(hw_lanes, port)?],
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

    pub(super) fn enable_auto_discovery(&mut self) {
        if self.xcvr_present {
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
    }

    pub(super) fn disable_auto_discovery(&mut self) {
        for port in self.ports.iter_mut() {
            port.sm = None;
        }
    }

    pub(super) fn auto_discovery_poll(&mut self) {
        // poll on xcvr state first
        self.xcvr_reconcile_state();
    }
}

#[derive(Debug, Clone)]
pub(super) struct LogicalPort<'a> {
    pub(super) port: Port<'a>,
    pub(super) hif: Option<HostInterface<'a>>,
    pub(super) rif: Option<RouterInterface<'a>>,
    pub(super) lanes: Vec<u32>,
    pub(super) oper_status: bool,
    pub(super) admin_state: bool,
    pub(super) auto_negotiation: bool,
    pub(super) speed: u32,
    pub(super) oper_speed: u32,
    pub(super) supported_speeds: Vec<u32>,
    pub(super) sm: Option<discovery::logicalport::DiscoveryStateMachine>,
}

impl<'a> LogicalPort<'a> {
    pub(super) fn new(hw_lanes: Vec<u32>, port: Port<'a>) -> Result<Self, PortError> {
        let oper_status: bool = port.get_oper_status()?.into();
        let admin_state = port.get_admin_state()?;
        let auto_negotiation = port.get_auto_neg_mode()?;
        let speed = port.get_speed()?;
        let oper_speed = port.get_oper_speed()?;
        let supported_speeds = port.get_supported_speeds()?;
        Ok(Self {
            port: port,
            hif: None,
            rif: None,
            lanes: hw_lanes,
            oper_status: oper_status,
            admin_state: admin_state,
            auto_negotiation: auto_negotiation,
            speed: speed,
            oper_speed: oper_speed,
            supported_speeds: supported_speeds,
            sm: None,
        })
    }

    pub(super) fn reconcile_state(&mut self) {
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
}

fn log_port_error(port: &Port<'_>, e: sai::Error) {
    log::error!("Port {}: SAI command failed: {:?}", port, e);
}

#[derive(Debug, Clone)]
pub(super) struct HostInterface<'a> {
    pub(super) intf: HostIf<'a>,
    pub(super) name: String,
    pub(super) oper_status: bool,
}

impl<'a> std::fmt::Display for HostInterface<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.intf)
    }
}
