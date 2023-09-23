use std::ffi::CString;
use std::process::ExitCode;
use std::str::FromStr;

use ipnet::IpNet;
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
use sai::port::PortID;
use sai::route::RouteEntryAttribute;
use sai::router_interface::RouterInterfaceAttribute;
use sai::router_interface::RouterInterfaceType;
use sai::switch::SwitchAttribute;
use sai::ObjectID;
use sai::PacketAction;
use sai::SAI;

fn main() -> ExitCode {
    if let Ok(version) = SAI::api_version() {
        println!("INFO: SAI version: {}", version);
    }

    // our profile
    let profile = vec![(
        CString::from_vec_with_nul(sai::SAI_KEY_INIT_CONFIG_FILE.to_vec()).unwrap(),
        CString::new("/root/saictl/etc/config.bcm").unwrap(),
    )];

    // init SAI
    let sai_api = match SAI::new(profile) {
        Ok(sai) => sai,
        Err(e) => {
            println!("ERROR: failed to initialize SAI: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    println!("INFO: successfully initialized SAI");

    // if let Err(e) = SAI::log_set_all(sai::LogLevel::Info) {
    //     println!("ERROR: failed to set log level for all APIs: {:?}", e);
    // }

    // now create switch
    let mac_address = [0x1c, 0x72, 0x1d, 0xec, 0x44, 0xa0];
    let switch = match sai_api.switch_create(vec![
        SwitchAttribute::InitSwitch(true),
        SwitchAttribute::SrcMacAddress(mac_address),
    ]) {
        Ok(sw_id) => sw_id,
        Err(e) => {
            println!("ERROR: failed to create switch: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    println!("INFO: successfully created switch: {:?}", switch);

    // port state change callback
    if let Err(e) = switch.set_port_state_change_callback(Box::new(|v| {
        v.iter()
            .for_each(|n| println!("INFO: Port State Change Event: {:?}", n));
    })) {
        println!("ERROR: failed to set port state change callback: {:?}", e);
    } else {
        println!("INFO: successfully set port state change callback");
    }

    // remove default vlan members
    let default_vlan = match switch.get_default_vlan() {
        Ok(vlan) => vlan,
        Err(e) => {
            println!("ERROR: failed to get default VLAN: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    println!(
        "INFO: default VLAN of switch {} is: {:?}",
        switch, default_vlan
    );
    match default_vlan.get_members() {
        Err(e) => {
            println!(
                "ERROR: failed to get VLAN members for default VLAN {}: {:?}",
                default_vlan, e
            );
        }
        Ok(members) => {
            for member in members {
                println!("INFO: Removing VLAN member {}...", member);
                if let Err(e) = member.remove() {
                    println!("ERROR: failed to remove VLAN member: {:?}", e);
                }
            }
        }
    };

    // remove default bridge ports
    let default_bridge = match switch.get_default_bridge() {
        Ok(bridge) => bridge,
        Err(e) => {
            println!("ERROR: failed to get dfeault bridge: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    println!(
        "INFO default bridge of switch {} is: {:?}",
        switch, default_bridge
    );
    match default_bridge.get_ports() {
        Err(e) => {
            println!(
                "ERROR: failed to get bridge ports for default bridge {}: {:?}",
                default_bridge, e
            );
        }
        Ok(ports) => {
            for bridge_port in ports {
                match bridge_port.get_type() {
                    // we only go ahead when this is a real port
                    Ok(bridge::port::Type::Port) => {}
                    Ok(v) => {
                        println!(
                            "INFO: not removing bridge port {} of type: {:?}",
                            bridge_port, v
                        );
                        continue;
                    }
                    Err(e) => {
                        println!(
                            "ERROR: failed to get bridge porty type of bridge port {}: {:?}",
                            bridge_port, e
                        );
                        continue;
                    }
                }

                println!("INFO: removing bridge port {}...", bridge_port);
                if let Err(e) = bridge_port.remove() {
                    println!("ERROR: failed to remove bridge port: {:?}", e);
                }
            }
        }
    };

    // program traps
    let default_trap_group = match switch.get_default_hostif_trap_group() {
        Ok(group) => group,
        Err(e) => {
            println!(
                "ERROR: failed to get default host interface trap group: {:?}",
                e
            );
            return ExitCode::FAILURE;
        }
    };
    let default_trap_group_id = default_trap_group.to_id();
    let _ip2me_trap = match switch.create_hostif_trap(vec![
        TrapAttribute::TrapType(TrapType::IP2ME),
        TrapAttribute::PacketAction(PacketAction::Trap),
        TrapAttribute::TrapGroup(default_trap_group_id),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: failed to create ip2me trap: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    let _arp_req_trap = match switch.create_hostif_trap(vec![
        TrapAttribute::TrapType(TrapType::ARPRequest),
        TrapAttribute::PacketAction(PacketAction::Copy),
        TrapAttribute::TrapGroup(default_trap_group_id),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: failed to create ip2me trap: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    let _arp_resp_trap = match switch.create_hostif_trap(vec![
        TrapAttribute::TrapType(TrapType::ARPResponse),
        TrapAttribute::PacketAction(PacketAction::Copy),
        TrapAttribute::TrapGroup(default_trap_group_id),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: failed to create ip2me trap: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    let _default_table_entry = match switch.create_hostif_table_entry(vec![
        TableEntryAttribute::Type(TableEntryType::Wildcard),
        TableEntryAttribute::ChannelType(ChannelType::NetdevPhysicalPort),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!(
                "ERROR: failed to create default host interface table entry: {:?}",
                e
            );
            return ExitCode::FAILURE;
        }
    };

    // get CPU port
    let cpu_port = match switch.get_cpu_port() {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: failed go get CPU port: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    let cpu_port_id = PortID::from(cpu_port);

    // create host interface for it
    let cpu_intf = match switch.create_hostif(vec![
        HostIfAttribute::Name("CPU".to_string()),
        HostIfAttribute::Type(HostIfType::Netdev),
        HostIfAttribute::ObjectID(cpu_port_id.into()),
        HostIfAttribute::OperStatus(true),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!(
                "ERROR: failed to create host interface for CPU port {}: {:?}",
                cpu_port_id, e
            );
            return ExitCode::FAILURE;
        }
    };

    // get the default virtual router
    let default_virtual_router = match switch.get_default_virtual_router() {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: failed to get default virtual router: {:?}", e);
            return ExitCode::FAILURE;
        }
    };

    // prep the router: create loopback interface
    // create initial routes
    let _lo_rif = match default_virtual_router.create_router_interface(vec![
        RouterInterfaceAttribute::Type(RouterInterfaceType::Loopback),
        RouterInterfaceAttribute::MTU(9100),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!(
                "ERROR: failed to get create loopback interface for virtual router {}: {:?}",
                default_virtual_router, e
            );
            return ExitCode::FAILURE;
        }
    };
    let _default_route_entry = match default_virtual_router.create_route_entry(
        IpNet::from_str("0.0.0.0/0").unwrap(),
        vec![RouteEntryAttribute::PacketAction(PacketAction::Drop)],
    ) {
        Ok(v) => v,
        Err(e) => {
            println!(
                "ERROR: failed to create default route entry for virtual router {}: {:?}",
                default_virtual_router, e
            );
            return ExitCode::FAILURE;
        }
    };
    let _myip_route_entry = match default_virtual_router.create_route_entry(
        IpNet::from_str("10.10.10.1/32").unwrap(),
        vec![
            RouteEntryAttribute::PacketAction(PacketAction::Forward),
            RouteEntryAttribute::NextHopID(cpu_port_id.into()),
        ],
    ) {
        Ok(v) => v,
        Err(e) => {
            println!(
                "ERROR: failed to create route entry for ourselves for virtual router {}: {:?}",
                default_virtual_router, e
            );
            return ExitCode::FAILURE;
        }
    };

    // get ports now
    let ports = match switch.get_ports() {
        Ok(v) => v,
        Err(e) => {
            println!(
                "ERROR: failed to get port list from switch {}: {:?}",
                switch, e
            );
            return ExitCode::FAILURE;
        }
    };

    let mut hostifs: Vec<HostIf> = Vec::with_capacity(ports.len());
    for (i, port) in ports.into_iter().enumerate() {
        let port_id = port.to_id();
        // create host interface
        let hostif = match switch.create_hostif(vec![
            HostIfAttribute::Type(HostIfType::Netdev),
            HostIfAttribute::ObjectID(port_id.into()),
            HostIfAttribute::Name(format!("Ethernet{}", i)),
        ]) {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "ERROR: failed to create host interface for port {} on switch {}: {:?}",
                    port, switch, e
                );
                return ExitCode::FAILURE;
            }
        };
        hostifs.push(hostif.clone());

        // check supported speeds, and set port to 10G if possible
        match port.get_supported_speeds() {
            Err(e) => {
                println!(
                    "ERROR: failed to query port {} for supported speeds: {:?}",
                    port, e
                );
            }
            Ok(speeds) => {
                if !speeds.contains(&10000) {
                    println!(
                        "WARN: port {} does not support 10G, only {:?}",
                        port, speeds
                    )
                } else {
                    match port.set_speed(10000) {
                        Ok(_) => {
                            println!("INFO: set port speed to 10G for port {}", port);
                        }
                        Err(e) => {
                            println!(
                                "ERROR: failed to set port speed to 10G for port {}: {:?}",
                                port, e
                            );
                        }
                    }
                }
            }
        }

        // set port up
        match port.set_admin_state(true) {
            Ok(_) => {
                println!("INFO: set admin state to true for port {}", port);
            }
            Err(e) => {
                println!(
                    "ERROR: failed to set admin state to true for port {}: {:?}",
                    port, e
                );
            }
        }

        // allow vlan tags on host interfaces
        match hostif.set_vlan_tag(VlanTag::Original) {
            Ok(_) => {
                println!(
                    "INFO: set vlan tag to keep original for host interface {}",
                    hostif
                );
            }
            Err(e) => {
                println!(
                    "ERROR: failed to set vlan tag to keep original for host interface {}: {:?}",
                    hostif, e
                );
            }
        }

        // bring host interface up
        match hostif.set_oper_status(true) {
            Ok(_) => {
                println!(
                    "INFO: set oper status to true for host interface {}",
                    hostif
                );
            }
            Err(e) => {
                println!(
                    "ERROR: failed to set oper status to true for host interface {}: {:?}",
                    hostif, e
                );
            }
        }

        // create router interface
        match default_virtual_router.create_router_interface(vec![
            RouterInterfaceAttribute::SrcMacAddress(mac_address),
            RouterInterfaceAttribute::Type(RouterInterfaceType::Port),
            RouterInterfaceAttribute::PortID(port.into()),
            RouterInterfaceAttribute::MTU(9100),
            RouterInterfaceAttribute::NATZoneID(0),
        ]) {
            Ok(v) => {
                println!("INFO: successfully created router interface {}", v);
            }
            Err(e) => {
                println!("ERROR: failed create router interface: {:?}", e);
            }
        }
    }

    match switch.enable_shell() {
        Ok(_) => {}
        Err(e) => {
            println!("ERROR: failed to enter switch shell: {:?}", e);
        }
    }

    // shutdown: remove things again
    hostifs.push(cpu_intf);
    for hostif in hostifs {
        let id = hostif.to_id();
        match hostif.remove() {
            Ok(_) => {
                println!("INFO: successfully removed host interface {}", id);
            }
            Err(e) => {
                println!("ERROR: failed to remove host interface {}: {:?}", id, e);
            }
        }
    }

    match switch.remove() {
        Ok(_) => {
            println!("INFO: successfully removed switch");
        }
        Err(e) => {
            println!("ERROR: failed to remove switch: {:?}", e);
        }
    }

    println!("INFO: Success");

    return ExitCode::SUCCESS;
}
