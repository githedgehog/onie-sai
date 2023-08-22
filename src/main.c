#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include <sys/types.h>
#ifdef __cplusplus
extern "C" {
#endif
#include <sai.h>
#include <saimetadata.h>
#include <s5212.h>
#ifdef __cplusplus
}
#endif


static const char* profile_get_value(_In_ sai_switch_profile_id_t profile_id, _In_ const char *variable);
static int profile_get_next_value(_In_ sai_switch_profile_id_t profile_id, _Out_ const char **variable, _Out_ const char **value);
static void dump_startup_data(int rec, sai_apis_t *apis, sai_object_id_t id);

static const sai_service_method_table_t smt = {
    .profile_get_value = profile_get_value,
    .profile_get_next_value = profile_get_next_value,
};

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
    sai_mac_t default_mac_addr = {1,2,3,4,5,6};
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
    printf("saictl: STARTING DUMP DATA\n");
    dump_startup_data(0, &apis, sw_id);
    printf("saictl: END DUMP DATA\n");
    //////// end dump some data

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

    printf("saictl[%d]: dumping data for %s -> %s\n", rec, ot_str, id_str);

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

        sai_attribute_t attr;
        attr.id = md->attrid;

        if (md->attrvaluetype == SAI_ATTR_VALUE_TYPE_OBJECT_ID) {
            if (md->objecttype == SAI_OBJECT_TYPE_STP && md->attrid == SAI_STP_ATTR_BRIDGE_ID) {
                // printf("saictl: skipping %s since it causes crash\n", md->attridname);
                continue;
            }

            // printf("saictl[%d]: getting %s for %s\n", rec, md->attridname, id_str);

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
                printf("saictl: const null, but got value %s on %s\n", attr_val_oid_str, md->attridname);
            }

            if (!md->allownullobjectid && attr.value.oid == SAI_NULL_OBJECT_ID) {
                printf("saictl: dont allow null, but got null on %s\n", md->attridname);
            }

            char val_str[128];
            sai_serialize_attribute_value(val_str, md, &attr.value);
            printf("%*ssaictl[%d]: result on %s: %s: %s\n", rec, "", rec, id_str, md->attridname, val_str);

            dump_startup_data(rec+1, apis, attr.value.oid); // recursion
        } else if (md->attrvaluetype == SAI_ATTR_VALUE_TYPE_OBJECT_LIST) {
            // printf("saictl: getting %s for %s\n", md->attridname, id_str);

            sai_object_id_t list[MAX_ELEMENTS];

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

            for (uint32_t i = 0; i < attr.value.objlist.count; i++)
            {
                char entry_id_str[128] = {0};
                sai_serialize_object_id(entry_id_str, attr.value.objlist.list[i]);
                printf("%*ssaictl[%d]: entry[%d/%d] for %s %s %s\n", rec, "", rec, i, attr.value.objlist.count, id_str, md->attridname, entry_id_str);
            }

            char val_str[128];
            sai_serialize_attribute_value(val_str, md, &attr.value);
            // discovered[id][md->attridname] = sai_serialize_attr_value(*md, attr);
            // printf("saictl[%d]: list count %s: %u\n", rec, md->attridname, attr.value.objlist.count);
            printf("%*ssaictl[%d]: result on %s: %s: %s\n", rec, "", rec, id_str, md->attridname, val_str);

            for (uint32_t i = 0; i < attr.value.objlist.count; i++)
            {
                char entry_id_str[128] = {0};
                sai_serialize_object_id(entry_id_str, attr.value.objlist.list[i]);
                printf("%*ssaictl[%d]: recursing for %s %s %s\n", rec, "", rec, id_str, md->attridname, entry_id_str);
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
                printf("%*ssaictl[%d]: result on %s: %s: %s\n", rec, "", rec, id_str, md->attridname, val_str);
            } else { ;
                // char str[128];
                // sai_serialize_status(str, status);
                // printf("saictl: get error: %s: %s\n", md->attridname, str);
            }
        }
    }
}