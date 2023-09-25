// export all modules from here
pub mod bridge;
pub mod hostif;
pub mod port;
pub mod route;
pub mod router_interface;
pub mod switch;
pub mod virtual_router;
pub mod vlan;

use port::Port;
use port::PortID;
// we are re-exporting some things here
pub use sai_sys::sai_ip_prefix_t;
pub use sai_sys::sai_mac_t;
pub use sai_sys::SAI_KEY_INIT_CONFIG_FILE;

// imports for here
use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex, RwLock};

use ipnet::IpNet;
use sai_sys::*;
use std::os::raw::{c_char, c_int};
use std::ptr::{null, null_mut};

pub trait ObjectID<ID>
where
    sai_object_id_t: From<ID>,
{
    fn to_id(&self) -> ID;
}

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

static SAI_INITIALIZED: Mutex<bool> = Mutex::new(false);

static SWITCH_CREATED: Mutex<bool> = Mutex::new(false);

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
    APIFunctionUnavailable,
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
    switch_api_backing: sai_switch_api_t,
    switch_api_ptr: Option<*const sai_switch_api_t>,
    vlan_api_backing: sai_vlan_api_t,
    vlan_api_ptr: Option<*const sai_vlan_api_t>,
    bridge_api_backing: sai_bridge_api_t,
    bridge_api_ptr: Option<*const sai_bridge_api_t>,
    port_api_backing: sai_port_api_t,
    port_api_ptr: Option<*const sai_port_api_t>,
    hostif_api_backing: sai_hostif_api_t,
    hostif_api_ptr: Option<*const sai_hostif_api_t>,
    router_interface_api_backing: sai_router_interface_api_t,
    router_interface_api_ptr: Option<*const sai_router_interface_api_t>,
    route_api_backing: sai_route_api_t,
    route_api_ptr: Option<*const sai_route_api_t>,
}

impl SAI {
    fn switch_api(&self) -> Option<sai_switch_api_t> {
        self.switch_api_ptr.map(|api| unsafe { *api })
    }

    fn vlan_api(&self) -> Option<sai_vlan_api_t> {
        self.vlan_api_ptr.map(|api| unsafe { *api })
    }

    fn bridge_api(&self) -> Option<sai_bridge_api_t> {
        self.bridge_api_ptr.map(|api| unsafe { *api })
    }

    fn port_api(&self) -> Option<sai_port_api_t> {
        self.port_api_ptr.map(|api| unsafe { *api })
    }

    fn hostif_api(&self) -> Option<sai_hostif_api_t> {
        self.hostif_api_ptr.map(|api| unsafe { *api })
    }

    fn router_interface_api(&self) -> Option<sai_router_interface_api_t> {
        self.router_interface_api_ptr.map(|api| unsafe { *api })
    }

    fn route_api(&self) -> Option<sai_route_api_t> {
        self.route_api_ptr.map(|api| unsafe { *api })
    }

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
                ret.apis_query()?;

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

    // TODO: this shouldn't necessarily fail the initialization. However, for our case right now this is exactly what we need/want. So let's just continue like this.
    fn apis_query(&mut self) -> Result<(), Status> {
        // NOTE: we are not using sai_metadata_apis_query on purpose, as we got burned by it. If you want to know details, talk to mheese about it.
        // As we are only using a few select APIs at this point in time, we can also easily afford to simply query only the APIs that we need.
        //
        // Here is a dilemma with the implementation unfortunately. While the docs talk about "Caller allocated method table", the reality of implementations clearly looks
        // different: at least Broadcom SAI is returning a pointer to their own allocated/managed table, and they are not using a provided table at all. We can detect this
        // by comparing the returned pointer to the one that we passed in essentially.
        //
        // Furthermore, here is the biggest problem: just because you get a pointer to a table back, does not mean that it is actually populated with functions already
        // even though the function is considered a success and returns with success.
        // This means that in Rust (like in C) we need to dereference the returned pointer every time we use it, and we can't simply store a copy of the table that the
        // returned pointer is pointer to. Affected APIs are at least: vlan_api, router_interface_api.
        //
        // switch API
        {
            self.switch_api_backing = Default::default();
            let switch_api_ptr_orig = &self.switch_api_backing as *const _;
            let mut switch_api_ptr = &mut self.switch_api_backing as *mut _;
            let switch_api_ptr_ptr = &mut switch_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_SWITCH, switch_api_ptr_ptr as _) };
            if st != SAI_STATUS_SUCCESS as i32 {
                return Err(Status::from(st));
            }
            if switch_api_ptr_orig != switch_api_ptr {
                log::debug!(
                    "sai_api_query(SAI_API_SWITCH) updated pointer away from our own table"
                );
            }
            self.switch_api_ptr = Some(switch_api_ptr);
        }

        // vlan API
        {
            self.vlan_api_backing = Default::default();
            let vlan_api_ptr_orig = &self.vlan_api_backing as *const _;
            let mut vlan_api_ptr = &mut self.vlan_api_backing as *mut _;
            let vlan_api_ptr_ptr = &mut vlan_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_VLAN, vlan_api_ptr_ptr as _) };
            if st != SAI_STATUS_SUCCESS as i32 {
                return Err(Status::from(st));
            }
            if vlan_api_ptr_orig != vlan_api_ptr {
                log::debug!("sai_api_query(SAI_API_VLAN) updated pointer away from our own table");
            }
            self.vlan_api_ptr = Some(vlan_api_ptr);
        }

        // bridge API
        {
            self.bridge_api_backing = Default::default();
            let bridge_api_ptr_orig = &self.bridge_api_backing as *const _;
            let mut bridge_api_ptr = &mut self.bridge_api_backing as *mut _;
            let bridge_api_ptr_ptr = &mut bridge_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_BRIDGE, bridge_api_ptr_ptr as _) };
            if st != SAI_STATUS_SUCCESS as i32 {
                return Err(Status::from(st));
            }
            if bridge_api_ptr_orig != bridge_api_ptr {
                log::debug!(
                    "sai_api_query(SAI_API_BRIDGE) updated pointer away from our own table"
                );
            }
            self.bridge_api_ptr = Some(bridge_api_ptr);
        }

        // port API
        {
            self.port_api_backing = Default::default();
            let port_api_ptr_orig = &self.port_api_backing as *const _;
            let mut port_api_ptr = &mut self.port_api_backing as *mut _;
            let port_api_ptr_ptr = &mut port_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_PORT, port_api_ptr_ptr as _) };
            if st != SAI_STATUS_SUCCESS as i32 {
                return Err(Status::from(st));
            }
            if port_api_ptr_orig != port_api_ptr {
                log::debug!("sai_api_query(SAI_API_PORT) updated pointer away from our own table");
            }
            self.port_api_ptr = Some(port_api_ptr);
        }

        // hostif API
        {
            self.hostif_api_backing = Default::default();
            let hostif_api_ptr_orig = &self.hostif_api_backing as *const _;
            let mut hostif_api_ptr = &mut self.hostif_api_backing as *mut _;
            let hostif_api_ptr_ptr = &mut hostif_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_HOSTIF, hostif_api_ptr_ptr as _) };
            if st != SAI_STATUS_SUCCESS as i32 {
                return Err(Status::from(st));
            }
            if hostif_api_ptr_orig == hostif_api_ptr {
                log::debug!(
                    "sai_api_query(SAI_API_HOSTIF) updated pointer away from our own table"
                );
            }
            self.hostif_api_ptr = Some(hostif_api_ptr);
        }

        // router interface API
        {
            self.router_interface_api_backing = Default::default();
            let router_interface_api_ptr_orig = &self.router_interface_api_backing as *const _;
            let mut router_interface_api_ptr = &mut self.router_interface_api_backing as *mut _;
            let router_interface_api_ptr_ptr = &mut router_interface_api_ptr as *mut *mut _;
            let st = unsafe {
                sai_api_query(
                    _sai_api_t_SAI_API_ROUTER_INTERFACE,
                    router_interface_api_ptr_ptr as _,
                )
            };
            if st != SAI_STATUS_SUCCESS as i32 {
                return Err(Status::from(st));
            }
            if router_interface_api_ptr_orig != router_interface_api_ptr {
                log::debug!("sai_api_query(SAI_API_ROUTER_INTERFACE) updated pointer away from our own table");
            }
            self.router_interface_api_ptr = Some(router_interface_api_ptr);
        }

        // route API
        {
            self.route_api_backing = Default::default();
            let route_api_ptr_orig = &self.route_api_backing as *const _;
            let mut route_api_ptr = &mut self.route_api_backing as *mut _;
            let route_api_ptr_ptr = &mut route_api_ptr as *mut *mut _;
            let st = unsafe { sai_api_query(_sai_api_t_SAI_API_ROUTE, route_api_ptr_ptr as _) };
            if st != SAI_STATUS_SUCCESS as i32 {
                return Err(Status::from(st));
            }
            if route_api_ptr_orig != route_api_ptr {
                log::debug!("sai_api_query(SAI_API_ROUTE) updated pointer away from our own table");
            }
            self.route_api_ptr = Some(route_api_ptr);
        }
        Ok(())
    }

    // NOTE: we abandoned this easy and convenient way of querying the APIs as we got burned by it.
    // Ask mheese if you are curious about the details.
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

    pub fn switch_create(
        &self,
        attrs: Vec<switch::SwitchAttribute>,
    ) -> Result<switch::Switch, Error> {
        // check first if a switch was created already
        let mut switch_created = SWITCH_CREATED.lock().unwrap();
        if *switch_created {
            return Err(Error::SwitchAlreadyCreated);
        }
        // check that API is available/callable
        let switch_api = self.switch_api().ok_or(Error::APIUnavailable)?;
        let create_switch = switch_api
            .create_switch
            .ok_or(Error::APIFunctionUnavailable)?;

        // now call it
        let mut sw_id: sai_object_id_t = 0;
        let attrs_arg: Vec<sai_attribute_t> = attrs.into_iter().map(Into::into).collect();
        // attrs_arg.push(sai_attribute_t {
        //     id: _sai_switch_attr_t_SAI_SWITCH_ATTR_SWITCH_STATE_CHANGE_NOTIFY,
        //     value: sai_attribute_value_t {
        //         ptr: switch_state_change_cb as sai_pointer_t,
        //     },
        // });
        let attrs_arg_ptr = attrs_arg.as_ptr();
        match unsafe { create_switch(&mut sw_id, attrs_arg.len() as u32, attrs_arg_ptr) } {
            0 => {
                *switch_created = true;
                Ok(switch::Switch {
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

// impl From<Counter<'_>> for CounterID {
//     fn from(value: Counter) -> Self {
//         Self { id: value.id }
//     }
// }

// #[derive(Clone, Copy)]
// pub struct Counter<'a> {
//     id: sai_object_id_t,
//     sai: &'a SAI,
// }

// impl std::fmt::Debug for Counter<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Counter(oid:{:#x})", self.id)
//     }
// }

// impl std::fmt::Display for Counter<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "oid:{:#x}", self.id)
//     }
// }

// impl<'a> Counter<'a> {}

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

// impl From<MirrorSession<'_>> for MirrorSessionID {
//     fn from(value: MirrorSession) -> Self {
//         Self { id: value.id }
//     }
// }

// pub struct MirrorSession<'a> {
//     id: sai_object_id_t,
//     sai: &'a SAI,
// }

// impl std::fmt::Debug for MirrorSession<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "MirrorSession(oid:{:#x})", self.id)
//     }
// }

// impl std::fmt::Display for MirrorSession<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "oid:{:#x}", self.id)
//     }
// }

// impl<'a> MirrorSession<'a> {}

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
