use std::thread::sleep;
use std::time::Duration;

use xcvr_sys::idx_t;
use xcvr_sys::xcvr_port_type_t;
use xcvr_sys::xcvr_status_t;

use crate::common::eeprom_file_path;
use crate::common::Eeprom;
use crate::common::MemoryMappedFile;

// PCI_RESOURCE_PATH is specific per platform, so we need to keep it in the specific module
const PCI_RESOURCE_PATH: &str = "/sys/bus/pci/devices/0000:04:00.0/resource0";

pub(super) fn xcvr_num_physical_ports() -> idx_t {
    56
}

pub(super) fn xcvr_get_supported_port_types(
    index: idx_t,
) -> Result<xcvr_port_type_t, xcvr_status_t> {
    match index {
        0..=47 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP28
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_PLUS
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP),
        48..=51 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFPDD),
        52..=55 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP28
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP_PLUS
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP),
        _ => Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    }
}

pub(super) fn xcvr_get_presence(index: idx_t) -> Result<bool, xcvr_status_t> {
    if index >= xcvr_num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    // port offset starts with 0x4004
    let port_offset: usize = 0x4004 + (index as usize * 16);

    let f = MemoryMappedFile::open(PCI_RESOURCE_PATH)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    let reg_value = f.pci_get_value(port_offset) as i32;

    // ModPrsL is active low
    // (1 << 4) is for checking invalid port num for QSFP ports
    // (1 << 0) is for checking invalid port num for SFP ports
    match index {
        0..=47 => Ok(reg_value & (1 << 0) == 0),
        48..=55 => Ok(reg_value & (1 << 4) == 0),
        _ => Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    }
}

pub(super) fn xcvr_get_reset_status(index: idx_t) -> Result<bool, xcvr_status_t> {
    if index >= xcvr_num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    // not necessary to check for SFP ports apparently
    if let 0..=47 = index {
        return Ok(false);
    }

    // port offset starts with 0x4000
    // NOTE: this is *not* the same as for presence!
    let port_offset: usize = 0x4000 + (index as usize * 16);

    let f = MemoryMappedFile::open(PCI_RESOURCE_PATH)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    let reg_value = f.pci_get_value(port_offset) as i32;

    // Mask off 4th bit for reset status
    Ok((reg_value & (1 << 4)) == 0)
}

pub(super) fn xcvr_get_low_power_mode(index: idx_t) -> Result<bool, xcvr_status_t> {
    if index >= xcvr_num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    // not necessary to check for SFP ports apparently
    if let 0..=47 = index {
        return Ok(false);
    }

    // port offset starts with 0x4000
    // NOTE: this is *not* the same as for presence!
    let port_offset: usize = 0x4000 + (index as usize * 16);

    let f = MemoryMappedFile::open(PCI_RESOURCE_PATH)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    let reg_value = f.pci_get_value(port_offset) as i32;

    // Mask off 6th bit for low power mode
    Ok((reg_value & (1 << 6)) == 0)
}

pub(super) fn xcvr_reset(index: idx_t) -> Result<(), xcvr_status_t> {
    if index >= xcvr_num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    // not necessary to check for SFP ports apparently
    if let 0..=47 = index {
        return Ok(());
    }

    // port offset starts with 0x4000
    // NOTE: this is *not* the same as for presence!
    let port_offset: usize = 0x4000 + (index as usize * 16);

    let mut f = MemoryMappedFile::open(PCI_RESOURCE_PATH)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    let reg_value = f.pci_get_value(port_offset) as i32;

    // Mask off 4th bit for reset
    let mask = 1 << 4;

    // ResetL is active low
    let reg_value = reg_value & !mask;

    // Convert our register value back to a hex string and write back
    f.pci_set_value(port_offset, reg_value as u32);

    // Sleep 1 second to allow it to settle
    sleep(Duration::from_secs(1));

    let reg_value = reg_value | mask;

    // Convert our register value back to a hex string and write back
    f.pci_set_value(port_offset, reg_value as u32);

    Ok(())
}

pub(super) fn xcvr_set_low_power_mode(
    index: idx_t,
    low_power_mode: bool,
) -> Result<(), xcvr_status_t> {
    if index >= xcvr_num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    // not necessary to check for SFP ports apparently
    if let 0..=47 = index {
        return Ok(());
    }

    // port offset starts with 0x4000
    // NOTE: this is *not* the same as for presence!
    let port_offset: usize = 0x4000 + (index as usize * 16);

    let mut f = MemoryMappedFile::open(PCI_RESOURCE_PATH)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    let reg_value = f.pci_get_value(port_offset) as i32;

    // Mask off 6th bit for lowpower mode
    let mask = 1 << 6;

    // LPMode is active high; set or clear the bit accordingly
    let reg_value = if low_power_mode {
        reg_value | mask
    } else {
        reg_value & !mask
    };

    // Convert our register value back to a hex string and write back
    f.pci_set_value(port_offset, reg_value as u32);

    Ok(())
}

/// This is special for the 5248 as it has QSFPDD ports.
/// For these the ASIC treats them as separate ports, however, there is only one transceiver
/// To accomodate this we're doing the same thing: we treat it as separate ports, but we need
/// to make sure that we get the index number here right because there is only one transceiver.
pub(super) fn xcvr_get_inserted_port_type<F>(
    num_physical_ports: F,
    index: idx_t,
) -> Result<xcvr_port_type_t, xcvr_status_t>
where
    F: Fn() -> idx_t,
{
    if index >= num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    let index = match index {
        v @ 0..=47 => v,
        48..=49 => 48, // first QSFPDD port
        50..=51 => 49, // second QSFPDD port
        v @ 52..=55 => v - 2,
        _ => return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    };

    let eeprom = Eeprom::new(eeprom_file_path(index));
    let id = eeprom.get_id()?;

    match id {
        0x03 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP),
        0x0C => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP),
        0x0D => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP_PLUS),
        0x11 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP28),
        0x18 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFPDD),
        0x19 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_OSFP),
        v => Ok(v as xcvr_port_type_t),
    }
}
