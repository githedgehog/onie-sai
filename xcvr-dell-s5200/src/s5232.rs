use memmap::MmapMut;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
// use std::sync::OnceLock;
use std::thread::sleep;
use std::time::Duration;

use xcvr_sys::idx_t;
use xcvr_sys::xcvr_port_type_t;
use xcvr_sys::xcvr_status_t;

// static INIT_LOGGER: OnceLock<()> = OnceLock::new();

// fn init_logger_func() {
//     // initialize logger, and log at debug level if RUST_LOG is not set
//     env_logger::init_from_env(
//         env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
//     );
// }

fn eeprom_file_path(index: idx_t) -> Result<String, xcvr_status_t> {
    if index >= xcvr_num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    // our I2C devices start at device "2", and always use the address 0x50
    let i = index + 2;
    Ok(format!("/sys/class/i2c-adapter/i2c-{i}/{i}-0050/eeprom"))
}

const VENDOR_NAME_OFFSET: usize = 129;
const VENDOR_PART_NUM_OFFSET: usize = 148;
const VENDOR_NAME_LENGTH: usize = 16;
const VENDOR_PART_NUM_LENGTH: usize = 16;

struct Eeprom {
    path: String,
}

impl Eeprom {
    fn read_eeprom(&self, offset: usize, num_bytes: usize) -> Result<Vec<u8>, xcvr_status_t> {
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

    fn write_eeprom(
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

    fn get_id(&self) -> Result<u8, xcvr_status_t> {
        let buffer = self.read_eeprom(0, 1)?;
        Ok(buffer[0])
    }
}

/*
    def read_eeprom(self, offset, num_bytes):
        try:
            with open(self.get_eeprom_path(), mode='rb', buffering=0) as f:
                f.seek(offset)
                return bytearray(f.read(num_bytes))
        except (OSError, IOError):
            return None

    def write_eeprom(self, offset, num_bytes, write_buffer):
        try:
            with open(self.get_eeprom_path(), mode='r+b', buffering=0) as f:
                f.seek(offset)
                f.write(write_buffer[0:num_bytes])
        except (OSError, IOError):
            return False
        return True
*/

const PCI_RESOURCE_PATH: &str = "/sys/bus/pci/devices/0000:04:00.0/resource0";

struct MemoryMappedFile {
    map: MmapMut,
}

impl MemoryMappedFile {
    fn open(file_path: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().read(true).write(true).open(file_path)?;
        let map = unsafe { MmapMut::map_mut(&file)? };
        Ok(MemoryMappedFile { map })
    }

    fn pci_mem_read_u32(&self, offset: usize) -> u32 {
        let mut buffer = [0; 4];
        buffer.copy_from_slice(&self.map[offset..offset + 4]);
        u32::from_le_bytes(buffer)
    }

    fn pci_mem_write_u32(&mut self, offset: usize, data: u32) {
        let data_bytes = data.to_le_bytes();
        self.map[offset..offset + 4].copy_from_slice(&data_bytes);
    }

    fn pci_get_value(&self, offset: usize) -> u32 {
        self.pci_mem_read_u32(offset)
    }

    fn pci_set_value(&mut self, offset: usize, val: u32) {
        self.pci_mem_write_u32(offset, val);
    }
}

pub(super) fn xcvr_num_physical_ports() -> idx_t {
    34
}

pub(super) fn xcvr_get_supported_port_types(
    index: idx_t,
) -> Result<xcvr_port_type_t, xcvr_status_t> {
    match index {
        0..=31 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP28
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP_PLUS
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP),
        32..=33 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_PLUS
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP),
        _ => Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    }
}

pub(super) fn xcvr_get_inserted_port_type(index: idx_t) -> Result<xcvr_port_type_t, xcvr_status_t> {
    if index >= xcvr_num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    let eeprom = Eeprom {
        path: eeprom_file_path(index)?,
    };

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
        0..=31 => Ok(reg_value & (1 << 4) == 0),
        32..=33 => Ok(reg_value & (1 << 0) == 0),
        _ => Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    }
}

pub(super) fn xcvr_get_oper_status(index: idx_t) -> Result<bool, xcvr_status_t> {
    xcvr_get_reset_status(index).map(|v| !v)
}

pub(super) fn xcvr_get_reset_status(index: idx_t) -> Result<bool, xcvr_status_t> {
    if index >= xcvr_num_physical_ports() {
        return Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL);
    }

    // not necessary to check for SFP ports apparently
    if let 32..=33 = index {
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
    if let 32..=33 = index {
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
    if let 32..=33 = index {
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
    if let 32..=33 = index {
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
