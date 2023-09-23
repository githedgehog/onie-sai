use super::*;
use sai_sys::*;

#[derive(Clone, Copy, Debug)]
pub enum RouterInterfaceType {
    Port,
    VLAN,
    Loopback,
    MplsRouter,
    SubPort,
    Bridge,
    QinQPort,
}

impl From<RouterInterfaceType> for i32 {
    fn from(value: RouterInterfaceType) -> Self {
        match value {
            RouterInterfaceType::Port => {
                _sai_router_interface_type_t_SAI_ROUTER_INTERFACE_TYPE_PORT as i32
            }
            RouterInterfaceType::VLAN => {
                _sai_router_interface_type_t_SAI_ROUTER_INTERFACE_TYPE_VLAN as i32
            }
            RouterInterfaceType::Loopback => {
                _sai_router_interface_type_t_SAI_ROUTER_INTERFACE_TYPE_LOOPBACK as i32
            }
            RouterInterfaceType::MplsRouter => {
                _sai_router_interface_type_t_SAI_ROUTER_INTERFACE_TYPE_MPLS_ROUTER as i32
            }
            RouterInterfaceType::SubPort => {
                _sai_router_interface_type_t_SAI_ROUTER_INTERFACE_TYPE_SUB_PORT as i32
            }
            RouterInterfaceType::Bridge => {
                _sai_router_interface_type_t_SAI_ROUTER_INTERFACE_TYPE_BRIDGE as i32
            }
            RouterInterfaceType::QinQPort => {
                _sai_router_interface_type_t_SAI_ROUTER_INTERFACE_TYPE_QINQ_PORT as i32
            }
        }
    }
}

// TODO: Ingress and Egress ACL need proper type
#[derive(Clone, Copy, Debug)]
pub enum RouterInterfaceAttribute {
    // VirtualRouterID(VirtualRouterID),
    Type(RouterInterfaceType),
    PortID(RouterInterfacePortID),
    VlanID(vlan::VLANID),
    OuterVlanID(u16),
    InnerVlanID(u16),
    BridgeID(bridge::BridgeID),
    SrcMacAddress(sai_mac_t),
    AdminV4State(bool),
    AdminV6State(bool),
    MTU(u32),
    IngressACL(sai_object_id_t),
    EgressACL(sai_object_id_t),
    NeighborMissPacketAction(PacketAction),
    V4McastEnable(bool),
    V6McastEnable(bool),
    LoopbackPacketAction(PacketAction),
    IsVirtual(bool),
    NATZoneID(u8),
    DisableDecrementTTL(bool),
    MplsState(bool),
}

impl From<RouterInterfaceAttribute> for sai_attribute_t {
    fn from(value: RouterInterfaceAttribute) -> Self {
        match value {
            // RouterInterfaceAttribute::VirtualRouterID(v) => sai_attribute_t {
            //     id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_VIRTUAL_ROUTER_ID,
            //     value: sai_attribute_value_t { oid: v.into() },
            // },
            RouterInterfaceAttribute::Type(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_TYPE,
                value: sai_attribute_value_t { s32: v.into() },
            },
            RouterInterfaceAttribute::PortID(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_PORT_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            RouterInterfaceAttribute::VlanID(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_VLAN_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            RouterInterfaceAttribute::OuterVlanID(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_OUTER_VLAN_ID,
                value: sai_attribute_value_t { u16_: v },
            },
            RouterInterfaceAttribute::InnerVlanID(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_INNER_VLAN_ID,
                value: sai_attribute_value_t { u16_: v },
            },
            RouterInterfaceAttribute::BridgeID(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_BRIDGE_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            RouterInterfaceAttribute::SrcMacAddress(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_SRC_MAC_ADDRESS,
                value: sai_attribute_value_t { mac: v.clone() },
            },
            RouterInterfaceAttribute::AdminV4State(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_ADMIN_V4_STATE,
                value: sai_attribute_value_t { booldata: v },
            },
            RouterInterfaceAttribute::AdminV6State(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_ADMIN_V6_STATE,
                value: sai_attribute_value_t { booldata: v },
            },
            RouterInterfaceAttribute::MTU(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_MTU,
                value: sai_attribute_value_t { u32_: v },
            },
            RouterInterfaceAttribute::IngressACL(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_INGRESS_ACL,
                value: sai_attribute_value_t { oid: v },
            },
            RouterInterfaceAttribute::EgressACL(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_EGRESS_ACL,
                value: sai_attribute_value_t { oid: v },
            },
            RouterInterfaceAttribute::NeighborMissPacketAction(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_NEIGHBOR_MISS_PACKET_ACTION,
                value: sai_attribute_value_t { s32: v.into() },
            },
            RouterInterfaceAttribute::V4McastEnable(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_V4_MCAST_ENABLE,
                value: sai_attribute_value_t { booldata: v },
            },
            RouterInterfaceAttribute::V6McastEnable(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_V6_MCAST_ENABLE,
                value: sai_attribute_value_t { booldata: v },
            },
            RouterInterfaceAttribute::LoopbackPacketAction(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_LOOPBACK_PACKET_ACTION,
                value: sai_attribute_value_t { s32: v.into() },
            },
            RouterInterfaceAttribute::IsVirtual(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_IS_VIRTUAL,
                value: sai_attribute_value_t { booldata: v },
            },
            RouterInterfaceAttribute::NATZoneID(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_NAT_ZONE_ID,
                value: sai_attribute_value_t { u8_: v },
            },
            RouterInterfaceAttribute::DisableDecrementTTL(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_DISABLE_DECREMENT_TTL,
                value: sai_attribute_value_t { booldata: v },
            },
            RouterInterfaceAttribute::MplsState(v) => sai_attribute_t {
                id: _sai_router_interface_attr_t_SAI_ROUTER_INTERFACE_ATTR_ADMIN_MPLS_STATE,
                value: sai_attribute_value_t { booldata: v },
            },
        }
    }
}

// TODO: needs to be extended to the other types
// * @objects SAI_OBJECT_TYPE_PORT, SAI_OBJECT_TYPE_LAG, SAI_OBJECT_TYPE_SYSTEM_PORT
// SAI_ROUTER_INTERFACE_ATTR_PORT_ID,
#[derive(Clone, Copy)]
pub struct RouterInterfacePortID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for RouterInterfacePortID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "routerinterface:oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for RouterInterfacePortID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<RouterInterfacePortID> for sai_object_id_t {
    fn from(value: RouterInterfacePortID) -> Self {
        value.id
    }
}

impl From<Port<'_>> for RouterInterfacePortID {
    fn from(value: Port) -> Self {
        Self { id: value.id }
    }
}

impl From<PortID> for RouterInterfacePortID {
    fn from(value: PortID) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone, Copy)]
pub struct RouterInterfaceID {
    pub(crate) id: sai_object_id_t,
}

impl std::fmt::Debug for RouterInterfaceID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for RouterInterfaceID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<RouterInterfaceID> for sai_object_id_t {
    fn from(value: RouterInterfaceID) -> Self {
        value.id
    }
}

impl From<RouterInterface<'_>> for RouterInterfaceID {
    fn from(value: RouterInterface) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone)]
pub struct RouterInterface<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for RouterInterface<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RouterInterface(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for RouterInterface<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> RouterInterface<'a> {
    pub fn remove(self) -> Result<(), Error> {
        let router_interface_api = self
            .sai
            .router_interface_api()
            .ok_or(Error::APIUnavailable)?;
        let remove_router_interface = router_interface_api
            .remove_router_interface
            .ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_router_interface(self.id) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}

impl ObjectID<RouterInterfaceID> for RouterInterface<'_> {
    fn to_id(&self) -> RouterInterfaceID {
        RouterInterfaceID { id: self.id }
    }
}
