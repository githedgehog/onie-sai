pub mod table_entry;
pub mod trap;
pub mod trap_group;
pub mod user_defined_trap;

use crate::port::{Port, PortID};

use super::*;
use sai_sys::*;

#[derive(Clone, Copy, Debug)]
pub enum HostIfType {
    Netdev,
    FD,
    Genetlink,
}

impl From<HostIfType> for i32 {
    fn from(value: HostIfType) -> Self {
        match value {
            HostIfType::Netdev => _sai_hostif_type_t_SAI_HOSTIF_TYPE_NETDEV as i32,
            HostIfType::FD => _sai_hostif_type_t_SAI_HOSTIF_TYPE_FD as i32,
            HostIfType::Genetlink => _sai_hostif_type_t_SAI_HOSTIF_TYPE_GENETLINK as i32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum VlanTag {
    Strip,
    Keep,
    Original,
}

impl From<VlanTag> for i32 {
    fn from(value: VlanTag) -> Self {
        match value {
            VlanTag::Strip => _sai_hostif_vlan_tag_t_SAI_HOSTIF_VLAN_TAG_STRIP as i32,
            VlanTag::Keep => _sai_hostif_vlan_tag_t_SAI_HOSTIF_VLAN_TAG_KEEP as i32,
            VlanTag::Original => _sai_hostif_vlan_tag_t_SAI_HOSTIF_VLAN_TAG_ORIGINAL as i32,
        }
    }
}

// TODO: still needs From implementations for the other object types
// * @type sai_object_id_t
// * @objects SAI_OBJECT_TYPE_PORT, SAI_OBJECT_TYPE_LAG, SAI_OBJECT_TYPE_VLAN, SAI_OBJECT_TYPE_SYSTEM_PORT
// SAI_HOSTIF_ATTR_OBJ_ID,
#[derive(Clone, Copy)]
pub struct HostIfObjectID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "hostif:oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<PortID> for HostIfObjectID {
    fn from(value: PortID) -> Self {
        Self { id: value.id }
    }
}

impl From<Port<'_>> for HostIfObjectID {
    fn from(value: Port) -> Self {
        Self { id: value.id }
    }
}

impl From<HostIfObjectID> for sai_object_id_t {
    fn from(value: HostIfObjectID) -> Self {
        value.id
    }
}

#[derive(Clone, Debug)]
pub enum HostIfAttribute {
    Type(HostIfType),
    ObjectID(HostIfObjectID),
    Name(String),
    OperStatus(bool),
    Queue(u32),
    VlanTag(VlanTag),
    GenetlinkMcgrpName(String),
}

impl From<HostIfAttribute> for sai_attribute_t {
    fn from(value: HostIfAttribute) -> Self {
        match value {
            HostIfAttribute::Type(v) => sai_attribute_t {
                id: _sai_hostif_attr_t_SAI_HOSTIF_ATTR_TYPE,
                value: sai_attribute_value_t { s32: v.into() },
            },
            HostIfAttribute::ObjectID(v) => sai_attribute_t {
                id: _sai_hostif_attr_t_SAI_HOSTIF_ATTR_OBJ_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            HostIfAttribute::Name(v) => {
                let b = v.as_bytes();
                let mut data: [i8; 32] = [0; 32];
                for i in 0..b.len() {
                    if i >= (SAI_HOSTIF_NAME_SIZE - 1) as usize {
                        break;
                    }
                    data[i] = b.get(i).unwrap().clone() as i8;
                }
                sai_attribute_t {
                    id: _sai_hostif_attr_t_SAI_HOSTIF_ATTR_NAME,
                    value: sai_attribute_value_t {
                        chardata: data.clone(),
                    },
                }
            }
            HostIfAttribute::OperStatus(v) => sai_attribute_t {
                id: _sai_hostif_attr_t_SAI_HOSTIF_ATTR_OPER_STATUS,
                value: sai_attribute_value_t { booldata: v },
            },
            HostIfAttribute::Queue(v) => sai_attribute_t {
                id: _sai_hostif_attr_t_SAI_HOSTIF_ATTR_QUEUE,
                value: sai_attribute_value_t { u32_: v },
            },
            HostIfAttribute::VlanTag(v) => sai_attribute_t {
                id: _sai_hostif_attr_t_SAI_HOSTIF_ATTR_VLAN_TAG,
                value: sai_attribute_value_t { s32: v.into() },
            },
            HostIfAttribute::GenetlinkMcgrpName(v) => {
                let b = v.as_bytes();
                let mut data: [i8; 32] = [0; 32];
                for i in 0..b.len() {
                    if i >= (SAI_HOSTIF_GENETLINK_MCGRP_NAME_SIZE - 1) as usize {
                        break;
                    }
                    data[i] = b.get(i).unwrap().clone() as i8;
                }
                sai_attribute_t {
                    id: _sai_hostif_attr_t_SAI_HOSTIF_ATTR_GENETLINK_MCGRP_NAME,
                    value: sai_attribute_value_t {
                        chardata: data.clone(),
                    },
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct HostIfID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfID> for sai_object_id_t {
    fn from(value: HostIfID) -> Self {
        value.id
    }
}

impl From<HostIf<'_>> for HostIfID {
    fn from(value: HostIf) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone)]
pub struct HostIf<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for HostIf<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIf(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for HostIf<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> HostIf<'a> {
    pub fn set_vlan_tag(&self, vlan_tag: VlanTag) -> Result<(), Error> {
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let set_hostif_attribute = hostif_api
            .set_hostif_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let attr = sai_attribute_t {
            id: _sai_hostif_attr_t_SAI_HOSTIF_ATTR_VLAN_TAG,
            value: sai_attribute_value_t {
                s32: vlan_tag.into(),
            },
        };

        let st = unsafe { set_hostif_attribute(self.id, &attr) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }

    pub fn set_oper_status(&self, oper_status: bool) -> Result<(), Error> {
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let set_hostif_attribute = hostif_api
            .set_hostif_attribute
            .ok_or(Error::APIFunctionUnavailable)?;

        let attr = sai_attribute_t {
            id: _sai_hostif_attr_t_SAI_HOSTIF_ATTR_OPER_STATUS,
            value: sai_attribute_value_t {
                booldata: oper_status,
            },
        };

        let st = unsafe { set_hostif_attribute(self.id, &attr) };
        if st != SAI_STATUS_SUCCESS as sai_status_t {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }

    pub fn remove(self) -> Result<(), Error> {
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let remove_hostif = hostif_api
            .remove_hostif
            .ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_hostif(self.id) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}

impl ObjectID<HostIfID> for HostIf<'_> {
    fn to_id(&self) -> HostIfID {
        HostIfID { id: self.id }
    }
}
