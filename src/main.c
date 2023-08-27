#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include <signal.h>
#include <sys/types.h>
#include <arpa/inet.h>
#include <sai.h>
#include <saimetadata.h>
#include <s5212.h>

static volatile sig_atomic_t stop;

static sai_ip4_t myip();
static sai_ip4_t mymask();

static const char* profile_get_value(_In_ sai_switch_profile_id_t profile_id, _In_ const char *variable);
static int profile_get_next_value(_In_ sai_switch_profile_id_t profile_id, _Out_ const char **variable, _Out_ const char **value);
static int register_callbacks(sai_apis_t *apis, sai_object_id_t sw_id);
static int remove_default_vlan_members(sai_apis_t *apis, sai_object_id_t sw_id);
static int remove_default_bridge_ports(sai_apis_t *apis, sai_object_id_t sw_id);
static size_t add_host_intfs(sai_apis_t *apis, sai_object_id_t sw_id, sai_object_id_t *hifs_ids, size_t *hifs_ids_count);
static int remove_host_intfs(sai_apis_t *apis, sai_object_id_t sw_id, sai_object_id_t *hifs_ids, size_t hifs_ids_count);
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

    // create switch
    // MAC as per SONiC: 1c:72:1d:ec:44:a0
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

    ret = remove_default_vlan_members(&apis, sw_id);
    if (ret < 0) {
        printf("saictl: removing default VLAN members failed: %d\n", ret);
    }

    ret = remove_default_bridge_ports(&apis, sw_id);
    if (ret < 0) {
        printf("saictl: removing default bridge ports failed: %d\n", ret);
    }

    //////// start creating stuff
    size_t hifs_ids_count = 0;
    sai_object_id_t hifs_ids[20];
    ret = add_host_intfs(&apis, sw_id, hifs_ids, &hifs_ids_count);
    if (ret < 0) {
        printf("saictl: creating stuff failed: %d\n", ret);
        stop = 1;
    }
    //////// end creating stuff

    // this enables Broadcom's "drivshell"
    // the call to `set_switch_attribute` is blocking in this case
    sai_attribute_t attr_shell;
    attr_shell.id = SAI_SWITCH_ATTR_SWITCH_SHELL_ENABLE;
    attr_shell.value.booldata = true;
    st = apis.switch_api->set_switch_attribute(sw_id, &attr_shell);
    if (st != SAI_STATUS_SUCCESS) {
        char str[128];
        sai_serialize_status(str, st);
        printf("saictl: switch shell failed: %s\n", str);
        stop = 1;
    }

    // wait for signal before we shut down
    signal(SIGINT, main_signal_handler);
    signal(SIGTERM, main_signal_handler);
    printf("saictl: waiting on SIGINT or SIGTERM\n");
    while (!stop) {
        sleep(5);
    }
    
    printf("saictl: shutting down...\n");

    //////// start remove stuff
    ret = remove_host_intfs(&apis, sw_id, hifs_ids, hifs_ids_count);
    if (ret < 0) {
        printf("saictl: removing stuff failed: %d\n", ret);
    }
    //////// end remove stuff

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

static sai_ip4_t myip() {
    struct in_addr ipv4Addr;
    if (inet_pton(AF_INET, "10.10.10.1", &ipv4Addr) <= 0) {
        printf("saictl: ERROR invalid IP address\n");
        return 0;
    }

    // SAI expects network byte order here
    //uint32_t ipAsUInt32 = ntohl(ipv4Addr.s_addr);
    return ipv4Addr.s_addr;
}

static sai_ip4_t mymask() {
    struct in_addr ipv4Addr;
    if (inet_pton(AF_INET, "255.255.255.255", &ipv4Addr) <= 0) {
        printf("saictl: ERROR invalid IP address for mask\n");
        return 0;
    }

    // SAI expects network byte order here
    //uint32_t ipAsUInt32 = ntohl(ipv4Addr.s_addr);
    return ipv4Addr.s_addr;
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

static int remove_default_vlan_members(sai_apis_t *apis, sai_object_id_t sw_id) {
    sai_status_t st;
    char err[128];

    sai_attribute_t attr_default_vlan;
    attr_default_vlan.id = SAI_SWITCH_ATTR_DEFAULT_VLAN_ID;

    st = apis->switch_api->get_switch_attribute(sw_id, 1, &attr_default_vlan);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to get default VLAN: %s\n", err);
        return -1;
    }

    sai_object_id_t default_vlan_id = attr_default_vlan.value.oid;
    char default_vlan_id_str[32];
    sai_serialize_object_id(default_vlan_id_str, default_vlan_id);
    printf("saictl: successfully retrieved default VLAN ID: %s\n", default_vlan_id_str);

    // get vlan members
    sai_object_id_t vlan_members[128];
    sai_attribute_t attr_vlan_members;
    attr_vlan_members.id = SAI_VLAN_ATTR_MEMBER_LIST;
    attr_vlan_members.value.objlist.count = 128;
    attr_vlan_members.value.objlist.list = vlan_members;

    st = apis->vlan_api->get_vlan_attribute(default_vlan_id, 1, &attr_vlan_members);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to get default VLAN %s member list: %s\n", default_vlan_id_str, err);
        return -1;
    }

    // now iterate over them and remove them
    int ret = 0;
    for (int i = 0; i < attr_vlan_members.value.objlist.count; i++) {
        sai_object_id_t vlan_member_id = attr_vlan_members.value.objlist.list[i];
        char vlan_member_id_str[32];
        sai_serialize_object_id(vlan_member_id_str, vlan_member_id);
        st = apis->vlan_api->remove_vlan_member(vlan_member_id);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictL: failed to remove VLAN member %s from VLAN %s: %s\n", vlan_member_id_str, default_vlan_id_str, err);
            ret = -1;
            continue;
        }
        printf("saictl: successfully removed VLAN member %s from VLAN %s\n", vlan_member_id_str, default_vlan_id_str);
    }
    return ret;
}

static int remove_default_bridge_ports(sai_apis_t *apis, sai_object_id_t sw_id) {
    sai_status_t st;
    char err[128];

    sai_attribute_t attr_default_bridge_id;
    attr_default_bridge_id.id = SAI_SWITCH_ATTR_DEFAULT_1Q_BRIDGE_ID;

    st = apis->switch_api->get_switch_attribute(sw_id, 1, &attr_default_bridge_id);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to get default bridge ID: %s\n", err);
        return -1;
    }

    sai_object_id_t default_bridge_id = attr_default_bridge_id.value.oid;
    char default_bridge_id_str[32];
    sai_serialize_object_id(default_bridge_id_str, default_bridge_id);
    printf("saictl: successfully retrieved default bridge ID: %s\n", default_bridge_id_str);

    sai_object_id_t bridge_port_list[128];
    sai_attribute_t attr_bridge_port_list;
    attr_bridge_port_list.id = SAI_BRIDGE_ATTR_PORT_LIST;
    attr_bridge_port_list.value.objlist.count = 128;
    attr_bridge_port_list.value.objlist.list = bridge_port_list;

    st = apis->bridge_api->get_bridge_attribute(default_bridge_id, 1, &attr_bridge_port_list);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to get bridge %s port list: %s\n", default_bridge_id_str, err);
        return -1;
    }

    // now iterate over them and remove them
    int ret = 0;
    for (int i =0; i < attr_bridge_port_list.value.objlist.count; i++) {
        sai_object_id_t bridge_port = attr_bridge_port_list.value.objlist.list[i];
        char bridge_port_str[32];
        sai_serialize_object_id(bridge_port_str, bridge_port);

        // check if this is a 
        sai_attribute_t attr_bridge_port_type;
        attr_bridge_port_type.id = SAI_BRIDGE_PORT_ATTR_TYPE;
        attr_bridge_port_type.value.s32 = SAI_NULL_OBJECT_ID;
        st = apis->bridge_api->get_bridge_port_attribute(bridge_port, 1, &attr_bridge_port_type);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to get bridge port type for bridge port %s: %s\n", bridge_port_str, err);
            ret = -1;
            continue;
        }

        if (attr_bridge_port_type.value.s32 != SAI_BRIDGE_PORT_TYPE_PORT) {
            printf("saictl: not removing bridge port %s from bridge %s as it is not of type SAI_BRIDGE_PORT_TYPE_PORT\n", bridge_port_str, default_bridge_id_str);
            continue;
        }

        st = apis->bridge_api->remove_bridge_port(bridge_port);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to remove bridge port %s from bridge %s: %s\n", bridge_port_str, default_bridge_id_str, err);
            ret = -1;
            continue;
        }
        printf("saictl: successfully removed bridge port %s from bridge %s\n", bridge_port_str, default_bridge_id_str);
    }
    return ret;
}

static size_t add_host_intfs(sai_apis_t *apis, sai_object_id_t sw_id, sai_object_id_t *hifs_ids, size_t *hifs_ids_count) {
    sai_status_t st;
    char err[128];

    // // get default vlan
    // sai_attribute_t attr_default_vlan;
    // attr_default_vlan.id = SAI_SWITCH_ATTR_DEFAULT_VLAN_ID;
    // st = apis->switch_api->get_switch_attribute(sw_id, 1, &attr_default_vlan);
    // if (st != SAI_STATUS_SUCCESS) {
    //     sai_serialize_status(err, st);
    //     printf("saictl: failed to get default VLAN: %s\n", err);
    //     return -1;
    // }
    // uint16_t default_vlan_id = attr_default_vlan.value.u16;

    // // get default bridge
    // sai_attribute_t attr_default_bridge_id;
    // attr_default_bridge_id.id = SAI_SWITCH_ATTR_DEFAULT_1Q_BRIDGE_ID;
    // st = apis->switch_api->get_switch_attribute(sw_id, 1, &attr_default_bridge_id);
    // if (st != SAI_STATUS_SUCCESS) {
    //     sai_serialize_status(err, st);
    //     printf("saictl: failed to get default bridge ID: %s\n", err);
    //     return -1;
    // }
    // sai_object_id_t default_bridge_id = attr_default_bridge_id.value.oid;
    // char default_bridge_id_str[32];
    // sai_serialize_object_id(default_bridge_id_str, default_bridge_id);
    // printf("saictl: default bridge ID is %s\n", default_bridge_id_str);

    // get default trap group
    sai_attribute_t attr_default_trap_group;
    attr_default_trap_group.id = SAI_SWITCH_ATTR_DEFAULT_TRAP_GROUP;
    attr_default_trap_group.value.oid = SAI_NULL_OBJECT_ID;

    st = apis->switch_api->get_switch_attribute(sw_id, 1, &attr_default_trap_group);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to get default trap group from switch: %s\n", err);
        return -1;
    }
    sai_object_id_t default_trap_group_id = attr_default_trap_group.value.oid;

    // create traps: ip2me, arp request and arp response
    sai_attribute_t attr_trap_ip2me[] = {
        {
            .id = SAI_HOSTIF_TRAP_ATTR_TRAP_TYPE,
            .value.s32 = SAI_HOSTIF_TRAP_TYPE_IP2ME,
        },
        {
            .id = SAI_HOSTIF_TRAP_ATTR_PACKET_ACTION,
            .value.s32 = SAI_PACKET_ACTION_TRAP,
        },
        {
            .id = SAI_HOSTIF_TRAP_ATTR_TRAP_GROUP,
            .value.oid = default_trap_group_id,
        }
    };
    sai_object_id_t trap_ip2me_id;
    st = apis->hostif_api->create_hostif_trap(&trap_ip2me_id, sw_id, 3, attr_trap_ip2me);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to create ip2me trap: %s\n", err);
        return -1;
    }

    sai_attribute_t attr_trap_arpreq[] = {
        {
            .id = SAI_HOSTIF_TRAP_ATTR_TRAP_TYPE,
            .value.s32 = SAI_HOSTIF_TRAP_TYPE_ARP_REQUEST,
        },
        {
            .id = SAI_HOSTIF_TRAP_ATTR_PACKET_ACTION,
            .value.s32 = SAI_PACKET_ACTION_COPY,
        },
        {
            .id = SAI_HOSTIF_TRAP_ATTR_TRAP_GROUP,
            .value.oid = default_trap_group_id,
        }
    };
    sai_object_id_t trap_arpreq_id;
    st = apis->hostif_api->create_hostif_trap(&trap_arpreq_id, sw_id, 3, attr_trap_arpreq);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to create arpreq trap: %s\n", err);
        return -1;
    }

    sai_attribute_t attr_trap_arpresp[] = {
        {
            .id = SAI_HOSTIF_TRAP_ATTR_TRAP_TYPE,
            .value.s32 = SAI_HOSTIF_TRAP_TYPE_ARP_RESPONSE,
        },
        {
            .id = SAI_HOSTIF_TRAP_ATTR_PACKET_ACTION,
            .value.s32 = SAI_PACKET_ACTION_COPY,
        },
        {
            .id = SAI_HOSTIF_TRAP_ATTR_TRAP_GROUP,
            .value.oid = default_trap_group_id,
        }
    };
    sai_object_id_t trap_arpresp_id;
    st = apis->hostif_api->create_hostif_trap(&trap_arpresp_id, sw_id, 3, attr_trap_arpresp);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to create arpresp trap: %s\n", err);
        return -1;
    }

    // create default host interface table entry like SONiC
    sai_object_id_t default_hostif_table_id;
    sai_attribute_t attrs_default_hostif_table[] = {
        {
            .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_TYPE,
            .value.s32 = SAI_HOSTIF_TABLE_ENTRY_TYPE_WILDCARD,
        },
        {
            .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_CHANNEL_TYPE,
            .value.s32 = SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_PHYSICAL_PORT,
        }
    };
    st = apis->hostif_api->create_hostif_table_entry(&default_hostif_table_id, sw_id, 2, attrs_default_hostif_table);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to create default host interface table entry: %s\n", err);
        return -1;
    }

    // get the CPU port first
    sai_attribute_t attr_cpu_port;
    attr_cpu_port.id = SAI_SWITCH_ATTR_CPU_PORT;
    attr_cpu_port.value.oid = SAI_NULL_OBJECT_ID;
    st = apis->switch_api->get_switch_attribute(sw_id, 1, &attr_cpu_port);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to get CPU port from switch: %s\n", err);
        return -1;
    }
    sai_object_id_t cpu_port_id = attr_cpu_port.value.oid;

    // // desperately try to create a bridge port for the CPU port
    // sai_object_id_t cpu_bridge_port_id;
    // sai_attribute_t attr_cpu_bridge_port[] = {
    //     {
    //         .id = SAI_BRIDGE_PORT_ATTR_TYPE,
    //         .value.s32 = SAI_BRIDGE_PORT_TYPE_PORT,
    //     },
    //     {
    //         .id = SAI_BRIDGE_PORT_ATTR_PORT_ID,
    //         .value.oid = cpu_port_id,
    //     },
    //     {
    //         .id = SAI_BRIDGE_PORT_ATTR_BRIDGE_ID,
    //         .value.oid = default_bridge_id,
    //     }
    // };
    // st = apis->bridge_api->create_bridge_port(&cpu_bridge_port_id, sw_id, 3, attr_cpu_bridge_port);
    // if (st != SAI_STATUS_SUCCESS) {
    //     sai_serialize_status(err, st);
    //     printf("saictl: failed to create bridge port for CPU port: %s\n", err);
    // } else {
    //     printf("saictl: successfully created bridge port for CPU port\n");
    // }

    // create host intf for CPU port (not sure why this is necessary - if at all - but SONiC does that)
    sai_attribute_t attrs_cpu_hif[] = {
        {
            .id = SAI_HOSTIF_ATTR_NAME,
        },
        {
            .id = SAI_HOSTIF_ATTR_TYPE,
            .value.s32 = SAI_HOSTIF_TYPE_NETDEV,
        },
        {
            .id = SAI_HOSTIF_ATTR_OBJ_ID,
            .value.oid = cpu_port_id,
        },
        {
            .id = SAI_HOSTIF_ATTR_OPER_STATUS,
            .value.booldata = true,
        },
    };
    strncpy(attrs_cpu_hif[0].value.chardata, "CPU\0", 4);

    sai_object_id_t cpu_hifs_id;
    st = apis->hostif_api->create_hostif(&cpu_hifs_id, sw_id, 4, attrs_cpu_hif);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to create host interface for CPU: %s\n", err);
        return -1;
    }
    hifs_ids[*hifs_ids_count] = cpu_hifs_id;
    *hifs_ids_count = *hifs_ids_count + 1;

    // // set admin state
    // sai_attribute_t attr_cpu_admin_state;
    // attr_cpu_admin_state.id = SAI_PORT_ATTR_ADMIN_STATE;
    // attr_cpu_admin_state.value.booldata = true;
    // st = apis->port_api->set_port_attribute(cpu_port_id, &attr_cpu_admin_state);
    // if (st != SAI_STATUS_SUCCESS) {
    //     sai_serialize_status(err, st);
    //     printf("saictl: failed to set admin state of CPU port to true: %s\n", err);
    // } else {
    //     printf("saictl: successfully set admin state of CPU port to true\n");
    // }

    // // set default vlan
    // sai_attribute_t attr_cpu_vlan;
    // attr_cpu_vlan.id = SAI_PORT_ATTR_PORT_VLAN_ID;
    // attr_cpu_vlan.value.u16 = default_vlan_id;
    // st = apis->port_api->set_port_attribute(cpu_port_id, &attr_cpu_vlan);
    // if (st != SAI_STATUS_SUCCESS) {
    //     sai_serialize_status(err, st);
    //     printf("saictl: failed to set default vlan of CPU port: %s\n", err);
    // } else {
    //     printf("saictl: successfully set default vlan of CPU port\n");
    // }

    // create generic netlink interface like SONiC for sflow
    sai_attribute_t attrs_nl_hif[] = {
        {
            .id = SAI_HOSTIF_ATTR_NAME,
        },
        {
            .id = SAI_HOSTIF_ATTR_GENETLINK_MCGRP_NAME,
        },
        {
            .id = SAI_HOSTIF_ATTR_TYPE,
            .value.s32 = SAI_HOSTIF_TYPE_GENETLINK,
        },
        {
            .id = SAI_HOSTIF_ATTR_OPER_STATUS,
            .value.booldata = true,
        },
    };
    strncpy(attrs_nl_hif[0].value.chardata, "psample\0", 8);
    strncpy(attrs_nl_hif[1].value.chardata, "packets\0", 8);

    sai_object_id_t nl_hifs_id;
    st = apis->hostif_api->create_hostif(&nl_hifs_id, sw_id, 4, attrs_nl_hif);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to create host interface for psample: %s\n", err);
        return -1;
    }
    hifs_ids[*hifs_ids_count] = nl_hifs_id;
    *hifs_ids_count = *hifs_ids_count + 1;

    // get default virtual router
    sai_attribute_t attr_default_router;
    attr_default_router.id = SAI_SWITCH_ATTR_DEFAULT_VIRTUAL_ROUTER_ID;
    attr_default_router.value.oid = SAI_NULL_OBJECT_ID;
    st = apis->switch_api->get_switch_attribute(sw_id, 1, &attr_default_router);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to get default virutal router: %s\n", err);
        return -1;
    }
    sai_object_id_t default_virtual_router_id = attr_default_router.value.oid;
    char default_virtual_router_id_str[32];
    sai_serialize_object_id(default_virtual_router_id_str, default_virtual_router_id);
    printf("saictl: received default virtual router ID: %s\n", default_virtual_router_id_str);

    // create a loopback router interface
    sai_attribute_t attr_rif_lo[] = {
        {
            .id = SAI_ROUTER_INTERFACE_ATTR_TYPE,
            .value.s32 = SAI_ROUTER_INTERFACE_TYPE_LOOPBACK,
        },
        {
            .id = SAI_ROUTER_INTERFACE_ATTR_VIRTUAL_ROUTER_ID,
            .value.oid = default_virtual_router_id,
        },
        {
            .id = SAI_ROUTER_INTERFACE_ATTR_MTU,
            .value.u32 = 9100,
        }
    };
    sai_object_id_t rif_lo_id;
    st = apis->router_interface_api->create_router_interface(&rif_lo_id, sw_id, 3, attr_rif_lo);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to create loopback router interface: %s\n", err);
        return -1;
    }
    printf("saictl: successfully created loopback router interface\n");

    // create default route entry (must be the first)
    sai_route_entry_t default_route_entry = {
        .switch_id = sw_id,
        .vr_id = default_virtual_router_id,
        .destination = {
            .addr_family = SAI_IP_ADDR_FAMILY_IPV4,
            .addr.ip4 = INADDR_ANY,
            .mask.ip4 = INADDR_ANY,
        },
    };
    sai_attribute_t attr_default_route_entry = {
        .id = SAI_ROUTE_ENTRY_ATTR_PACKET_ACTION,
        .value.s32 = SAI_PACKET_ACTION_DROP,
    };
    st = apis->route_api->create_route_entry(&default_route_entry, 1, &attr_default_route_entry);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to add default route entry to default virtual router: %s\n", err);
        return -1;
    }
    char default_route_entry_str[128];
    sai_serialize_route_entry(default_route_entry_str, &default_route_entry);
    printf("saictl: successfully added default route %s\n", default_route_entry_str);

    // now create route for ourselves
    sai_route_entry_t route_entry = {
        .switch_id = sw_id,
        .vr_id = default_virtual_router_id,
        .destination = {
            .addr_family = SAI_IP_ADDR_FAMILY_IPV4,
            .addr.ip4 = myip(),
            .mask.ip4 = mymask(),
        },
    };
    sai_attribute_t attr_route_entry[] = {
        {
            .id = SAI_ROUTE_ENTRY_ATTR_PACKET_ACTION,
            .value.s32 = SAI_PACKET_ACTION_FORWARD,
        },
        {
            .id = SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID,
            .value.oid = cpu_port_id,
        }
    };
    st = apis->route_api->create_route_entry(&route_entry, 2, attr_route_entry);
    if (st != SAI_STATUS_SUCCESS) {
        sai_serialize_status(err, st);
        printf("saictl: failed to add our route entry to default virtual router: %s\n", err);
        return -1;
    }
    char route_entry_str[128];
    sai_serialize_route_entry(route_entry_str, &route_entry);
    printf("saictl: successfully added route %s\n", route_entry_str);

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

        // now create the host interface
        sai_object_id_t hostif_id;
        st = apis->hostif_api->create_hostif(&hostif_id, sw_id, 3, attrs);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to create host interface for %s: %s\n", ifname, err);
            continue;
        }
        hifs_ids[*hifs_ids_count] = hostif_id;
        *hifs_ids_count = *hifs_ids_count + 1;

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
            printf("saictl: successfully set port type of port %s to SFI\n", port_str);
        }

        // // set default vlan
        // sai_attribute_t attr_vlan;
        // attr_vlan.id = SAI_PORT_ATTR_PORT_VLAN_ID;
        // attr_vlan.value.u16 = default_vlan_id;
        // st = apis->port_api->set_port_attribute(port_id, &attr_vlan);
        // if (st != SAI_STATUS_SUCCESS) {
        //     sai_serialize_status(err, st);
        //     printf("saictl: failed to set default vlan of port %s: %s\n", port_str, err);
        // } else {
        //     printf("saictl: successfully set default vlan of port %s\n", port_str);
        // }

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

        // create router interface
        sai_mac_t default_mac_addr = {0x1c, 0x72, 0x1d, 0xec, 0x44, 0xa0};
        sai_attribute_t attr_rif[] = {
            {
                .id = SAI_ROUTER_INTERFACE_ATTR_SRC_MAC_ADDRESS,
            },
            {
                .id = SAI_ROUTER_INTERFACE_ATTR_TYPE,
                .value.s32 = SAI_ROUTER_INTERFACE_TYPE_PORT,
            },
            {
                .id = SAI_ROUTER_INTERFACE_ATTR_PORT_ID,
                .value.oid = port_id,
            },
            {
                .id = SAI_ROUTER_INTERFACE_ATTR_VIRTUAL_ROUTER_ID,
                .value.oid = default_virtual_router_id,
            },
            {
                .id = SAI_ROUTER_INTERFACE_ATTR_MTU,
                .value.u32 = 9100,
            },
            {
                .id = SAI_ROUTER_INTERFACE_ATTR_NAT_ZONE_ID,
                .value.u8 = 0,
            }
        };
        attr_rif[0].value.mac[0] = default_mac_addr[0];
        attr_rif[0].value.mac[1] = default_mac_addr[1];
        attr_rif[0].value.mac[2] = default_mac_addr[2];
        attr_rif[0].value.mac[3] = default_mac_addr[3];
        attr_rif[0].value.mac[4] = default_mac_addr[4];
        attr_rif[0].value.mac[5] = default_mac_addr[5];
        sai_object_id_t rif_id;
        st = apis->router_interface_api->create_router_interface(&rif_id, sw_id, 6, attr_rif);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to create router interface for %s: %s\n", ifname, err);
        } else {
            printf("saictl: successfully created router interface for %s\n", ifname);
        }

        // // add host interface table entry
        // sai_object_id_t table_entry_arpreq_id;
        // sai_attribute_t attr_table_entry_arpreq[] = {
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_TYPE,
        //         .value.s32 = SAI_HOSTIF_TABLE_ENTRY_TYPE_PORT,
        //     },
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_OBJ_ID,
        //         .value.oid = port_id,
        //     },
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_TRAP_ID,
        //         .value.oid = trap_arpreq_id,
        //     },
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_CHANNEL_TYPE,
        //         .value.s32 = SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_PHYSICAL_PORT,
        //     }
        // };
        // st = apis->hostif_api->create_hostif_table_entry(&table_entry_arpreq_id, sw_id, 4, attr_table_entry_arpreq);
        // if (st != SAI_STATUS_SUCCESS) {
        //     sai_serialize_status(err, st);
        //     printf("saictl: failed to create host interface table entry for ARP requests for %s: %s\n", ifname, err);
        // } else {
        //     printf("saictl: successfully created host interface table entry for ARP requests for %s\n", ifname);
        // }

        // sai_object_id_t table_entry_arpresp_id;
        // sai_attribute_t attr_table_entry_arpresp[] = {
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_TYPE,
        //         .value.s32 = SAI_HOSTIF_TABLE_ENTRY_TYPE_PORT,
        //     },
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_OBJ_ID,
        //         .value.oid = port_id,
        //     },
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_TRAP_ID,
        //         .value.oid = trap_arpresp_id,
        //     },
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_CHANNEL_TYPE,
        //         .value.s32 = SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_PHYSICAL_PORT,
        //     }
        // };
        // st = apis->hostif_api->create_hostif_table_entry(&table_entry_arpresp_id, sw_id, 4, attr_table_entry_arpresp);
        // if (st != SAI_STATUS_SUCCESS) {
        //     sai_serialize_status(err, st);
        //     printf("saictl: failed to create host interface table entry for ARP responses for %s: %s\n", ifname, err);
        // } else {
        //     printf("saictl: successfully created host interface table entry for ARP responses for %s\n", ifname);
        // }

        // sai_object_id_t table_entry_id;
        // sai_attribute_t attr_table_entry[] = {
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_TYPE,
        //         .value.s32 = SAI_HOSTIF_TABLE_ENTRY_TYPE_PORT,
        //     },
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_OBJ_ID,
        //         .value.oid = port_id,
        //     },
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_TRAP_ID,
        //         .value.oid = trap_ip2me_id,
        //     },
        //     {
        //         .id = SAI_HOSTIF_TABLE_ENTRY_ATTR_CHANNEL_TYPE,
        //         .value.s32 = SAI_HOSTIF_TABLE_ENTRY_CHANNEL_TYPE_NETDEV_PHYSICAL_PORT,
        //     }
        // };
        // st = apis->hostif_api->create_hostif_table_entry(&table_entry_id, sw_id, 4, attr_table_entry);
        // if (st != SAI_STATUS_SUCCESS) {
        //     sai_serialize_status(err, st);
        //     printf("saictl: failed to create host interface table entry for IP2ME for %s: %s\n", ifname, err);
        // } else {
        //     printf("saictl: successfully created host interface table entry for IP2ME for %s\n", ifname);
        // }
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

    return *hifs_ids_count;
}

static int remove_host_intfs(sai_apis_t *apis, sai_object_id_t sw_id, sai_object_id_t *hifs_ids, size_t hifs_ids_count) {
    sai_status_t st;
    char err[128];
    int ret = 0;

    for (int i = 0; i < hifs_ids_count; i++) {
        sai_object_id_t hifs_id = hifs_ids[i];
        char hifs_id_str[64];
        sai_serialize_object_id(hifs_id_str, hifs_id);
        st = apis->hostif_api->remove_hostif(hifs_id);
        if (st != SAI_STATUS_SUCCESS) {
            sai_serialize_status(err, st);
            printf("saictl: failed to remove host interface %s: %s\n", hifs_id_str, err);
            ret = -1;
            continue;
        }
        printf("saictl: successfully removed host interface %s\n", hifs_id_str);
    }
    return ret;
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
    char str[256];
    sai_serialize_port_oper_status_notification(str, data);
    printf("saictl: port_state_change_cb: %s\n", str);
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