use libloading::{Library, Symbol};
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::NulError;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::path::Path;
use thiserror::Error;

use xcvr_sys::idx_t;
use xcvr_sys::xcvr_port_type_t;
use xcvr_sys::xcvr_status_t;
use xcvr_sys::xcvr_transceiver_info_t;
use xcvr_sys::xcvr_transceiver_status_t;

type XcvrLibraryNameFunc = extern "C" fn() -> *const c_char;
type XcvrIsSupportedPlatformFunc = extern "C" fn(platform: *const c_char) -> bool;
type XcvrSupportedPlatformsFunc =
    extern "C" fn(supported_platforms: *mut *const c_char, supported_platforms_count: *mut usize);
type XcvrNumPhysicalPortsFunc =
    extern "C" fn(platform: *const c_char, num: *mut idx_t) -> xcvr_status_t;
type XcvrGetPresenceFunc =
    extern "C" fn(platform: *const c_char, index: idx_t, is_present: *mut bool) -> xcvr_status_t;
type XcvrGetSupportedPortTypesFunc = extern "C" fn(
    platform: *const c_char,
    index: idx_t,
    supported_port_types: *mut xcvr_port_type_t,
) -> xcvr_status_t;
type XcvrGetInsertedPortTypeFunc = extern "C" fn(
    platform: *const c_char,
    index: idx_t,
    supported_port_types: *mut xcvr_port_type_t,
) -> xcvr_status_t;
type XcvrGetOperStatusFunc =
    extern "C" fn(platform: *const c_char, index: idx_t, oper_status: *mut bool) -> xcvr_status_t;
type XcvrGetResetStatusFunc =
    extern "C" fn(platform: *const c_char, index: idx_t, reset_status: *mut bool) -> xcvr_status_t;
type XcvrResetFunc = extern "C" fn(platform: *const c_char, index: idx_t) -> xcvr_status_t;
type XcvrGetLowPowerModeFunc = extern "C" fn(
    platform: *const c_char,
    index: idx_t,
    low_power_mode: *mut bool,
) -> xcvr_status_t;
type XcvrSetLowPowerModeFunc =
    extern "C" fn(platform: *const c_char, index: idx_t, low_power_mode: bool) -> xcvr_status_t;
type XcvrGetTransceiverInfoFunc = extern "C" fn(
    platform: *const c_char,
    index: idx_t,
    transceiver_info: *mut xcvr_transceiver_info_t,
) -> xcvr_status_t;
type XcvrGetTransceiverStatusFunc = extern "C" fn(
    platform: *const c_char,
    index: idx_t,
    transceiver_status: *mut xcvr_transceiver_status_t,
) -> xcvr_status_t;

#[derive(Error, Debug)]
pub enum XcvrError {
    #[error("dynamic library error")]
    LoadingDynamicLibrary(libloading::Error),
    #[error("")]
    GettingDynamicLibrarySymbol(#[from] libloading::Error),
    #[error("library returned NULL pointer")]
    NullReturn,
    #[error("string argument contained null characters")]
    InvalidStringArgument(NulError),
}

pub struct XcvrLibraryLoader {
    lib: Library,
}

impl XcvrLibraryLoader {
    pub fn new(path: &Path) -> Result<Self, XcvrError> {
        Ok(Self {
            lib: unsafe { Library::new(path.as_os_str()) }
                .map_err(|e| XcvrError::LoadingDynamicLibrary(e))?,
        })
    }

    pub fn lib(&self) -> Result<XcvrLibrary, XcvrError> {
        XcvrLibrary::new(&self.lib)
    }
}

pub struct XcvrLibrary<'a> {
    xcvr_library_name: Symbol<'a, XcvrLibraryNameFunc>,
    xcvr_is_supported_platform: Symbol<'a, XcvrIsSupportedPlatformFunc>,
    xcvr_supported_platforms: Symbol<'a, XcvrSupportedPlatformsFunc>,
    xcvr_num_physical_ports: Symbol<'a, XcvrNumPhysicalPortsFunc>,
    xcvr_get_presence: Symbol<'a, XcvrGetPresenceFunc>,
    xcvr_get_supported_port_types: Symbol<'a, XcvrGetSupportedPortTypesFunc>,
    xcvr_get_inserted_port_type: Symbol<'a, XcvrGetInsertedPortTypeFunc>,
    xcvr_get_oper_status: Symbol<'a, XcvrGetOperStatusFunc>,
    xcvr_get_reset_status: Symbol<'a, XcvrGetResetStatusFunc>,
    xcvr_reset: Symbol<'a, XcvrResetFunc>,
    xcvr_get_low_power_mode: Symbol<'a, XcvrGetLowPowerModeFunc>,
    xcvr_set_low_power_mode: Symbol<'a, XcvrSetLowPowerModeFunc>,
    xcvr_get_transceiver_info: Symbol<'a, XcvrGetTransceiverInfoFunc>,
    xcvr_get_transceiver_status: Symbol<'a, XcvrGetTransceiverStatusFunc>,
}

impl<'a> XcvrLibrary<'a> {
    pub fn new(lib: &'a Library) -> Result<Self, XcvrError> {
        Ok(Self {
            xcvr_library_name: unsafe { lib.get(b"xcvr_library_name\0") }?,
            xcvr_is_supported_platform: unsafe { lib.get(b"xcvr_is_supported_platform\0") }?,
            xcvr_supported_platforms: unsafe { lib.get(b"xcvr_supported_platforms\0") }?,
            xcvr_num_physical_ports: unsafe { lib.get(b"xcvr_num_physical_ports\0") }?,
            xcvr_get_presence: unsafe { lib.get(b"xcvr_get_presence\0") }?,
            xcvr_get_supported_port_types: unsafe { lib.get(b"xcvr_get_supported_port_types\0") }?,
            xcvr_get_inserted_port_type: unsafe { lib.get(b"xcvr_get_inserted_port_type\0") }?,
            xcvr_get_oper_status: unsafe { lib.get(b"xcvr_get_oper_status\0") }?,
            xcvr_get_reset_status: unsafe { lib.get(b"xcvr_get_reset_status\0") }?,
            xcvr_reset: unsafe { lib.get(b"xcvr_reset\0") }?,
            xcvr_get_low_power_mode: unsafe { lib.get(b"xcvr_get_low_power_mode\0") }?,
            xcvr_set_low_power_mode: unsafe { lib.get(b"xcvr_set_low_power_mode\0") }?,
            xcvr_get_transceiver_info: unsafe { lib.get(b"xcvr_get_transceiver_info\0") }?,
            xcvr_get_transceiver_status: unsafe { lib.get(b"xcvr_get_transceiver_status\0") }?,
        })
    }

    pub fn library_name(&self) -> Result<String, XcvrError> {
        let ret = (self.xcvr_library_name)();
        if ret.is_null() {
            return Err(XcvrError::NullReturn);
        }
        Ok(unsafe { CStr::from_ptr(ret) }.to_string_lossy().to_string())
    }

    pub fn is_supported_platform(&self, platform: &str) -> Result<bool, XcvrError> {
        let arg = CString::new(platform).map_err(|e| XcvrError::InvalidStringArgument(e))?;
        let ret = (self.xcvr_is_supported_platform)(arg.as_ptr());
        Ok(ret)
    }

    pub fn supported_platforms(&self) -> Result<Vec<String>, XcvrError> {
        // prep our arguments for the call
        let mut count: usize = 0;
        let mut sp: MaybeUninit<*const *const i8> = MaybeUninit::uninit();

        // do the call
        (self.xcvr_supported_platforms)(sp.as_mut_ptr() as _, &mut count);

        // building the strings from the double pointer can be daunting
        let sp = unsafe { sp.assume_init() };
        let mut ret: Vec<String> = Vec::with_capacity(count);
        for i in 0..count {
            let entry = unsafe { *sp.offset(i as isize) };
            let cstr = unsafe { CStr::from_ptr(entry) };
            let str = cstr.to_str().unwrap().to_string();
            ret.push(str);
        }
        Ok(ret)
    }
}
