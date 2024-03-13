use std::ffi::CStr;
use std::os::raw::c_char;

use xcvr_sys::idx_t;
use xcvr_sys::xcvr_port_type_t;
use xcvr_sys::xcvr_status_t;
use xcvr_sys::xcvr_transceiver_info_t;
use xcvr_sys::xcvr_transceiver_status_t;

mod common;

static LIBRARY_NAME: &[u8; 19] = b"xcvr-cel-seastone2\0";

static SUPPORTED_PLATFORMS: [&[u8; 25]; 1] = [b"x86_64-cel_seastone_2-r0\0"];

#[no_mangle]
pub extern "C" fn xcvr_library_name() -> *const c_char {
    LIBRARY_NAME.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn xcvr_is_supported_platform(platform: *const c_char) -> bool {
    if platform.is_null() {
        return false;
    }

    let platform_bytes = unsafe { CStr::from_ptr(platform) }.to_bytes_with_nul();

    SUPPORTED_PLATFORMS
        .iter()
        .map(|v| v.as_slice())
        .find(|supported_platform| *supported_platform == platform_bytes)
        .is_some()
}

#[no_mangle]
pub extern "C" fn xcvr_supported_platforms(
    supported_platforms: *mut *const c_char,
    supported_platforms_count: *mut usize,
) {
    if supported_platforms.is_null() {
        return;
    }

    if supported_platforms_count.is_null() {
        return;
    }

    unsafe { *supported_platforms_count = SUPPORTED_PLATFORMS.len() };
    unsafe { *supported_platforms = SUPPORTED_PLATFORMS.as_ptr() as *const c_char };
}

#[no_mangle]
pub extern "C" fn xcvr_num_physical_ports(
    platform: *const c_char,
    num: *mut idx_t,
) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    if num.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
    }

    unsafe { *num = 33 };
    xcvr_sys::XCVR_STATUS_SUCCESS
}

#[no_mangle]
pub extern "C" fn xcvr_get_presence(
    platform: *const c_char,
    index: idx_t,
    is_present: *mut bool,
) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    if is_present.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
    }
    common::get_presence(index)
        .map(|v| {
            unsafe { *is_present = v };
            xcvr_sys::XCVR_STATUS_SUCCESS
        })
        .unwrap_or_else(|err| err)
}

#[no_mangle]
pub extern "C" fn xcvr_get_supported_port_types(
    platform: *const c_char,
    index: idx_t,
    supported_port_types: *mut xcvr_port_type_t,
) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    if supported_port_types.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
    }
    let v = match index {
        0..=31 => {
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP28
                | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP_PLUS
                | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP
        }
        32 => {
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP28
                | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_PLUS
                | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP
        }
        _ => {
            return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
        }
    };
    unsafe { *supported_port_types = v };
    xcvr_sys::XCVR_STATUS_SUCCESS
}

#[no_mangle]
pub extern "C" fn xcvr_get_inserted_port_type(
    platform: *const c_char,
    index: idx_t,
    inserted_port_type: *mut xcvr_port_type_t,
) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    if inserted_port_type.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
    }
    common::get_inserted_port_type(index)
        .map(|v| {
            unsafe { *inserted_port_type = v };
            xcvr_sys::XCVR_STATUS_SUCCESS
        })
        .unwrap_or_else(|err| err)
}

#[no_mangle]
pub extern "C" fn xcvr_get_oper_status(
    platform: *const c_char,
    index: idx_t,
    oper_status: *mut bool,
) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    if oper_status.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
    }
    // this is the reverse of reset status
    common::get_reset_status(index)
        .map(|v| {
            unsafe { *oper_status = !v };
            xcvr_sys::XCVR_STATUS_SUCCESS
        })
        .unwrap_or_else(|err| err)
}

#[no_mangle]
pub extern "C" fn xcvr_get_reset_status(
    platform: *const c_char,
    index: idx_t,
    reset_status: *mut bool,
) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    if reset_status.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
    }
    common::get_reset_status(index)
        .map(|v| {
            unsafe { *reset_status = v };
            xcvr_sys::XCVR_STATUS_SUCCESS
        })
        .unwrap_or_else(|err| err)
}

#[no_mangle]
pub extern "C" fn xcvr_reset(platform: *const c_char, index: idx_t) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    if let Err(err) = common::reset(index) {
        return err;
    }
    xcvr_sys::XCVR_STATUS_SUCCESS
}

#[no_mangle]
pub extern "C" fn xcvr_get_low_power_mode(
    platform: *const c_char,
    index: idx_t,
    low_power_mode: *mut bool,
) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    if low_power_mode.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
    }
    common::get_lpmode(index)
        .map(|v| {
            unsafe { *low_power_mode = v };
            xcvr_sys::XCVR_STATUS_SUCCESS
        })
        .unwrap_or_else(|err| err)
}

#[no_mangle]
pub extern "C" fn xcvr_set_low_power_mode(
    platform: *const c_char,
    index: idx_t,
    low_power_mode: bool,
) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    if let Err(err) = common::set_lpmode(index, low_power_mode) {
        return err;
    }
    xcvr_sys::XCVR_STATUS_SUCCESS
}

#[no_mangle]
pub extern "C" fn xcvr_get_transceiver_info(
    _platform: *const c_char,
    _index: idx_t,
    _transceiver_info: *mut xcvr_transceiver_info_t,
) -> xcvr_status_t {
    xcvr_sys::XCVR_STATUS_ERROR_UNIMPLEMENTED
}

#[no_mangle]
pub extern "C" fn xcvr_get_transceiver_status(
    _platform: *const c_char,
    _index: idx_t,
    _transceiver_status: *mut xcvr_transceiver_status_t,
) -> xcvr_status_t {
    xcvr_sys::XCVR_STATUS_ERROR_UNIMPLEMENTED
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    use super::*;

    #[test]
    fn test_xcvr_library_name() {
        let lib_name = "xcvr-cel-seastone2";
        let get_lib_name_ptr = xcvr_library_name();
        let get_lib_name = unsafe { CStr::from_ptr(get_lib_name_ptr) };
        let get_lib_name_str = get_lib_name.to_str().unwrap();
        assert_eq!(lib_name, get_lib_name_str);
    }

    #[test]
    fn test_xcvr_is_supported_platform() {
        let p = CString::new("x86_64-cel_seastone_2-r0").unwrap();
        assert!(xcvr_is_supported_platform(p.as_ptr()));
        let p = CString::new("x86_64-dellemc_s5249f_c3538-r0").unwrap();
        assert!(!xcvr_is_supported_platform(p.as_ptr()));
        let p = CString::new("also not supported").unwrap();
        assert!(!xcvr_is_supported_platform(p.as_ptr()));
    }

    #[test]
    fn test_xcvr_supported_platforms() {
        // prep our arguments for the call
        let mut count: usize = 0;
        let mut sp: MaybeUninit<*const *const i8> = MaybeUninit::uninit();
        xcvr_supported_platforms(sp.as_mut_ptr() as _, &mut count);

        // count must be 1
        assert_eq!(count, 1);

        // building the strings from the double pointer can be daunting
        let sp = unsafe { sp.assume_init() };
        let mut ret: Vec<String> = Vec::with_capacity(count);
        for i in 0..count {
            let entry = unsafe { *sp.offset(i as isize) };
            let cstr = unsafe { CStr::from_ptr(entry) };
            let str = cstr.to_str().unwrap().to_string();
            ret.push(str);
        }

        // now we can compare
        let cmp = vec!["x86_64-cel_seastone_2-r0".to_string()];
        assert_eq!(cmp, ret);
    }
}
