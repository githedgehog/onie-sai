use crate::{
    bridge::Bridge,
    hostif::{
        table_entry::TableEntry, table_entry::TableEntryAttribute, trap::Trap, trap::TrapAttribute,
        trap_group::TrapGroup, HostIf, HostIfAttribute,
    },
    port::Port,
    virtual_router::VirtualRouter,
    vlan::VLAN,
};

use super::*;
use sai_sys::*;
use std::sync::RwLock;

static SWITCH_STATE_CHANGE_CALLBACK: RwLock<
    Option<Box<dyn Fn(sai_object_id_t, sai_switch_oper_status_t) + Send + Sync>>,
> = RwLock::new(None);

extern "C" fn switch_state_change_cb(
    switch_id: sai_object_id_t,
    switch_oper_status: sai_switch_oper_status_t,
) {
    let cb_read_lock = SWITCH_STATE_CHANGE_CALLBACK.read().unwrap();
    if let Some(ref callback) = *cb_read_lock {
        callback(switch_id, switch_oper_status);
    }
}

static SWITCH_SHUTDOWN_REQUEST_CALLBACK: RwLock<
    Option<Box<dyn Fn(sai_object_id_t) + Send + Sync>>,
> = RwLock::new(None);

extern "C" fn switch_shutdown_request_cb(switch_id: sai_object_id_t) {
    let cb_read_lock = SWITCH_SHUTDOWN_REQUEST_CALLBACK.read().unwrap();
    if let Some(ref callback) = *cb_read_lock {
        callback(switch_id);
    }
}

static FDB_EVENT_CALLBACK: RwLock<
    Option<Box<dyn Fn(Vec<sai_fdb_event_notification_data_t>) + Send + Sync>>,
> = RwLock::new(None);

extern "C" fn fdb_event_cb(count: u32, data: *const sai_fdb_event_notification_data_t) {
    let cb_read_lock = FDB_EVENT_CALLBACK.read().unwrap();
    if let Some(ref callback) = *cb_read_lock {
        let mut arg: Vec<sai_fdb_event_notification_data_t> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let elem = unsafe { *data.offset(i as isize) };
            arg.push(elem);
        }
        callback(arg);
    }
}

static NAT_EVENT_CALLBACK: RwLock<
    Option<Box<dyn Fn(Vec<sai_nat_event_notification_data_t>) + Send + Sync>>,
> = RwLock::new(None);

extern "C" fn nat_event_cb(count: u32, data: *const sai_nat_event_notification_data_t) {
    let cb_read_lock = NAT_EVENT_CALLBACK.read().unwrap();
    if let Some(ref callback) = *cb_read_lock {
        let mut arg: Vec<sai_nat_event_notification_data_t> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let elem = unsafe { *data.offset(i as isize) };
            arg.push(elem);
        }
        callback(arg);
    }
}

static PORT_STATE_CHANGE_CALLBACK: RwLock<
    Option<Box<dyn Fn(Vec<sai_port_oper_status_notification_t>) + Send + Sync>>,
> = RwLock::new(None);

extern "C" fn port_state_change_cb(count: u32, data: *const sai_port_oper_status_notification_t) {
    let cb_read_lock = PORT_STATE_CHANGE_CALLBACK.read().unwrap();
    if let Some(ref callback) = *cb_read_lock {
        let mut arg: Vec<sai_port_oper_status_notification_t> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let elem = unsafe { *data.offset(i as isize) };
            arg.push(elem);
        }
        callback(arg);
    }
}

static QUEUE_PFC_DEADLOCK_CALLBACK: RwLock<
    Option<Box<dyn Fn(Vec<sai_queue_deadlock_notification_data_t>) + Send + Sync>>,
> = RwLock::new(None);

extern "C" fn queue_pfc_deadlock_cb(
    count: u32,
    data: *const sai_queue_deadlock_notification_data_t,
) {
    let cb_read_lock = QUEUE_PFC_DEADLOCK_CALLBACK.read().unwrap();
    if let Some(ref callback) = *cb_read_lock {
        let mut arg: Vec<sai_queue_deadlock_notification_data_t> =
            Vec::with_capacity(count as usize);
        for i in 0..count {
            let elem = unsafe { *data.offset(i as isize) };
            arg.push(elem);
        }
        callback(arg);
    }
}

static BFD_SESSION_STATE_CHANGE_CALLBACK: RwLock<
    Option<Box<dyn Fn(Vec<sai_bfd_session_state_notification_t>) + Send + Sync>>,
> = RwLock::new(None);

extern "C" fn bfd_session_state_change_cb(
    count: u32,
    data: *const sai_bfd_session_state_notification_t,
) {
    let cb_read_lock = BFD_SESSION_STATE_CHANGE_CALLBACK.read().unwrap();
    if let Some(ref callback) = *cb_read_lock {
        let mut arg: Vec<sai_bfd_session_state_notification_t> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let elem = unsafe { *data.offset(i as isize) };
            arg.push(elem);
        }
        callback(arg);
    }
}

#[derive(Clone, Copy)]
pub struct SwitchID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for SwitchID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for SwitchID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<SwitchID> for sai_object_id_t {
    fn from(value: SwitchID) -> Self {
        value.id
    }
}

impl From<Switch<'_>> for SwitchID {
    fn from(value: Switch) -> Self {
        Self { id: value.id }
    }
}

pub struct Switch<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for Switch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Switch(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for Switch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> Switch<'a> {
    pub fn get_default_vlan(&self) -> Result<VLAN<'a>, Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let get_switch_attribute = switch_api
            .get_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_DEFAULT_VLAN_ID,
            value: sai_attribute_value_t { oid: 0 },
        };

        let st = unsafe { get_switch_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(VLAN {
            id: unsafe { attr.value.oid },
            sai: self.sai,
        })
    }

    pub fn get_default_bridge(&self) -> Result<Bridge<'a>, Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let get_switch_attribute = switch_api
            .get_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_DEFAULT_1Q_BRIDGE_ID,
            value: sai_attribute_value_t { oid: 0 },
        };

        let st = unsafe { get_switch_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(Bridge {
            id: unsafe { attr.value.oid },
            sai: self.sai,
        })
    }

    pub fn get_default_hostif_trap_group(&self) -> Result<TrapGroup<'a>, Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let get_switch_attribute = switch_api
            .get_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_DEFAULT_TRAP_GROUP,
            value: sai_attribute_value_t { oid: 0 },
        };

        let st = unsafe { get_switch_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(TrapGroup {
            id: unsafe { attr.value.oid },
            sai: self.sai,
        })
    }

    pub fn create_hostif_trap(&self, attrs: Vec<TrapAttribute>) -> Result<Trap<'a>, Error> {
        // check that API is available/callable
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let create_hostif_trap = hostif_api
            .create_hostif_trap
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut exclude_port_list_backing: Vec<sai_object_id_t> = Vec::new();
        let mut mirror_session_backing: Vec<sai_object_id_t> = Vec::new();
        let args: Vec<sai_attribute_t> = attrs
            .into_iter()
            .map(|v| {
                v.to_sai_attribute_t(&mut exclude_port_list_backing, &mut mirror_session_backing)
            })
            .collect();

        let mut oid: sai_object_id_t = 0;
        let st = unsafe {
            create_hostif_trap(
                &mut oid as *mut _,
                self.id,
                args.len() as u32,
                args.as_ptr(),
            )
        };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(Trap {
            id: oid,
            sai: self.sai,
        })
    }

    pub fn create_hostif_table_entry(
        &self,
        attrs: Vec<TableEntryAttribute>,
    ) -> Result<TableEntry<'a>, Error> {
        // check that API is available/callable
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let create_hostif_table_entry = hostif_api
            .create_hostif_table_entry
            .ok_or(Error::APIFunctionUnavailable)?;

        let args: Vec<sai_attribute_t> = attrs.into_iter().map(|v| v.into()).collect();

        let mut oid: sai_object_id_t = 0;
        let st = unsafe {
            create_hostif_table_entry(&mut oid, self.id, args.len() as u32, args.as_ptr())
        };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(TableEntry {
            id: oid,
            sai: self.sai,
        })
    }

    pub fn get_cpu_port(&self) -> Result<Port<'a>, Error> {
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let get_switch_attribute = switch_api
            .get_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_CPU_PORT,
            value: sai_attribute_value_t { oid: 0 },
        };

        let st = unsafe { get_switch_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(Port {
            id: unsafe { attr.value.oid },
            sai: self.sai,
        })
    }

    pub fn create_hostif(&self, attrs: Vec<HostIfAttribute>) -> Result<HostIf, Error> {
        // check that API is available/callable
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let create_hostif = hostif_api
            .create_hostif
            .ok_or(Error::APIFunctionUnavailable)?;

        let args: Vec<sai_attribute_t> = attrs.into_iter().map(|v| v.into()).collect();

        let mut oid: sai_object_id_t = 0;
        let st = unsafe { create_hostif(&mut oid, self.id, args.len() as u32, args.as_ptr()) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(HostIf {
            id: oid,
            sai: self.sai,
        })
    }

    pub fn get_default_virtual_router(&self) -> Result<VirtualRouter, Error> {
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let get_switch_attribute = switch_api
            .get_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_DEFAULT_VIRTUAL_ROUTER_ID,
            value: sai_attribute_value_t { oid: 0 },
        };

        let st = unsafe { get_switch_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(VirtualRouter {
            id: unsafe { attr.value.oid },
            switch_id: self.id,
            sai: self.sai,
        })
    }

    pub fn get_ports(&self) -> Result<Vec<Port<'a>>, Error> {
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let get_switch_attribute = switch_api
            .get_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let mut ports: Vec<sai_object_id_t> = vec![0u64; 128];
        let mut attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_PORT_LIST,
            value: sai_attribute_value_t {
                objlist: sai_object_list_t {
                    count: 128,
                    list: ports.as_mut_ptr(),
                },
            },
        };

        let st = unsafe { get_switch_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // iterate over the returned list and build the vector for return
        let count = unsafe { attr.value.objlist.count };
        let list = unsafe { attr.value.objlist.list };
        let mut ret: Vec<Port> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let oid: sai_object_id_t = unsafe { *list.offset(i as isize) };
            ret.push(Port {
                id: oid,
                sai: self.sai,
            });
        }
        Ok(ret)
    }

    pub fn enable_shell(&self) -> Result<(), Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let set_switch_attribute = switch_api
            .set_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_SWITCH_SHELL_ENABLE,
            value: sai_attribute_value_t { booldata: true },
        };
        let st: sai_status_t = unsafe { set_switch_attribute(1, &attr as *const _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }

    pub fn remove(self) -> Result<(), Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let remove_switch = switch_api
            .remove_switch
            .ok_or(Error::APIFunctionUnavailable)?;

        let st: sai_status_t = unsafe { remove_switch(self.id) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }

    pub fn set_switch_state_change_callback(
        &self,
        cb: Box<dyn Fn(sai_object_id_t, sai_switch_oper_status_t) + Send + Sync>,
    ) -> Result<(), Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let set_switch_attribute = switch_api
            .set_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        // we acquire this lock and will not let it go before the end of this function which is correct
        let mut cb_write_lock = SWITCH_STATE_CHANGE_CALLBACK.write().unwrap();

        let attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_SWITCH_STATE_CHANGE_NOTIFY,
            value: sai_attribute_value_t {
                ptr: switch_state_change_cb as sai_pointer_t,
            },
        };
        let st: sai_status_t = unsafe { set_switch_attribute(1, &attr as *const _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // only set the closure when we know that the call to SAI was successful
        *cb_write_lock = Some(cb);
        Ok(())
    }

    pub fn set_switch_shutdown_request_callback(
        &self,
        cb: Box<dyn Fn(sai_object_id_t) + Send + Sync>,
    ) -> Result<(), Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let set_switch_attribute = switch_api
            .set_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        // we acquire this lock and will not let it go before the end of this function which is correct
        let mut cb_write_lock = SWITCH_SHUTDOWN_REQUEST_CALLBACK.write().unwrap();

        let attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_SWITCH_SHUTDOWN_REQUEST_NOTIFY,
            value: sai_attribute_value_t {
                ptr: switch_shutdown_request_cb as sai_pointer_t,
            },
        };
        let st: sai_status_t = unsafe { set_switch_attribute(1, &attr as *const _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // only set the closure when we know that the call to SAI was successful
        *cb_write_lock = Some(cb);
        Ok(())
    }

    pub fn set_fdb_event_callback(
        &self,
        cb: Box<dyn Fn(Vec<sai_fdb_event_notification_data_t>) + Send + Sync>,
    ) -> Result<(), Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let set_switch_attribute = switch_api
            .set_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        // we acquire this lock and will not let it go before the end of this function which is correct
        let mut cb_write_lock = FDB_EVENT_CALLBACK.write().unwrap();

        let attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_FDB_EVENT_NOTIFY,
            value: sai_attribute_value_t {
                ptr: fdb_event_cb as sai_pointer_t,
            },
        };
        let st: sai_status_t = unsafe { set_switch_attribute(1, &attr as *const _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // only set the closure when we know that the call to SAI was successful
        *cb_write_lock = Some(cb);
        Ok(())
    }

    pub fn set_nat_event_callback(
        &self,
        cb: Box<dyn Fn(Vec<sai_nat_event_notification_data_t>) + Send + Sync>,
    ) -> Result<(), Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let set_switch_attribute = switch_api
            .set_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        // we acquire this lock and will not let it go before the end of this function which is correct
        let mut cb_write_lock = NAT_EVENT_CALLBACK.write().unwrap();

        let attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_NAT_EVENT_NOTIFY,
            value: sai_attribute_value_t {
                ptr: nat_event_cb as sai_pointer_t,
            },
        };
        let st: sai_status_t = unsafe { set_switch_attribute(1, &attr as *const _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // only set the closure when we know that the call to SAI was successful
        *cb_write_lock = Some(cb);
        Ok(())
    }

    pub fn set_port_state_change_callback(
        &self,
        cb: Box<dyn Fn(Vec<sai_port_oper_status_notification_t>) + Send + Sync>,
    ) -> Result<(), Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let set_switch_attribute = switch_api
            .set_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        // we acquire this lock and will not let it go before the end of this function which is correct
        let mut cb_write_lock = PORT_STATE_CHANGE_CALLBACK.write().unwrap();

        let attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_PORT_STATE_CHANGE_NOTIFY,
            value: sai_attribute_value_t {
                ptr: port_state_change_cb as sai_pointer_t,
            },
        };
        let st: sai_status_t = unsafe { set_switch_attribute(1, &attr as *const _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // only set the closure when we know that the call to SAI was successful
        *cb_write_lock = Some(cb);
        Ok(())
    }

    pub fn set_queue_pfc_deadlock_callback(
        &self,
        cb: Box<dyn Fn(Vec<sai_queue_deadlock_notification_data_t>) + Send + Sync>,
    ) -> Result<(), Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let set_switch_attribute = switch_api
            .set_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        // we acquire this lock and will not let it go before the end of this function which is correct
        let mut cb_write_lock = QUEUE_PFC_DEADLOCK_CALLBACK.write().unwrap();

        let attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_QUEUE_PFC_DEADLOCK_NOTIFY,
            value: sai_attribute_value_t {
                ptr: queue_pfc_deadlock_cb as sai_pointer_t,
            },
        };
        let st: sai_status_t = unsafe { set_switch_attribute(1, &attr as *const _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // only set the closure when we know that the call to SAI was successful
        *cb_write_lock = Some(cb);
        Ok(())
    }

    pub fn set_bfd_session_state_change_callback(
        &self,
        cb: Box<dyn Fn(Vec<sai_bfd_session_state_notification_t>) + Send + Sync>,
    ) -> Result<(), Error> {
        // check that API is available/callable
        let switch_api = self.sai.switch_api().ok_or(Error::APIUnavailable)?;
        let set_switch_attribute = switch_api
            .set_switch_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        // we acquire this lock and will not let it go before the end of this function which is correct
        let mut cb_write_lock = BFD_SESSION_STATE_CHANGE_CALLBACK.write().unwrap();

        let attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_BFD_SESSION_STATE_CHANGE_NOTIFY,
            value: sai_attribute_value_t {
                ptr: bfd_session_state_change_cb as sai_pointer_t,
            },
        };
        let st: sai_status_t = unsafe { set_switch_attribute(1, &attr as *const _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // only set the closure when we know that the call to SAI was successful
        *cb_write_lock = Some(cb);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum SwitchAttribute {
    InitSwitch(bool),
    SrcMacAddress(sai_mac_t),
}

impl From<SwitchAttribute> for sai_attribute_t {
    fn from(value: SwitchAttribute) -> Self {
        match value {
            SwitchAttribute::InitSwitch(v) => Self {
                id: _sai_switch_attr_t_SAI_SWITCH_ATTR_INIT_SWITCH,
                value: sai_attribute_value_t { booldata: v },
            },
            SwitchAttribute::SrcMacAddress(v) => Self {
                id: _sai_switch_attr_t_SAI_SWITCH_ATTR_SRC_MAC_ADDRESS,
                value: _sai_attribute_value_t { mac: v },
            },
        }
    }
}

impl ObjectID<SwitchID> for Switch<'_> {
    fn to_id(&self) -> SwitchID {
        SwitchID { id: self.id }
    }
}
