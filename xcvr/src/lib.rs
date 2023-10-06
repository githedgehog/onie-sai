use libloading::Symbol;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::NulError;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::path::Path;

pub use xcvr_sys::idx_t;
pub use xcvr_sys::xcvr_port_type_t;
pub use xcvr_sys::xcvr_status_t;
use xcvr_sys::xcvr_transceiver_info_t;
use xcvr_sys::xcvr_transceiver_status_t;
use xcvr_sys::XCVR_STATUS_SUCCESS;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    General,
    Blocking,
    PowerBudgetExceeded,
    I2CStuck,
    BadEEPROM,
    UnsupportedCable,
    HighTemp,
    BadCable,
    UnsupportedPlatform,
    Unimplemented,
    Unknown(xcvr_status_t),
}

impl From<xcvr_status_t> for Status {
    fn from(value: xcvr_status_t) -> Self {
        match value {
            xcvr_sys::XCVR_STATUS_ERROR_GENERAL => Status::General,
            xcvr_sys::XCVR_STATUS_ERROR_BLOCKING => Status::Blocking,
            xcvr_sys::XCVR_STATUS_ERROR_POWER_BUDGET_EXCEEDED => Status::PowerBudgetExceeded,
            xcvr_sys::XCVR_STATUS_ERROR_I2C_STUCK => Status::I2CStuck,
            xcvr_sys::XCVR_STATUS_ERROR_BAD_EEPROM => Status::BadEEPROM,
            xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_CABLE => Status::UnsupportedCable,
            xcvr_sys::XCVR_STATUS_ERROR_HIGH_TEMP => Status::HighTemp,
            xcvr_sys::XCVR_STATUS_ERROR_BAD_CABLE => Status::BadCable,
            xcvr_sys::XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM => Status::UnsupportedPlatform,
            xcvr_sys::XCVR_STATUS_ERROR_UNIMPLEMENTED => Status::Unimplemented,
            v => Self::Unknown(v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PortType {
    RJ45,
    SFP,
    XFP,
    SFPPlus,
    QSFP,
    CFP,
    QSFPPlus,
    QSFP28,
    SFP28,
    CFP2,
    QSFP56,
    QSFPDD,
    OSFP,
    SFPDD,
    Unknown(xcvr_port_type_t),
}

impl From<PortType> for xcvr_port_type_t {
    fn from(value: PortType) -> Self {
        match value {
            PortType::RJ45 => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_RJ45,
            PortType::SFP => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP,
            PortType::XFP => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_XFP,
            PortType::SFPPlus => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_PLUS,
            PortType::QSFP => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP,
            PortType::CFP => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_CFP,
            PortType::QSFPPlus => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP_PLUS,
            PortType::QSFP28 => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP28,
            PortType::SFP28 => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP28,
            PortType::CFP2 => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_CFP2,
            PortType::QSFP56 => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP56,
            PortType::QSFPDD => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFPDD,
            PortType::OSFP => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_OSFP,
            PortType::SFPDD => xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_DD,
            PortType::Unknown(v) => v,
        }
    }
}

impl From<xcvr_port_type_t> for PortType {
    fn from(value: xcvr_port_type_t) -> Self {
        match value {
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_RJ45 => PortType::RJ45,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP => PortType::SFP,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_XFP => PortType::XFP,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_PLUS => PortType::SFPPlus,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP => PortType::QSFP,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_CFP => PortType::CFP,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP_PLUS => PortType::QSFPPlus,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP28 => PortType::QSFP28,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP28 => PortType::SFP28,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_CFP2 => PortType::CFP2,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP56 => PortType::QSFP56,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFPDD => PortType::QSFPDD,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_OSFP => PortType::OSFP,
            xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_DD => PortType::SFPDD,
            v => PortType::Unknown(v),
        }
    }
}

impl PortType {
    pub fn from_mask(mask: xcvr_port_type_t) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_RJ45 == 1 {
            ret.push(PortType::RJ45);
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP == 1 {
            ret.push(PortType::SFP)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_XFP == 1 {
            ret.push(PortType::XFP)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_PLUS == 1 {
            ret.push(PortType::SFPPlus)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP == 1 {
            ret.push(PortType::QSFP)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_CFP == 1 {
            ret.push(PortType::CFP)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP_PLUS == 1 {
            ret.push(PortType::QSFPPlus)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP28 == 1 {
            ret.push(PortType::QSFP28)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP28 == 1 {
            ret.push(PortType::SFP28)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_CFP2 == 1 {
            ret.push(PortType::CFP2)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP56 == 1 {
            ret.push(PortType::QSFP56)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFPDD == 1 {
            ret.push(PortType::QSFPDD)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_OSFP == 1 {
            ret.push(PortType::OSFP)
        }
        if mask & xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_DD == 1 {
            ret.push(PortType::SFPDD)
        }
        ret
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransceiverInfo {
    pub typ: String,
    pub type_abbrv_name: String,
    pub hardware_rev: String,
    pub vendor_rev: String,
    pub serial: String,
    pub manufacturer: String,
    pub model: String,
    pub connector: String,
    pub encoding: String,
    pub ext_identifier: String,
    pub ext_rateselect_compliance: String,
    pub cable_length: u32,
    pub nominal_bit_rate: u32,
    pub specification_compliance: String,
    pub vendor_date: String,
    pub vendor_oui: String,
    pub application_advertisement: String,
}

impl From<xcvr_transceiver_info_t> for TransceiverInfo {
    fn from(value: xcvr_transceiver_info_t) -> Self {
        Self {
            typ: char_array_to_string(value.type_),
            type_abbrv_name: char_array_to_string(value.type_abbrv_name),
            hardware_rev: char_array_to_string(value.hardware_rev),
            vendor_rev: char_array_to_string(value.vendor_rev),
            serial: char_array_to_string(value.serial),
            manufacturer: char_array_to_string(value.manufacturer),
            model: char_array_to_string(value.model),
            connector: char_array_to_string(value.connector),
            encoding: char_array_to_string(value.encoding),
            ext_identifier: char_array_to_string(value.ext_identifier),
            ext_rateselect_compliance: char_array_to_string(value.ext_rateselect_compliance),
            cable_length: value.cable_length,
            nominal_bit_rate: value.nominal_bit_rate,
            specification_compliance: char_array_to_string(value.specification_compliance),
            vendor_date: char_array_to_string(value.vendor_date),
            vendor_oui: char_array_to_string(value.vendor_oui),
            application_advertisement: char_array_to_string(value.application_advertisement),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransceiverStatus {
    pub module_state: String,
    pub module_fault_cause: String,
    pub datapath_firmware_fault: bool,
    pub module_firmware_fault: bool,
    pub module_state_changed: bool,
    pub datapath_hostlane1: String,
    pub datapath_hostlane2: String,
    pub datapath_hostlane3: String,
    pub datapath_hostlane4: String,
    pub datapath_hostlane5: String,
    pub datapath_hostlane6: String,
    pub datapath_hostlane7: String,
    pub datapath_hostlane8: String,
    pub txoutput_status: bool,
    pub rxoutput_status_hostlane1: bool,
    pub rxoutput_status_hostlane2: bool,
    pub rxoutput_status_hostlane3: bool,
    pub rxoutput_status_hostlane4: bool,
    pub rxoutput_status_hostlane5: bool,
    pub rxoutput_status_hostlane6: bool,
    pub rxoutput_status_hostlane7: bool,
    pub rxoutput_status_hostlane8: bool,
    pub txfault: bool,
    pub txlos_hostlane1: bool,
    pub txlos_hostlane2: bool,
    pub txlos_hostlane3: bool,
    pub txlos_hostlane4: bool,
    pub txlos_hostlane5: bool,
    pub txlos_hostlane6: bool,
    pub txlos_hostlane7: bool,
    pub txlos_hostlane8: bool,
    pub txcdrlol_hostlane1: bool,
    pub txcdrlol_hostlane2: bool,
    pub txcdrlol_hostlane3: bool,
    pub txcdrlol_hostlane4: bool,
    pub txcdrlol_hostlane5: bool,
    pub txcdrlol_hostlane6: bool,
    pub txcdrlol_hostlane7: bool,
    pub txcdrlol_hostlane8: bool,
    pub rxlos: bool,
    pub rxcdrlol: bool,
    pub config_state_hostlane1: String,
    pub config_state_hostlane2: String,
    pub config_state_hostlane3: String,
    pub config_state_hostlane4: String,
    pub config_state_hostlane5: String,
    pub config_state_hostlane6: String,
    pub config_state_hostlane7: String,
    pub config_state_hostlane8: String,
    pub dpinit_pending_hostlane1: bool,
    pub dpinit_pending_hostlane2: bool,
    pub dpinit_pending_hostlane3: bool,
    pub dpinit_pending_hostlane4: bool,
    pub dpinit_pending_hostlane5: bool,
    pub dpinit_pending_hostlane6: bool,
    pub dpinit_pending_hostlane7: bool,
    pub dpinit_pending_hostlane8: bool,
    pub temphighalarm_flag: bool,
    pub temphighwarning_flag: bool,
    pub templowalarm_flag: bool,
    pub templowwarning_flag: bool,
    pub vcchighalarm_flag: bool,
    pub vcchighwarning_flag: bool,
    pub vcclowalarm_flag: bool,
    pub vcclowwarning_flag: bool,
    pub txpowerhighalarm_flag: bool,
    pub txpowerlowalarm_flag: bool,
    pub txpowerhighwarning_flag: bool,
    pub txpowerlowwarning_flag: bool,
    pub rxpowerhighalarm_flag: bool,
    pub rxpowerlowalarm_flag: bool,
    pub rxpowerhighwarning_flag: bool,
    pub rxpowerlowwarning_flag: bool,
    pub txbiashighalarm_flag: bool,
    pub txbiaslowalarm_flag: bool,
    pub txbiashighwarning_flag: bool,
    pub txbiaslowwarning_flag: bool,
    pub lasertemphighalarm_flag: bool,
    pub lasertemplowalarm_flag: bool,
    pub lasertemphighwarning_flag: bool,
    pub lasertemplowwarning_flag: bool,
    pub prefecberhighalarm_flag: bool,
    pub prefecberlowalarm_flag: bool,
    pub prefecberhighwarning_flag: bool,
    pub prefecberlowwarning_flag: bool,
    pub postfecberhighalarm_flag: bool,
    pub postfecberlowalarm_flag: bool,
    pub postfecberhighwarning_flag: bool,
    pub postfecberlowwarning_flag: bool,
}

fn char_array_to_string(v: [i8; 255]) -> String {
    v.iter()
        .take_while(|&&c| c != 0) // Take characters until null terminator
        .map(|&c| c as u8 as char) // Convert i8 to u8 and then to char
        .collect()
}
impl From<xcvr_transceiver_status_t> for TransceiverStatus {
    fn from(value: xcvr_transceiver_status_t) -> Self {
        // let a = String::from_utf8_lossy(&value.module_state);
        Self {
            module_state: char_array_to_string(value.module_state),
            module_fault_cause: char_array_to_string(value.module_fault_cause),
            datapath_firmware_fault: value.datapath_firmware_fault,
            module_firmware_fault: value.module_firmware_fault,
            module_state_changed: value.module_state_changed,
            datapath_hostlane1: char_array_to_string(value.datapath_hostlane1),
            datapath_hostlane2: char_array_to_string(value.datapath_hostlane2),
            datapath_hostlane3: char_array_to_string(value.datapath_hostlane3),
            datapath_hostlane4: char_array_to_string(value.datapath_hostlane4),
            datapath_hostlane5: char_array_to_string(value.datapath_hostlane5),
            datapath_hostlane6: char_array_to_string(value.datapath_hostlane6),
            datapath_hostlane7: char_array_to_string(value.datapath_hostlane7),
            datapath_hostlane8: char_array_to_string(value.datapath_hostlane8),
            txoutput_status: value.txoutput_status,
            rxoutput_status_hostlane1: value.rxoutput_status_hostlane1,
            rxoutput_status_hostlane2: value.rxoutput_status_hostlane2,
            rxoutput_status_hostlane3: value.rxoutput_status_hostlane3,
            rxoutput_status_hostlane4: value.rxoutput_status_hostlane4,
            rxoutput_status_hostlane5: value.rxoutput_status_hostlane5,
            rxoutput_status_hostlane6: value.rxoutput_status_hostlane6,
            rxoutput_status_hostlane7: value.rxoutput_status_hostlane7,
            rxoutput_status_hostlane8: value.rxoutput_status_hostlane8,
            txfault: value.txfault,
            txlos_hostlane1: value.txlos_hostlane1,
            txlos_hostlane2: value.txlos_hostlane2,
            txlos_hostlane3: value.txlos_hostlane3,
            txlos_hostlane4: value.txlos_hostlane4,
            txlos_hostlane5: value.txlos_hostlane5,
            txlos_hostlane6: value.txlos_hostlane6,
            txlos_hostlane7: value.txlos_hostlane7,
            txlos_hostlane8: value.txlos_hostlane8,
            txcdrlol_hostlane1: value.txcdrlol_hostlane1,
            txcdrlol_hostlane2: value.txcdrlol_hostlane2,
            txcdrlol_hostlane3: value.txcdrlol_hostlane3,
            txcdrlol_hostlane4: value.txcdrlol_hostlane4,
            txcdrlol_hostlane5: value.txcdrlol_hostlane5,
            txcdrlol_hostlane6: value.txcdrlol_hostlane6,
            txcdrlol_hostlane7: value.txcdrlol_hostlane7,
            txcdrlol_hostlane8: value.txcdrlol_hostlane8,
            rxlos: value.rxlos,
            rxcdrlol: value.rxcdrlol,
            config_state_hostlane1: char_array_to_string(value.config_state_hostlane1),
            config_state_hostlane2: char_array_to_string(value.config_state_hostlane2),
            config_state_hostlane3: char_array_to_string(value.config_state_hostlane3),
            config_state_hostlane4: char_array_to_string(value.config_state_hostlane4),
            config_state_hostlane5: char_array_to_string(value.config_state_hostlane5),
            config_state_hostlane6: char_array_to_string(value.config_state_hostlane6),
            config_state_hostlane7: char_array_to_string(value.config_state_hostlane7),
            config_state_hostlane8: char_array_to_string(value.config_state_hostlane8),
            dpinit_pending_hostlane1: value.dpinit_pending_hostlane1,
            dpinit_pending_hostlane2: value.dpinit_pending_hostlane2,
            dpinit_pending_hostlane3: value.dpinit_pending_hostlane3,
            dpinit_pending_hostlane4: value.dpinit_pending_hostlane4,
            dpinit_pending_hostlane5: value.dpinit_pending_hostlane5,
            dpinit_pending_hostlane6: value.dpinit_pending_hostlane6,
            dpinit_pending_hostlane7: value.dpinit_pending_hostlane7,
            dpinit_pending_hostlane8: value.dpinit_pending_hostlane8,
            temphighalarm_flag: value.temphighalarm_flag,
            temphighwarning_flag: value.temphighwarning_flag,
            templowalarm_flag: value.templowalarm_flag,
            templowwarning_flag: value.templowwarning_flag,
            vcchighalarm_flag: value.vcchighalarm_flag,
            vcchighwarning_flag: value.vcchighwarning_flag,
            vcclowalarm_flag: value.vcclowalarm_flag,
            vcclowwarning_flag: value.vcclowwarning_flag,
            txpowerhighalarm_flag: value.txpowerhighalarm_flag,
            txpowerlowalarm_flag: value.txpowerlowalarm_flag,
            txpowerhighwarning_flag: value.txpowerhighwarning_flag,
            txpowerlowwarning_flag: value.txpowerlowwarning_flag,
            rxpowerhighalarm_flag: value.rxpowerhighalarm_flag,
            rxpowerlowalarm_flag: value.rxpowerlowalarm_flag,
            rxpowerhighwarning_flag: value.rxpowerhighwarning_flag,
            rxpowerlowwarning_flag: value.rxpowerlowwarning_flag,
            txbiashighalarm_flag: value.txbiashighalarm_flag,
            txbiaslowalarm_flag: value.txbiaslowalarm_flag,
            txbiashighwarning_flag: value.txbiashighwarning_flag,
            txbiaslowwarning_flag: value.txbiaslowwarning_flag,
            lasertemphighalarm_flag: value.lasertemphighalarm_flag,
            lasertemplowalarm_flag: value.lasertemplowalarm_flag,
            lasertemphighwarning_flag: value.lasertemphighwarning_flag,
            lasertemplowwarning_flag: value.lasertemplowwarning_flag,
            prefecberhighalarm_flag: value.prefecberhighalarm_flag,
            prefecberlowalarm_flag: value.prefecberlowalarm_flag,
            prefecberhighwarning_flag: value.prefecberhighwarning_flag,
            prefecberlowwarning_flag: value.prefecberlowwarning_flag,
            postfecberhighalarm_flag: value.postfecberhighalarm_flag,
            postfecberlowalarm_flag: value.postfecberlowalarm_flag,
            postfecberhighwarning_flag: value.postfecberhighwarning_flag,
            postfecberlowwarning_flag: value.postfecberlowwarning_flag,
        }
    }
}

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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("dynamic library error")]
    LoadingDynamicLibrary(libloading::Error),
    #[error("")]
    GettingDynamicLibrarySymbol(#[from] libloading::Error),
    #[error("library returned NULL pointer")]
    NullReturn,
    #[error("platform string argument contained null characters")]
    InvalidPlatformString(NulError),
    #[error("unsupported platform {found:?}, supported platforms: {supported:?}")]
    PlatformUnsupported {
        found: String,
        supported: Vec<String>,
    },
    #[error("xcvr library call failed: {0:?}")]
    Status(Status),
}

pub struct LibraryLoader {
    lib: libloading::Library,
}

impl LibraryLoader {
    pub fn new(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            lib: unsafe { libloading::Library::new(path.as_os_str()) }
                .map_err(|e| Error::LoadingDynamicLibrary(e))?,
        })
    }

    pub fn lib(&self) -> Result<Library, Error> {
        Library::new(&self.lib)
    }
}

pub trait Context {
    fn library_name(&self) -> Result<String, Error>;
    fn is_supported_platform(&self, platform: &str) -> Result<bool, Error>;
    fn supported_platforms(&self) -> Result<Vec<String>, Error>;
}

pub trait PlatformContext {
    fn num_physical_ports(&self) -> Result<idx_t, Error>;
    fn get_presence(&self, port_index: idx_t) -> Result<bool, Error>;
    fn get_supported_port_types(&self, port_index: idx_t) -> Result<Vec<PortType>, Error>;
    fn get_inserted_port_type(&self, port_index: idx_t) -> Result<PortType, Error>;
    fn get_oper_status(&self, port_index: idx_t) -> Result<bool, Error>;
    fn get_reset_status(&self, port_index: idx_t) -> Result<bool, Error>;
    fn reset(&self, port_index: idx_t) -> Result<(), Error>;
    fn get_low_power_mode(&self, port_index: idx_t) -> Result<bool, Error>;
    fn set_low_power_mode(&self, port_index: idx_t, low_power_mode: bool) -> Result<(), Error>;
    fn get_transceiver_info(&self, port_index: idx_t) -> Result<TransceiverInfo, Error>;
    fn get_transceiver_status(&self, port_index: idx_t) -> Result<TransceiverStatus, Error>;
}

pub struct Library<'a> {
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

impl<'a> Library<'a> {
    fn new(lib: &'a libloading::Library) -> Result<Self, Error> {
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

    pub fn platform_lib(&self, platform: &str) -> Result<PlatformLibrary, Error> {
        if !self.is_supported_platform(platform)? {
            let supported = match self.supported_platforms() {
                Ok(v) => v,
                Err(e) => vec![format!("unknown (error: {})", e)],
            };
            return Err(Error::PlatformUnsupported {
                found: platform.to_string(),
                supported: supported,
            });
        }
        let platform = CString::new(platform).map_err(|e| Error::InvalidPlatformString(e))?;
        Ok(PlatformLibrary::new(self, platform))
    }
}

impl<'a> Context for Library<'a> {
    fn library_name(&self) -> Result<String, Error> {
        let ret = (self.xcvr_library_name)();
        if ret.is_null() {
            return Err(Error::NullReturn);
        }
        Ok(unsafe { CStr::from_ptr(ret) }.to_string_lossy().to_string())
    }

    fn is_supported_platform(&self, platform: &str) -> Result<bool, Error> {
        let arg = CString::new(platform).map_err(|e| Error::InvalidPlatformString(e))?;
        let ret = (self.xcvr_is_supported_platform)(arg.as_ptr());
        Ok(ret)
    }

    fn supported_platforms(&self) -> Result<Vec<String>, Error> {
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

pub struct PlatformLibrary<'a> {
    platform: CString,
    xcvr_num_physical_ports: &'a Symbol<'a, XcvrNumPhysicalPortsFunc>,
    xcvr_get_presence: &'a Symbol<'a, XcvrGetPresenceFunc>,
    xcvr_get_supported_port_types: &'a Symbol<'a, XcvrGetSupportedPortTypesFunc>,
    xcvr_get_inserted_port_type: &'a Symbol<'a, XcvrGetInsertedPortTypeFunc>,
    xcvr_get_oper_status: &'a Symbol<'a, XcvrGetOperStatusFunc>,
    xcvr_get_reset_status: &'a Symbol<'a, XcvrGetResetStatusFunc>,
    xcvr_reset: &'a Symbol<'a, XcvrResetFunc>,
    xcvr_get_low_power_mode: &'a Symbol<'a, XcvrGetLowPowerModeFunc>,
    xcvr_set_low_power_mode: &'a Symbol<'a, XcvrSetLowPowerModeFunc>,
    xcvr_get_transceiver_info: &'a Symbol<'a, XcvrGetTransceiverInfoFunc>,
    xcvr_get_transceiver_status: &'a Symbol<'a, XcvrGetTransceiverStatusFunc>,
}

impl<'a> PlatformLibrary<'a> {
    fn new(lib: &'a Library, platform: CString) -> Self {
        Self {
            platform: platform,
            xcvr_num_physical_ports: &lib.xcvr_num_physical_ports,
            xcvr_get_presence: &lib.xcvr_get_presence,
            xcvr_get_supported_port_types: &lib.xcvr_get_supported_port_types,
            xcvr_get_inserted_port_type: &lib.xcvr_get_inserted_port_type,
            xcvr_get_oper_status: &lib.xcvr_get_oper_status,
            xcvr_get_reset_status: &lib.xcvr_get_reset_status,
            xcvr_reset: &lib.xcvr_reset,
            xcvr_get_low_power_mode: &lib.xcvr_get_low_power_mode,
            xcvr_set_low_power_mode: &lib.xcvr_set_low_power_mode,
            xcvr_get_transceiver_info: &lib.xcvr_get_transceiver_info,
            xcvr_get_transceiver_status: &lib.xcvr_get_transceiver_status,
        }
    }
}

impl<'a> PlatformContext for PlatformLibrary<'a> {
    fn num_physical_ports(&self) -> Result<idx_t, Error> {
        let mut v: idx_t = 0;
        let ret = (self.xcvr_num_physical_ports)(self.platform.as_ptr(), &mut v);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(v)
    }

    fn get_presence(&self, port_index: idx_t) -> Result<bool, Error> {
        let mut v: bool = false;
        let ret = (self.xcvr_get_presence)(self.platform.as_ptr(), port_index, &mut v);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(v)
    }

    fn get_supported_port_types(&self, port_index: idx_t) -> Result<Vec<PortType>, Error> {
        let mut v: xcvr_port_type_t = 0;
        let ret = (self.xcvr_get_supported_port_types)(self.platform.as_ptr(), port_index, &mut v);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(PortType::from_mask(v))
    }

    fn get_inserted_port_type(&self, port_index: idx_t) -> Result<PortType, Error> {
        let mut v: xcvr_port_type_t = 0;
        let ret = (self.xcvr_get_inserted_port_type)(self.platform.as_ptr(), port_index, &mut v);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(PortType::from(v))
    }

    fn get_oper_status(&self, port_index: idx_t) -> Result<bool, Error> {
        let mut v: bool = false;
        let ret = (self.xcvr_get_oper_status)(self.platform.as_ptr(), port_index, &mut v);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(v)
    }

    fn get_reset_status(&self, port_index: idx_t) -> Result<bool, Error> {
        let mut v: bool = false;
        let ret = (self.xcvr_get_reset_status)(self.platform.as_ptr(), port_index, &mut v);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(v)
    }

    fn reset(&self, port_index: idx_t) -> Result<(), Error> {
        let ret = (self.xcvr_reset)(self.platform.as_ptr(), port_index);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(())
    }

    fn get_low_power_mode(&self, port_index: idx_t) -> Result<bool, Error> {
        let mut v: bool = false;
        let ret = (self.xcvr_get_low_power_mode)(self.platform.as_ptr(), port_index, &mut v);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(v)
    }

    fn set_low_power_mode(&self, port_index: idx_t, low_power_mode: bool) -> Result<(), Error> {
        let ret =
            (self.xcvr_set_low_power_mode)(self.platform.as_ptr(), port_index, low_power_mode);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(())
    }

    fn get_transceiver_info(&self, port_index: idx_t) -> Result<TransceiverInfo, Error> {
        let mut v: xcvr_transceiver_info_t = Default::default();
        let ret = (self.xcvr_get_transceiver_info)(self.platform.as_ptr(), port_index, &mut v);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(TransceiverInfo::from(v))
    }

    fn get_transceiver_status(&self, port_index: idx_t) -> Result<TransceiverStatus, Error> {
        let mut v: xcvr_transceiver_status_t = Default::default();
        let ret = (self.xcvr_get_transceiver_status)(self.platform.as_ptr(), port_index, &mut v);
        if ret != XCVR_STATUS_SUCCESS {
            return Err(Error::Status(Status::from(ret)));
        }
        Ok(TransceiverStatus::from(v))
    }
}

pub struct FallbackPlatformLibrary {}

impl PlatformContext for FallbackPlatformLibrary {
    fn num_physical_ports(&self) -> Result<idx_t, Error> {
        Ok(0)
    }

    fn get_presence(&self, _port_index: idx_t) -> Result<bool, Error> {
        Ok(true)
    }

    fn get_supported_port_types(&self, _port_index: idx_t) -> Result<Vec<PortType>, Error> {
        Ok(vec![])
    }

    fn get_inserted_port_type(&self, _port_index: idx_t) -> Result<PortType, Error> {
        Ok(PortType::SFP)
    }

    fn get_oper_status(&self, _port_index: idx_t) -> Result<bool, Error> {
        Ok(true)
    }

    fn get_reset_status(&self, _port_index: idx_t) -> Result<bool, Error> {
        Ok(false)
    }

    fn reset(&self, _port_index: idx_t) -> Result<(), Error> {
        Ok(())
    }

    fn get_low_power_mode(&self, _port_index: idx_t) -> Result<bool, Error> {
        Ok(false)
    }

    fn set_low_power_mode(&self, _port_index: idx_t, _low_power_mode: bool) -> Result<(), Error> {
        Ok(())
    }

    fn get_transceiver_info(&self, _port_index: idx_t) -> Result<TransceiverInfo, Error> {
        Ok(TransceiverInfo::default())
    }

    fn get_transceiver_status(&self, _port_index: idx_t) -> Result<TransceiverStatus, Error> {
        Ok(TransceiverStatus::default())
    }
}
