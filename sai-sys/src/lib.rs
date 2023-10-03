#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use ipnet::IpNet;
use ipnet::Ipv4Net;
use ipnet::Ipv6Net;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl From<IpNet> for sai_ip_prefix_t {
    fn from(value: IpNet) -> Self {
        match value {
            IpNet::V4(v) => sai_ip_prefix_t {
                addr_family: _sai_ip_addr_family_t_SAI_IP_ADDR_FAMILY_IPV4,
                addr: sai_ip_addr_t {
                    ip4: u32::from(v.addr()).to_be(),
                },
                mask: sai_ip_addr_t {
                    ip4: u32::from(v.netmask()).to_be(),
                },
            },
            IpNet::V6(v) => sai_ip_prefix_t {
                addr_family: _sai_ip_addr_family_t_SAI_IP_ADDR_FAMILY_IPV6,
                addr: sai_ip_addr_t {
                    // TODO: this might need .to_be_bytes() as well
                    ip6: v.addr().octets(),
                },
                mask: sai_ip_addr_t {
                    ip6: v.netmask().octets(),
                },
            },
        }
    }
}

impl From<sai_ip_prefix_t> for IpNet {
    fn from(value: sai_ip_prefix_t) -> Self {
        match value.addr_family {
            _sai_ip_addr_family_t_SAI_IP_ADDR_FAMILY_IPV4 => {
                let addr: Ipv4Addr = From::from(unsafe { value.addr.ip4 });
                let mask: Ipv4Addr = From::from(unsafe { value.mask.ip4 });
                let mask_prefix = match ipnet::ipv4_mask_to_prefix(mask) {
                    Ok(v) => v,
                    Err(e) => {
                        panic!("the IPv4 mask within sai_ip_prefix_t produces an invalid prefix length error: {:?}", e);
                    }
                };
                // the unwrap is safe as the ipv4_mask_to_prefix is already performing the same check
                IpNet::V4(Ipv4Net::new(addr, mask_prefix).unwrap())
            }
            _sai_ip_addr_family_t_SAI_IP_ADDR_FAMILY_IPV6 => {
                let addr: Ipv6Addr = From::from(unsafe { value.addr.ip6 });
                let mask: Ipv6Addr = From::from(unsafe { value.mask.ip6 });
                let mask_prefix = match ipnet::ipv6_mask_to_prefix(mask) {
                    Ok(v) => v,
                    Err(e) => {
                        panic!("the IPv6 mask within sai_ip_prefix_t produces an invalid prefix lenght error: {:?}", e);
                    }
                };
                IpNet::V6(Ipv6Net::new(addr, mask_prefix).unwrap())
            }
            unknown_addr_family => {
                panic!(
                    "unknown addr_family within sai_ip_prefix_t: {}",
                    unknown_addr_family
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::MaybeUninit;

    use super::*;

    /// profile_get_next_value is used by the SMT (service method table)
    unsafe extern "C" fn profile_get_next_value(
        _profile_id: sai_switch_profile_id_t,
        _variable: *mut *const ::std::os::raw::c_char,
        _value: *mut *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int {
        0
    }

    /// profile_get_value is used by the SMT (service method table)
    unsafe extern "C" fn profile_get_value(
        _profile_id: sai_switch_profile_id_t,
        _variable: *const ::std::os::raw::c_char,
    ) -> *const ::std::os::raw::c_char {
        std::ptr::null()
    }

    #[test]
    fn basic_api_initialization_and_teardown() {
        // query API version
        let mut version: sai_api_version_t = 0;
        let st = unsafe { sai_query_api_version(&mut version as *mut _) };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        // initialize API
        let smt = sai_service_method_table_t {
            profile_get_next_value: Some(profile_get_next_value),
            profile_get_value: Some(profile_get_value),
        };
        let st = unsafe { sai_api_initialize(0, &smt) };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);

        let mut apis_backing = MaybeUninit::<sai_apis_t>::uninit();
        let st = unsafe { sai_metadata_apis_query(Some(sai_api_query), apis_backing.as_mut_ptr()) };
        assert_eq!(st, 3);
        let apis = unsafe { apis_backing.assume_init() };

        // query available functionalities
        // there would be `sai_metadata_apis_query()`, but one must stay away from this function as it is not backwards compatible
        // so we'll query simply for all the APIs that we care about, and stay away from the others that we don't care about
        // switch API
        let mut switch_api_backing = MaybeUninit::<sai_switch_api_t>::uninit();
        let switch_api_ptr_orig = switch_api_backing.as_ptr();
        let mut switch_api_ptr = switch_api_backing.as_mut_ptr();
        assert_eq!(switch_api_ptr_orig, switch_api_ptr);
        let switch_api_ptr_ptr = &mut switch_api_ptr as *mut *mut sai_switch_api_t;
        let st = unsafe { sai_api_query(_sai_api_t_SAI_API_SWITCH, switch_api_ptr_ptr as _) };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        let switch_api = if switch_api_ptr_orig == switch_api_ptr {
            println!("pointer the same");
            unsafe { switch_api_backing.assume_init() }
        } else {
            println!("pointer changed");
            unsafe {
                switch_api_backing.assume_init_drop();
                *switch_api_ptr
            }
        };

        // vlan API
        // let mut vlan_api_backing: sai_vlan_api_t = Default::default();
        // let vlan_api_ptr_orig = &vlan_api_backing as *const _;
        // let mut vlan_api_ptr = &mut vlan_api_backing as *mut _;
        // assert_eq!(vlan_api_ptr_orig, vlan_api_ptr);
        // let vlan_api_ptr_ptr = &mut vlan_api_ptr as *mut *mut _;
        // let st = unsafe { sai_api_query(_sai_api_t_SAI_API_VLAN, vlan_api_ptr_ptr as _) };
        // assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        // let vlan_api: sai_vlan_api_t = if vlan_api_ptr_orig == vlan_api_ptr {
        //     vlan_api_backing
        // } else {
        //     println!("in here for gods sake");
        //     unsafe { *vlan_api_ptr }
        // };
        let mut vlan_api_backing = MaybeUninit::<sai_vlan_api_t>::uninit();
        let vlan_api_ptr_orig = vlan_api_backing.as_ptr();
        let mut vlan_api_ptr = vlan_api_backing.as_mut_ptr();
        assert_eq!(vlan_api_ptr_orig, vlan_api_ptr);
        let vlan_api_ptr_ptr = &mut vlan_api_ptr as *mut *mut sai_vlan_api_t;
        let st = unsafe { sai_api_query(_sai_api_t_SAI_API_VLAN, vlan_api_ptr_ptr as _) };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        let vlan_api = if vlan_api_ptr_orig == vlan_api_ptr {
            println!("pointer the same");
            unsafe { vlan_api_backing.assume_init() }
        } else {
            println!("pointer changed");
            unsafe {
                vlan_api_backing.assume_init_drop();
                *vlan_api_ptr
            }
        };

        // bridge API
        let mut bridge_api_backing = MaybeUninit::<sai_bridge_api_t>::uninit();
        let bridge_api_ptr_orig = bridge_api_backing.as_ptr();
        let mut bridge_api_ptr = bridge_api_backing.as_mut_ptr();
        assert_eq!(bridge_api_ptr_orig, bridge_api_ptr);
        let bridge_api_ptr_ptr = &mut bridge_api_ptr as *mut *mut sai_bridge_api_t;
        let st = unsafe { sai_api_query(_sai_api_t_SAI_API_BRIDGE, bridge_api_ptr_ptr as _) };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        let bridge_api = if bridge_api_ptr_orig == bridge_api_ptr {
            println!("pointer the same");
            unsafe { bridge_api_backing.assume_init() }
        } else {
            println!("pointer changed");
            unsafe {
                bridge_api_backing.assume_init_drop();
                *bridge_api_ptr
            }
        };

        // port API
        let mut port_api_backing = MaybeUninit::<sai_port_api_t>::uninit();
        let port_api_ptr_orig = port_api_backing.as_ptr();
        let mut port_api_ptr = port_api_backing.as_mut_ptr();
        assert_eq!(port_api_ptr_orig, port_api_ptr);
        let port_api_ptr_ptr = &mut port_api_ptr as *mut *mut sai_port_api_t;
        let st = unsafe { sai_api_query(_sai_api_t_SAI_API_PORT, port_api_ptr_ptr as _) };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        let port_api = if port_api_ptr_orig == port_api_ptr {
            println!("pointer the same");
            unsafe { port_api_backing.assume_init() }
        } else {
            println!("pointer changed");
            unsafe {
                port_api_backing.assume_init_drop();
                *port_api_ptr
            }
        };

        // HostIf API
        let mut hostif_api_backing = MaybeUninit::<sai_hostif_api_t>::uninit();
        let hostif_api_ptr_orig = hostif_api_backing.as_ptr();
        let mut hostif_api_ptr = hostif_api_backing.as_mut_ptr();
        assert_eq!(hostif_api_ptr_orig, hostif_api_ptr);
        let hostif_api_ptr_ptr = &mut hostif_api_ptr as *mut *mut sai_hostif_api_t;
        let st = unsafe { sai_api_query(_sai_api_t_SAI_API_HOSTIF, hostif_api_ptr_ptr as _) };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        let hostif_api = if hostif_api_ptr_orig == hostif_api_ptr {
            println!("pointer the same");
            unsafe { hostif_api_backing.assume_init() }
        } else {
            println!("pointer changed");
            unsafe {
                hostif_api_backing.assume_init_drop();
                *hostif_api_ptr
            }
        };

        // router interface API
        let mut router_interface_api_backing = MaybeUninit::<sai_router_interface_api_t>::uninit();
        let router_interface_api_ptr_orig = router_interface_api_backing.as_ptr();
        let mut router_interface_api_ptr = router_interface_api_backing.as_mut_ptr();
        assert_eq!(router_interface_api_ptr_orig, router_interface_api_ptr);
        let router_interface_api_ptr_ptr =
            &mut router_interface_api_ptr as *mut *mut sai_router_interface_api_t;
        let st = unsafe {
            sai_api_query(
                _sai_api_t_SAI_API_ROUTER_INTERFACE,
                router_interface_api_ptr_ptr as _,
            )
        };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        let router_interface_api = if router_interface_api_ptr_orig == router_interface_api_ptr {
            println!("pointer the same");
            unsafe { router_interface_api_backing.assume_init() }
        } else {
            println!("pointer changed");
            unsafe {
                router_interface_api_backing.assume_init_drop();
                *router_interface_api_ptr
            }
        };

        // route API
        let mut route_api_backing = MaybeUninit::<sai_route_api_t>::uninit();
        let route_api_ptr_orig = route_api_backing.as_ptr();
        let mut route_api_ptr = route_api_backing.as_mut_ptr();
        assert_eq!(route_api_ptr_orig, route_api_ptr);
        let route_api_ptr_ptr = &mut route_api_ptr as *mut *mut sai_route_api_t;
        let st = unsafe { sai_api_query(_sai_api_t_SAI_API_ROUTE, route_api_ptr_ptr as _) };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        let route_api = if route_api_ptr_orig == route_api_ptr {
            println!("pointer the same");
            unsafe { route_api_backing.assume_init() }
        } else {
            println!("pointer changed");
            unsafe {
                route_api_backing.assume_init_drop();
                *route_api_ptr
            }
        };

        // simply check that the functions that we want to use are available
        // NOTE: we can't assume that this works in a unit test
        // assert!(switch_api.create_switch.is_some());
        // assert!(vlan_api.get_vlan_attribute.is_some());
        // assert!(vlan_api.remove_vlan_member.is_some());
        // assert!(bridge_api.remove_bridge_port.is_some());
        // assert!(port_api.set_port_attribute.is_some());
        // assert!(port_api.get_port_attribute.is_some());
        // assert!(hostif_api.create_hostif.is_some());
        // assert!(hostif_api.create_hostif_trap.is_some());
        // assert!(hostif_api.create_hostif_table_entry.is_some());
        // assert!(hostif_api.get_hostif_attribute.is_some());
        // assert!(hostif_api.set_hostif_attribute.is_some());
        // assert!(hostif_api.remove_hostif.is_some());
        // assert!(rif_api.create_router_interface.is_some());
        // assert!(route_api.create_route_entry.is_some());
        // assert!(route_api.remove_route_entry.is_some());
        //
        println!("{:?}", switch_api);
        println!("{:?}", vlan_api);
        println!("{:?}", bridge_api);
        println!("{:?}", port_api);
        println!("{:?}", hostif_api);
        println!("{:?}", router_interface_api);
        println!("{:?}", route_api);
        println!("{:?}", apis);

        // teardown API again
        let st = unsafe { sai_api_uninitialize() };
        assert_eq!(st, SAI_STATUS_SUCCESS as i32);
    }
}
