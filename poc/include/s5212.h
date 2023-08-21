
typedef struct {
    char* k;
    char* v;
} kv;

kv s5212_profile[] = {
    {SAI_KEY_INIT_CONFIG_FILE, "/root/saictl/etc/config.bcm"},
};

#define s5212_profile_length (sizeof(s5212_profile) / sizeof(kv))