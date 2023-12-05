mod common;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use std::thread;
use std::time::Duration;

use xcvr_sys::idx_t;
use xcvr_sys::xcvr_port_type_t;
use xcvr_sys::xcvr_status_t;
use xcvr_sys::xcvr_transceiver_info_t;
use xcvr_sys::xcvr_transceiver_status_t;

static LIBRARY_NAME: &[u8; 10] = b"xcvr-pddf\0";

/// List of *known* supported platforms (that we have tested this with).
/// NOTE: there are potentially more platforms supported. Use the `xcvr_is_supported_platform()`
/// function to check if a platform is supported. It checks for the existence of the configuration
/// file which is technically all that is required and is what makes this portable.
static SUPPORTED_PLATFORMS: [&[u8; 30]; 3] = [
    b"x86_64-accton_as4630_54npe-r0\0",
    b"x86_64-accton_as7326_56x-r0\0\0\0",
    b"x86_64-accton_as7726_32x-r0\0\0\0",
];

#[no_mangle]
pub extern "C" fn xcvr_library_name() -> *const c_char {
    LIBRARY_NAME.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn xcvr_is_supported_platform(platform: *const c_char) -> bool {
    if platform.is_null() {
        return false;
    }

    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();

    // NOTE: instead of checking the list of supported platforms, we simply check for the
    // existence of the configuration file. This is ultimately more portable.
    // we simply test if the configuration file exists as a test
    Path::new(format!("/etc/platform/{}/pddf_xcvr_settings.json", platform).as_str()).exists()
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
    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();

    let ports = match crate::common::get_ports(platform.as_ref()) {
        Ok(ports) => ports,
        Err(_) => return xcvr_sys::XCVR_STATUS_ERROR_GENERAL,
    };

    unsafe { *num = ports.len() as idx_t };
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
    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();
    let port = match crate::common::get_port(platform.as_ref(), index) {
        Ok(port) => port,
        Err(_) => return xcvr_sys::XCVR_STATUS_ERROR_GENERAL,
    };

    // if this is a port which does not have a transceiver, then we'll just return true
    if !port.has_transceiver {
        unsafe { *is_present = true };
        return xcvr_sys::XCVR_STATUS_SUCCESS;
    }

    // otherwise we'll go through the proper check of the PDDF kernel module
    let presence = match crate::common::get_presence(&port) {
        Ok(presence) => presence,
        Err(status) => return status,
    };
    unsafe { *is_present = presence };
    xcvr_sys::XCVR_STATUS_SUCCESS
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
    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();
    let port = match crate::common::get_port(platform.as_ref(), index) {
        Ok(port) => port,
        Err(_) => return xcvr_sys::XCVR_STATUS_ERROR_GENERAL,
    };

    // we'll just OR them all together
    let spt = port.supported_port_types.iter().fold(0, |acc, &v| acc | v);
    unsafe { *supported_port_types = spt };
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
    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();
    let port = match crate::common::get_port(platform.as_ref(), index) {
        Ok(port) => port,
        Err(_) => return xcvr_sys::XCVR_STATUS_ERROR_GENERAL,
    };

    // if this is a port which does not have a transceiver, then we'll just return
    // the folded supported port types
    if !port.has_transceiver {
        let spt = port.supported_port_types.iter().fold(0, |acc, &v| acc | v);
        unsafe { *inserted_port_type = spt };
        return xcvr_sys::XCVR_STATUS_SUCCESS;
    }

    // otherwise we'll go through the proper check by reading it from the EEPROM of the transceiver
    let port_type = match crate::common::get_inserted_port_type(&port) {
        Ok(port_type) => port_type,
        Err(status) => return status,
    };
    unsafe { *inserted_port_type = port_type };
    xcvr_sys::XCVR_STATUS_SUCCESS
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
    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();
    let port = match crate::common::get_port(platform.as_ref(), index) {
        Ok(port) => port,
        Err(_) => return xcvr_sys::XCVR_STATUS_ERROR_GENERAL,
    };

    // if this is a port which does not have a transceiver, then we'll just return true
    if !port.has_transceiver {
        unsafe { *oper_status = true };
        return xcvr_sys::XCVR_STATUS_SUCCESS;
    }

    // otherwise we'll go through the proper check of the PDDF kernel module
    let reset = match crate::common::get_reset(&port) {
        Ok(reset) => reset,
        Err(status) => return status,
    };
    unsafe { *oper_status = !reset };
    xcvr_sys::XCVR_STATUS_SUCCESS
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
    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();
    let port = match crate::common::get_port(platform.as_ref(), index) {
        Ok(port) => port,
        Err(_) => return xcvr_sys::XCVR_STATUS_ERROR_GENERAL,
    };

    // if this is a port which does not have a transceiver, then we'll just return false
    if !port.has_transceiver {
        unsafe { *reset_status = false };
        return xcvr_sys::XCVR_STATUS_SUCCESS;
    }

    // otherwise we'll go through the proper check of the PDDF kernel module
    let reset = match crate::common::get_reset(&port) {
        Ok(reset) => reset,
        Err(status) => return status,
    };
    unsafe { *reset_status = reset };
    xcvr_sys::XCVR_STATUS_SUCCESS
}

#[no_mangle]
pub extern "C" fn xcvr_reset(platform: *const c_char, index: idx_t) -> xcvr_status_t {
    if platform.is_null() {
        return xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM;
    }
    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();
    let port = match crate::common::get_port(platform.as_ref(), index) {
        Ok(port) => port,
        Err(_) => return xcvr_sys::XCVR_STATUS_ERROR_GENERAL,
    };

    // if this is a port which does not have a transceiver, then we'll just return
    // with an error here
    if !port.has_transceiver {
        return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
    }

    // otherwise we'll go through the proper reset procedure
    if let Err(status) = crate::common::set_reset(&port, true) {
        return status;
    }
    thread::sleep(Duration::from_millis(1000));
    if let Err(status) = crate::common::set_reset(&port, false) {
        return status;
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
    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();
    let port = match crate::common::get_port(platform.as_ref(), index) {
        Ok(port) => port,
        Err(_) => return xcvr_sys::XCVR_STATUS_ERROR_GENERAL,
    };

    // if this is a port which does not have a transceiver, then we'll just return false
    if !port.has_transceiver {
        unsafe { *low_power_mode = false };
        return xcvr_sys::XCVR_STATUS_SUCCESS;
    }

    // otherwise we'll go through the proper value from the PDDF kernel module
    let lp = match crate::common::get_low_power_mode(&port) {
        Ok(lp) => lp,
        Err(status) => return status,
    };
    unsafe { *low_power_mode = lp };
    xcvr_sys::XCVR_STATUS_SUCCESS
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
    let platform = unsafe { CStr::from_ptr(platform) }.to_string_lossy();
    let port = match crate::common::get_port(platform.as_ref(), index) {
        Ok(port) => port,
        Err(_) => return xcvr_sys::XCVR_STATUS_ERROR_GENERAL,
    };

    // if this is a port which does not have a transceiver, then we'll just return
    // with an error here
    if !port.has_transceiver {
        return xcvr_sys::XCVR_STATUS_ERROR_GENERAL;
    }

    if let Err(status) = crate::common::set_low_power_mode(&port, low_power_mode) {
        return status;
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
