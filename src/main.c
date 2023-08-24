#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include <signal.h>
#include <sys/types.h>
#include <sai.h>
#include <saimetadata.h>
#include <s5212.h>

static volatile sig_atomic_t stop;

static const char* profile_get_value(_In_ sai_switch_profile_id_t profile_id, _In_ const char *variable);
static int profile_get_next_value(_In_ sai_switch_profile_id_t profile_id, _Out_ const char **variable, _Out_ const char **value);
static int register_callbacks(sai_apis_t *apis, sai_object_id_t sw_id);
static int add_host_intfs(sai_apis_t *apis, sai_object_id_t sw_id);
static void dump_startup_data(int rec, sai_apis_t *apis, sai_object_id_t id);

static void switch_state_change_cb(_In_ sai_object_id_t switch_id, _In_ sai_switch_oper_status_t switch_oper_status);
static void switch_shutdown_request_cb(_In_ sai_object_id_t switch_id);
static void fdb_event_cb(_In_ uint32_t count, _In_ const sai_fdb_event_notification_data_t *data);
static void nat_event_cb(_In_ uint32_t count, _In_ const sai_nat_event_notification_data_t *data);
static void port_state_change_cb(_In_ uint32_t count, _In_ const sai_port_oper_status_notification_t *data);
static void queue_pfc_deadlock_cb(_In_ uint32_t count, _In_ const sai_queue_deadlock_notification_data_t *data);
static void bfd_session_state_change_cb(_In_ uint32_t count, _In_ const sai_bfd_session_state_notification_t *data);

static const sai_service_method_table_t smt = {
    .profile_get_value = profile_get_value,
    .profile_get_next_value = profile_get_next_value,
};

static void main_signal_handler(int signum) {
    stop = 1;
}

int main(int argc, char *argv[]) {
    sai_status_t st;

    sai_api_version_t version;

    st = sai_query_api_version(&version);
    if (st != SAI_STATUS_SUCCESS) {
        printf("saictl: sai_query_api_version error: 0x%x\n", st);
        return EXIT_FAILURE;
    }
    printf("saictl: SAI Version: 0x%lx\n", version);

    st = sai_api_initialize(0, &smt);
    if (st != SAI_STATUS_SUCCESS) {
        printf("saictl: sai_api_initialize error: 0x%x\n", st);
        return EXIT_FAILURE;
    }
    printf("saictl: sai_api_initialize success\n");

    sai_apis_t apis;
    memset(&apis, 0, sizeof(sai_apis_t));

    st = sai_metadata_apis_query(sai_api_query, &apis);
    if (st != SAI_STATUS_SUCCESS) {
        printf("saictl: sai_metadata_apis_query error: 0x%x\n", st);
        // return EXIT_FAILURE;
    }
    for (int i=1; i < SAI_API_MAX; i++) {
        st = sai_log_set((sai_api_t)i, SAI_LOG_LEVEL_INFO);
        if (st != SAI_STATUS_SUCCESS) {
            printf("saictl: sai_log_set(0x%x) error: 0x%x\n", i, st);
        }
    }
    // st = sai_log_set(SAI_API_SWITCH, SAI_LOG_LEVEL_DEBUG);
    // if (st != SAI_STATUS_SUCCESS) {
    //     printf("saictl: sai_log_set debug (0x%x) error: 0x%x\n", SAI_API_SWITCH, st);
    // }
    // st = sai_log_set(SAI_API_PORT, SAI_LOG_LEVEL_DEBUG);
    // if (st != SAI_STATUS_SUCCESS) {
    //     printf("saictl: sai_log_set debug (0x%x) error: 0x%x\n", SAI_API_PORT, st);
    // }

    // create switch
    // 1c:72:1d:ec:44:a0
    sai_mac_t default_mac_addr = {0x1c, 0x72, 0x1d, 0xec, 0x44, 0xa0};
    sai_object_id_t sw_id;
    sai_attribute_t sw_create_attr[2];
    sw_create_attr[0].id = SAI_SWITCH_ATTR_INIT_SWITCH;
    sw_create_attr[0].value.booldata = true;
    sw_create_attr[1].id = SAI_SWITCH_ATTR_SRC_MAC_ADDRESS;
    sw_create_attr[1].value.mac[0] = default_mac_addr[0];
    sw_create_attr[1].value.mac[1] = default_mac_addr[1];
    sw_create_attr[1].value.mac[2] = default_mac_addr[2];
    sw_create_attr[1].value.mac[3] = default_mac_addr[3];
    sw_create_attr[1].value.mac[4] = default_mac_addr[4];
    sw_create_attr[1].value.mac[5] = default_mac_addr[5];

    printf("saictl: creating switch...\n");
    st = apis.switch_api->create_switch(&sw_id, 2, sw_create_attr);
    if (st != SAI_STATUS_SUCCESS) {
        char str[128];
        sai_serialize_status(str, st);
        printf("saictl: create_switch error: 0x%x %s\n", st, str);
        return EXIT_FAILURE;
    }
    printf("saictl: create_switch success\n");

    //////// start dump some data
    // printf("saictl: STARTING DUMP DATA\n");
    // dump_startup_data(0, &apis, sw_id);
    // printf("saictl: END DUMP DATA\n");
    //////// end dump some data

    //////// register callbacks
    int ret = register_callbacks(&apis, sw_id);
    if (ret < 0) {
        printf("saictl: registering callbacks failed\n");
    }

    //////// start creating stuff
    ret = add_host_intfs(&apis, sw_id);
    if (ret < 0) {
        printf("saictl: creating stuff failed: %d\n", ret);
        stop = 1;
    }
    //////// end creating stuff

    // wait for signal before we shut down
    signal(SIGINT, main_signal_handler);
    signal(SIGTERM, main_signal_handler);
    printf("saictl: waiting on SIGINT or SIGTERM\n");
    while (!stop)
        pause();
    
    printf("saictl: shutting down...\n");

    // remove switch

    st = apis.switch_api->remove_switch(sw_id);
    if (st != SAI_STATUS_SUCCESS) {
        printf("saictl: remove_switch error: 0x%x\n", st);
    } else {
        printf("saictl: remove_switch success\n");
    }

    st = sai_api_uninitialize();
    if (st != SAI_STATUS_SUCCESS) {
        printf("saictl: sai_api_uninitialize error: 0x%x\n", st);
        return EXIT_FAILURE;
    }
    printf("saictl: sai_api_uninitialize success\n");

    return EXIT_SUCCESS;
}

static const char* profile_get_value(_In_ sai_switch_profile_id_t profile_id, _In_ const char *variable) {
    if (variable == NULL) {
        return NULL;
    }
    printf("saictl: profile_get_value 0x%x %s\n", profile_id, variable);

    for (size_t i=0; i<s5212_profile_length; i++) {
        if (strcmp(s5212_profile[i].k, variable) == 0) {
            return s5212_profile[i].v;
        }
    }

    return NULL;
}

static size_t profile_iter = 0;

static int profile_get_next_value(_In_ sai_switch_profile_id_t profile_id, _Out_ const char **variable, _Out_ const char **value) {
    // printf("saictl: profile_get_next_value 0x%x\n", profile_id);
    if (value == NULL) {
        printf("saictl: resetting profile map iterator\n");
        profile_iter = 0;
        return 0;
    }

    if (variable == NULL) {
        printf("saictl: variable is null\n");
        return -1;
    }

    if (profile_iter == s5212_profile_length) {
        printf("saictl: iterator reached end\n");
        return -1;
    }

    printf("saictl: profile_get_next_value: %s=%s\n", s5212_profile[profile_iter].k, s5212_profile[profile_iter].v);
    *variable = s5212_profile[profile_iter].k;
    *value = s5212_profile[profile_iter].v;

    profile_iter++;

    return 0;
}

#define DEFAULT_HOSTIF_TX_QUEUE 7

static int register_callbacks(sai_apis_t *apis, sai_object_id_t sw_id) {
    sai_attribute_t attrs[] = {
        {
            .id = SAI_SWITCH_ATTR_SWITCH_STATE_CHANGE_NOTIFY, 
            .value.ptr = switch_state_change_cb,
        },
        {
            .id = SAI_SWITCH_ATTR_SHUTDOWN_REQUEST_NOTIFY, 
            .value.ptr = switch_shutdown_request_cb,
        },
        {
            .id = SAI_SWITCH_ATTR_FDB_EVENT_NOTIFY, 
            .value.ptr = fdb_event_cb,
        },
        {
            .id = SAI_SWITCH_ATTR_NAT_EVENT_NOTIFY, 
            .value.ptr = nat_event_cb,
        },
        {
            .id = SAI_SWITCH_ATTR_PORT_STATE_CHANGE_NOTIFY, 
            .value.ptr = port_state_change_cb,
        },
        {
            .id = SAI_SWITCH_ATTR_QUEUE_PFC_DEADLOCK_NOTIFY, 
            .value.ptr = queue_pfc_deadlock_cb,
        },
        {
            .id = SAI_SWITCH_ATTR_BFD_SESSION_STATE_CHANGE_NOTIFY,
            .value.ptr = bfd_session_state_change_cb,
        }
    };

    int ret = 1;
    sai_status_t st;
    for (int i = 0; i < 7; i++) {
        sai_attribute_t attr = attrs[i];
        st = apis->switch_api->set_switch_attribute(sw_id, &attr);
        if (st != SAI_STATUS_SUCCESS) {
            char err[64];
            sai_serialize_status(err, st);
            printf("saictl: failed to set callback[%d]: %s\n", i, err);
            ret = -1;
        }
    }
    return ret;
}

static int add_host_intfs(sai_apis_t *apis, sai_object_id_t sw_id) {
    sai_status_t st;
    char err[128];

    // check if this switch supports queues
    // sai_attr_capability_t queue_cap;
    bool has_queues = true;
    // st = sai_query_attribute_capability(sw_id, SAI_OBJECT_TYPE_HOSTIF, SAI_HOSTIF_ATTR_QUEUE, &queue_cap);
    // if (st != SAI_STATUS_SUCCESS) {
    //     sai_serialize_status(err, st);
    //     printf("saictl: failed to query queue capability of switch: %s\n", err);
    // } else {
    //     // ths is how SONiC checks the capabilities, always through the set capability
    //     has_queues = queue_cap.set_implemented;
    // }

    // get the port list from the switch
    sai_object_id_t port_list[128];
    sai_attribute_t attr_port_list;
    attr_port_list.id = SAI_SWITCH_ATTR_PORT_LIST;
    attr_port_list.value.objlist.count = 128;
    attr_port_list.value.objlist.list = port_list;

    st = apis->switch_api->get_switch_attribute(sw_id, 1, &attr_port_list);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to get port list from switch: %s\n", err);
        return -1;
    }

    // now iterate over the ports
    for (int i = 0; i < attr_port_list.value.objlist.count; i++) {
        // get the port ID from the list
        sai_object_id_t port_id = attr_port_list.value.objlist.list[i];

        // prep an interface name
        char ifname[SAI_HOSTIF_NAME_SIZE] = {0};
        snprintf(ifname, SAI_HOSTIF_NAME_SIZE, "Ethernet%d", i);

        // build attribute list
        int attrs_count = 3; // potentially 4
        sai_attribute_t attrs[4] = {
            {
                .id = SAI_HOSTIF_ATTR_TYPE,
                .value.s32 = SAI_HOSTIF_TYPE_NETDEV,
            },
            {
                .id = SAI_HOSTIF_ATTR_OBJ_ID,
                .value.oid = port_id,
            },
            {
                .id = SAI_HOSTIF_ATTR_NAME,
            }
        };
        strncpy(attrs[2].value.chardata, ifname, SAI_HOSTIF_NAME_SIZE);
        if (has_queues) {
            attrs[3].id = SAI_HOSTIF_ATTR_QUEUE;
            attrs[3].value.u32 = DEFAULT_HOSTIF_TX_QUEUE;
            attrs_count++;
        }

        // now create the host interface
        sai_object_id_t hostif_id;
        st = apis->hostif_api->create_hostif(&hostif_id, sw_id, attrs_count, attrs);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to create host interface for %s: %s\n", ifname, err);
            return -1;
        }

        char port_str[64];
        sai_serialize_object_id(port_str, port_id);
        char hostif_id_str[64];
        sai_serialize_object_id(hostif_id_str, hostif_id);
        printf("saictl: created host interface %s -> %s for port ID %s\n", hostif_id_str, ifname, port_str);

        // set the speed to 10G if possible
        sai_attribute_t attr_supported_speed;
        uint32_t supported_speed_list[16];
        attr_supported_speed.id = SAI_PORT_ATTR_SUPPORTED_SPEED;
        attr_supported_speed.value.u32list.count = 16;
        attr_supported_speed.value.u32list.list = supported_speed_list;
        
        st = apis->port_api->get_port_attribute(port_id, 1, &attr_supported_speed);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to query port %s for supported speeds: %s\n", port_str, err);
        } else {
            bool has_speed = false;
            for (int j = 0; j < attr_supported_speed.value.u32list.count; j++) {
                uint32_t speed = attr_supported_speed.value.u32list.list[j];
                if (speed == 10000) {
                    has_speed = true;
                    break;
                }
            }
            if (!has_speed) {
                printf("saictl: port %s does not support 10000 speed\n", port_str);
            } else {
                sai_attribute_t attr_speed;
                attr_speed.id = SAI_PORT_ATTR_SPEED;
                attr_speed.value.u32 = 10000;
                st = apis->port_api->set_port_attribute(port_id, &attr_speed);
                if (st != SAI_STATUS_SUCCESS) {
                    sai_serialize_status(err, st);
                    printf("saictl: failed to set speed for port %s to 10000: %s\n", port_str, err);
                } else {
                    printf("saictl: successfully set speed for port %s to 10000\n", port_str);
                }
            }
        }

        // set interface type
        sai_attribute_t attr_intf_type;
        attr_intf_type.id = SAI_PORT_ATTR_INTERFACE_TYPE;
        attr_intf_type.value.s32 = SAI_PORT_INTERFACE_TYPE_SFI;
        st = apis->port_api->set_port_attribute(port_id, &attr_intf_type);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to set port type of port %s to SFI: %s\n", port_str, err);
        } else {
            printf("saictl: successfully set port type of port %s to SFIn", port_str);
        }

        // set admin state
        sai_attribute_t attr_admin_state;
        attr_admin_state.id = SAI_PORT_ATTR_ADMIN_STATE;
        attr_admin_state.value.booldata = true;
        st = apis->port_api->set_port_attribute(port_id, &attr_admin_state);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to set admin state of port %s to true: %s\n", port_str, err);
        } else {
            printf("saictl: successfully set admin state of port %s to true\n", port_str);
        }

        // bring host interface up
        sai_attribute_t attr_oper_status;
        attr_oper_status.id = SAI_HOSTIF_ATTR_OPER_STATUS;
        attr_oper_status.value.booldata = true;
        st = apis->hostif_api->set_hostif_attribute(hostif_id, &attr_oper_status);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to bring host interface up for %s: %s\n", ifname, err);
        } else {
            printf("saictl: successfully brought host interface for %s\n", ifname);
        }
    }

    // get
    // SAI_PORT_ATTR_ADMIN_STATE
    // SAI_PORT_ATTR_SPEED
    // SAI_PORT_ATTR_OPER_SPEED
    // SAI_PORT_ATTR_OPER_STATUS
    for (int i = 0; i < attr_port_list.value.objlist.count; i++) {
        // get the port ID from the list
        sai_object_id_t port_id = attr_port_list.value.objlist.list[i];
        char port_str[64];
        sai_serialize_object_id(port_str, port_id);

        // admin state
        sai_attribute_t attr_admin_state = {
            .id = SAI_PORT_ATTR_ADMIN_STATE,
        };
        st = apis->port_api->get_port_attribute(port_id, 1, &attr_admin_state);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to query admin state of port %s: %s\n", port_str, err);
        } else {
            printf("saictl: successfully queried admin state of port %s: %d\n", port_str, attr_admin_state.value.booldata);
        }

        // speed
        sai_attribute_t attr_speed = {
            .id = SAI_PORT_ATTR_SPEED,
        };
        st = apis->port_api->get_port_attribute(port_id, 1, &attr_speed);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to query speed of port %s: %s\n", port_str, err);
        } else {
            printf("saictl: successfully queried speed of port %s: %d\n", port_str, attr_speed.value.u32);
        }

        // oper speed
        sai_attribute_t attr_oper_speed = {
            .id = SAI_PORT_ATTR_OPER_SPEED,
        };
        st = apis->port_api->get_port_attribute(port_id, 1, &attr_oper_speed);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to query oper speed of port %s: %s\n", port_str, err);
        } else {
            printf("saictl: successfully queried oper speed of port %s: %d\n", port_str, attr_oper_speed.value.u32);
        }

        // oper status
        sai_attribute_t attr_oper_status = {
            .id = SAI_PORT_ATTR_OPER_STATUS,
        };
        st = apis->port_api->get_port_attribute(port_id, 1, &attr_oper_status);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to query oper status of port %s: %s\n", port_str, err);
        } else {
            printf("saictl: successfully queried oper status of port %s: %d\n", port_str, attr_oper_status.value.s32);
        }
    }

    // set 
    // SAI_PORT_ATTR_PKT_TX_ENABLE
    // SAI_PORT_ATTR_ADMIN_STATE

    return 0;
}

static void switch_state_change_cb(_In_ sai_object_id_t switch_id, _In_ sai_switch_oper_status_t switch_oper_status) {
    printf("saictl: switch_state_change_cb\n");
    return;
}
static void switch_shutdown_request_cb(_In_ sai_object_id_t switch_id) {
    printf("saictl: switch_shutdown_request_cb\n");
    return;
}
static void fdb_event_cb(_In_ uint32_t count, _In_ const sai_fdb_event_notification_data_t *data) {
    printf("saictl: fdb_event_cb\n");
    return;
}
static void nat_event_cb(_In_ uint32_t count, _In_ const sai_nat_event_notification_data_t *data) {
    printf("saictl: nat_event_cb\n");
    return;
}
static void port_state_change_cb(_In_ uint32_t count, _In_ const sai_port_oper_status_notification_t *data) {
    printf("saictl: port_state_change_cb\n");
    return;
}
static void queue_pfc_deadlock_cb(_In_ uint32_t count, _In_ const sai_queue_deadlock_notification_data_t *data) {
    printf("saictl: queue_pfc_deadlock_cb\n");
    return;
}
static void bfd_session_state_change_cb(_In_ uint32_t count, _In_ const sai_bfd_session_state_notification_t *data) {
    printf("saictl: bfd_session_state_change_cb\n");
    return;
}

#define MAX_ELEMENTS 1024

typedef struct {
    sai_object_type_t ot;
    sai_object_id_t id;
} arrkv;

#define DATA_ARR_ELEMENTS 2048
static int dump_data_arr_idx = 0;
static arrkv dump_data_arr[DATA_ARR_ELEMENTS];

static bool add_oid(sai_object_type_t ot, sai_object_id_t id) {
    if (dump_data_arr_idx >= DATA_ARR_ELEMENTS) {
        return false;
    }
    arrkv tmp = { .ot = ot, .id = id};
    dump_data_arr[dump_data_arr_idx] = tmp;
    dump_data_arr_idx++;
    return true;
}

static bool has_oid(sai_object_type_t ot, sai_object_id_t id) {
    for (int i=0; i < DATA_ARR_ELEMENTS; i++) {
        arrkv entry = dump_data_arr[i];
        if (ot == entry.ot && id == entry.id) {
            return true;
        }
    }
    return false;
}

// this is in a nutshell exactly what the "saidiscover" application is doing in the sairedis repo
static void dump_startup_data(int rec, sai_apis_t *apis, sai_object_id_t id) {
    char id_str[128] = {0};
    sai_serialize_object_id(id_str, id);

    //////// start dump some data
    sai_object_type_t ot = sai_object_type_query(id);
    if (ot == SAI_OBJECT_TYPE_NULL) {
        // printf("saictl: sai_object_type_query(%s) returned SAI_OBJECT_TYPE_NULL\n", id_str);
        return;
    }

    if (has_oid(ot, id)) {
        // printf("saictl[%d]: already dumped for oid %s\n", rec, id_str);
        return;
    }

    if (!add_oid(ot, id)) {
        printf("saictl[%d]: cannot track OIDs anymore, array full", rec);
        return;
    }

    char ot_str[128] = {0};
    sai_serialize_object_type(ot_str, ot);

    // printf("saictl[%d]: dumping data for %s -> %s\n", rec, ot_str, id_str);

    const sai_object_type_info_t *info = sai_metadata_all_object_type_infos[ot];

    for (int idx = 0; info->attrmetadata[idx] != NULL; ++idx) {
        const sai_attr_metadata_t *md = info->attrmetadata[idx];

        if (md->objecttype == SAI_OBJECT_TYPE_PORT && md->attrid == SAI_PORT_ATTR_HW_LANE_LIST) {
            // XXX workaround for brcm
            // printf("saictl: workaround for brcm\n");
            continue;
        }

        if (md->objecttype == SAI_OBJECT_TYPE_HOSTIF_USER_DEFINED_TRAP /*&& md->attrid == SAI_HOSTIF_USER_DEFINED_TRAP_ATTR_TYPE*/) {
            // XXX workaround for brcm
            // printf("saictl: workaround 2 for brcm\n");
            continue;
        }

        if (md->objecttype == SAI_OBJECT_TYPE_HOSTIF_TRAP) {
            // XXX workaround for brcm
            // printf("saictl: workaround 3 for brcm\n");
            continue;
        }

        if (md->objecttype == SAI_OBJECT_TYPE_MY_MAC) {
            // XXX workaround for brcm
            // printf("saictl: workaround 4 for brcm\n");
            continue;
        }

        if (md->objecttype == SAI_OBJECT_TYPE_QUEUE) {
            continue;
        }

        sai_attribute_t attr;
        attr.id = md->attrid;

        // if (md->attrid == SAI_QUEUE_ATTR_PFC_DLR_PACKET_ACTION) {
        //     continue;
        // }
        // printf("%*s%s %s                                                                        \n", rec, "", md->attridname, id_str);
        if (md->attrvaluetype == SAI_ATTR_VALUE_TYPE_OBJECT_ID) {
            if (md->objecttype == SAI_OBJECT_TYPE_STP && md->attrid == SAI_STP_ATTR_BRIDGE_ID) {
                // printf("saictl: skipping %s since it causes crash\n", md->attridname);
                continue;
            }

            

            sai_object_meta_key_t mk = { .objecttype = ot, .objectkey = { .key = { .object_id = id } } };
            sai_status_t status = info->get(&mk, 1, &attr);
            if (status != SAI_STATUS_SUCCESS) {
                // char str[128];
                // sai_serialize_status(str, status);
                // printf("saictl: get error: %s: %s\n", md->attridname, str);
                continue;
            }

            if (md->defaultvaluetype == SAI_DEFAULT_VALUE_TYPE_CONST && attr.value.oid != SAI_NULL_OBJECT_ID) {
                char attr_val_oid_str[128];
                sai_serialize_object_id(attr_val_oid_str, attr.value.oid);
                // printf("saictl: const null, but got value %s on %s\n", attr_val_oid_str, md->attridname);
            }

            if (!md->allownullobjectid && attr.value.oid == SAI_NULL_OBJECT_ID) {
                printf("saictl: dont allow null, but got null on %s\n", md->attridname);
            }

            char val_str[128];
            sai_serialize_attribute_value(val_str, md, &attr.value);
            printf("%*ssaictl[%d]: result on %s->%s: %s: %s\n", rec, "", rec, ot_str, id_str, md->attridname, val_str);

            dump_startup_data(rec+1, apis, attr.value.oid); // recursion
        } else if (md->attrvaluetype == SAI_ATTR_VALUE_TYPE_OBJECT_LIST) {
            // printf("saictl: getting %s for %s\n", md->attridname, id_str);

            sai_object_id_t list[MAX_ELEMENTS];
            
            // sai_object_id_t ser_list[MAX_ELEMENTS];
            // sai_attribute_t ser_attr;
            

            attr.value.objlist.count = MAX_ELEMENTS;
            attr.value.objlist.list = list;


            sai_object_meta_key_t mk = { .objecttype = ot, .objectkey = { .key = { .object_id = id } } };
            sai_status_t status = info->get(&mk, 1, &attr);
            if (status != SAI_STATUS_SUCCESS) {
                // char str[128];
                // sai_serialize_status(str, status);
                // printf("saictl: get error: %s: %s\n", md->attridname, str);
                continue;
            }

            if (md->defaultvaluetype == SAI_DEFAULT_VALUE_TYPE_EMPTY_LIST && attr.value.objlist.count != 0) {
                printf("saictl: default is empty list, but got count %u on %s\n", attr.value.objlist.count, md->attridname);
            }

            // ser_attr.id = md->attrid;
            // ser_attr.value.objlist.count = attr.value.objlist.count;
            // ser_attr.value.objlist.list = ser_list;
            for (uint32_t i = 0; i < attr.value.objlist.count; i++)
            {
                // ser_attr.value.objlist.list[i] = attr.value.objlist.list[i];
                char entry_id_str[128] = {0};
                sai_serialize_object_id(entry_id_str, attr.value.objlist.list[i]);
                printf("%*ssaictl[%d]: result on %s->%s[%d/%d]: %s: %s\n", rec, "", rec, ot_str, id_str, i+1, attr.value.objlist.count, md->attridname, entry_id_str);
            }

            // char val_str[128];
            // sai_serialize_attribute_value(val_str, md, &ser_attr.value);
            // discovered[id][md->attridname] = sai_serialize_attr_value(*md, attr);
            // printf("saictl[%d]: list count %s: %u\n", rec, md->attridname, attr.value.objlist.count);
            // printf("%*ssaictl[%d]: result on %s: %s: %s\n", rec, "", rec, id_str, md->attridname, val_str);

            for (uint32_t i = 0; i < attr.value.objlist.count; i++)
            {
                char entry_id_str[128] = {0};
                sai_serialize_object_id(entry_id_str, attr.value.objlist.list[i]);
                // printf("%*ssaictl[%d]: recursing for %s %s %s\n", rec, "", rec, id_str, md->attridname, entry_id_str);
                dump_startup_data(rec+1, apis, attr.value.objlist.list[i]); // recursion
            }
        } else {
            if ((md->objecttype == SAI_OBJECT_TYPE_PORT && md->attrid == SAI_PORT_ATTR_FEC_MODE) ||
                (md->objecttype == SAI_OBJECT_TYPE_PORT && md->attrid == SAI_PORT_ATTR_GLOBAL_FLOW_CONTROL_MODE) ||
                (md->objecttype == SAI_OBJECT_TYPE_SWITCH && md->attrid == SAI_SWITCH_ATTR_INIT_SWITCH))
            {
                // workaround since return invalid values
                // printf("saictl: workaround since return invalid values\n");
                continue;
            }

            /*
             * Discover non oid attributes as well.
             *
             * TODO lists!
             */

            sai_object_id_t list[MAX_ELEMENTS];

            switch (md->attrvaluetype)
            {
                case SAI_ATTR_VALUE_TYPE_INT8:
                case SAI_ATTR_VALUE_TYPE_INT16:
                case SAI_ATTR_VALUE_TYPE_INT32:
                case SAI_ATTR_VALUE_TYPE_INT64:
                case SAI_ATTR_VALUE_TYPE_UINT8:
                case SAI_ATTR_VALUE_TYPE_UINT16:
                case SAI_ATTR_VALUE_TYPE_UINT32:
                case SAI_ATTR_VALUE_TYPE_UINT64:
                case SAI_ATTR_VALUE_TYPE_POINTER:
                case SAI_ATTR_VALUE_TYPE_BOOL:
                case SAI_ATTR_VALUE_TYPE_UINT32_RANGE:
                case SAI_ATTR_VALUE_TYPE_MAC:
                    break;

                case SAI_ATTR_VALUE_TYPE_INT8_LIST:
                case SAI_ATTR_VALUE_TYPE_INT32_LIST:
                case SAI_ATTR_VALUE_TYPE_UINT32_LIST:
                case SAI_ATTR_VALUE_TYPE_VLAN_LIST:

                    attr.value.objlist.count = MAX_ELEMENTS;
                    attr.value.objlist.list = list;
                    break;

                case SAI_ATTR_VALUE_TYPE_ACL_CAPABILITY:

                    attr.value.aclcapability.action_list.count = MAX_ELEMENTS;
                    attr.value.aclcapability.action_list.list = (int32_t*)list;
                    break;

                default: ;

                    // char attr_value_type_str[128];
                    // sai_serialize_attr_value_type(attr_value_type_str, md->attrvaluetype);
                    // printf("saictl: attr value: %s not supported\n", attr_value_type_str);
                    continue;
            }

            // printf("saictl: getting %s for %s\n", md->attridname, id_str);

            sai_object_meta_key_t mk = { .objecttype = ot, .objectkey = { .key = { .object_id = id } } };
            sai_status_t status = info->get(&mk, 1, &attr);
            if (status == SAI_STATUS_SUCCESS) {
                char val_str[128];
                sai_serialize_attribute_value(val_str, md, &attr.value);
                // discovered[id][md->attridname] = sai_serialize_attr_value(*md, attr);
                printf("%*ssaictl[%d]: result on %s->%s: %s: %s\n", rec, "", rec, ot_str, id_str, md->attridname, val_str);
            } else { ;
                // char str[128];
                // sai_serialize_status(str, status);
                // printf("saictl: get error: %s: %s\n", md->attridname, str);
            }
        }
    }
}