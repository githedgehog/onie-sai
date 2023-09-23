use crate::*;
use sai_sys::*;

#[derive(Clone, Copy)]
pub struct TrapID {
    pub(crate) id: sai_object_id_t,
}

impl std::fmt::Debug for TrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl std::fmt::Display for TrapID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl From<Trap<'_>> for TrapID {
    fn from(value: Trap) -> Self {
        Self { id: value.id }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TrapType {
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

impl From<TrapType> for i32 {
    fn from(value: TrapType) -> Self {
        match value {
            TrapType::STP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_STP as i32,
            TrapType::LACP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LACP as i32,
            TrapType::EAPOL => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_EAPOL as i32,
            TrapType::LLDP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LLDP as i32,
            TrapType::PVRST => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PVRST as i32,
            TrapType::IGMPTypeQuery => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_QUERY as i32
            }
            TrapType::IGMPTypeLeave => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_LEAVE as i32
            }
            TrapType::IGMPTypeV1Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_V1_REPORT as i32
            }
            TrapType::IGMPTypeV2Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_V2_REPORT as i32
            }
            TrapType::IGMPTypeV3Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IGMP_TYPE_V3_REPORT as i32
            }
            TrapType::SamplePacket => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SAMPLEPACKET as i32
            }
            TrapType::UDLD => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_UDLD as i32,
            TrapType::CDP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_CDP as i32,
            TrapType::VTP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_VTP as i32,
            TrapType::DTP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DTP as i32,
            TrapType::PAGP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PAGP as i32,
            TrapType::PTP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PTP as i32,
            TrapType::PTPTxEvent => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PTP_TX_EVENT as i32
            }
            TrapType::DHCPL2 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCP_L2 as i32,
            TrapType::DHCPv6L2 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCPV6_L2 as i32,
            TrapType::SwitchCustomRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SWITCH_CUSTOM_RANGE_BASE as i32
            }
            TrapType::ARPRequest => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ARP_REQUEST as i32,
            TrapType::ARPResponse => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ARP_RESPONSE as i32
            }
            TrapType::DHCP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCP as i32,
            TrapType::OSPF => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_OSPF as i32,
            TrapType::PIM => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIM as i32,
            TrapType::VRRP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_VRRP as i32,
            TrapType::DHCPv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DHCPV6 as i32,
            TrapType::OSPFv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_OSPFV6 as i32,
            TrapType::VRRPv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_VRRPV6 as i32,
            TrapType::IPv6NeighborDiscovery => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_NEIGHBOR_DISCOVERY as i32
            }
            TrapType::IPv6MLDv1v2 => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_MLD_V1_V2 as i32
            }
            TrapType::IPv6MLDv1Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_MLD_V1_REPORT as i32
            }
            TrapType::IPv6MLDv1Done => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_MLD_V1_DONE as i32
            }
            TrapType::MLDv2Report => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MLD_V2_REPORT as i32
            }
            TrapType::UnknownL3Multicast => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_UNKNOWN_L3_MULTICAST as i32
            }
            TrapType::SNATMiss => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SNAT_MISS as i32,
            TrapType::DNATMiss => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_DNAT_MISS as i32,
            TrapType::NATHairpin => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_NAT_HAIRPIN as i32,
            TrapType::IPv6NeighborSolicitation => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_NEIGHBOR_SOLICITATION as i32
            }
            TrapType::IPv6NeighborAdvertisement => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IPV6_NEIGHBOR_ADVERTISEMENT as i32
            }
            TrapType::ISIS => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ISIS as i32,
            TrapType::RouterCustomRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_ROUTER_CUSTOM_RANGE_BASE as i32
            }
            TrapType::IP2ME => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_IP2ME as i32,
            TrapType::SSH => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SSH as i32,
            TrapType::SNMP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_SNMP as i32,
            TrapType::BGP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BGP as i32,
            TrapType::BGPv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BGPV6 as i32,
            TrapType::BFD => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFD as i32,
            TrapType::BFDv6 => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFDV6 as i32,
            TrapType::BFDMicro => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFD_MICRO as i32,
            TrapType::BFDv6Micro => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_BFDV6_MICRO as i32,
            TrapType::LDP => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LDP as i32,
            TrapType::GNMI => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_GNMI as i32,
            TrapType::P4rt => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_P4RT as i32,
            TrapType::NTPClient => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_NTPCLIENT as i32,
            TrapType::NTPServer => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_NTPSERVER as i32,
            TrapType::LocalIPCustomRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_LOCAL_IP_CUSTOM_RANGE_BASE as i32
            }
            TrapType::L3MTUError => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_L3_MTU_ERROR as i32
            }
            TrapType::TTLError => _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_TTL_ERROR as i32,
            TrapType::StaticFDBMove => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_STATIC_FDB_MOVE as i32
            }
            TrapType::PipelineDiscardEgressBuffer => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIPELINE_DISCARD_EGRESS_BUFFER as i32
            }
            TrapType::PipelineDiscardWRED => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIPELINE_DISCARD_WRED as i32
            }
            TrapType::PipelineDiscardRouter => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_PIPELINE_DISCARD_ROUTER as i32
            }
            TrapType::MPLSTTLError => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MPLS_TTL_ERROR as i32
            }
            TrapType::MPLSRouterAlertLabel => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MPLS_ROUTER_ALERT_LABEL as i32
            }
            TrapType::MPLSLabelLookupMiss => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_MPLS_LABEL_LOOKUP_MISS as i32
            }
            TrapType::CustomExceptionRangeBase => {
                _sai_hostif_trap_type_t_SAI_HOSTIF_TRAP_TYPE_CUSTOM_EXCEPTION_RANGE_BASE as i32
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TrapAttribute {
    TrapType(TrapType),
    PacketAction(PacketAction),
    TrapPriority(u32),
    ExcludePortList(Vec<PortID>),
    TrapGroup(super::trap_group::TrapGroupID),
    MirrorSession(Vec<MirrorSessionID>),
    Counter(CounterID),
}

impl TrapAttribute {
    pub(crate) fn to_sai_attribute_t(
        &self,
        exclude_port_list_backing: &mut Vec<sai_object_id_t>,
        mirror_session_backing: &mut Vec<sai_object_id_t>,
    ) -> sai_attribute_t {
        match self {
            TrapAttribute::TrapType(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_TRAP_TYPE,
                value: sai_attribute_value_t { s32: (*v).into() },
            },
            TrapAttribute::PacketAction(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_PACKET_ACTION,
                value: sai_attribute_value_t { s32: (*v).into() },
            },
            TrapAttribute::TrapPriority(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_TRAP_PRIORITY,
                value: sai_attribute_value_t { u32_: *v },
            },
            TrapAttribute::ExcludePortList(v) => {
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
            TrapAttribute::TrapGroup(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_TRAP_GROUP,
                value: sai_attribute_value_t { oid: (*v).into() },
            },
            TrapAttribute::MirrorSession(v) => {
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
            TrapAttribute::Counter(v) => sai_attribute_t {
                id: _sai_hostif_trap_attr_t_SAI_HOSTIF_TRAP_ATTR_COUNTER_ID,
                value: sai_attribute_value_t { oid: (*v).into() },
            },
        }
    }
}

#[derive(Clone)]
pub struct Trap<'a> {
    pub(crate) id: sai_object_id_t,
    pub(crate) sai: &'a SAI,
}

impl std::fmt::Debug for Trap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HostIfTrap(oid:{:#x})", self.id)
    }
}

impl std::fmt::Display for Trap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oid:{:#x}", self.id)
    }
}

impl<'a> Trap<'a> {
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
