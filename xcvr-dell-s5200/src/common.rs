use memmap::MmapMut;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
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

    fn get_id(&self) -> Result<u8, xcvr_status_t> {
        let buffer = self.read_eeprom(0, 1)?;
        Ok(buffer[0])
    }
}

pub(crate) struct MemoryMappedFile {
    map: MmapMut,
}

impl MemoryMappedFile {
    pub(crate) fn open(file_path: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().read(true).write(true).open(file_path)?;
        let map = unsafe { MmapMut::map_mut(&file)? };
        Ok(MemoryMappedFile { map })
    }

    pub(crate) fn pci_mem_read_u32(&self, offset: usize) -> u32 {
        let mut buffer = [0; 4];
        buffer.copy_from_slice(&self.map[offset..offset + 4]);
        u32::from_le_bytes(buffer)
    }

    pub(crate) fn pci_mem_write_u32(&mut self, offset: usize, data: u32) {
        let data_bytes = data.to_le_bytes();
        self.map[offset..offset + 4].copy_from_slice(&data_bytes);
    }

    pub(crate) fn pci_get_value(&self, offset: usize) -> u32 {
        self.pci_mem_read_u32(offset)
    }

    pub(crate) fn pci_set_value(&mut self, offset: usize, val: u32) {
        self.pci_mem_write_u32(offset, val);
    }
}

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
