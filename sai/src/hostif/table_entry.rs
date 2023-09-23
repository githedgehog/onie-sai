use super::trap;
use crate::*;
use sai_sys::*;

#[derive(Clone, Copy, Debug)]
pub enum TableEntryType {
    Port,
    LAG,
    VLAN,
    TrapID,
    Wildcard,
}

impl From<TableEntryType> for i32 {
    fn from(value: TableEntryType) -> Self {
        match value {
            TableEntryType::Port => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_PORT as i32
            }
            TableEntryType::LAG => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_LAG as i32
            }
            TableEntryType::VLAN => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_VLAN as i32
            }
            TableEntryType::TrapID => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_TRAP_ID as i32
            }
            TableEntryType::Wildcard => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_WILDCARD as i32
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChannelType {
    CB,
    FD,
    NetdevPhysicalPort,
    NetdevLogicalPort,
    NetdevL3,
    Genetlink,
}

impl From<ChannelType> for i32 {
    fn from(value: ChannelType) -> Self {
        match value {
            ChannelType::CB => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_CB as i32
            }
            ChannelType::FD => _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_FD as i32,
            ChannelType::NetdevPhysicalPort => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_PHYSICAL_PORT as i32
            }
            ChannelType::NetdevLogicalPort => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_LOGICAL_PORT as i32
            }
            ChannelType::NetdevL3 => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_L3 as i32
            }
            ChannelType::Genetlink => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_GENETLINK as i32
            }
        }
    }
}

// TODO: still needs From implementation for all types
// * @objects SAI_OBJECT_TYPE_PORT, SAI_OBJECT_TYPE_LAG, SAI_OBJECT_TYPE_ROUTER_INTERFACE
// SAI_HOSTIF_TABLE_ENTRY_ATTR_OBJ_ID,
#[derive(Clone, Copy)]
pub struct TableEntryObjectID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for TableEntryObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "tableentry:oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for TableEntryObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<PortID> for TableEntryObjectID {
    fn from(value: PortID) -> Self {
        Self { id: value.id }
    }
}

impl From<TableEntryObjectID> for sai_object_id_t {
    fn from(value: TableEntryObjectID) -> Self {
        value.id
    }
}

// TODO: still needs From implementation for all types
// * @objects SAI_OBJECT_TYPE_HOSTIF_TRAP, SAI_OBJECT_TYPE_HOSTIF_USER_DEFINED_TRAP
// SAI_HOSTIF_TABLE_ENTRY_ATTR_TRAP_ID,
#[derive(Clone, Copy)]
pub struct TableEntryTrapID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for TableEntryTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for TableEntryTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<trap::TrapID> for TableEntryTrapID {
    fn from(value: trap::TrapID) -> Self {
        Self { id: value.id }
    }
}

impl From<TableEntryTrapID> for sai_object_id_t {
    fn from(value: TableEntryTrapID) -> Self {
        value.id
    }
}

#[derive(Clone, Debug)]
pub enum TableEntryAttribute {
    Type(TableEntryType),
    ObjectID(TableEntryObjectID),
    TrapID(TableEntryTrapID),
    ChannelType(ChannelType),
    HostIf(super::HostIfID),
}

impl From<TableEntryAttribute> for sai_attribute_t {
    fn from(value: TableEntryAttribute) -> Self {
        match value {
            TableEntryAttribute::Type(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_TYPE,
                value: sai_attribute_value_t { s32: v.into() },
            },
            TableEntryAttribute::ObjectID(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_OBJ_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            TableEntryAttribute::TrapID(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_TRAP_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            TableEntryAttribute::ChannelType(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_CHANNEL_TYPE,
                value: sai_attribute_value_t { s32: v.into() },
            },
            TableEntryAttribute::HostIf(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_HOST_IF,
                value: sai_attribute_value_t { oid: v.into() },
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct TableEntryID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for TableEntryID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for TableEntryID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<TableEntryID> for sai_object_id_t {
    fn from(value: TableEntryID) -> Self {
        value.id
    }
}

impl From<TableEntry<'_>> for TableEntryID {
    fn from(value: TableEntry) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone)]
pub struct TableEntry<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for TableEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TableEntry(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for TableEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> TableEntry<'a> {
    pub fn remove(self) -> Result<(), Error> {
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let remove_hostif_table_entry = hostif_api
            .remove_hostif_table_entry
            .ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_hostif_table_entry(self.id) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}

impl ObjectID<TableEntryID> for TableEntry<'_> {
    fn to_id(&self) -> TableEntryID {
        TableEntryID { id: self.id }
    }
}
