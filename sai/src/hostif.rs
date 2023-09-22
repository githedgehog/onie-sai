use crate::port::{Port, PortID};

use super::*;
use sai_sys::*;

#[derive(Clone, Copy)]
pub struct HostIfTrapGroupID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTrapGroupID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTrapGroupID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfTrapGroupID> for sai_object_id_t {
    fn from(value: HostIfTrapGroupID) -> Self {
        value.id
    }
}

impl From<HostIfTrapGroup<'_>> for HostIfTrapGroupID {
    fn from(value: HostIfTrapGroup) -> Self {
        Self { id: value.id }
    }
}

#[derive(Copy, Clone)]
pub struct HostIfTrapGroup<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for HostIfTrapGroup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIfTrapGroup(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for HostIfTrapGroup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> HostIfTrapGroup<'a> {
    pub fn remove(self) -> Result<(), Error> {
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let remove_hostif_trap_group = hostif_api
            .remove_hostif_trap_group
            .ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_hostif_trap_group(self.id) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy)]
pub struct HostIfTrapID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfTrap<'_>> for HostIfTrapID {
    fn from(value: HostIfTrap) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HostIfTrapType {
    STP,
    LACP,
    EAPOL,
    LLDP,
    PVRST,
    IGMPTypeQuery,
    IGMPTypeLeave,
    IGMPTypeV1Report,
    IGMPTypeV2Report,
    IGMPTypeV3Report,
    SamplePacket,
    UDLD,
    CDP,
    VTP,
    DTP,
    PAGP,
    PTP,
    PTPTxEvent,
    DHCPL2,
    DHCPv6L2,
    SwitchCustomRangeBase,
    ARPRequest,
    ARPResponse,
    DHCP,
    OSPF,
    PIM,
    VRRP,
    DHCPv6,
    OSPFv6,
    VRRPv6,
    IPv6NeighborDiscovery,
    IPv6MLDv1v2,
    IPv6MLDv1Report,
    IPv6MLDv1Done,
    MLDv2Report,
    UnknownL3Multicast,
    SNATMiss,
    DNATMiss,
    NATHairpin,
    IPv6NeighborSolicitation,
    IPv6NeighborAdvertisement,
    ISIS,
    RouterCustomRangeBase,
    IP2ME,
    SSH,
    SNMP,
    BGP,
    BGPv6,
    BFD,
    BFDv6,
    BFDMicro,
    BFDv6Micro,
    LDP,
    GNMI,
    P4rt,
    NTPClient,
    NTPServer,
    LocalIPCustomRangeBase,
    L3MTUError,
    TTLError,
    StaticFDBMove,
    PipelineDiscardEgressBuffer,
    PipelineDiscardWRED,
    PipelineDiscardRouter,
    MPLSTTLError,
    MPLSRouterAlertLabel,
    MPLSLabelLookupMiss,
    CustomExceptionRangeBase,
}

impl From<HostIfTrapType> for i32 {
    fn from(value: HostIfTrapType) -> Self {
        match value {
            HostIfTrapType::STP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_STP as i32,
            HostIfTrapType::LACP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LACP as i32,
            HostIfTrapType::EAPOL => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_EAPOL as i32,
            HostIfTrapType::LLDP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LLDP as i32,
            HostIfTrapType::PVRST => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PVRST as i32,
            HostIfTrapType::IGMPTypeQuery => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_QUERY as i32
            }
            HostIfTrapType::IGMPTypeLeave => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_LEAVE as i32
            }
            HostIfTrapType::IGMPTypeV1Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_V1_REPORT as i32
            }
            HostIfTrapType::IGMPTypeV2Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_V2_REPORT as i32
            }
            HostIfTrapType::IGMPTypeV3Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_V3_REPORT as i32
            }
            HostIfTrapType::SamplePacket => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SAMPLEPACKET as i32
            }
            HostIfTrapType::UDLD => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_UDLD as i32,
            HostIfTrapType::CDP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_CDP as i32,
            HostIfTrapType::VTP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_VTP as i32,
            HostIfTrapType::DTP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DTP as i32,
            HostIfTrapType::PAGP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PAGP as i32,
            HostIfTrapType::PTP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PTP as i32,
            HostIfTrapType::PTPTxEvent => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PTP_TX_EVENT as i32
            }
            HostIfTrapType::DHCPL2 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCP_L2 as i32,
            HostIfTrapType::DHCPv6L2 => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCPV6_L2 as i32
            }
            HostIfTrapType::SwitchCustomRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SWITCH_CUSTOM_RANGE_BASE as i32
            }
            HostIfTrapType::ARPRequest => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ARP_REQUEST as i32
            }
            HostIfTrapType::ARPResponse => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ARP_RESPONSE as i32
            }
            HostIfTrapType::DHCP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCP as i32,
            HostIfTrapType::OSPF => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_OSPF as i32,
            HostIfTrapType::PIM => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIM as i32,
            HostIfTrapType::VRRP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_VRRP as i32,
            HostIfTrapType::DHCPv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCPV6 as i32,
            HostIfTrapType::OSPFv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_OSPFV6 as i32,
            HostIfTrapType::VRRPv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_VRRPV6 as i32,
            HostIfTrapType::IPv6NeighborDiscovery => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_NEIGHBOR_DISCOVERY as i32
            }
            HostIfTrapType::IPv6MLDv1v2 => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_MLD_V1_V2 as i32
            }
            HostIfTrapType::IPv6MLDv1Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_MLD_V1_REPORT as i32
            }
            HostIfTrapType::IPv6MLDv1Done => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_MLD_V1_DONE as i32
            }
            HostIfTrapType::MLDv2Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MLD_V2_REPORT as i32
            }
            HostIfTrapType::UnknownL3Multicast => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_UNKNOWN_L3_MULTICAST as i32
            }
            HostIfTrapType::SNATMiss => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SNAT_MISS as i32
            }
            HostIfTrapType::DNATMiss => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DNAT_MISS as i32
            }
            HostIfTrapType::NATHairpin => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_NAT_HAIRPIN as i32
            }
            HostIfTrapType::IPv6NeighborSolicitation => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_NEIGHBOR_SOLICITATION as i32
            }
            HostIfTrapType::IPv6NeighborAdvertisement => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_NEIGHBOR_ADVERTISEMENT as i32
            }
            HostIfTrapType::ISIS => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ISIS as i32,
            HostIfTrapType::RouterCustomRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ROUTER_CUSTOM_RANGE_BASE as i32
            }
            HostIfTrapType::IP2ME => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IP2ME as i32,
            HostIfTrapType::SSH => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SSH as i32,
            HostIfTrapType::SNMP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SNMP as i32,
            HostIfTrapType::BGP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BGP as i32,
            HostIfTrapType::BGPv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BGPV6 as i32,
            HostIfTrapType::BFD => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFD as i32,
            HostIfTrapType::BFDv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFDV6 as i32,
            HostIfTrapType::BFDMicro => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFD_MICRO as i32
            }
            HostIfTrapType::BFDv6Micro => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFDV6_MICRO as i32
            }
            HostIfTrapType::LDP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LDP as i32,
            HostIfTrapType::GNMI => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_GNMI as i32,
            HostIfTrapType::P4rt => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_P4RT as i32,
            HostIfTrapType::NTPClient => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_NTPCLIENT as i32
            }
            HostIfTrapType::NTPServer => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_NTPSERVER as i32
            }
            HostIfTrapType::LocalIPCustomRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LOCAL_IP_CUSTOM_RANGE_BASE as i32
            }
            HostIfTrapType::L3MTUError => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_L3_MTU_ERROR as i32
            }
            HostIfTrapType::TTLError => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_TTL_ERROR as i32
            }
            HostIfTrapType::StaticFDBMove => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_STATIC_FDB_MOVE as i32
            }
            HostIfTrapType::PipelineDiscardEgressBuffer => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIPELINE_DISCARD_EGRESS_BUFFER as i32
            }
            HostIfTrapType::PipelineDiscardWRED => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIPELINE_DISCARD_WRED as i32
            }
            HostIfTrapType::PipelineDiscardRouter => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIPELINE_DISCARD_ROUTER as i32
            }
            HostIfTrapType::MPLSTTLError => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MPLS_TTL_ERROR as i32
            }
            HostIfTrapType::MPLSRouterAlertLabel => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MPLS_ROUTER_ALERT_LABEL as i32
            }
            HostIfTrapType::MPLSLabelLookupMiss => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MPLS_LABEL_LOOKUP_MISS as i32
            }
            HostIfTrapType::CustomExceptionRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_CUSTOM_EXCEPTION_RANGE_BASE as i32
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum HostIfTrapAttribute {
    TrapType(HostIfTrapType),
    PacketAction(PacketAction),
    TrapPriority(u32),
    ExcludePortList(Vec<PortID>),
    TrapGroup(HostIfTrapGroupID),
    MirrorSession(Vec<MirrorSessionID>),
    Counter(CounterID),
}

impl HostIfTrapAttribute {
    pub(crate) fn to_sai_attribute_t(
        &self,
        exclude_port_list_backing: &mut Vec<sai_object_id_t>,
        mirror_session_backing: &mut Vec<sai_object_id_t>,
    ) -> sai_attribute_t {
        match self {
            HostIfTrapAttribute::TrapType(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_TRAP_TYPE,
                value: sai_attribute_value_t { s32: (*v).into() },
            },
            HostIfTrapAttribute::PacketAction(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_PACKET_ACTION,
                value: sai_attribute_value_t { s32: (*v).into() },
            },
            HostIfTrapAttribute::TrapPriority(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_TRAP_PRIORITY,
                value: sai_attribute_value_t { u32_: *v },
            },
            HostIfTrapAttribute::ExcludePortList(v) => {
                // let mut arg: Vec<sai_object_id_t> = v.into_iter().map(|v| v.into()).collect();
                exclude_port_list_backing.clear();
                v.iter()
                    .for_each(|port_id| exclude_port_list_backing.push(port_id.id));
                sai_attribute_t {
                    id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_EXCLUDE_PORT_LIST,
                    value: sai_attribute_value_t {
                        objlist: sai_object_list_t {
                            count: exclude_port_list_backing.len() as u32,
                            list: exclude_port_list_backing.as_mut_ptr(),
                        },
                    },
                }
            }
            HostIfTrapAttribute::TrapGroup(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_TRAP_GROUP,
                value: sai_attribute_value_t { oid: (*v).into() },
            },
            HostIfTrapAttribute::MirrorSession(v) => {
                // let mut arg: Vec<sai_object_id_t> = v.into_iter().map(|v| v.into()).collect();
                mirror_session_backing.clear();
                v.iter().for_each(|mirror_session_id| {
                    mirror_session_backing.push(mirror_session_id.id)
                });
                sai_attribute_t {
                    id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_MIRROR_SESSION,
                    value: sai_attribute_value_t {
                        objlist: sai_object_list_t {
                            count: mirror_session_backing.len() as u32,
                            list: mirror_session_backing.as_mut_ptr(),
                        },
                    },
                }
            }
            HostIfTrapAttribute::Counter(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_COUNTER_ID,
                value: sai_attribute_value_t { oid: (*v).into() },
            },
        }
    }
}

#[derive(Copy, Clone)]
pub struct HostIfTrap<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for HostIfTrap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIfTrap(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for HostIfTrap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> HostIfTrap<'a> {
    pub fn remove(self) -> Result<(), Error> {
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let remove_hostif_trap = hostif_api
            .remove_hostif_trap
            .ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_hostif_trap(self.id) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy)]
pub struct HostIfUserDefinedTrapID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfUserDefinedTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfUserDefinedTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfUserDefinedTrapID> for sai_object_id_t {
    fn from(value: HostIfUserDefinedTrapID) -> Self {
        value.id
    }
}

impl From<HostIfUserDefinedTrap<'_>> for HostIfUserDefinedTrapID {
    fn from(value: HostIfUserDefinedTrap<'_>) -> Self {
        Self { id: value.id }
    }
}

#[derive(Copy, Clone)]
pub struct HostIfUserDefinedTrap<'a> {
    id: sai_object_id_t,
    sai: &'a SAI,
}

impl std::fmt::Debug for HostIfUserDefinedTrap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIfUserDefinedTrap(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for HostIfUserDefinedTrap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> HostIfUserDefinedTrap<'a> {
    pub fn remove(self) -> Result<(), Error> {
        let hostif_api = self.sai.hostif_api().ok_or(Error::APIUnavailable)?;
        let remove_hostif_user_defined_trap = hostif_api
            .remove_hostif_user_defined_trap
            .ok_or(Error::APIFunctionUnavailable)?;

        let st = unsafe { remove_hostif_user_defined_trap(self.id) };
        if st != SAI_STATUS_SUCCESS as i32 {
            Err(Error::SAI(Status::from(st)))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HostIfTableEntryType {
    Port,
    LAG,
    VLAN,
    TrapID,
    Wildcard,
}

impl From<HostIfTableEntryType> for i32 {
    fn from(value: HostIfTableEntryType) -> Self {
        match value {
            HostIfTableEntryType::Port => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_PORT as i32
            }
            HostIfTableEntryType::LAG => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_LAG as i32
            }
            HostIfTableEntryType::VLAN => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_VLAN as i32
            }
            HostIfTableEntryType::TrapID => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_TRAP_ID as i32
            }
            HostIfTableEntryType::Wildcard => {
                _sai_hostif_table_entry_type_t_SAI_HOSTIF_TABLE_ENTRY_TYPE_WILDCARD as i32
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HostIfTableEntryChannelType {
    CB,
    FD,
    NetdevPhysicalPort,
    NetdevLogicalPort,
    NetdevL3,
    Genetlink,
}

impl From<HostIfTableEntryChannelType> for i32 {
    fn from(value: HostIfTableEntryChannelType) -> Self {
        match value {
            HostIfTableEntryChannelType::CB => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_CB as i32
            }
            HostIfTableEntryChannelType::FD => _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_FD as i32,
            HostIfTableEntryChannelType::NetdevPhysicalPort => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_PHYSICAL_PORT as i32
            }
            HostIfTableEntryChannelType::NetdevLogicalPort => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_LOGICAL_PORT as i32
            }
            HostIfTableEntryChannelType::NetdevL3 => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_L3 as i32
            }
            HostIfTableEntryChannelType::Genetlink => {
                _sai_hostif_table_entry_channel_type_t_SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_GENETLINK as i32
            }
        }
    }
}

// TODO: still needs From implementation for all types
// * @objects SAI_OBJECT_TYPE_PORT, SAI_OBJECT_TYPE_LAG, SAI_OBJECT_TYPE_ROUTER_INTERFACE
// SAI_HOSTIF_TABLE_ENTRY_ATTR_OBJ_ID,
#[derive(Clone, Copy)]
pub struct HostIfTableEntryObjectID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTableEntryObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTableEntryObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<PortID> for HostIfTableEntryObjectID {
    fn from(value: PortID) -> Self {
        Self { id: value.id }
    }
}

impl From<HostIfTableEntryObjectID> for sai_object_id_t {
    fn from(value: HostIfTableEntryObjectID) -> Self {
        value.id
    }
}

// TODO: still needs From implementation for all types
// * @objects SAI_OBJECT_TYPE_HOSTIF_TRAP, SAI_OBJECT_TYPE_HOSTIF_USER_DEFINED_TRAP
// SAI_HOSTIF_TABLE_ENTRY_ATTR_TRAP_ID,
#[derive(Clone, Copy)]
pub struct HostIfTableEntryTrapID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTableEntryTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTableEntryTrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfTrapID> for HostIfTableEntryTrapID {
    fn from(value: HostIfTrapID) -> Self {
        Self { id: value.id }
    }
}

impl From<HostIfTableEntryTrapID> for sai_object_id_t {
    fn from(value: HostIfTableEntryTrapID) -> Self {
        value.id
    }
}

#[derive(Clone, Debug)]
pub enum HostIfTableEntryAttribute {
    Type(HostIfTableEntryType),
    ObjectID(HostIfTableEntryObjectID),
    TrapID(HostIfTableEntryTrapID),
    ChannelType(HostIfTableEntryChannelType),
    HostIf(HostIfID),
}

impl From<HostIfTableEntryAttribute> for sai_attribute_t {
    fn from(value: HostIfTableEntryAttribute) -> Self {
        match value {
            HostIfTableEntryAttribute::Type(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_TYPE,
                value: sai_attribute_value_t { s32: v.into() },
            },
            HostIfTableEntryAttribute::ObjectID(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_OBJ_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            HostIfTableEntryAttribute::TrapID(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_TRAP_ID,
                value: sai_attribute_value_t { oid: v.into() },
            },
            HostIfTableEntryAttribute::ChannelType(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_CHANNEL_TYPE,
                value: sai_attribute_value_t { s32: v.into() },
            },
            HostIfTableEntryAttribute::HostIf(v) => Self {
                id: _sai_hostif_table_entry_attr_t_SAI_HOSTIF_TABLE_ENTRY_ATTR_HOST_IF,
                value: sai_attribute_value_t { oid: v.into() },
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct HostIfTableEntryID {
    id: sai_object_id_t,
}

impl std::fmt::Debug for HostIfTableEntryID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for HostIfTableEntryID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<HostIfTableEntryID> for sai_object_id_t {
    fn from(value: HostIfTableEntryID) -> Self {
        value.id
    }
}

impl From<HostIfTableEntry<'_>> for HostIfTableEntryID {
    fn from(value: HostIfTableEntry) -> Self {
        Self { id: value.id }
    }
}

#[derive(Copy, Clone)]
pub struct HostIfTableEntry<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for HostIfTableEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIfTableEntry(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for HostIfTableEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> HostIfTableEntry<'a> {
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
pub enum HostIfVlanTag {
    Strip,
    Keep,
    Original,
}

impl From<HostIfVlanTag> for i32 {
    fn from(value: HostIfVlanTag) -> Self {
        match value {
            HostIfVlanTag::Strip => _sai_hostif_vlan_tag_t_SAI_HOSTIF_VLAN_TAG_STRIP as i32,
            HostIfVlanTag::Keep => _sai_hostif_vlan_tag_t_SAI_HOSTIF_VLAN_TAG_KEEP as i32,
            HostIfVlanTag::Original => _sai_hostif_vlan_tag_t_SAI_HOSTIF_VLAN_TAG_ORIGINAL as i32,
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
        write!(f, "oid:{:#x}", self.id)
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
    VlanTag(HostIfVlanTag),
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

#[derive(Clone, Copy)]
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
    pub fn set_vlan_tag(&self, vlan_tag: HostIfVlanTag) -> Result<(), Error> {
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
