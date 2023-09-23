use crate::*;
use sai_sys::*;

#[derive(Clone)]
pub struct BridgePort<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for BridgePort<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BridgePort(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for BridgePort<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> BridgePort<'a> {
    pub fn get_type(&self) -> Result<Type, Error> {
        // check that API is available/callable
        let bridge_api = self.sai.bridge_api().ok_or(Error::APIUnavailable)?;
        let get_bridge_port_attribute = bridge_api
            .get_bridge_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_bridge_port_attr_t_SAI_BRIDGE_PORT_ATTR_TYPE,
            value: sai_attribute_value_t { u32_: 0 },
        };

        let st = unsafe { get_bridge_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        let bridge_port_type = unsafe { attr.value.u32_ };
        Ok(Type::from(bridge_port_type))
    }

    pub fn remove(self) -> Result<(), Error> {
        // check that API is available/callable
        let bridge_api = self.sai.bridge_api().ok_or(Error::APIUnavailable)?;
        let remove_bridge_port = bridge_api
            .remove_bridge_port
            .ok_or(Error::APIFunctionUnavailable)?;

        match unsafe { remove_bridge_port(self.id) } {
            0 => Ok(()),
            v => Err(Error::SAI(Status::from(v))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Port,
    SubPort,
    Dot1QRouter,
    Dot1DRouter,
    Tunnel,
    Unknown(sai_bridge_port_type_t),
}

impl From<sai_bridge_port_type_t> for Type {
    fn from(value: sai_bridge_port_type_t) -> Self {
        match value {
            sai_sys::_sai_bridge_port_type_t_SAI_BRIDGE_PORT_TYPE_PORT => Self::Port,
            sai_sys::_sai_bridge_port_type_t_SAI_BRIDGE_PORT_TYPE_SUB_PORT => Self::SubPort,
            sai_sys::_sai_bridge_port_type_t_SAI_BRIDGE_PORT_TYPE_1Q_ROUTER => Self::Dot1QRouter,
            sai_sys::_sai_bridge_port_type_t_SAI_BRIDGE_PORT_TYPE_1D_ROUTER => Self::Dot1DRouter,
            sai_sys::_sai_bridge_port_type_t_SAI_BRIDGE_PORT_TYPE_TUNNEL => Self::Tunnel,
            v => Self::Unknown(v),
        }
    }
}
