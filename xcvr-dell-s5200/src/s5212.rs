use std::fs::File;

use xcvr_sys::idx_t;
use xcvr_sys::xcvr_port_type_t;
use xcvr_sys::xcvr_status_t;

pub(super) fn xcvr_num_physical_ports() -> idx_t {
    15
}

pub(super) fn xcvr_get_presence(index: idx_t) -> Result<bool, xcvr_status_t> {
    if index >= xcvr_num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    // our I2C devices start at device "2", and always use the address 0x50
    // If we want to know if there is a transceiver present, we can simply
    // check if we can open the eeprom file in sysfs
    let i = index + 2;
    let eeprom_file_path = format!("/sys/class/i2c-adapter/i2c-{i}/{i}-0050/eeprom");
    match File::open(eeprom_file_path) {
        Ok(_) => Ok(true),
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                // if the file cannot be found, then there is something wrong with the setup
                Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL)
            } else {
                // otherwise this means that there is no transceiver present
                // TODO: there are probably more cases where we should fail
                Ok(false)
            }
        }
    }
}

pub(super) fn xcvr_get_supported_port_types(
    index: idx_t,
) -> Result<xcvr_port_type_t, xcvr_status_t> {
    match index {
        0..=11 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP28
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_PLUS
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP),
        12..=14 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP28
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP_PLUS
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP),
        _ => Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    }
}
