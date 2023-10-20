use xcvr_sys::idx_t;
use xcvr_sys::xcvr_port_type_t;
use xcvr_sys::xcvr_status_t;

pub(super) fn xcvr_num_physical_ports() -> idx_t {
    54
}

pub(super) fn xcvr_get_supported_port_types(
    index: idx_t,
) -> Result<xcvr_port_type_t, xcvr_status_t> {
    match index {
        0..=47 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP28
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP_PLUS
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_SFP),
        48..=49 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFPDD),
        50..=53 => Ok(xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP28
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP_PLUS
            | xcvr_sys::_xcvr_port_type_t_XCVR_PORT_TYPE_QSFP),
        _ => Err(xcvr_sys::XCVR_STATUS_ERROR_GENERAL),
    }
}

pub(super) fn xcvr_get_presence(_index: idx_t) -> Result<bool, xcvr_status_t> {
    Err(xcvr_sys::XCVR_STATUS_ERROR_UNIMPLEMENTED)
}

pub(super) fn xcvr_get_oper_status(index: idx_t) -> Result<bool, xcvr_status_t> {
    xcvr_get_reset_status(index).map(|v| !v)
}

pub(super) fn xcvr_get_reset_status(_index: idx_t) -> Result<bool, xcvr_status_t> {
    Err(xcvr_sys::XCVR_STATUS_ERROR_UNIMPLEMENTED)
}

pub(super) fn xcvr_get_low_power_mode(_index: idx_t) -> Result<bool, xcvr_status_t> {
    Err(xcvr_sys::XCVR_STATUS_ERROR_UNIMPLEMENTED)
}

pub(super) fn xcvr_reset(_index: idx_t) -> Result<(), xcvr_status_t> {
    Err(xcvr_sys::XCVR_STATUS_ERROR_UNIMPLEMENTED)
}

pub(super) fn xcvr_set_low_power_mode(
    _index: idx_t,
    _low_power_mode: bool,
) -> Result<(), xcvr_status_t> {
    Err(xcvr_sys::XCVR_STATUS_ERROR_UNIMPLEMENTED)
}
