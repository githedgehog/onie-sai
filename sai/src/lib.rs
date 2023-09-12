use std::ffi::{CStr, CString};
use std::sync::{Arc, RwLock, Mutex};

use sai_sys::*;
use std::os::raw::{c_char, c_int};
use std::ptr::{null, null_mut};

static PROFILE_GET_NEXT_VALUE_CALLBACK: RwLock<Option<Box<dyn Fn(sai_switch_profile_id_t, *mut *const c_char, *mut *const c_char) -> c_int + Send + Sync>>> = RwLock::new(None);

extern "C" fn profile_get_next_value_cb(profile_id: sai_switch_profile_id_t,
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

static PROFILE_GET_VALUE_CALLBACK: RwLock<Option<Box<dyn Fn(sai_switch_profile_id_t, *const c_char) -> *const c_char + Send + Sync>>> = RwLock::new(None);

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
struct SAIProfile {
    profile_idx: Mutex<usize>,
    profile: Vec<(CString, CString)>,
}

impl SAIProfile {
    fn profile_get_next_value(&self, _profile_id: sai_switch_profile_id_t, variable: *mut *const c_char, value: *mut *const c_char) -> c_int {
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

    fn profile_get_value(&self, _profile_id: sai_switch_profile_id_t, variable: *const c_char) -> *const c_char {
        if variable == null() {
            return null();
        }

        // convert variable to Rust string
        let var_cstr = unsafe { CStr::from_ptr(variable) };
        let var = var_cstr.to_owned();

        // NOTE: this is kind of unsafe because it assumes that the vector isn't being dropped
        // however, we know that because we own it, and this closure is never being called again when
        // the profile is being dropped
        self.profile.iter()
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
            -0x00010000..=0x0001FFFF => Status::InvalidAttribute(value.abs()-0x00010000),
            -0x00020000..=0x0002FFFF => Status::InvalidAttributeValue(value.abs()-0x00020000),
            -0x00030000..=0x0003FFFF => Status::AttributeNotImplemented(value.abs()-0x00030000),
            -0x00040000..=0x0004FFFF => Status::UnknownAttribute(value.abs()-0x00040000),
            -0x00050000..=0x0005FFFF => Status::AttributeNotSupported(value.abs()-0x00050000),
            _ => Status::Unknown(value),
        }
    }
}

#[derive(Debug)]
pub struct SAI {}

impl SAI {

    pub fn new(profile: Vec<(CString, CString)>) -> Result<SAI, InitError> {
        let init_lock = SAI_INITIALIZED.try_lock();
        if let Ok(mut sai_initialized) = init_lock {
            // SAI is a singleton, so if this is true, it means it was already initialized
            if *sai_initialized {
                Err(InitError::AlreadyInitialized)
            } else {
                // we will return this, and the whole SAI_INITIALIZED lock is just there to ensure it is a singleton
                let ret = SAI {};

                // deal with the profile, and making sure there is a closure which can be called for it which has access to the map
                let p1 = Arc::new(SAIProfile {
                    profile_idx: Mutex::new(0),
                    profile: profile,
                });
                let p2 = Arc::clone(&p1);
                {
                    let mut cb_write_lock = PROFILE_GET_NEXT_VALUE_CALLBACK.write().unwrap();
                    *cb_write_lock = Some(Box::new(move |profile_id: sai_switch_profile_id_t, variable: *mut *const c_char, value: *mut *const c_char| { p1.profile_get_next_value(profile_id, variable, value) }));
                }
                {
                    let mut cb_write_lock = PROFILE_GET_VALUE_CALLBACK.write().unwrap();
                    *cb_write_lock = Some(Box::new(move |profile_id: sai_switch_profile_id_t, variable: *const c_char| { p2.profile_get_value(profile_id, variable) }));
                }

                // this calls the the underlying C function
                ret.init()?;

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
    fn sai_new_and_drop() {
        // this is our profile we're going to clone and pass around
        let profile = vec![(CString::from_vec_with_nul(SAI_KEY_INIT_CONFIG_FILE.to_vec()).unwrap(), CString::new("/does/not/exist").unwrap())];

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
