use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;
use xcvr_sys::idx_t;
use xcvr_sys::xcvr_port_type_t;
use xcvr_sys::xcvr_status_t;

/*

    type xcvrSettings struct {
        Idx                uint   `json:"idx"`
        DisplayName        string `json:"display_name"`
        HasTransceiver     bool   `json:"has_transceiver"`
        EEPROMDir          string `json:"eeprom_dir,omitempty"`
        CTRLDir            string `json:"ctrl_dir,omitempty"`
        SupportedPortTypes []uint `json:"supported_port_types,omitempty"`
    }

*/

/// serializes the xcvrSettings struct to json
#[derive(Serialize, Deserialize, Clone)]
pub struct Port {
    pub idx: idx_t,
    pub display_name: String,
    pub has_transceiver: bool,
    pub eeprom_dir: Option<String>,
    pub ctrl_dir: Option<String>,
    pub supported_port_types: Vec<xcvr_port_type_t>,
}

pub enum ReadSettingsError {
    Io(std::io::Error),
    Json(serde_json::Error),
    IndexOutOfBounds,
}

pub fn get_ports(platform: &str) -> Result<Vec<Port>, ReadSettingsError> {
    let path = format!("/etc/platform/{}/pddf_xcvr_settings.json", platform);
    let settings_path = Path::new(path.as_str());
    let settings_file = std::fs::File::open(settings_path).map_err(|e| ReadSettingsError::Io(e))?;
    let ports: Vec<Port> =
        serde_json::from_reader(settings_file).map_err(|e| ReadSettingsError::Json(e))?;
    Ok(ports)
}

pub fn get_port(platform: &str, index: idx_t) -> Result<Port, ReadSettingsError> {
    let path = format!("/etc/platform/{}/pddf_xcvr_settings.json", platform);
    let settings_path = Path::new(path.as_str());
    let settings_file = std::fs::File::open(settings_path).map_err(|e| ReadSettingsError::Io(e))?;
    let ports: Vec<Port> =
        serde_json::from_reader(settings_file).map_err(|e| ReadSettingsError::Json(e))?;
    if index >= ports.len() as idx_t {
        return Err(ReadSettingsError::IndexOutOfBounds);
    }
    Ok(ports[index as usize].clone())
}

pub struct Eeprom {
    path: String,
}

impl Eeprom {
    pub fn new(path: String) -> Self {
        Self { path: path }
    }

    pub fn read_eeprom(&self, offset: usize, num_bytes: usize) -> Result<Vec<u8>, xcvr_status_t> {
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
    pub fn write_eeprom(
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

    pub fn get_id(&self) -> Result<u8, xcvr_status_t> {
        let buffer = self.read_eeprom(0, 1)?;
        Ok(buffer[0])
    }
}

pub fn get_inserted_port_type(port: &Port) -> Result<xcvr_port_type_t, xcvr_status_t> {
    let eeprom_dir = match port.eeprom_dir {
        Some(ref dir) => dir,
        None => return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    };
    let eeprom = Eeprom::new(format!("{}/eeprom", eeprom_dir));
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

pub fn get_presence(port: &Port) -> Result<bool, xcvr_status_t> {
    let ctrl_dir = match port.ctrl_dir {
        Some(ref dir) => dir,
        None => return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    };
    let p_str = format!("{}/xcvr_present", ctrl_dir);
    let p = Path::new(p_str.as_str());
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(p)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    let num = buf
        .trim()
        .parse::<i32>()
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    // TODO: this should get mapped according to the pd-plugin.json file
    if num >= 1 {
        return Ok(true);
    }
    Ok(false)
}

pub fn get_reset(port: &Port) -> Result<bool, xcvr_status_t> {
    let ctrl_dir = match port.ctrl_dir {
        Some(ref dir) => dir,
        None => return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    };
    let p_str = format!("{}/xcvr_reset", ctrl_dir);
    let p = Path::new(p_str.as_str());
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(p)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    let num = buf
        .trim()
        .parse::<i32>()
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    // TODO: this should get mapped according to the pd-plugin.json file
    if num >= 1 {
        return Ok(true);
    }
    Ok(false)
}

pub fn set_reset(port: &Port, reset: bool) -> Result<(), xcvr_status_t> {
    let ctrl_dir = match port.ctrl_dir {
        Some(ref dir) => dir,
        None => return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    };
    let p_str = format!("{}/xcvr_reset", ctrl_dir);
    let p = Path::new(p_str.as_str());
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(p)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    // TODO: this should get mapped according to the pd-plugin.json file
    let num = if reset {
        "1".to_string()
    } else {
        "0".to_string()
    };
    f.write_all(num.as_bytes())
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    Ok(())
}

pub fn get_low_power_mode(port: &Port) -> Result<bool, xcvr_status_t> {
    let ctrl_dir = match port.ctrl_dir {
        Some(ref dir) => dir,
        None => return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    };
    let p_str = format!("{}/xcvr_lpmode", ctrl_dir);
    let p = Path::new(p_str.as_str());
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(p)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    let num = buf
        .trim()
        .parse::<i32>()
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    // TODO: this should get mapped according to the pd-plugin.json file
    if num >= 1 {
        return Ok(true);
    }
    Ok(false)
}

pub fn set_low_power_mode(port: &Port, low_power_mode: bool) -> Result<(), xcvr_status_t> {
    let ctrl_dir = match port.ctrl_dir {
        Some(ref dir) => dir,
        None => return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    };
    let p_str = format!("{}/xcvr_lpmode", ctrl_dir);
    let p = Path::new(p_str.as_str());
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(p)
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;

    // TODO: this should get mapped according to the pd-plugin.json file
    let num = if low_power_mode {
        "1".to_string()
    } else {
        "0".to_string()
    };
    f.write_all(num.as_bytes())
        .map_err(|_| xcvr_sys::XCVR_STATUS_ERROR_GENERAL)?;
    Ok(())
}
