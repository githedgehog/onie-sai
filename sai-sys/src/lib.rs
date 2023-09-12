#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ptr::null_mut;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl Default for sai_apis_t {
    fn default() -> Self {
        Self {
            switch_api: null_mut(),
            port_api: null_mut(),
            fdb_api: null_mut(),
            vlan_api: null_mut(),
            virtual_router_api: null_mut(),
            route_api: null_mut(),
            next_hop_api: null_mut(),
            next_hop_group_api: null_mut(),
            router_interface_api: null_mut(),
            neighbor_api: null_mut(),
            acl_api: null_mut(),
            hostif_api: null_mut(),
            mirror_api: null_mut(),
            samplepacket_api: null_mut(),
            stp_api: null_mut(),
            lag_api: null_mut(),
            policer_api: null_mut(),
            wred_api: null_mut(),
            qos_map_api: null_mut(),
            queue_api: null_mut(),
            scheduler_api: null_mut(),
            scheduler_group_api: null_mut(),
            buffer_api: null_mut(),
            hash_api: null_mut(),
            udf_api: null_mut(),
            tunnel_api: null_mut(),
            l2mc_api: null_mut(),
            ipmc_api: null_mut(),
            rpf_group_api: null_mut(),
            l2mc_group_api: null_mut(),
            ipmc_group_api: null_mut(),
            mcast_fdb_api: null_mut(),
            bridge_api: null_mut(),
            tam_api: null_mut(),
            srv6_api: null_mut(),
            mpls_api: null_mut(),
            dtel_api: null_mut(),
            bfd_api: null_mut(),
            isolation_group_api: null_mut(),
            nat_api: null_mut(),
            counter_api: null_mut(),
            debug_counter_api: null_mut(),
            macsec_api: null_mut(),
            system_port_api: null_mut(),
            my_mac_api: null_mut(),
            ipsec_api: null_mut(),
            bmtor_api: null_mut(),
        }
    }
}

#[cfg(test)]
mod tests {
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
        unsafe {
            // initialize API
            let smt = sai_service_method_table_t {
                profile_get_next_value: Some(profile_get_next_value),
                profile_get_value: Some(profile_get_value),
            };
            let st = sai_api_initialize(0, &smt);
            assert_eq!(st, SAI_STATUS_SUCCESS as i32);

            // query available functionality
            let mut apis: sai_apis_t = Default::default();
            let _st = sai_metadata_apis_query(Some(sai_api_query), &mut apis);

            // it's expected that some of the api queries will actually fail
            // so we cannot rely on a success result here
            // assert_eq!(st, SAI_STATUS_SUCCESS as i32);

            // however, there are some APIs which must be there
            // in fact we should simply check for all of them which we care for
            assert_ne!(apis.switch_api, std::ptr::null_mut());
            assert_ne!(apis.vlan_api, std::ptr::null_mut());
            assert_ne!(apis.bridge_api, std::ptr::null_mut());
            assert_ne!(apis.port_api, std::ptr::null_mut());
            assert_ne!(apis.hostif_api, std::ptr::null_mut());
            assert_ne!(apis.router_interface_api, std::ptr::null_mut());
            assert_ne!(apis.route_api, std::ptr::null_mut());

            // teardown API again
            let st = sai_api_uninitialize();
            assert_eq!(st, SAI_STATUS_SUCCESS as i32);
        }
    }
}
