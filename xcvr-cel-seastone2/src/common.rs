use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use xcvr_sys::idx_t;
use xcvr_sys::xcvr_port_type_t;
use xcvr_sys::xcvr_status_t;

// use std::sync::OnceLock;
// static INIT_LOGGER: OnceLock<()> = OnceLock::new();

// fn init_logger_func() {
//     // initialize logger, and log at debug level if RUST_LOG is not set
//     env_logger::init_from_env(
//         env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
//     );
// }

// const VENDOR_NAME_OFFSET: usize = 129;
// const VENDOR_PART_NUM_OFFSET: usize = 148;
// const VENDOR_NAME_LENGTH: usize = 16;
// const VENDOR_PART_NUM_LENGTH: usize = 16;

// eeprom_file_path returns the path to the eeprom file for the given port index in sysfs.
// NOTE: this is the same scheme for every port across the whole S5200 series.
pub(crate) fn eeprom_file_path(index: idx_t) -> String {
    // our I2C devices start at device "2", and always use the address 0x50
    let i = index + 2;
    format!("/sys/class/i2c-adapter/i2c-{i}/{i}-0050/eeprom")
}

pub(crate) fn port_name(index: idx_t) -> Result<String, xcvr_status_t> {
    match index {
        0..=31 => Ok(format!("QSFP{}", index + 1)),
        32 => Ok("SFP1".to_string()),
        _ => Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    }
}

pub(crate) struct Eeprom {
    path: String,
}

impl Eeprom {
    pub(crate) fn new(path: String) -> Self {
        Self { path: path }
    }

    pub(crate) fn read_eeprom(
        &self,
        offset: usize,
        num_bytes: usize,
    ) -> Result<Vec<u8>, xcvr_status_t> {
        let mut f = OpenOptions::new()
            .read(true)
            .open(self.path.as_str())
            .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
        let mut buffer = vec![0; num_bytes];
        f.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
        f.read_exact(&mut buffer)
            .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
        Ok(buffer)
    }

    #[allow(dead_code)]
    pub(crate) fn write_eeprom(
        &self,
        offset: usize,
        num_bytes: usize,
        write_buffer: &[u8],
    ) -> Result<(), xcvr_status_t> {
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.path.as_str())
            .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
        f.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
        f.write_all(&write_buffer[0..num_bytes])
            .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
        Ok(())
    }

    pub(crate) fn get_id(&self) -> Result<u8, xcvr_status_t> {
        let buffer = self.read_eeprom(0, 1)?;
        Ok(buffer[0])
    }
}

pub(super) fn get_inserted_port_type(index: idx_t) -> Result<xcvr_port_type_t, xcvr_status_t> {
    if index >= 33 {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

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

pub(super) fn get_presence(index: idx_t) -> Result<bool, xcvr_status_t> {
    let pn = port_name(index)?;
    let presence_file = if index == 32 {
        "sfp_modabs"
    } else {
        "qsfp_modprs"
    };
    let result: i32 = std::fs::read_to_string(format!(
        "/sys/devices/platform/switchboard/SFF/{pn}/{presence_file}"
    ))
    .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?
    .parse()
    .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    // present is active low
    Ok(result == 0)
}

pub(super) fn get_lpmode(index: idx_t) -> Result<bool, xcvr_status_t> {
    if index == 32 {
        return Ok(false);
    }
    let pn = port_name(index)?;
    let result: i32 = std::fs::read_to_string(format!(
        "/sys/devices/platform/switchboard/SFF/{pn}/qsfp_lpmode"
    ))
    .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?
    .parse()
    .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    Ok(result == 1)
}

pub(super) fn get_reset_status(index: idx_t) -> Result<bool, xcvr_status_t> {
    if index == 32 {
        return Ok(false);
    }
    let pn = port_name(index)?;
    let result: i32 = std::fs::read_to_string(format!(
        "/sys/devices/platform/switchboard/SFF/{pn}/qsfp_reset"
    ))
    .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?
    .parse()
    .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    // ModPrsL is active low
    Ok(result == 0)
}

pub(super) fn reset(index: idx_t) -> Result<(), xcvr_status_t> {
    if index == 32 {
        return Ok(());
    }
    let pn = port_name(index)?;
    std::fs::write(
        format!("/sys/devices/platform/switchboard/SFF/{pn}/qsfp_reset"),
        "0x0",
    )
    .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    // Sleep 1 second to allow it to settle
    sleep(Duration::from_secs(1));
    std::fs::write(
        format!("/sys/devices/platform/switchboard/SFF/{pn}/qsfp_reset"),
        "0x1",
    )
    .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    Ok(())
}

pub(super) fn set_lpmode(index: idx_t, lpmode: bool) -> Result<(), xcvr_status_t> {
    if index == 32 {
        return Ok(());
    }
    let pn = port_name(index)?;
    let num = if lpmode { "0x1" } else { "0x0" };
    std::fs::write(
        format!("/sys/devices/platform/switchboard/SFF/{pn}/qsfp_lpmode"),
        num,
    )
    .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    Ok(())
}
