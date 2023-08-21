# Documentation and Notes

We keep this folder to contain research and development notes.

For example, it is useful to extract the following from running SONiC instances:

- `asic_state.json`: contains the ASIC_STATE redis dump from within a running SONiC instance. This can be achieved by running `redis-dump -d 1 -k "ASIC_STATE*" -y > asic_state.json` on the SONiC instance.
- `dump-data-after-start`: contains a dump of the SAI state (as much as was dumpable without crashing) by using the `dump_startup_data()` function within the POC code.
- `config_db.json`: is simply the SONiC config db from a running SONiC instance
- `sonic-system-state-debug`: it's a collection of system state of a running SONiC instance. This way we discovered that `/dev/shm` needs to be mounted for Broadcom SAI to work.
- `td3-s5212f-25g.config.bcm`: is the broadcom configuration file. This must be referenced by a SAI profile value from the `SAI_KEY_INIT_CONFIG_FILE` setting for Broadcom SAI.
