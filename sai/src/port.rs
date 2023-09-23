use super::*;
use sai_sys::*;

#[derive(Clone, Copy)]
pub struct PortID {
    pub(crate) id: sai_object_id_t,
}

impl std::fmt::Debug for PortID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for PortID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<PortID> for sai_object_id_t {
    fn from(value: PortID) -> Self {
        value.id
    }
}

impl From<Port<'_>> for PortID {
    fn from(value: Port) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone)]
pub struct Port<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for Port<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Port(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for Port<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> Port<'a> {
    pub fn get_supported_speeds(&self) -> Result<Vec<u32>, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut speeds: Vec<u32> = vec![0u32; 16];
        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_SUPPORTED_SPEED,
            value: sai_attribute_value_t {
                u32list: sai_u32_list_t {
                    count: speeds.len() as u32,
                    list: speeds.as_mut_ptr(),
                },
            },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // iterate over the returned list and build the vector for return
        let count = unsafe { attr.value.u32list.count };
        let list = unsafe { attr.value.u32list.list };
        let mut ret: Vec<u32> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let speed: u32 = unsafe { *list.offset(i as isize) };
            ret.push(speed);
        }
        Ok(ret)
    }

    pub fn set_speed(&self, speed: u32) -> Result<(), Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let set_port_attribute = port_api
            .set_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_SPEED,
            value: sai_attribute_value_t { u32_: speed },
        };

        let st = unsafe { set_port_attribute(self.id, &attr) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }

    pub fn set_admin_state(&self, admin_state: bool) -> Result<(), Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let set_port_attribute = port_api
            .set_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_ADMIN_STATE,
            value: sai_attribute_value_t {
                booldata: admin_state,
            },
        };

        let st = unsafe { set_port_attribute(self.id, &attr) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }

    pub fn remove(self) -> Result<(), Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let remove_port = port_api.remove_port.ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_port(self.id) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}

impl ObjectID<PortID> for Port<'_> {
    fn to_id(&self) -> PortID {
        PortID { id: self.id }
    }
}
