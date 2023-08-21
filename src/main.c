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
        char str[512];
        sai_serialize_status(str, st);
        printf("saictl: create_switch error: 0x%x %s\n", st, str);
        return EXIT_FAILURE;
    }
    printf("saictl: create_switch success\n");

    // remove switch

    st = apis.switch_api->remove_switch(sw_id);
    if (st != SAI_STATUS_SUCCESS) {
        printf("saictl: remove_switch error: 0x%x\n", st);
    }
    printf("saictl: remove_switch success\n");

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