use crate::router_interface::{RouterInterface, RouterInterfaceID};

use super::*;
use sai_sys::*;

// TODO: implement From for all types
// * @type sai_object_id_t
// * @objects SAI_OBJECT_TYPE_NEXT_HOP, SAI_OBJECT_TYPE_NEXT_HOP_GROUP, SAI_OBJECT_TYPE_ROUTER_INTERFACE, SAI_OBJECT_TYPE_PORT
// SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID,
#[derive(Clone, Copy)]
pub struct RouteEntryNextHopID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for RouteEntryNextHopID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for RouteEntryNextHopID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<RouteEntryNextHopID> for sai_object_id_t {
    fn from(value: RouteEntryNextHopID) -> Self {
        value.id
    }
}

impl From<PortID> for RouteEntryNextHopID {
    fn from(value: PortID) -> Self {
        Self { id: value.id }
    }
}

impl From<Port<'_>> for RouteEntryNextHopID {
    fn from(value: Port<'_>) -> Self {
        Self { id: value.id }
    }
}

impl From<RouterInterfaceID> for RouteEntryNextHopID {
    fn from(value: RouterInterfaceID) -> Self {
        Self { id: value.id }
    }
}

impl From<RouterInterface<'_>> for RouteEntryNextHopID {
    fn from(value: RouterInterface<'_>) -> Self {
        Self { id: value.id }
    }
}

/*
    *
     * @type sai_packet_action_t
    SAI_ROUTE_ENTRY_ATTR_PACKET_ACTION = SAI_ROUTE_ENTRY_ATTR_START,
     * @type sai_object_id_t
     * @objects SAI_OBJECT_TYPE_HOSTIF_USER_DEFINED_TRAP
    SAI_ROUTE_ENTRY_ATTR_USER_TRAP_ID,
     * @type sai_object_id_t
     * @objects SAI_OBJECT_TYPE_NEXT_HOP, SAI_OBJECT_TYPE_NEXT_HOP_GROUP, SAI_OBJECT_TYPE_ROUTER_INTERFACE, SAI_OBJECT_TYPE_PORT
    SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID,
     * @type sai_uint32_t
    SAI_ROUTE_ENTRY_ATTR_META_DATA,
     * @type sai_ip_addr_family_t
    SAI_ROUTE_ENTRY_ATTR_IP_ADDR_FAMILY,
     * @type sai_object_id_t
     * @objects SAI_OBJECT_TYPE_COUNTER
    SAI_ROUTE_ENTRY_ATTR_COUNTER_ID,
*/

#[derive(Clone, Copy, Debug)]
pub enum RouteEntryAttribute {
    PacketAction(PacketAction),
    UserDefinedTrap(hostif::HostIfUserDefinedTrapID),
    NextHopID(RouteEntryNextHopID),
    MetaData(u32),
    AddrFamily(sai_ip_addr_family_t),
    CounterID(CounterID),
}

impl From<RouteEntryAttribute> for sai_attribute_t {
    fn from(value: RouteEntryAttribute) -> Self {
        match value {
            RouteEntryAttribute::PacketAction(v) => sai_attribute_t {
                id: _sai_route_entry_attr_t_SAI_ROUTE_ENTRY_ATTR_PACKET_ACTION,
                value: sai_attribute_value_t { s32: v.into() },
            },
            RouteEntryAttribute::UserDefinedTrap(v) => sai_attribute_t {
                id: _sai_route_entry_attr_t_SAI_ROUTE_ENTRY_ATTR_USER_TRAP_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            RouteEntryAttribute::NextHopID(v) => sai_attribute_t {
                id: _sai_route_entry_attr_t_SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            RouteEntryAttribute::MetaData(v) => sai_attribute_t {
                id: _sai_route_entry_attr_t_SAI_ROUTE_ENTRY_ATTR_META_DATA,
                value: sai_attribute_value_t { u32_: v },
            },
            RouteEntryAttribute::AddrFamily(v) => sai_attribute_t {
                id: _sai_route_entry_attr_t_SAI_ROUTE_ENTRY_ATTR_IP_ADDR_FAMILY,
                value: sai_attribute_value_t { u32_: v },
            },
            RouteEntryAttribute::CounterID(v) => sai_attribute_t {
                id: _sai_route_entry_attr_t_SAI_ROUTE_ENTRY_ATTR_COUNTER_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct RouteEntry<'a> {
    pub(crate) entry: sai_route_entry_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for RouteEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        write!(
            f,
            "RouteEntry(switch_id:oid:{:#x}, vr_id:oid:{:#x})",
            self.entry.switch_id, self.entry.vr_id
        )
    }
}

impl std::fmt::Display for RouteEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        write!(
            f,
            "RouteEntry: switch_id:oid:{:#x}, vr_id:oid:{:#x})",
            self.entry.switch_id, self.entry.vr_id
        )
    }
}

impl<'a> RouteEntry<'a> {
    pub fn remove(self) -> Result<(), Error> {
        let route_api = self.sai.route_api().ok_or(Error::APIUnavailable)?;
        let remove_route_entry = route_api
            .remove_route_entry
            .ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_route_entry(&self.entry) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}
