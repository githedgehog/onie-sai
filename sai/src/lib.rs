use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex, RwLock};

use sai_sys::*;
use std::os::raw::{c_char, c_int};
use std::ptr::{null, null_mut};

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
        unsafe {
            *variable = entry.0.as_ptr() as *const c_char;
            *value = entry.1.as_ptr() as *const c_char
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

#[derive(Debug)]
pub struct SAI {
    apis: sai_apis_t,
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
                let mut ret = SAI {
                    apis: Default::default(),
                };

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
                ret.metadata_api_query()?;

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

    fn metadata_api_query(&mut self) -> Result<i32, Status> {
        // query available functionality
        unsafe {
            match sai_metadata_apis_query(Some(sai_api_query), &mut self.apis) {
                v if v >= 0 => Ok(v),
                v => Err(Status::from(v)),
            }
        }
    }

    fn uninit(&self) -> Result<(), Status> {
        unsafe {
            match sai_api_uninitialize() {
                0 => Ok(()),
                v => Err(Status::from(v)),
            }
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
