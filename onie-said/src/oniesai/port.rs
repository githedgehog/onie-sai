use thiserror::Error;

use sai::hostif::HostIf;
use sai::port::Port;
use sai::router_interface::RouterInterface;

#[derive(Debug, Error)]
pub(super) enum PortError {
    #[error("SAI command failed")]
    SAIError(sai::Error),
    // #[error("port validation failed")]
    // Validation,
}

impl From<sai::Error> for PortError {
    fn from(value: sai::Error) -> Self {
        PortError::SAIError(value)
    }
}

#[derive(Debug, Clone)]
pub(super) struct PhysicalPort<'a> {
    pub(super) idx: usize,
    pub(super) ports: Vec<LogicalPort<'a>>,
    pub(super) lanes: Vec<u32>,
}

impl<'a> PhysicalPort<'a> {
    pub(super) fn from_ports(ports: Vec<Port<'a>>) -> Result<Vec<PhysicalPort<'a>>, PortError> {
        let mut ret: Vec<PhysicalPort<'a>> = Vec::with_capacity(ports.len());
        for (i, port) in ports.into_iter().enumerate() {
            let hw_lanes = port.get_hw_lanes()?;
            ret.push(PhysicalPort {
                idx: i,
                ports: vec![LogicalPort {
                    port: port,
                    hif: None,
                    rif: None,
                }],
                lanes: hw_lanes,
            })
        }
        Ok(ret)
    }
}

#[derive(Debug, Clone)]
pub(super) struct LogicalPort<'a> {
    pub(super) port: Port<'a>,
    pub(super) hif: Option<HostInterface<'a>>,
    pub(super) rif: Option<RouterInterface<'a>>,
}

#[derive(Debug, Clone)]
pub(super) struct HostInterface<'a> {
    pub(super) name: String,
    pub(super) intf: HostIf<'a>,
}

impl<'a> std::fmt::Display for HostInterface<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.intf)
    }
}
