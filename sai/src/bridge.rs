use super::*;
use sai_sys::*;

#[derive(Clone, Copy)]
pub struct BridgeID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for BridgeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for BridgeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<BridgeID> for sai_object_id_t {
    fn from(value: BridgeID) -> Self {
        value.id
    }
}

impl From<Bridge<'_>> for BridgeID {
    fn from(value: Bridge) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone, Copy)]
pub struct Bridge<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for Bridge<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bridge(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for Bridge<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> Bridge<'a> {
    pub fn get_ports(&self) -> Result<Vec<BridgePort>, Error> {
        // check that API is available/callable
        let bridge_api = self.sai.bridge_api().ok_or(Error::APIUnavailable)?;
        let get_bridge_attribute = bridge_api
            .get_bridge_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut members: Vec<sai_object_id_t> = vec![0u64; 128];
        let mut attr = sai_attribute_t {
            id: _sai_bridge_attr_t_SAI_BRIDGE_ATTR_PORT_LIST,
            value: sai_attribute_value_t {
                objlist: sai_object_list_t {
                    count: 128,
                    list: members.as_mut_ptr(),
                },
            },
        };

        let st = unsafe { get_bridge_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // iterate over the returned list and build the vector for return
        let count = unsafe { attr.value.objlist.count };
        let list = unsafe { attr.value.objlist.list };
        let mut ret: Vec<BridgePort> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let oid: sai_object_id_t = unsafe { *list.offset(i as isize) };
            ret.push(BridgePort {
                id: oid,
                sai: self.sai,
            });
        }
        Ok(ret)
    }
}

pub struct BridgePort<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
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
    pub fn get_type(&self) -> Result<BridgePortType, Error> {
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
        Ok(BridgePortType::from(bridge_port_type))
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
pub enum BridgePortType {
    Port,
    SubPort,
    Dot1QRouter,
    Dot1DRouter,
    Tunnel,
    Unknown(sai_bridge_port_type_t),
}

impl From<sai_bridge_port_type_t> for BridgePortType {
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
