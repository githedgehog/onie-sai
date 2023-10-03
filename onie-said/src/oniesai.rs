mod port;

use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use anyhow::anyhow;
use ipnet::IpNet;
use onie_sai_rpc::onie_sai;
use sai::bridge;
use sai::hostif::table_entry::ChannelType;
use sai::hostif::table_entry::TableEntryAttribute;
use sai::hostif::table_entry::TableEntryType;
use sai::hostif::trap::TrapAttribute;
use sai::hostif::trap::TrapType;
use sai::hostif::HostIf;
use sai::hostif::HostIfAttribute;
use sai::hostif::HostIfType;
use sai::hostif::VlanTag;
use sai::port::OperStatus;
use sai::port::PortID;
use sai::route::RouteEntryAttribute;
use sai::router_interface::RouterInterfaceAttribute;
use sai::router_interface::RouterInterfaceType;
use sai::switch::Switch;
use sai::switch::SwitchAttribute;
use sai::ObjectID;
use sai::PacketAction;
use sai::SAI;

use anyhow::Context;
use sai::sai_mac_t;
use sai::virtual_router::VirtualRouter;

use thiserror::Error;

use self::port::PhysicalPort;

#[derive(Error, Debug)]
pub(crate) enum ProcessError {
    #[error("SAI status returned unsuccessful")]
    SAIStatus(#[from] sai::Status),
    #[error("SAI command failed")]
    SAIError(#[from] sai::Error),
}

pub(crate) enum ProcessRequest {
    Shutdown,
    Version(
        (
            onie_sai::VersionRequest,
            Sender<Result<onie_sai::VersionResponse, ProcessError>>,
        ),
    ),
    LogicalPortStateChange((PortID, OperStatus)),
}

pub(crate) struct Processor<'a> {
    switch: Switch<'a>,
    virtual_router: VirtualRouter<'a>,
    cpu_port_id: PortID,
    cpu_hostif: HostIf<'a>,
    ports: Vec<PhysicalPort<'a>>,
    rx: Receiver<ProcessRequest>,
    tx: Sender<ProcessRequest>,
}

impl<'a> Processor<'a> {
    pub(crate) fn new(sai_api: &'a SAI, mac_address: sai_mac_t) -> anyhow::Result<Self> {
        // now create switch
        let switch: Switch<'a> = sai_api
            .switch_create(vec![
                SwitchAttribute::InitSwitch(true),
                SwitchAttribute::SrcMacAddress(mac_address),
            ])
            .context("failed to create switch")?;
        log::info!("successfully created switch: {:?}", switch);

        // the processor channel
        let (tx, rx) = channel();
        let psc_cb_tx = tx.clone();

        // port state change callback
        switch
            .set_port_state_change_callback(Box::new(move |notifications| {
                for notification in notifications {
                    let port_id = PortID::from(notification);
                    let port_state = OperStatus::from(notification);
                    log::info!(
                        "Port State Change Event: port_id = {:?}, port_state = {:?}",
                        port_id,
                        port_state
                    );
                    if let Err(e) = psc_cb_tx.send(ProcessRequest::LogicalPortStateChange((port_id, port_state))) {
                        log::error!("Port State Change Event: failed to submit port state change event to processor (port_id = {:?}, port_state = {:?}): {}", port_id, port_state, e);
                    }
                }
            }))
            .context("failed to set port state change callback")?;

        // remove default vlan members
        let default_vlan = switch
            .get_default_vlan()
            .context("failed to get default VLAN")?;
        log::info!("default VLAN of switch {} is: {:?}", switch, default_vlan);
        let members = default_vlan.get_members().context(format!(
            "failed to get VLAN members for default VLAN {}",
            default_vlan
        ))?;
        for member in members {
            log::info!("Removing VLAN member {}...", member);
            member.remove().context("failed to remove VLAN member")?;
        }

        // remove default bridge ports
        let default_bridge = switch
            .get_default_bridge()
            .context("failed to get dfeault bridge")?;
        log::info!(
            "default bridge of switch {} is: {:?}",
            switch,
            default_bridge
        );
        let bridge_ports = default_bridge.get_ports().context(format!(
            "failed to get bridge ports for default bridge {}",
            default_bridge
        ))?;
        for bridge_port in bridge_ports {
            match bridge_port.get_type() {
                // we only go ahead when this is a real port
                Ok(bridge::port::Type::Port) => {}
                Ok(v) => {
                    log::info!("not removing bridge port {} of type: {:?}", bridge_port, v);
                    continue;
                }
                Err(e) => {
                    return Err(anyhow!(
                        "failed to get bridge port type of bridge port {}: {:?}",
                        bridge_port,
                        e
                    ));
                }
            }

            log::info!("removing bridge port {}...", bridge_port);
            bridge_port
                .remove()
                .context("failed to remove bridge port")?;
        }

        // program traps
        let default_trap_group = switch
            .get_default_hostif_trap_group()
            .context("failed to get default host interface trap group")?;
        let default_trap_group_id = default_trap_group.to_id();
        let _ip2me_trap = switch
            .create_hostif_trap(vec![
                TrapAttribute::TrapType(TrapType::IP2ME),
                TrapAttribute::PacketAction(PacketAction::Trap),
                TrapAttribute::TrapGroup(default_trap_group_id),
            ])
            .context("failed to create ip2me trap")?;
        let _arp_req_trap = switch
            .create_hostif_trap(vec![
                TrapAttribute::TrapType(TrapType::ARPRequest),
                TrapAttribute::PacketAction(PacketAction::Copy),
                TrapAttribute::TrapGroup(default_trap_group_id),
            ])
            .context("failed to create ARP request trap")?;
        let _arp_resp_trap = switch
            .create_hostif_trap(vec![
                TrapAttribute::TrapType(TrapType::ARPResponse),
                TrapAttribute::PacketAction(PacketAction::Copy),
                TrapAttribute::TrapGroup(default_trap_group_id),
            ])
            .context("failed to create ARP response trap")?;

        let _default_table_entry = switch
            .create_hostif_table_entry(vec![
                TableEntryAttribute::Type(TableEntryType::Wildcard),
                TableEntryAttribute::ChannelType(ChannelType::NetdevPhysicalPort),
            ])
            .context("failed to create default host interface table entry")?;

        // get CPU port
        let cpu_port = switch.get_cpu_port().context("failed to get CPU port")?;
        let cpu_port_id = PortID::from(cpu_port);

        // create host interface for it
        let cpu_intf: HostIf<'a> = switch
            .create_hostif(vec![
                HostIfAttribute::Name("CPU".to_string()),
                HostIfAttribute::Type(HostIfType::Netdev),
                HostIfAttribute::ObjectID(cpu_port_id.into()),
                HostIfAttribute::OperStatus(true),
            ])
            .context(format!(
                "failed to create host interface for CPU port {}",
                cpu_port_id
            ))?;

        // get the default virtual router
        let default_virtual_router: VirtualRouter<'a> = switch
            .get_default_virtual_router()
            .context("failed to get default virtual router")?;

        // prep the router: create loopback interface
        // create initial routes
        let _lo_rif = default_virtual_router
            .create_router_interface(vec![
                RouterInterfaceAttribute::Type(RouterInterfaceType::Loopback),
                RouterInterfaceAttribute::MTU(9100),
            ])
            .context(format!(
                "failed to get create loopback interface for virtual router {}",
                default_virtual_router
            ))?;

        let _default_route_entry = default_virtual_router
            .create_route_entry(
                IpNet::from_str("0.0.0.0/0").unwrap(),
                vec![RouteEntryAttribute::PacketAction(PacketAction::Drop)],
            )
            .context(format!(
                "failed to create default route entry for virtual router {}",
                default_virtual_router
            ))?;

        // get ports now
        let ports = switch
            .get_ports()
            .context(format!("failed to get port list from switch {}", switch))?;

        let ports = PhysicalPort::from_ports(ports)
            .context("failed to create physical ports from SAI ports")?;

        Ok(Processor {
            switch: switch,
            virtual_router: default_virtual_router,
            cpu_port_id: cpu_port_id,
            cpu_hostif: cpu_intf,
            ports: ports,
            rx: rx,
            tx: tx,
        })
    }

    pub(crate) fn get_sender(&self) -> Sender<ProcessRequest> {
        self.tx.clone()
    }

    pub(crate) fn process(self) {
        while let Ok(req) = self.rx.recv() {
            match req {
                ProcessRequest::Shutdown => return,
                ProcessRequest::Version((r, resp_tx)) => {
                    let resp = self.process_version_request(r);
                    if let Err(e) = resp_tx.send(resp) {
                        log::error!(
                            "processor: failed to send version response to rpc server: {:?}",
                            e
                        );
                    };
                }
                ProcessRequest::LogicalPortStateChange((port_id, port_state)) => {
                    self.process_logical_port_state_change(port_id, port_state)
                }
            }
        }
    }

    fn process_version_request(
        &self,
        _: onie_sai::VersionRequest,
    ) -> Result<onie_sai::VersionResponse, ProcessError> {
        match SAI::api_version() {
            Err(e) => Err(ProcessError::SAIStatus(e)),
            Ok(v) => Ok(onie_sai::VersionResponse {
                onie_said_version: "0.1.0".to_string(),
                sai_version: v.to_string(),
                ..Default::default()
            }),
        }
    }

    fn process_logical_port_state_change(&self, port_id: PortID, port_state: OperStatus) {
        let mut found = false;
        for phy_port in self.ports.iter() {
            for log_port in phy_port.ports.iter() {
                if log_port.port == port_id {
                    found = true;
                    match log_port.hif.as_ref() {
                        Some(hif) => {
                            let hif_oper_state = bool::from(port_state);
                            match hif.intf.set_oper_status(hif_oper_state) {
                                Ok(_) => log::info!("processor: set host interface {} operational status to {} for port {}", hif.intf, hif_oper_state, port_id),
                                Err(e) => log::error!("processor: failed to set host interface {} operational status to {} for port {}: {:?}", hif, hif_oper_state, port_id, e),
                            }
                        }
                        None => log::warn!(
                            "processor: port {} has no associated host interface created yet",
                            port_id
                        ),
                    }
                }
            }
        }
        if !found {
            log::warn!(
                "processor: port {} not found during port state change event",
                port_id
            );
        }
    }
}

impl<'a> Drop for Processor<'a> {
    fn drop(&mut self) {
        // TODO: the `clone()`s here are ugly, but there is no real good other solution (that I know of)
        log::info!("Shutting down ONIE SAI processor...");

        // removing CPU host interface
        let cpu_hostif_id = self.cpu_hostif.to_id();
        match self.cpu_hostif.clone().remove() {
            Ok(_) => log::info!("processor: removed CPU host interface {}", cpu_hostif_id),
            Err(e) => log::error!(
                "processor: failed to remove CPU host interface {}: {:?}",
                cpu_hostif_id,
                e
            ),
        };

        // removing host interfaces for all ports
        for phy_port in self.ports.clone() {
            for port in phy_port.ports {
                let port_id = port.port.to_id();
                if let Some(hif) = port.hif {
                    let hif_id = hif.intf.to_id();
                    match hif.intf.remove() {
                        Ok(_) => log::info!(
                            "processor: removed host interface {} for port {}",
                            hif_id,
                            port_id
                        ),
                        Err(e) => log::error!(
                            "processor: failed to remove host interface {} for port {}: {:?}",
                            hif_id,
                            port_id,
                            e
                        ),
                    }
                }
            }
        }
    }
}

// let _myip_route_entry = match default_virtual_router.create_route_entry(
//     IpNet::from_str("10.10.10.1/32").unwrap(),
//     vec![
//         RouteEntryAttribute::PacketAction(PacketAction::Forward),
//         RouteEntryAttribute::NextHopID(cpu_port_id.into()),
//     ],
// ) {
//     Ok(v) => v,
//     Err(e) => {
//         log::error!(
//             "failed to create route entry for ourselves for virtual router {}: {:?}",
//             default_virtual_router,
//             e
//         );
//         return ExitCode::FAILURE;
//     }
// };

// let mut hostifs: Vec<HostIf> = Vec::with_capacity(ports.len());
// for (i, port) in ports.into_iter().enumerate() {
//     let port_id = port.to_id();
//     // create host interface
//     let hostif = match switch.create_hostif(vec![
//         HostIfAttribute::Type(HostIfType::Netdev),
//         HostIfAttribute::ObjectID(port_id.into()),
//         HostIfAttribute::Name(format!("Ethernet{}", i)),
//     ]) {
//         Ok(v) => v,
//         Err(e) => {
//             log::error!(
//                 "failed to create host interface for port {} on switch {}: {:?}",
//                 port,
//                 switch,
//                 e
//             );
//             return ExitCode::FAILURE;
//         }
//     };
//     hostifs.push(hostif.clone());

//     // check supported speeds, and set port to 10G if possible
//     match port.get_supported_speeds() {
//         Err(e) => {
//             log::error!(
//                 "failed to query port {} for supported speeds: {:?}",
//                 port,
//                 e
//             );
//         }
//         Ok(speeds) => {
//             if !speeds.contains(&10000) {
//                 log::warn!("port {} does not support 10G, only {:?}", port, speeds)
//             } else {
//                 match port.set_speed(10000) {
//                     Ok(_) => {
//                         log::info!("set port speed to 10G for port {}", port);
//                     }
//                     Err(e) => {
//                         log::error!(
//                             "failed to set port speed to 10G for port {}: {:?}",
//                             port,
//                             e
//                         );
//                     }
//                 }
//             }
//         }
//     }

//     // set port up
//     match port.set_admin_state(true) {
//         Ok(_) => {
//             log::info!("set admin state to true for port {}", port);
//         }
//         Err(e) => {
//             log::error!(
//                 "failed to set admin state to true for port {}: {:?}",
//                 port,
//                 e
//             );
//         }
//     }

//     // allow vlan tags on host interfaces
//     match hostif.set_vlan_tag(VlanTag::Original) {
//         Ok(_) => {
//             log::info!(
//                 "set vlan tag to keep original for host interface {}",
//                 hostif
//             );
//         }
//         Err(e) => {
//             log::error!(
//                 "failed to set vlan tag to keep original for host interface {}: {:?}",
//                 hostif,
//                 e
//             );
//         }
//     }

//     // bring host interface up
//     match hostif.set_oper_status(true) {
//         Ok(_) => {
//             log::info!("set oper status to true for host interface {}", hostif);
//         }
//         Err(e) => {
//             log::error!(
//                 "failed to set oper status to true for host interface {}: {:?}",
//                 hostif,
//                 e
//             );
//         }
//     }

//     // create router interface
//     match default_virtual_router.create_router_interface(vec![
//         RouterInterfaceAttribute::SrcMacAddress(mac_address),
//         RouterInterfaceAttribute::Type(RouterInterfaceType::Port),
//         RouterInterfaceAttribute::PortID(port.into()),
//         RouterInterfaceAttribute::MTU(9100),
//         RouterInterfaceAttribute::NATZoneID(0),
//     ]) {
//         Ok(v) => {
//             log::info!("successfully created router interface {}", v);
//         }
//         Err(e) => {
//             log::error!("failed create router interface: {:?}", e);
//         }
//     }
// }

// match switch.enable_shell() {
//     Ok(_) => {}
//     Err(e) => {
//         log::error!("failed to enter switch shell: {:?}", e);
//     }
// }

// shutdown: remove things again
// hostifs.push(cpu_intf);
// for hostif in hostifs {
//     let id = hostif.to_id();
//     match hostif.remove() {
//         Ok(_) => {
//             log::info!("successfully removed host interface {}", id);
//         }
//         Err(e) => {
//             log::error!("failed to remove host interface {}: {:?}", id, e);
//         }
//     }
// }

// match switch.remove() {
//     Ok(_) => {
//         log::info!("successfully removed switch");
//     }
//     Err(e) => {
//         log::error!("failed to remove switch: {:?}", e);
//     }
// }
