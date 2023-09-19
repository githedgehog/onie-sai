use std::ffi::CString;
use std::process::ExitCode;

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

    if let Err(e) = SAI::log_set_all(sai::LogLevel::Info) {
        println!("ERROR: failed to set log level for all APIs: {:?}", e);
    }

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
    let default_vlan_members = match default_vlan.get_members() {
        Ok(members) => members,
        Err(e) => {
            println!(
                "ERROR: failed to get VLAN members for default VLAN {}: {:?}",
                default_vlan, e
            );
            return ExitCode::FAILURE;
        }
    };
    for member in default_vlan_members {
        println!("INFO Removing VLAN member {}...", member);
        if let Err(e) = member.remove() {
            println!("ERROR: failed to remove VLAN member: {:?}", e);
        }
    }

    return ExitCode::SUCCESS;
}
