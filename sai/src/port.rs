use super::*;
use sai_sys::*;

#[derive(Clone, Copy, Debug)]
pub enum OperStatus {
    Unknown,
    Up,
    Down,
    Testing,
    NotPresent,
}

impl From<OperStatus> for i32 {
    fn from(value: OperStatus) -> Self {
        match value {
            OperStatus::Unknown => _sai_port_oper_status_t_SAI_PORT_OPER_STATUS_UNKNOWN as i32,
            OperStatus::Up => _sai_port_oper_status_t_SAI_PORT_OPER_STATUS_UP as i32,
            OperStatus::Down => _sai_port_oper_status_t_SAI_PORT_OPER_STATUS_DOWN as i32,
            OperStatus::Testing => _sai_port_oper_status_t_SAI_PORT_OPER_STATUS_TESTING as i32,
            OperStatus::NotPresent => {
                _sai_port_oper_status_t_SAI_PORT_OPER_STATUS_NOT_PRESENT as i32
            }
        }
    }
}

impl From<i32> for OperStatus {
    fn from(value: i32) -> Self {
        match value {
            x if x == sai_sys::_sai_port_oper_status_t_SAI_PORT_OPER_STATUS_UNKNOWN as i32 => {
                OperStatus::Unknown
            }
            x if x == _sai_port_oper_status_t_SAI_PORT_OPER_STATUS_UP as i32 => OperStatus::Up,
            x if x == _sai_port_oper_status_t_SAI_PORT_OPER_STATUS_DOWN as i32 => OperStatus::Down,
            x if x == _sai_port_oper_status_t_SAI_PORT_OPER_STATUS_TESTING as i32 => {
                OperStatus::Testing
            }
            x if x == _sai_port_oper_status_t_SAI_PORT_OPER_STATUS_NOT_PRESENT as i32 => {
                OperStatus::NotPresent
            }
            _ => OperStatus::Unknown,
        }
    }
}

impl From<sai_port_oper_status_notification_t> for OperStatus {
    fn from(value: sai_port_oper_status_notification_t) -> Self {
        OperStatus::from(value.port_state as i32)
    }
}

impl From<OperStatus> for bool {
    fn from(value: OperStatus) -> Self {
        match value {
            OperStatus::Up => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BreakoutModeType {
    OneLane,
    TwoLanes,
    FourLanes,
    Unknown(i32),
}

impl From<BreakoutModeType> for i32 {
    fn from(value: BreakoutModeType) -> Self {
        match value {
            BreakoutModeType::OneLane => {
                _sai_port_breakout_mode_type_t_SAI_PORT_BREAKOUT_MODE_TYPE_1_LANE as i32
            }
            BreakoutModeType::TwoLanes => {
                _sai_port_breakout_mode_type_t_SAI_PORT_BREAKOUT_MODE_TYPE_2_LANE as i32
            }
            BreakoutModeType::FourLanes => {
                _sai_port_breakout_mode_type_t_SAI_PORT_BREAKOUT_MODE_TYPE_4_LANE as i32
            }
            BreakoutModeType::Unknown(v) => v,
        }
    }
}

impl From<i32> for BreakoutModeType {
    fn from(value: i32) -> Self {
        match value {
            x if x
                == sai_sys::_sai_port_breakout_mode_type_t_SAI_PORT_BREAKOUT_MODE_TYPE_1_LANE
                    as i32 =>
            {
                BreakoutModeType::OneLane
            }
            x if x
                == sai_sys::_sai_port_breakout_mode_type_t_SAI_PORT_BREAKOUT_MODE_TYPE_2_LANE
                    as i32 =>
            {
                BreakoutModeType::TwoLanes
            }
            x if x
                == sai_sys::_sai_port_breakout_mode_type_t_SAI_PORT_BREAKOUT_MODE_TYPE_4_LANE
                    as i32 =>
            {
                BreakoutModeType::FourLanes
            }
            x => BreakoutModeType::Unknown(x),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AutoNegConfigMode {
    Disabled,
    Auto,
    Slave,
    Master,
    Unknown(i32),
}

impl From<AutoNegConfigMode> for i32 {
    fn from(value: AutoNegConfigMode) -> Self {
        match value {
            AutoNegConfigMode::Disabled => {
                _sai_port_auto_neg_config_mode_t_SAI_PORT_AUTO_NEG_CONFIG_MODE_DISABLED as i32
            }
            AutoNegConfigMode::Auto => {
                _sai_port_auto_neg_config_mode_t_SAI_PORT_AUTO_NEG_CONFIG_MODE_AUTO as i32
            }
            AutoNegConfigMode::Slave => {
                _sai_port_auto_neg_config_mode_t_SAI_PORT_AUTO_NEG_CONFIG_MODE_SLAVE as i32
            }
            AutoNegConfigMode::Master => {
                _sai_port_auto_neg_config_mode_t_SAI_PORT_AUTO_NEG_CONFIG_MODE_MASTER as i32
            }
            AutoNegConfigMode::Unknown(v) => v,
        }
    }
}

impl From<i32> for AutoNegConfigMode {
    fn from(value: i32) -> Self {
        match value {
            x if x
                == sai_sys::_sai_port_auto_neg_config_mode_t_SAI_PORT_AUTO_NEG_CONFIG_MODE_DISABLED
                    as i32 =>
            {
                AutoNegConfigMode::Disabled
            }
            x if x
                == sai_sys::_sai_port_auto_neg_config_mode_t_SAI_PORT_AUTO_NEG_CONFIG_MODE_AUTO
                    as i32 =>
            {
                AutoNegConfigMode::Auto
            }
            x if x
                == sai_sys::_sai_port_auto_neg_config_mode_t_SAI_PORT_AUTO_NEG_CONFIG_MODE_SLAVE
                    as i32 =>
            {
                AutoNegConfigMode::Slave
            }
            x if x
                == sai_sys::_sai_port_auto_neg_config_mode_t_SAI_PORT_AUTO_NEG_CONFIG_MODE_MASTER
                    as i32 =>
            {
                AutoNegConfigMode::Master
            }
            x => AutoNegConfigMode::Unknown(x),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PortID {
    pub(crate) id: sai_object_id_t,
}

impl std::fmt::Debug for PortID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "port:oid:{:#x}", self.id)
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

impl From<sai_port_oper_status_notification_t> for PortID {
    fn from(value: sai_port_oper_status_notification_t) -> Self {
        Self { id: value.port_id }
    }
}

impl PartialEq<PortID> for Port<'_> {
    fn eq(&self, other: &PortID) -> bool {
        self.id == other.id
    }
}

impl PartialEq<Port<'_>> for PortID {
    fn eq(&self, other: &Port<'_>) -> bool {
        self.id == other.id
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
    /// get the operational status of the port
    pub fn get_oper_status(&self) -> Result<OperStatus, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_OPER_STATUS,
            value: sai_attribute_value_t { s32: 0 },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(OperStatus::from(unsafe { attr.value.s32 }))
    }

    /// get the admin state of the port
    pub fn get_admin_state(&self) -> Result<bool, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_ADMIN_STATE,
            value: sai_attribute_value_t { booldata: false },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(unsafe { attr.value.booldata })
    }

    /// get a list of the supported speeds of the port
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

    /// get the operating speed of the port
    /// NOTE: this is different from the configured port speed
    pub fn get_oper_speed(&self) -> Result<u32, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_OPER_SPEED,
            value: sai_attribute_value_t { u32_: 0 },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(unsafe { attr.value.u32_ })
    }

    /// get the configured port speed
    /// NOTE: this is different from the operating port speed
    pub fn get_speed(&self) -> Result<u32, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_SPEED,
            value: sai_attribute_value_t { u32_: 0 },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(unsafe { attr.value.u32_ })
    }

    /// returns true if auto neg mode is supported by this port
    pub fn get_supported_auto_neg_mode(&self) -> Result<bool, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_SUPPORTED_AUTO_NEG_MODE,
            value: sai_attribute_value_t { booldata: false },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(unsafe { attr.value.booldata })
    }

    /// returns true if auto neg mode is advertised by this port
    pub fn get_advertised_auto_neg_mode(&self) -> Result<bool, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_ADVERTISED_AUTO_NEG_MODE,
            value: sai_attribute_value_t { booldata: false },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(unsafe { attr.value.booldata })
    }

    /// returns true if auto neg mode is advertised by the *remote's* port
    pub fn get_remote_advertised_auto_neg_mode(&self) -> Result<bool, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_REMOTE_ADVERTISED_AUTO_NEG_MODE,
            value: sai_attribute_value_t { booldata: false },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(unsafe { attr.value.booldata })
    }

    /// returns true if auto neg mode is activated (configured) for this port
    pub fn get_auto_neg_mode(&self) -> Result<bool, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_AUTO_NEG_MODE,
            value: sai_attribute_value_t { booldata: false },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(unsafe { attr.value.booldata })
    }

    /// get auto neg configuration mode for this port
    pub fn get_auto_neg_config_mode(&self) -> Result<AutoNegConfigMode, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_AUTO_NEG_CONFIG_MODE,
            value: sai_attribute_value_t { s32: 0 },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(AutoNegConfigMode::from(unsafe { attr.value.s32 }))
    }

    /// returns true if auto negotiation completed (Auto negotiation (AN) done state)
    pub fn get_auto_neg_status(&self) -> Result<bool, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_AUTO_NEG_STATUS,
            value: sai_attribute_value_t { booldata: false },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(unsafe { attr.value.booldata })
    }

    pub fn get_supported_breakout_modes(&self) -> Result<Vec<BreakoutModeType>, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut breakout_mode_types: Vec<i32> = vec![0i32; 4];
        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_SUPPORTED_BREAKOUT_MODE_TYPE,
            value: sai_attribute_value_t {
                s32list: sai_s32_list_t {
                    count: breakout_mode_types.len() as u32,
                    list: breakout_mode_types.as_mut_ptr(),
                },
            },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // iterate over the returned list and build the vector for return
        let count = unsafe { attr.value.s32list.count };
        let list = unsafe { attr.value.s32list.list };
        let mut ret: Vec<BreakoutModeType> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let breakout_mode_type: i32 = unsafe { *list.offset(i as isize) };
            ret.push(BreakoutModeType::from(breakout_mode_type));
        }
        Ok(ret)
    }

    pub fn get_current_breakout_mode(&self) -> Result<BreakoutModeType, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_CURRENT_BREAKOUT_MODE_TYPE,
            value: sai_attribute_value_t { s32: 0 },
        };

        let st = unsafe { get_port_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(BreakoutModeType::from(unsafe { attr.value.s32 }))
    }

    pub fn get_hw_lanes(&self) -> Result<Vec<u32>, Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let get_port_attribute = port_api
            .get_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut lanes: Vec<u32> = vec![0u32; 16];
        let mut attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_HW_LANE_LIST,
            value: sai_attribute_value_t {
                u32list: sai_u32_list_t {
                    count: lanes.len() as u32,
                    list: lanes.as_mut_ptr(),
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
            let lane: u32 = unsafe { *list.offset(i as isize) };
            ret.push(lane);
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

    /// enables/disables advertisement of auto neg mode for this port
    /// NOTE: this is different from enabling/disabling auto neg mode for the port (this is just advertising it)!
    pub fn set_advertised_auto_neg_mode(&self, enable: bool) -> Result<(), Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let set_port_attribute = port_api
            .set_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_ADVERTISED_AUTO_NEG_MODE,
            value: sai_attribute_value_t { booldata: enable },
        };

        let st = unsafe { set_port_attribute(self.id, &attr) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }

    /// enables/disables auto neg mode for this port
    /// NOTE: this is different from auto neg mode advertisement!
    pub fn set_auto_neg_mode(&self, enable: bool) -> Result<(), Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let set_port_attribute = port_api
            .set_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_AUTO_NEG_MODE,
            value: sai_attribute_value_t { booldata: enable },
        };

        let st = unsafe { set_port_attribute(self.id, &attr) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }

    /// enables/disables auto neg mode for this port
    /// NOTE: this is different from auto neg mode advertisement!
    pub fn set_auto_neg_config_mode(&self, config_mode: AutoNegConfigMode) -> Result<(), Error> {
        let port_api = self.sai.port_api().ok_or(Error::APIUnavailable)?;
        let set_port_attribute = port_api
            .set_port_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let attr = sai_attribute_t {
            id: _sai_port_attr_t_SAI_PORT_ATTR_AUTO_NEG_CONFIG_MODE,
            value: sai_attribute_value_t {
                s32: config_mode.into(),
            },
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
