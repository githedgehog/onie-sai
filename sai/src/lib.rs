use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex, RwLock};

use sai_sys::*;
use std::os::raw::{c_char, c_int};
use std::ptr::{null, null_mut};

// we are re-exporting some things here
pub use sai_sys::sai_mac_t;
pub use sai_sys::SAI_KEY_INIT_CONFIG_FILE;

static PROFILE_GET_NEXT_VALUE_CALLBACK: RwLock<
    Option<
        Box<
            dyn Fn(sai_switch_profile_id_t, *mut *const c_char, *mut *const c_char) -> c_int
                + Send
                + Sync,
        >,
    >,
> = RwLock::new(None);

extern "C" fn profile_get_next_value_cb(
    profile_id: sai_switch_profile_id_t,
    variable: *mut *const c_char,
    value: *mut *const c_char,
) -> c_int {
    // Check if the callback is set
    let cb_read_lock = PROFILE_GET_NEXT_VALUE_CALLBACK.read().unwrap();
    if let Some(ref callback) = *cb_read_lock {
        callback(profile_id, variable, value)
    } else {
        0
    }
}

static PROFILE_GET_VALUE_CALLBACK: RwLock<
    Option<Box<dyn Fn(sai_switch_profile_id_t, *const c_char) -> *const c_char + Send + Sync>>,
> = RwLock::new(None);

extern "C" fn profile_get_value_cb(
    profile_id: sai_switch_profile_id_t,
    variable: *const c_char,
) -> *const c_char {
    // Check if the callback is set
    let cb_read_lock = PROFILE_GET_VALUE_CALLBACK.read().unwrap();
    if let Some(ref callback) = *cb_read_lock {
        callback(profile_id, variable)
    } else {
        null()
    }
}

static PROFILE_SMT: sai_service_method_table_t = sai_service_method_table_t {
    profile_get_next_value: Some(profile_get_next_value_cb),
    profile_get_value: Some(profile_get_value_cb),
};

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

#[derive(Debug)]
struct Profile {
    profile_idx: Mutex<usize>,
    profile: Vec<(CString, CString)>,
}

impl Profile {
    fn profile_get_next_value(
        &self,
        _profile_id: sai_switch_profile_id_t,
        variable: *mut *const c_char,
        value: *mut *const c_char,
    ) -> c_int {
        if value == null_mut() {
            // resetting profile map iterator
            *self.profile_idx.lock().unwrap() = 0;
            return 0;
        }

        if variable == null_mut() {
            // variable is null
            return -1;
        }

        let idx = { *self.profile_idx.lock().unwrap() };
        if self.profile.len() == idx {
            // iterator reached end
            return -1;
        }

        // here comes the scary part: set the C variable and value to the Rust ones
        //
        // NOTE: this is kind of unsafe because it assumes that the vector isn't being dropped
        // however, we know that because we own it, and this closure is never being called again when
        // the profile is being dropped
        //
        // the unwrap() is safe, as we were testing above if we reached the end
        let entry = self.profile.get(idx).unwrap();
        let entry_var = entry.0.as_ptr();
        let entry_val = entry.1.as_ptr();
        unsafe {
            *variable = entry_var;
            *value = entry_val;
        }
        *self.profile_idx.lock().unwrap() += 1;

        // the 0 return value denotes to SAI to continue with the next value
        0
    }

    fn profile_get_value(
        &self,
        _profile_id: sai_switch_profile_id_t,
        variable: *const c_char,
    ) -> *const c_char {
        if variable == null() {
            return null();
        }

        // convert variable to Rust string
        let var_cstr = unsafe { CStr::from_ptr(variable) };
        let var = var_cstr.to_owned();

        // NOTE: this is kind of unsafe because it assumes that the vector isn't being dropped
        // however, we know that because we own it, and this closure is never being called again when
        // the profile is being dropped
        self.profile
            .iter()
            .find(|&x| x.0 == var)
            .map_or(null(), |x| x.1.as_ptr() as *const c_char)
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        // nothing right now, we had this here for debugging
    }
}

static SAI_INITIALIZED: Mutex<bool> = Mutex::new(false);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitError {
    Initializing,
    AlreadyInitialized,
    SAI(Status),
}

impl From<Status> for InitError {
    fn from(value: Status) -> Self {
        InitError::SAI(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    SwitchAlreadyCreated,
    APIUnavailable,
    SAI(Status),
}

impl From<Status> for Error {
    fn from(value: Status) -> Self {
        Error::SAI(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Success,
    Failure,
    NotSupported,
    NoMemory,
    InsufficientResources,
    InvalidParameter,
    ItemAlreadyExists,
    ItemNotFound,
    BufferOverflow,
    InvalidPortNumber,
    InvalidPortMember,
    InvalidVlanID,
    Uninitialized,
    TableFull,
    MandatoryAttributeMissing,
    NotImplemented,
    AddrNotFound,
    ObjectInUse,
    InvalidObjectType,
    InvalidObjectId,
    InvalidNvStorage,
    NvStorageFull,
    SwUpgradeVersionMismatch,
    NotExecuted,
    StageMismatch,
    InvalidAttribute(i32),
    InvalidAttributeValue(i32),
    AttributeNotImplemented(i32),
    UnknownAttribute(i32),
    AttributeNotSupported(i32),
    Unknown(i32),
}

impl From<sai_status_t> for Status {
    fn from(value: sai_status_t) -> Self {
        // TODO: figure out why `bindgen` is not generating consts for those
        match value {
            0 => Status::Success,
            -0x00000001 => Status::Failure,
            -0x00000002 => Status::NotSupported,
            -0x00000003 => Status::NoMemory,
            -0x00000004 => Status::InsufficientResources,
            -0x00000005 => Status::InvalidParameter,
            -0x00000006 => Status::ItemAlreadyExists,
            -0x00000007 => Status::ItemNotFound,
            -0x00000008 => Status::BufferOverflow,
            -0x00000009 => Status::InvalidPortNumber,
            -0x0000000A => Status::InvalidPortMember,
            -0x0000000B => Status::InvalidVlanID,
            -0x0000000C => Status::Uninitialized,
            -0x0000000D => Status::TableFull,
            -0x0000000E => Status::MandatoryAttributeMissing,
            -0x0000000F => Status::NotImplemented,
            -0x00000010 => Status::AddrNotFound,
            -0x00000011 => Status::ObjectInUse,
            -0x00000012 => Status::InvalidObjectType,
            -0x00000013 => Status::InvalidObjectId,
            -0x00000014 => Status::InvalidNvStorage,
            -0x00000015 => Status::NvStorageFull,
            -0x00000016 => Status::SwUpgradeVersionMismatch,
            -0x00000017 => Status::NotExecuted,
            -0x00000018 => Status::StageMismatch,
            -0x00010000..=0x0001FFFF => Status::InvalidAttribute(value.abs() - 0x00010000),
            -0x00020000..=0x0002FFFF => Status::InvalidAttributeValue(value.abs() - 0x00020000),
            -0x00030000..=0x0003FFFF => Status::AttributeNotImplemented(value.abs() - 0x00030000),
            -0x00040000..=0x0004FFFF => Status::UnknownAttribute(value.abs() - 0x00040000),
            -0x00050000..=0x0005FFFF => Status::AttributeNotSupported(value.abs() - 0x00050000),
            _ => Status::Unknown(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Notice,
    Warn,
    Error,
    Critical,
}

impl From<LogLevel> for sai_log_level_t {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Debug => _sai_log_level_t_SAI_LOG_LEVEL_DEBUG,
            LogLevel::Info => _sai_log_level_t_SAI_LOG_LEVEL_INFO,
            LogLevel::Notice => _sai_log_level_t_SAI_LOG_LEVEL_NOTICE,
            LogLevel::Warn => _sai_log_level_t_SAI_LOG_LEVEL_WARN,
            LogLevel::Error => _sai_log_level_t_SAI_LOG_LEVEL_ERROR,
            LogLevel::Critical => _sai_log_level_t_SAI_LOG_LEVEL_CRITICAL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum API {
    Unspecified,
    Switch,
    Port,
    FDB,
    VLAN,
    VirtualRouter,
    Route,
    NextHop,
    NextHopGroup,
    RouterInterface,
    Neighbor,
    ACL,
    HostIf,
    Mirror,
    SamplePacket,
    STP,
    LAG,
    Policer,
    WRED,
    QosMap,
    Queue,
    Scheduler,
    SchedulerGroup,
    Buffer,
    Hash,
    UDF,
    Tunnel,
    L2MC,
    IPMC,
    RpfGroup,
    L2mcGroup,
    IpmcGroup,
    McastFdb,
    Bridge,
    TAM,
    SRV6,
    MPLS,
    DTEL,
    BFD,
    IsolationGroup,
    NAT,
    Counter,
    DebugCounter,
    MACSEC,
    SystemPort,
    MyMac,
    IPSEC,
    GenericProgrammable,
    Unknown(u32),
}

impl From<sai_api_t> for API {
    fn from(value: sai_api_t) -> Self {
        match value {
            sai_sys::_sai_api_t_SAI_API_UNSPECIFIED => API::Unspecified,
            sai_sys::_sai_api_t_SAI_API_SWITCH => API::Switch,
            sai_sys::_sai_api_t_SAI_API_PORT => API::Port,
            sai_sys::_sai_api_t_SAI_API_FDB => API::FDB,
            sai_sys::_sai_api_t_SAI_API_VLAN => API::VLAN,
            sai_sys::_sai_api_t_SAI_API_VIRTUAL_ROUTER => API::VirtualRouter,
            sai_sys::_sai_api_t_SAI_API_ROUTE => API::Route,
            sai_sys::_sai_api_t_SAI_API_NEXT_HOP => API::NextHop,
            sai_sys::_sai_api_t_SAI_API_NEXT_HOP_GROUP => API::NextHopGroup,
            sai_sys::_sai_api_t_SAI_API_ROUTER_INTERFACE => API::RouterInterface,
            sai_sys::_sai_api_t_SAI_API_NEIGHBOR => API::Neighbor,
            sai_sys::_sai_api_t_SAI_API_ACL => API::ACL,
            sai_sys::_sai_api_t_SAI_API_HOSTIF => API::HostIf,
            sai_sys::_sai_api_t_SAI_API_MIRROR => API::Mirror,
            sai_sys::_sai_api_t_SAI_API_SAMPLEPACKET => API::SamplePacket,
            sai_sys::_sai_api_t_SAI_API_STP => API::STP,
            sai_sys::_sai_api_t_SAI_API_LAG => API::LAG,
            sai_sys::_sai_api_t_SAI_API_POLICER => API::Policer,
            sai_sys::_sai_api_t_SAI_API_WRED => API::WRED,
            sai_sys::_sai_api_t_SAI_API_QOS_MAP => API::QosMap,
            sai_sys::_sai_api_t_SAI_API_QUEUE => API::Queue,
            sai_sys::_sai_api_t_SAI_API_SCHEDULER => API::Scheduler,
            sai_sys::_sai_api_t_SAI_API_SCHEDULER_GROUP => API::SchedulerGroup,
            sai_sys::_sai_api_t_SAI_API_BUFFER => API::Buffer,
            sai_sys::_sai_api_t_SAI_API_HASH => API::Hash,
            sai_sys::_sai_api_t_SAI_API_UDF => API::UDF,
            sai_sys::_sai_api_t_SAI_API_TUNNEL => API::Tunnel,
            sai_sys::_sai_api_t_SAI_API_L2MC => API::L2MC,
            sai_sys::_sai_api_t_SAI_API_IPMC => API::IPMC,
            sai_sys::_sai_api_t_SAI_API_RPF_GROUP => API::RpfGroup,
            sai_sys::_sai_api_t_SAI_API_L2MC_GROUP => API::L2mcGroup,
            sai_sys::_sai_api_t_SAI_API_IPMC_GROUP => API::IpmcGroup,
            sai_sys::_sai_api_t_SAI_API_MCAST_FDB => API::McastFdb,
            sai_sys::_sai_api_t_SAI_API_BRIDGE => API::Bridge,
            sai_sys::_sai_api_t_SAI_API_TAM => API::TAM,
            sai_sys::_sai_api_t_SAI_API_SRV6 => API::SRV6,
            sai_sys::_sai_api_t_SAI_API_MPLS => API::MPLS,
            sai_sys::_sai_api_t_SAI_API_DTEL => API::DTEL,
            sai_sys::_sai_api_t_SAI_API_BFD => API::BFD,
            sai_sys::_sai_api_t_SAI_API_ISOLATION_GROUP => API::IsolationGroup,
            sai_sys::_sai_api_t_SAI_API_NAT => API::NAT,
            sai_sys::_sai_api_t_SAI_API_COUNTER => API::Counter,
            sai_sys::_sai_api_t_SAI_API_DEBUG_COUNTER => API::DebugCounter,
            sai_sys::_sai_api_t_SAI_API_MACSEC => API::MACSEC,
            sai_sys::_sai_api_t_SAI_API_SYSTEM_PORT => API::SystemPort,
            sai_sys::_sai_api_t_SAI_API_MY_MAC => API::MyMac,
            sai_sys::_sai_api_t_SAI_API_IPSEC => API::IPSEC,
            sai_sys::_sai_api_t_SAI_API_GENERIC_PROGRAMMABLE => API::GenericProgrammable,
            v => API::Unknown(v),
        }
    }
}

impl From<API> for sai_api_t {
    fn from(value: API) -> Self {
        match value {
            API::Unspecified => sai_sys::_sai_api_t_SAI_API_UNSPECIFIED,
            API::Switch => sai_sys::_sai_api_t_SAI_API_SWITCH,
            API::Port => sai_sys::_sai_api_t_SAI_API_PORT,
            API::FDB => sai_sys::_sai_api_t_SAI_API_FDB,
            API::VLAN => sai_sys::_sai_api_t_SAI_API_VLAN,
            API::VirtualRouter => sai_sys::_sai_api_t_SAI_API_VIRTUAL_ROUTER,
            API::Route => sai_sys::_sai_api_t_SAI_API_ROUTE,
            API::NextHop => sai_sys::_sai_api_t_SAI_API_NEXT_HOP,
            API::NextHopGroup => sai_sys::_sai_api_t_SAI_API_NEXT_HOP_GROUP,
            API::RouterInterface => sai_sys::_sai_api_t_SAI_API_ROUTER_INTERFACE,
            API::Neighbor => sai_sys::_sai_api_t_SAI_API_NEIGHBOR,
            API::ACL => sai_sys::_sai_api_t_SAI_API_ACL,
            API::HostIf => sai_sys::_sai_api_t_SAI_API_HOSTIF,
            API::Mirror => sai_sys::_sai_api_t_SAI_API_MIRROR,
            API::SamplePacket => sai_sys::_sai_api_t_SAI_API_SAMPLEPACKET,
            API::STP => sai_sys::_sai_api_t_SAI_API_STP,
            API::LAG => sai_sys::_sai_api_t_SAI_API_LAG,
            API::Policer => sai_sys::_sai_api_t_SAI_API_POLICER,
            API::WRED => sai_sys::_sai_api_t_SAI_API_WRED,
            API::QosMap => sai_sys::_sai_api_t_SAI_API_QOS_MAP,
            API::Queue => sai_sys::_sai_api_t_SAI_API_QUEUE,
            API::Scheduler => sai_sys::_sai_api_t_SAI_API_SCHEDULER,
            API::SchedulerGroup => sai_sys::_sai_api_t_SAI_API_SCHEDULER_GROUP,
            API::Buffer => sai_sys::_sai_api_t_SAI_API_BUFFER,
            API::Hash => sai_sys::_sai_api_t_SAI_API_HASH,
            API::UDF => sai_sys::_sai_api_t_SAI_API_UDF,
            API::Tunnel => sai_sys::_sai_api_t_SAI_API_TUNNEL,
            API::L2MC => sai_sys::_sai_api_t_SAI_API_L2MC,
            API::IPMC => sai_sys::_sai_api_t_SAI_API_IPMC,
            API::RpfGroup => sai_sys::_sai_api_t_SAI_API_RPF_GROUP,
            API::L2mcGroup => sai_sys::_sai_api_t_SAI_API_L2MC_GROUP,
            API::IpmcGroup => sai_sys::_sai_api_t_SAI_API_IPMC_GROUP,
            API::McastFdb => sai_sys::_sai_api_t_SAI_API_MCAST_FDB,
            API::Bridge => sai_sys::_sai_api_t_SAI_API_BRIDGE,
            API::TAM => sai_sys::_sai_api_t_SAI_API_TAM,
            API::SRV6 => sai_sys::_sai_api_t_SAI_API_SRV6,
            API::MPLS => sai_sys::_sai_api_t_SAI_API_MPLS,
            API::DTEL => sai_sys::_sai_api_t_SAI_API_DTEL,
            API::BFD => sai_sys::_sai_api_t_SAI_API_BFD,
            API::IsolationGroup => sai_sys::_sai_api_t_SAI_API_ISOLATION_GROUP,
            API::NAT => sai_sys::_sai_api_t_SAI_API_NAT,
            API::Counter => sai_sys::_sai_api_t_SAI_API_COUNTER,
            API::DebugCounter => sai_sys::_sai_api_t_SAI_API_DEBUG_COUNTER,
            API::MACSEC => sai_sys::_sai_api_t_SAI_API_MACSEC,
            API::SystemPort => sai_sys::_sai_api_t_SAI_API_SYSTEM_PORT,
            API::MyMac => sai_sys::_sai_api_t_SAI_API_MY_MAC,
            API::IPSEC => sai_sys::_sai_api_t_SAI_API_IPSEC,
            API::GenericProgrammable => sai_sys::_sai_api_t_SAI_API_GENERIC_PROGRAMMABLE,
            API::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Default)]
pub struct SAI {
    switch_api: sai_switch_api_t,
    vlan_api: sai_vlan_api_t,
    bridge_api: sai_bridge_api_t,
    port_api: sai_port_api_t,
    hostif_api: sai_hostif_api_t,
    router_interface_api: sai_router_interface_api_t,
    route_api: sai_route_api_t,
}

impl SAI {
    pub fn api_version() -> Result<u64, Status> {
        let mut version: sai_api_version_t = 0;
        unsafe {
            match sai_query_api_version(&mut version) {
                0 => Ok(version),
                v => Err(Status::from(v)),
            }
        }
    }

    pub fn log_set(api: API, lvl: LogLevel) -> Result<(), Status> {
        unsafe {
            match sai_log_set(sai_api_t::from(api), sai_log_level_t::from(lvl)) {
                0 => Ok(()),
                v => Err(Status::from(v)),
            }
        }
    }

    pub fn log_set_all(lvl: LogLevel) -> Result<(), Vec<(API, Status)>> {
        let mut ret_err: Vec<(API, Status)> = vec![];
        for api in 1.._sai_api_t_SAI_API_MAX {
            unsafe {
                match sai_log_set(api, sai_log_level_t::from(lvl)) {
                    0 => {}
                    v => {
                        ret_err.push((API::from(api), Status::from(v)));
                    }
                };
            }
        }
        if ret_err.is_empty() {
            Ok(())
        } else {
            Err(ret_err)
        }
    }

    pub fn new(profile: Vec<(CString, CString)>) -> Result<SAI, InitError> {
        let init_lock = SAI_INITIALIZED.try_lock();
        if let Ok(mut sai_initialized) = init_lock {
            // SAI is a singleton, so if this is true, it means it was already initialized
            if *sai_initialized {
                Err(InitError::AlreadyInitialized)
            } else {
                // we will return this, and the whole SAI_INITIALIZED lock is just there to ensure it is a singleton
                let mut ret: SAI = Default::default();

                // deal with the profile, and making sure there is a closure which can be called for it which has access to the map
                let p1 = Arc::new(Profile {
                    profile_idx: Mutex::new(0),
                    profile: profile,
                });
                let p2 = Arc::clone(&p1);
                {
                    let mut cb_write_lock = PROFILE_GET_NEXT_VALUE_CALLBACK.write().unwrap();
                    *cb_write_lock = Some(Box::new(
                        move |profile_id: sai_switch_profile_id_t,
                              variable: *mut *const c_char,
                              value: *mut *const c_char| {
                            p1.profile_get_next_value(profile_id, variable, value)
                        },
                    ));
                }
                {
                    let mut cb_write_lock = PROFILE_GET_VALUE_CALLBACK.write().unwrap();
                    *cb_write_lock = Some(Box::new(
                        move |profile_id: sai_switch_profile_id_t, variable: *const c_char| {
                            p2.profile_get_value(profile_id, variable)
                        },
                    ));
                }

                // this calls the the underlying C function
                ret.init()?;
                ret.apis_query();

                // we lock our singleton
                *sai_initialized = true;

                // and return with it
                Ok(ret)
            }
        } else {
            Err(InitError::Initializing)
        }
    }

    fn init(&self) -> Result<(), Status> {
        unsafe {
            match sai_api_initialize(0, &PROFILE_SMT) {
                0 => Ok(()),
                v => Err(Status::from(v)),
            }
        }
    }

    fn apis_query(&mut self) {
        // switch API
        {
            let mut switch_api_backing: sai_switch_api_t = Default::default();
            let switch_api_ptr_orig = &switch_api_backing as *const _;
            let mut switch_api_ptr = &mut switch_api_backing as *mut _;
            let switch_api_ptr_ptr = &mut switch_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_SWITCH, switch_api_ptr_ptr as _) };
            if st == SAI_STATUS_SUCCESS as i32 {
                self.switch_api = if switch_api_ptr_orig == switch_api_ptr {
                    switch_api_backing
                } else {
                    unsafe { *switch_api_ptr }
                };
            }
        }

        // vlan API
        {
            let mut vlan_api_backing: sai_vlan_api_t = Default::default();
            let vlan_api_ptr_orig = &vlan_api_backing as *const _;
            let mut vlan_api_ptr = &mut vlan_api_backing as *mut _;
            let vlan_api_ptr_ptr = &mut vlan_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_VLAN, vlan_api_ptr_ptr as _) };
            if st == SAI_STATUS_SUCCESS as i32 {
                self.vlan_api = if vlan_api_ptr_orig == vlan_api_ptr {
                    vlan_api_backing
                } else {
                    unsafe { *vlan_api_ptr }
                };
            }
        }

        // bridge API
        {
            let mut bridge_api_backing: sai_bridge_api_t = Default::default();
            let bridge_api_ptr_orig = &bridge_api_backing as *const _;
            let mut bridge_api_ptr = &mut bridge_api_backing as *mut _;
            let bridge_api_ptr_ptr = &mut bridge_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_BRIDGE, bridge_api_ptr_ptr as _) };
            if st == SAI_STATUS_SUCCESS as i32 {
                self.bridge_api = if bridge_api_ptr_orig == bridge_api_ptr {
                    bridge_api_backing
                } else {
                    unsafe { *bridge_api_ptr }
                };
            }
        }

        // port API
        {
            let mut port_api_backing: sai_port_api_t = Default::default();
            let port_api_ptr_orig = &port_api_backing as *const _;
            let mut port_api_ptr = &mut port_api_backing as *mut _;
            let port_api_ptr_ptr = &mut port_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_PORT, port_api_ptr_ptr as _) };
            if st == SAI_STATUS_SUCCESS as i32 {
                self.port_api = if port_api_ptr_orig == port_api_ptr {
                    port_api_backing
                } else {
                    unsafe { *port_api_ptr }
                };
            }
        }

        // hostif API
        {
            let mut hostif_api_backing: sai_hostif_api_t = Default::default();
            let hostif_api_ptr_orig = &hostif_api_backing as *const _;
            let mut hostif_api_ptr = &mut hostif_api_backing as *mut _;
            let hostif_api_ptr_ptr = &mut hostif_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_HOSTIF, hostif_api_ptr_ptr as _) };
            if st == SAI_STATUS_SUCCESS as i32 {
                self.hostif_api = if hostif_api_ptr_orig == hostif_api_ptr {
                    hostif_api_backing
                } else {
                    unsafe { *hostif_api_ptr }
                };
            }
        }

        // router interface API
        {
            let mut router_interface_api_backing: sai_router_interface_api_t = Default::default();
            let router_interface_api_ptr_orig = &router_interface_api_backing as *const _;
            let mut router_interface_api_ptr = &mut router_interface_api_backing as *mut _;
            let router_interface_api_ptr_ptr = &mut router_interface_api_ptr as *mut *mut _;
            let st = unsafe {
                sai_api_query(
                    _sai_api_t_SAI_API_ROUTER_INTERFACE,
                    router_interface_api_ptr_ptr as _,
                )
            };
            if st == SAI_STATUS_SUCCESS as i32 {
                self.router_interface_api =
                    if router_interface_api_ptr_orig == router_interface_api_ptr {
                        router_interface_api_backing
                    } else {
                        unsafe { *router_interface_api_ptr }
                    };
            }
        }

        // route API
        {
            let mut route_api_backing: sai_route_api_t = Default::default();
            let route_api_ptr_orig = &route_api_backing as *const _;
            let mut route_api_ptr = &mut route_api_backing as *mut _;
            let route_api_ptr_ptr = &mut route_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_ROUTE, route_api_ptr_ptr as _) };
            if st == SAI_STATUS_SUCCESS as i32 {
                self.route_api = if route_api_ptr_orig == route_api_ptr {
                    route_api_backing
                } else {
                    unsafe { *route_api_ptr }
                };
            }
        }
    }

    // fn metadata_api_query(&mut self) -> Result<i32, Status> {
    //     // query available functionality
    //     unsafe {
    //         match sai_metadata_apis_query(Some(sai_api_query), &mut self.apis) {
    //             v if v >= 0 => Ok(v),
    //             v => Err(Status::from(v)),
    //         }
    //     }
    // }

    fn uninit(&self) -> Result<(), Status> {
        unsafe {
            match sai_api_uninitialize() {
                0 => Ok(()),
                v => Err(Status::from(v)),
            }
        }
    }

    pub fn switch_create(&self, attrs: Vec<SwitchAttribute>) -> Result<Switch, Error> {
        // check first if a switch was created already
        let mut switch_created = SWITCH_CREATED.lock().unwrap();
        if *switch_created {
            return Err(Error::SwitchAlreadyCreated);
        }
        // check that API is available/callable
        let create_switch = self.switch_api.create_switch.ok_or(Error::APIUnavailable)?;

        // now call it
        let mut sw_id: sai_object_id_t = 0;
        let mut attrs_arg: Vec<sai_attribute_t> = attrs.into_iter().map(Into::into).collect();
        attrs_arg.push(sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_SWITCH_STATE_CHANGE_NOTIFY,
            value: sai_attribute_value_t {
                ptr: switch_state_change_cb as sai_pointer_t,
            },
        });
        let attrs_arg_ptr = attrs_arg.as_ptr();
        match unsafe { create_switch(&mut sw_id, attrs_arg.len() as u32, attrs_arg_ptr) } {
            0 => {
                *switch_created = true;
                Ok(Switch {
                    id: sw_id,
                    sai: &self,
                })
            }
            v => Err(Error::SAI(Status::from(v))),
        }
    }
}

impl Drop for SAI {
    fn drop(&mut self) {
        // call uninit - even if this fails, there is not much that we can do about it
        let _ = self.uninit();

        // now set all global variables back to None/false
        {
            let mut cb_write_lock = PROFILE_GET_NEXT_VALUE_CALLBACK.write().unwrap();
            *cb_write_lock = None;
        }
        {
            let mut cb_write_lock = PROFILE_GET_VALUE_CALLBACK.write().unwrap();
            *cb_write_lock = None;
        }
        *SAI_INITIALIZED.lock().unwrap() = false;
    }
}

static SWITCH_CREATED: Mutex<bool> = Mutex::new(false);

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

impl From<Switch<'_>> for SwitchID {
    fn from(value: Switch) -> Self {
        Self { id: value.id }
    }
}

pub struct Switch<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
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
        let get_switch_attribute = self
            .sai
            .switch_api
            .get_switch_attribute
            .ok_or(Error::APIUnavailable)?;

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
        let get_switch_attribute = self
            .sai
            .switch_api
            .get_switch_attribute
            .ok_or(Error::APIUnavailable)?;

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

    pub fn get_default_hostif_trap_group(&self) -> Result<HostIfTrapGroup<'a>, Error> {
        // check that API is available/callable
        let get_switch_attribute = self
            .sai
            .switch_api
            .get_switch_attribute
            .ok_or(Error::APIUnavailable)?;

        let mut attr = sai_attribute_t {
            id: _sai_switch_attr_t_SAI_SWITCH_ATTR_DEFAULT_TRAP_GROUP,
            value: sai_attribute_value_t { oid: 0 },
        };

        let st = unsafe { get_switch_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(HostIfTrapGroup {
            id: unsafe { attr.value.oid },
            sai: self.sai,
        })
    }

    pub fn create_hostif_trap(
        &self,
        attrs: Vec<HostIfTrapAttribute>,
    ) -> Result<HostIfTrap<'a>, Error> {
        // check that API is available/callable
        let create_hostif_trap = self
            .sai
            .hostif_api
            .create_hostif_trap
            .ok_or(Error::APIUnavailable)?;

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

        Ok(HostIfTrap {
            id: oid,
            sai: self.sai,
        })
    }

    pub fn create_hostif_table_entry(
        &self,
        attrs: Vec<HostIfTableEntryAttribute>,
    ) -> Result<HostIfTableEntry<'a>, Error> {
        // check that API is available/callable
        let create_hostif_table_entry = self
            .sai
            .hostif_api
            .create_hostif_table_entry
            .ok_or(Error::APIUnavailable)?;

        let args: Vec<sai_attribute_t> = attrs.into_iter().map(|v| v.into()).collect();

        let mut oid: sai_object_id_t = 0;
        let st = unsafe {
            create_hostif_table_entry(&mut oid, self.id, args.len() as u32, args.as_ptr())
        };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        Ok(HostIfTableEntry {
            id: oid,
            sai: self.sai,
        })
    }

    pub fn set_switch_state_change_callback(
        &self,
        cb: Box<dyn Fn(sai_object_id_t, sai_switch_oper_status_t) + Send + Sync>,
    ) -> Result<(), Error> {
        // check that API is available/callable
        let set_switch_attribute = self
            .sai
            .switch_api
            .set_switch_attribute
            .ok_or(Error::APIUnavailable)?;

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
        let set_switch_attribute = self
            .sai
            .switch_api
            .set_switch_attribute
            .ok_or(Error::APIUnavailable)?;

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
        let set_switch_attribute = self
            .sai
            .switch_api
            .set_switch_attribute
            .ok_or(Error::APIUnavailable)?;

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
        let set_switch_attribute = self
            .sai
            .switch_api
            .set_switch_attribute
            .ok_or(Error::APIUnavailable)?;

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
        let set_switch_attribute = self
            .sai
            .switch_api
            .set_switch_attribute
            .ok_or(Error::APIUnavailable)?;

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
        let set_switch_attribute = self
            .sai
            .switch_api
            .set_switch_attribute
            .ok_or(Error::APIUnavailable)?;

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
        let set_switch_attribute = self
            .sai
            .switch_api
            .set_switch_attribute
            .ok_or(Error::APIUnavailable)?;

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

pub struct VLAN<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for VLAN<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VLAN(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for VLAN<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> VLAN<'a> {
    pub fn get_members(&self) -> Result<Vec<VLANMember>, Error> {
        // check that API is available/callable
        let get_vlan_attribute = self
            .sai
            .vlan_api
            .get_vlan_attribute
            .ok_or(Error::APIUnavailable)?;

        let mut members: Vec<sai_object_id_t> = vec![0u64; 128];
        let mut attr = sai_attribute_t {
            id: _sai_vlan_attr_t_SAI_VLAN_ATTR_MEMBER_LIST,
            value: sai_attribute_value_t {
                objlist: sai_object_list_t {
                    count: 128,
                    list: members.as_mut_ptr(),
                },
            },
        };

        let st = unsafe { get_vlan_attribute(self.id, 1, &mut attr as *mut _) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            return Err(Error::SAI(Status::from(st)));
        }

        // iterate over the returned list and build the vector for return
        let count = unsafe { attr.value.objlist.count };
        let list = unsafe { attr.value.objlist.list };
        let mut ret: Vec<VLANMember> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let oid: sai_object_id_t = unsafe { *list.offset(i as isize) };
            ret.push(VLANMember {
                id: oid,
                sai: self.sai,
            });
        }
        Ok(ret)
    }
}

pub struct VLANMember<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for VLANMember<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VLANMember(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for VLANMember<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> VLANMember<'a> {
    pub fn remove(self) -> Result<(), Error> {
        // check that API is available/callable
        let remove_vlan_member = self
            .sai
            .vlan_api
            .remove_vlan_member
            .ok_or(Error::APIUnavailable)?;

        match unsafe { remove_vlan_member(self.id) } {
            0 => Ok(()),
            v => Err(Error::SAI(Status::from(v))),
        }
    }
}

pub struct Bridge<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
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
        let get_bridge_attribute = self
            .sai
            .bridge_api
            .get_bridge_attribute
            .ok_or(Error::APIUnavailable)?;

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
        let get_bridge_port_attribute = self
            .sai
            .bridge_api
            .get_bridge_port_attribute
            .ok_or(Error::APIUnavailable)?;

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
        let remove_bridge_port = self
            .sai
            .bridge_api
            .remove_bridge_port
            .ok_or(Error::APIUnavailable)?;

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

#[derive(Clone, Copy)]
pub struct HostIfTrapGroupID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTrapGroupID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTrapGroupID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfTrapGroupID> for sai_object_id_t {
    fn from(value: HostIfTrapGroupID) -> Self {
        value.id
    }
}

impl From<HostIfTrapGroup<'_>> for HostIfTrapGroupID {
    fn from(value: HostIfTrapGroup) -> Self {
        Self { id: value.id }
    }
}

#[derive(Copy, Clone)]
pub struct HostIfTrapGroup<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for HostIfTrapGroup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIfTrapGroup(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for HostIfTrapGroup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> HostIfTrapGroup<'a> {}

#[derive(Clone, Copy)]
pub struct HostIfTrapID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfTrap<'_>> for HostIfTrapID {
    fn from(value: HostIfTrap) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PacketAction {
    Drop,
    Forward,
    Copy,
    CopyCancel,
    Trap,
    Log,
    Deny,
    Transit,
    DoNotDrop,
}

impl From<PacketAction> for i32 {
    fn from(value: PacketAction) -> Self {
        match value {
            PacketAction::Drop => _sai_packet_action_t_SAI_PACKET_ACTION_DROP as i32,
            PacketAction::Forward => _sai_packet_action_t_SAI_PACKET_ACTION_FORWARD as i32,
            PacketAction::Copy => _sai_packet_action_t_SAI_PACKET_ACTION_COPY as i32,
            PacketAction::CopyCancel => _sai_packet_action_t_SAI_PACKET_ACTION_COPY_CANCEL as i32,
            PacketAction::Trap => _sai_packet_action_t_SAI_PACKET_ACTION_TRAP as i32,
            PacketAction::Log => _sai_packet_action_t_SAI_PACKET_ACTION_LOG as i32,
            PacketAction::Deny => _sai_packet_action_t_SAI_PACKET_ACTION_DENY as i32,
            PacketAction::Transit => _sai_packet_action_t_SAI_PACKET_ACTION_TRANSIT as i32,
            PacketAction::DoNotDrop => _sai_packet_action_t_SAI_PACKET_ACTION_DONOTDROP as i32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HostIfTrapType {
    STP,
    LACP,
    EAPOL,
    LLDP,
    PVRST,
    IGMPTypeQuery,
    IGMPTypeLeave,
    IGMPTypeV1Report,
    IGMPTypeV2Report,
    IGMPTypeV3Report,
    SamplePacket,
    UDLD,
    CDP,
    VTP,
    DTP,
    PAGP,
    PTP,
    PTPTxEvent,
    DHCPL2,
    DHCPv6L2,
    SwitchCustomRangeBase,
    ARPRequest,
    ARPResponse,
    DHCP,
    OSPF,
    PIM,
    VRRP,
    DHCPv6,
    OSPFv6,
    VRRPv6,
    IPv6NeighborDiscovery,
    IPv6MLDv1v2,
    IPv6MLDv1Report,
    IPv6MLDv1Done,
    MLDv2Report,
    UnknownL3Multicast,
    SNATMiss,
    DNATMiss,
    NATHairpin,
    IPv6NeighborSolicitation,
    IPv6NeighborAdvertisement,
    ISIS,
    RouterCustomRangeBase,
    IP2ME,
    SSH,
    SNMP,
    BGP,
    BGPv6,
    BFD,
    BFDv6,
    BFDMicro,
    BFDv6Micro,
    LDP,
    GNMI,
    P4rt,
    NTPClient,
    NTPServer,
    LocalIPCustomRangeBase,
    L3MTUError,
    TTLError,
    StaticFDBMove,
    PipelineDiscardEgressBuffer,
    PipelineDiscardWRED,
    PipelineDiscardRouter,
    MPLSTTLError,
    MPLSRouterAlertLabel,
    MPLSLabelLookupMiss,
    CustomExceptionRangeBase,
}

impl From<HostIfTrapType> for i32 {
    fn from(value: HostIfTrapType) -> Self {
        match value {
            HostIfTrapType::STP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_STP as i32,
            HostIfTrapType::LACP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LACP as i32,
            HostIfTrapType::EAPOL => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_EAPOL as i32,
            HostIfTrapType::LLDP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LLDP as i32,
            HostIfTrapType::PVRST => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PVRST as i32,
            HostIfTrapType::IGMPTypeQuery => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_QUERY as i32
            }
            HostIfTrapType::IGMPTypeLeave => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_LEAVE as i32
            }
            HostIfTrapType::IGMPTypeV1Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_V1_REPORT as i32
            }
            HostIfTrapType::IGMPTypeV2Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_V2_REPORT as i32
            }
            HostIfTrapType::IGMPTypeV3Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_V3_REPORT as i32
            }
            HostIfTrapType::SamplePacket => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SAMPLEPACKET as i32
            }
            HostIfTrapType::UDLD => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_UDLD as i32,
            HostIfTrapType::CDP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_CDP as i32,
            HostIfTrapType::VTP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_VTP as i32,
            HostIfTrapType::DTP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DTP as i32,
            HostIfTrapType::PAGP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PAGP as i32,
            HostIfTrapType::PTP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PTP as i32,
            HostIfTrapType::PTPTxEvent => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PTP_TX_EVENT as i32
            }
            HostIfTrapType::DHCPL2 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCP_L2 as i32,
            HostIfTrapType::DHCPv6L2 => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCPV6_L2 as i32
            }
            HostIfTrapType::SwitchCustomRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SWITCH_CUSTOM_RANGE_BASE as i32
            }
            HostIfTrapType::ARPRequest => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ARP_REQUEST as i32
            }
            HostIfTrapType::ARPResponse => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ARP_RESPONSE as i32
            }
            HostIfTrapType::DHCP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCP as i32,
            HostIfTrapType::OSPF => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_OSPF as i32,
            HostIfTrapType::PIM => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIM as i32,
            HostIfTrapType::VRRP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_VRRP as i32,
            HostIfTrapType::DHCPv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCPV6 as i32,
            HostIfTrapType::OSPFv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_OSPFV6 as i32,
            HostIfTrapType::VRRPv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_VRRPV6 as i32,
            HostIfTrapType::IPv6NeighborDiscovery => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_NEIGHBOR_DISCOVERY as i32
            }
            HostIfTrapType::IPv6MLDv1v2 => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_MLD_V1_V2 as i32
            }
            HostIfTrapType::IPv6MLDv1Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_MLD_V1_REPORT as i32
            }
            HostIfTrapType::IPv6MLDv1Done => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_MLD_V1_DONE as i32
            }
            HostIfTrapType::MLDv2Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MLD_V2_REPORT as i32
            }
            HostIfTrapType::UnknownL3Multicast => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_UNKNOWN_L3_MULTICAST as i32
            }
            HostIfTrapType::SNATMiss => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SNAT_MISS as i32
            }
            HostIfTrapType::DNATMiss => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DNAT_MISS as i32
            }
            HostIfTrapType::NATHairpin => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_NAT_HAIRPIN as i32
            }
            HostIfTrapType::IPv6NeighborSolicitation => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_NEIGHBOR_SOLICITATION as i32
            }
            HostIfTrapType::IPv6NeighborAdvertisement => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_NEIGHBOR_ADVERTISEMENT as i32
            }
            HostIfTrapType::ISIS => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ISIS as i32,
            HostIfTrapType::RouterCustomRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ROUTER_CUSTOM_RANGE_BASE as i32
            }
            HostIfTrapType::IP2ME => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IP2ME as i32,
            HostIfTrapType::SSH => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SSH as i32,
            HostIfTrapType::SNMP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SNMP as i32,
            HostIfTrapType::BGP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BGP as i32,
            HostIfTrapType::BGPv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BGPV6 as i32,
            HostIfTrapType::BFD => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFD as i32,
            HostIfTrapType::BFDv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFDV6 as i32,
            HostIfTrapType::BFDMicro => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFD_MICRO as i32
            }
            HostIfTrapType::BFDv6Micro => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFDV6_MICRO as i32
            }
            HostIfTrapType::LDP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LDP as i32,
            HostIfTrapType::GNMI => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_GNMI as i32,
            HostIfTrapType::P4rt => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_P4RT as i32,
            HostIfTrapType::NTPClient => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_NTPCLIENT as i32
            }
            HostIfTrapType::NTPServer => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_NTPSERVER as i32
            }
            HostIfTrapType::LocalIPCustomRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LOCAL_IP_CUSTOM_RANGE_BASE as i32
            }
            HostIfTrapType::L3MTUError => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_L3_MTU_ERROR as i32
            }
            HostIfTrapType::TTLError => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_TTL_ERROR as i32
            }
            HostIfTrapType::StaticFDBMove => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_STATIC_FDB_MOVE as i32
            }
            HostIfTrapType::PipelineDiscardEgressBuffer => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIPELINE_DISCARD_EGRESS_BUFFER as i32
            }
            HostIfTrapType::PipelineDiscardWRED => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIPELINE_DISCARD_WRED as i32
            }
            HostIfTrapType::PipelineDiscardRouter => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIPELINE_DISCARD_ROUTER as i32
            }
            HostIfTrapType::MPLSTTLError => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MPLS_TTL_ERROR as i32
            }
            HostIfTrapType::MPLSRouterAlertLabel => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MPLS_ROUTER_ALERT_LABEL as i32
            }
            HostIfTrapType::MPLSLabelLookupMiss => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MPLS_LABEL_LOOKUP_MISS as i32
            }
            HostIfTrapType::CustomExceptionRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_CUSTOM_EXCEPTION_RANGE_BASE as i32
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum HostIfTrapAttribute {
    TrapType(HostIfTrapType),
    PacketAction(PacketAction),
    TrapPriority(u32),
    ExcludePortList(Vec<PortID>),
    TrapGroup(HostIfTrapGroupID),
    MirrorSession(Vec<MirrorSessionID>),
    Counter(CounterID),
}

impl HostIfTrapAttribute {
    fn to_sai_attribute_t(
        &self,
        exclude_port_list_backing: &mut Vec<sai_object_id_t>,
        mirror_session_backing: &mut Vec<sai_object_id_t>,
    ) -> sai_attribute_t {
        match self {
            HostIfTrapAttribute::TrapType(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_TRAP_TYPE,
                value: sai_attribute_value_t { s32: (*v).into() },
            },
            HostIfTrapAttribute::PacketAction(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_PACKET_ACTION,
                value: sai_attribute_value_t { s32: (*v).into() },
            },
            HostIfTrapAttribute::TrapPriority(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_TRAP_PRIORITY,
                value: sai_attribute_value_t { u32_: *v },
            },
            HostIfTrapAttribute::ExcludePortList(v) => {
                // let mut arg: Vec<sai_object_id_t> = v.into_iter().map(|v| v.into()).collect();
                exclude_port_list_backing.clear();
                v.iter()
                    .for_each(|port_id| exclude_port_list_backing.push(port_id.id));
                sai_attribute_t {
                    id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_EXCLUDE_PORT_LIST,
                    value: sai_attribute_value_t {
                        objlist: sai_object_list_t {
                            count: exclude_port_list_backing.len() as u32,
                            list: exclude_port_list_backing.as_mut_ptr(),
                        },
                    },
                }
            }
            HostIfTrapAttribute::TrapGroup(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_TRAP_GROUP,
                value: sai_attribute_value_t { oid: (*v).into() },
            },
            HostIfTrapAttribute::MirrorSession(v) => {
                // let mut arg: Vec<sai_object_id_t> = v.into_iter().map(|v| v.into()).collect();
                mirror_session_backing.clear();
                v.iter().for_each(|mirror_session_id| {
                    mirror_session_backing.push(mirror_session_id.id)
                });
                sai_attribute_t {
                    id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_MIRROR_SESSION,
                    value: sai_attribute_value_t {
                        objlist: sai_object_list_t {
                            count: mirror_session_backing.len() as u32,
                            list: mirror_session_backing.as_mut_ptr(),
                        },
                    },
                }
            }
            HostIfTrapAttribute::Counter(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_COUNTER_ID,
                value: sai_attribute_value_t { oid: (*v).into() },
            },
        }
    }
}

pub struct HostIfTrap<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for HostIfTrap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIfTrap(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for HostIfTrap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> HostIfTrapGroup<'a> {}

#[derive(Clone, Copy, Debug)]
pub enum HostIfTableEntryType {
    Port,
    LAG,
    VLAN,
    TrapID,
    Wildcard,
}

impl From<HostIfTableEntryType> for i32 {
    fn from(value: HostIfTableEntryType) -> Self {
        match value {
            HostIfTableEntryType::Port => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_PORT as i32
            }
            HostIfTableEntryType::LAG => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_LAG as i32
            }
            HostIfTableEntryType::VLAN => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_VLAN as i32
            }
            HostIfTableEntryType::TrapID => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_TRAP_ID as i32
            }
            HostIfTableEntryType::Wildcard => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_WILDCARD as i32
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HostIfTableEntryChannelType {
    CB,
    FD,
    NetdevPhysicalPort,
    NetdevLogicalPort,
    NetdevL3,
    Genetlink,
}

impl From<HostIfTableEntryChannelType> for i32 {
    fn from(value: HostIfTableEntryChannelType) -> Self {
        match value {
            HostIfTableEntryChannelType::CB => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_CB as i32
            }
            HostIfTableEntryChannelType::FD => _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_FD as i32,
            HostIfTableEntryChannelType::NetdevPhysicalPort => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_PHYSICAL_PORT as i32
            }
            HostIfTableEntryChannelType::NetdevLogicalPort => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_LOGICAL_PORT as i32
            }
            HostIfTableEntryChannelType::NetdevL3 => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_L3 as i32
            }
            HostIfTableEntryChannelType::Genetlink => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_GENETLINK as i32
            }
        }
    }
}

// TODO: still needs From implementation for all types
// * @objects SAI_OBJECT_TYPE_PORT, SAI_OBJECT_TYPE_LAG, SAI_OBJECT_TYPE_ROUTER_INTERFACE
// SAI_HOSTIF_TABLE_ENTRY_ATTR_OBJ_ID,
#[derive(Clone, Copy)]
pub struct HostIfTableEntryObjectID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTableEntryObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTableEntryObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<PortID> for HostIfTableEntryObjectID {
    fn from(value: PortID) -> Self {
        Self { id: value.id }
    }
}

impl From<HostIfTableEntryObjectID> for sai_object_id_t {
    fn from(value: HostIfTableEntryObjectID) -> Self {
        value.id
    }
}

// TODO: still needs From implementation for all types
// * @objects SAI_OBJECT_TYPE_HOSTIF_TRAP, SAI_OBJECT_TYPE_HOSTIF_USER_DEFINED_TRAP
// SAI_HOSTIF_TABLE_ENTRY_ATTR_TRAP_ID,
#[derive(Clone, Copy)]
pub struct HostIfTableEntryTrapID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTableEntryTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTableEntryTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfTrapID> for HostIfTableEntryTrapID {
    fn from(value: HostIfTrapID) -> Self {
        Self { id: value.id }
    }
}

impl From<HostIfTableEntryTrapID> for sai_object_id_t {
    fn from(value: HostIfTableEntryTrapID) -> Self {
        value.id
    }
}

#[derive(Clone, Debug)]
pub enum HostIfTableEntryAttribute {
    Type(HostIfTableEntryType),
    ObjectID(HostIfTableEntryObjectID),
    TrapID(HostIfTableEntryTrapID),
    ChannelType(HostIfTableEntryChannelType),
    HostIf(HostIfID),
}

impl From<HostIfTableEntryAttribute> for sai_attribute_t {
    fn from(value: HostIfTableEntryAttribute) -> Self {
        match value {
            HostIfTableEntryAttribute::Type(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_TYPE,
                value: sai_attribute_value_t { s32: v.into() },
            },
            HostIfTableEntryAttribute::ObjectID(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_OBJ_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            HostIfTableEntryAttribute::TrapID(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_TRAP_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            HostIfTableEntryAttribute::ChannelType(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_CHANNEL_TYPE,
                value: sai_attribute_value_t { s32: v.into() },
            },
            HostIfTableEntryAttribute::HostIf(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_HOST_IF,
                value: sai_attribute_value_t { oid: v.into() },
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct HostIfTableEntryID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTableEntryID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTableEntryID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfTableEntryID> for sai_object_id_t {
    fn from(value: HostIfTableEntryID) -> Self {
        value.id
    }
}

impl From<HostIfTableEntry<'_>> for HostIfTableEntryID {
    fn from(value: HostIfTableEntry) -> Self {
        Self { id: value.id }
    }
}

#[derive(Copy, Clone)]
pub struct HostIfTableEntry<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for HostIfTableEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIfTableEntry(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for HostIfTableEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> HostIfTableEntry<'a> {}

#[derive(Clone, Copy)]
pub struct HostIfID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfID> for sai_object_id_t {
    fn from(value: HostIfID) -> Self {
        value.id
    }
}

impl From<HostIf<'_>> for HostIfID {
    fn from(value: HostIf) -> Self {
        Self { id: value.id }
    }
}

pub struct HostIf<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for HostIf<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIf(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for HostIf<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> HostIf<'a> {}

#[derive(Clone, Copy)]
pub struct PortID {
    id: sai_object_id_t,
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

pub struct Port<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
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

impl<'a> Port<'a> {}

#[derive(Clone, Copy)]
pub struct CounterID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for CounterID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for CounterID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<CounterID> for sai_object_id_t {
    fn from(value: CounterID) -> Self {
        value.id
    }
}

impl From<Counter<'_>> for CounterID {
    fn from(value: Counter) -> Self {
        Self { id: value.id }
    }
}

pub struct Counter<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for Counter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Counter(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for Counter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> Counter<'a> {}

#[derive(Clone, Copy)]
pub struct MirrorSessionID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for MirrorSessionID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for MirrorSessionID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<MirrorSessionID> for sai_object_id_t {
    fn from(value: MirrorSessionID) -> Self {
        value.id
    }
}

impl From<MirrorSession<'_>> for MirrorSessionID {
    fn from(value: MirrorSession) -> Self {
        Self { id: value.id }
    }
}

pub struct MirrorSession<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for MirrorSession<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MirrorSession(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for MirrorSession<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> MirrorSession<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sai_log_set() {
        let res = SAI::log_set_all(LogLevel::Info);
        assert!(res.is_ok());
        let res = SAI::log_set(API::Switch, LogLevel::Debug);
        assert!(res.is_ok());
    }

    #[test]
    fn sai_query_api_version() {
        let res = SAI::api_version();
        assert!(res.is_ok());
        // TODO: this works only for certain SAIs obviously
        assert!(res.unwrap() >= 11100);
    }

    #[test]
    fn sai_new_and_drop() {
        // this is our profile we're going to clone and pass around
        let profile = vec![(
            CString::from_vec_with_nul(SAI_KEY_INIT_CONFIG_FILE.to_vec()).unwrap(),
            CString::new("/does/not/exist").unwrap(),
        )];

        // first initialization must work
        let res = SAI::new(profile.clone());
        assert!(res.is_ok());

        // second initialization must fail
        let res2 = SAI::new(profile.clone());
        assert!(res2.is_err());
        let err = res2.unwrap_err();
        assert_eq!(err, InitError::AlreadyInitialized);

        // dropping first init, and initializing again must work
        drop(res);
        let res = SAI::new(profile.clone());
        assert!(res.is_ok());
    }
}
