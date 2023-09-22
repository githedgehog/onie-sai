use std::ffi::CString;
use std::process::ExitCode;

use sai::BridgePortType;
use sai::HostIfAttribute;
use sai::HostIfTableEntryAttribute;
use sai::HostIfTableEntryChannelType;
use sai::HostIfTableEntryType;
use sai::HostIfTrapAttribute;
use sai::HostIfTrapType;
use sai::HostIfType;
use sai::PacketAction;
use sai::RouterInterfaceAttribute;
use sai::RouterInterfaceType;
use sai::SwitchAttribute;
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
    let switch = match sai_api.switch_create(vec![
        SwitchAttribute::InitSwitch(true),
        SwitchAttribute::SrcMacAddress([0x1c, 0x72, 0x1d, 0xec, 0x44, 0xa0]),
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
                    Ok(BridgePortType::Port) => {}
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
    let _ip2me_trap = match switch.create_hostif_trap(vec![
        HostIfTrapAttribute::TrapType(HostIfTrapType::IP2ME),
        HostIfTrapAttribute::PacketAction(PacketAction::Trap),
        HostIfTrapAttribute::TrapGroup(default_trap_group.into()),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: failed to create ip2me trap: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    let _arp_req_trap = match switch.create_hostif_trap(vec![
        HostIfTrapAttribute::TrapType(HostIfTrapType::ARPRequest),
        HostIfTrapAttribute::PacketAction(PacketAction::Copy),
        HostIfTrapAttribute::TrapGroup(default_trap_group.into()),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: failed to create ip2me trap: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    let _arp_resp_trap = match switch.create_hostif_trap(vec![
        HostIfTrapAttribute::TrapType(HostIfTrapType::ARPResponse),
        HostIfTrapAttribute::PacketAction(PacketAction::Copy),
        HostIfTrapAttribute::TrapGroup(default_trap_group.into()),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: failed to create ip2me trap: {:?}", e);
            return ExitCode::FAILURE;
        }
    };
    let _default_table_entry = match switch.create_hostif_table_entry(vec![
        HostIfTableEntryAttribute::Type(HostIfTableEntryType::Wildcard),
        HostIfTableEntryAttribute::ChannelType(HostIfTableEntryChannelType::NetdevPhysicalPort),
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

    // create host interface for it
    let _cpu_intf = match switch.create_hostif(vec![
        HostIfAttribute::Name("CPU".to_string()),
        HostIfAttribute::Type(HostIfType::Netdev),
        HostIfAttribute::ObjectID(cpu_port.into()),
        HostIfAttribute::OperStatus(true),
    ]) {
        Ok(v) => v,
        Err(e) => {
            println!(
                "ERROR: failed to create host interface for CPU port {}: {:?}",
                cpu_port, e
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
    let _lo_rif = match switch.create_router_interface(vec![
        RouterInterfaceAttribute::Type(RouterInterfaceType::Loopback),
        RouterInterfaceAttribute::VirtualRouterID(default_virtual_router.into()),
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

    println!("INFO: Success");

    return ExitCode::SUCCESS;
}
