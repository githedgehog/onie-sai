# SFP Ports

In SONiC SFP port handling is abstracted away through Python classes (e.g. in sfp_base.py and chassis_base.py), as well as in their specific platform implementation for every device.

We need to adopt a C-style library which provides similar functionality (we don't care about all of the features there as of now) that provides similar functionality.
We will load it as a dynamically loaded library per platform, and incorporate the functionality into port handling and information.

We want to port the following function list:

- supported_platform
- num_sfps
- get_presence
- get_port_or_cage_type
- get_inserted_media_type
- get_oper_status
- get_reset_status
- reset
- get_low_power_mode
- set_low_power_mode

as well as the bigger functions which come from the "sonic_xcvr" APIs:

```python
    def get_transceiver_info(self):
        """
        Retrieves transceiver info of this SFP

        Returns:
            A dict which contains following keys/values :
        ================================================================================
        keys                       |Value Format   |Information
        ---------------------------|---------------|----------------------------
        type                       |1*255VCHAR     |type of SFP
        type_abbrv_name            |1*255VCHAR     |type of SFP, abbreviated
        hardware_rev               |1*255VCHAR     |hardware version of SFP
        vendor_rev                 |1*255VCHAR     |vendor revision of SFP
        serial                     |1*255VCHAR     |serial number of the SFP
        manufacturer               |1*255VCHAR     |SFP vendor name
        model                      |1*255VCHAR     |SFP model name
        connector                  |1*255VCHAR     |connector information
        encoding                   |1*255VCHAR     |encoding information
        ext_identifier             |1*255VCHAR     |extend identifier
        ext_rateselect_compliance  |1*255VCHAR     |extended rateSelect compliance
        cable_length               |INT            |cable length in m
        nominal_bit_rate           |INT            |nominal bit rate by 100Mbs
        specification_compliance   |1*255VCHAR     |specification compliance
        vendor_date                |1*255VCHAR     |vendor date
        vendor_oui                 |1*255VCHAR     |vendor OUI
        application_advertisement  |1*255VCHAR     |supported applications advertisement
        ================================================================================
```

```python
    def get_transceiver_status(self):
        """
        Retrieves transceiver status of this SFP

        Returns:
            A dict which may contain following keys/values (there could be more for C-CMIS) :
        ================================================================================
        key                          = TRANSCEIVER_STATUS|ifname        ; Error information for module on port
        ; field                      = value
        module_state                 = 1*255VCHAR                       ; current module state (ModuleLowPwr, ModulePwrUp, ModuleReady, ModulePwrDn, Fault)
        module_fault_cause           = 1*255VCHAR                       ; reason of entering the module fault state
        datapath_firmware_fault      = BOOLEAN                          ; datapath (DSP) firmware fault
        module_firmware_fault        = BOOLEAN                          ; module firmware fault
        module_state_changed         = BOOLEAN                          ; module state changed
        datapath_hostlane1           = 1*255VCHAR                       ; data path state indicator on host lane 1
        datapath_hostlane2           = 1*255VCHAR                       ; data path state indicator on host lane 2
        datapath_hostlane3           = 1*255VCHAR                       ; data path state indicator on host lane 3
        datapath_hostlane4           = 1*255VCHAR                       ; data path state indicator on host lane 4
        datapath_hostlane5           = 1*255VCHAR                       ; data path state indicator on host lane 5
        datapath_hostlane6           = 1*255VCHAR                       ; data path state indicator on host lane 6
        datapath_hostlane7           = 1*255VCHAR                       ; data path state indicator on host lane 7
        datapath_hostlane8           = 1*255VCHAR                       ; data path state indicator on host lane 8
        txoutput_status              = BOOLEAN                          ; tx output status on media lane
        rxoutput_status_hostlane1    = BOOLEAN                          ; rx output status on host lane 1
        rxoutput_status_hostlane2    = BOOLEAN                          ; rx output status on host lane 2
        rxoutput_status_hostlane3    = BOOLEAN                          ; rx output status on host lane 3
        rxoutput_status_hostlane4    = BOOLEAN                          ; rx output status on host lane 4
        rxoutput_status_hostlane5    = BOOLEAN                          ; rx output status on host lane 5
        rxoutput_status_hostlane6    = BOOLEAN                          ; rx output status on host lane 6
        rxoutput_status_hostlane7    = BOOLEAN                          ; rx output status on host lane 7
        rxoutput_status_hostlane8    = BOOLEAN                          ; rx output status on host lane 8
        txfault                      = BOOLEAN                          ; tx fault flag on media lane
        txlos_hostlane1              = BOOLEAN                          ; tx loss of signal flag on host lane 1
        txlos_hostlane2              = BOOLEAN                          ; tx loss of signal flag on host lane 2
        txlos_hostlane3              = BOOLEAN                          ; tx loss of signal flag on host lane 3
        txlos_hostlane4              = BOOLEAN                          ; tx loss of signal flag on host lane 4
        txlos_hostlane5              = BOOLEAN                          ; tx loss of signal flag on host lane 5
        txlos_hostlane6              = BOOLEAN                          ; tx loss of signal flag on host lane 6
        txlos_hostlane7              = BOOLEAN                          ; tx loss of signal flag on host lane 7
        txlos_hostlane8              = BOOLEAN                          ; tx loss of signal flag on host lane 8
        txcdrlol_hostlane1           = BOOLEAN                          ; tx clock and data recovery loss of lock on host lane 1
        txcdrlol_hostlane2           = BOOLEAN                          ; tx clock and data recovery loss of lock on host lane 2
        txcdrlol_hostlane3           = BOOLEAN                          ; tx clock and data recovery loss of lock on host lane 3
        txcdrlol_hostlane4           = BOOLEAN                          ; tx clock and data recovery loss of lock on host lane 4
        txcdrlol_hostlane5           = BOOLEAN                          ; tx clock and data recovery loss of lock on host lane 5
        txcdrlol_hostlane6           = BOOLEAN                          ; tx clock and data recovery loss of lock on host lane 6
        txcdrlol_hostlane7           = BOOLEAN                          ; tx clock and data recovery loss of lock on host lane 7
        txcdrlol_hostlane8           = BOOLEAN                          ; tx clock and data recovery loss of lock on host lane 8
        rxlos                        = BOOLEAN                          ; rx loss of signal flag on media lane
        rxcdrlol                     = BOOLEAN                          ; rx clock and data recovery loss of lock on media lane
        config_state_hostlane1       = 1*255VCHAR                       ; configuration status for the data path of host line 1
        config_state_hostlane2       = 1*255VCHAR                       ; configuration status for the data path of host line 2
        config_state_hostlane3       = 1*255VCHAR                       ; configuration status for the data path of host line 3
        config_state_hostlane4       = 1*255VCHAR                       ; configuration status for the data path of host line 4
        config_state_hostlane5       = 1*255VCHAR                       ; configuration status for the data path of host line 5
        config_state_hostlane6       = 1*255VCHAR                       ; configuration status for the data path of host line 6
        config_state_hostlane7       = 1*255VCHAR                       ; configuration status for the data path of host line 7
        config_state_hostlane8       = 1*255VCHAR                       ; configuration status for the data path of host line 8
        dpinit_pending_hostlane1     = BOOLEAN                          ; data path configuration updated on host lane 1
        dpinit_pending_hostlane2     = BOOLEAN                          ; data path configuration updated on host lane 2
        dpinit_pending_hostlane3     = BOOLEAN                          ; data path configuration updated on host lane 3
        dpinit_pending_hostlane4     = BOOLEAN                          ; data path configuration updated on host lane 4
        dpinit_pending_hostlane5     = BOOLEAN                          ; data path configuration updated on host lane 5
        dpinit_pending_hostlane6     = BOOLEAN                          ; data path configuration updated on host lane 6
        dpinit_pending_hostlane7     = BOOLEAN                          ; data path configuration updated on host lane 7
        dpinit_pending_hostlane8     = BOOLEAN                          ; data path configuration updated on host lane 8
        temphighalarm_flag           = BOOLEAN                          ; temperature high alarm flag
        temphighwarning_flag         = BOOLEAN                          ; temperature high warning flag
        templowalarm_flag            = BOOLEAN                          ; temperature low alarm flag
        templowwarning_flag          = BOOLEAN                          ; temperature low warning flag
        vcchighalarm_flag            = BOOLEAN                          ; vcc high alarm flag
        vcchighwarning_flag          = BOOLEAN                          ; vcc high warning flag
        vcclowalarm_flag             = BOOLEAN                          ; vcc low alarm flag
        vcclowwarning_flag           = BOOLEAN                          ; vcc low warning flag
        txpowerhighalarm_flag        = BOOLEAN                          ; tx power high alarm flag
        txpowerlowalarm_flag         = BOOLEAN                          ; tx power low alarm flag
        txpowerhighwarning_flag      = BOOLEAN                          ; tx power high warning flag
        txpowerlowwarning_flag       = BOOLEAN                          ; tx power low alarm flag
        rxpowerhighalarm_flag        = BOOLEAN                          ; rx power high alarm flag
        rxpowerlowalarm_flag         = BOOLEAN                          ; rx power low alarm flag
        rxpowerhighwarning_flag      = BOOLEAN                          ; rx power high warning flag
        rxpowerlowwarning_flag       = BOOLEAN                          ; rx power low warning flag
        txbiashighalarm_flag         = BOOLEAN                          ; tx bias high alarm flag
        txbiaslowalarm_flag          = BOOLEAN                          ; tx bias low alarm flag
        txbiashighwarning_flag       = BOOLEAN                          ; tx bias high warning flag
        txbiaslowwarning_flag        = BOOLEAN                          ; tx bias low warning flag
        lasertemphighalarm_flag      = BOOLEAN                          ; laser temperature high alarm flag
        lasertemplowalarm_flag       = BOOLEAN                          ; laser temperature low alarm flag
        lasertemphighwarning_flag    = BOOLEAN                          ; laser temperature high warning flag
        lasertemplowwarning_flag     = BOOLEAN                          ; laser temperature low warning flag
        prefecberhighalarm_flag      = BOOLEAN                          ; prefec ber high alarm flag
        prefecberlowalarm_flag       = BOOLEAN                          ; prefec ber low alarm flag
        prefecberhighwarning_flag    = BOOLEAN                          ; prefec ber high warning flag
        prefecberlowwarning_flag     = BOOLEAN                          ; prefec ber low warning flag
        postfecberhighalarm_flag     = BOOLEAN                          ; postfec ber high alarm flag
        postfecberlowalarm_flag      = BOOLEAN                          ; postfec ber low alarm flag
        postfecberhighwarning_flag   = BOOLEAN                          ; postfec ber high warning flag
        postfecberlowwarning_flag    = BOOLEAN                          ; postfec ber low warning flag
        ================================================================================

        If there is an issue with reading the xcvr, None should be returned.
        """
        raise NotImplementedError
```

Furthermore, we should probably port the following types/constants:

```python
    # Device type definition. Note, this is a constant.
    DEVICE_TYPE = "sfp"

    # Generic error types definition
    SFP_STATUS_INITIALIZING                         = 'Initializing'
    SFP_STATUS_OK                                   = 'OK'
    SFP_STATUS_UNPLUGGED                            = 'Unplugged'
    SFP_STATUS_DISABLED                             = 'Disabled'
    SFP_ERROR_DESCRIPTION_BLOCKING                  = 'Blocking EEPROM from being read'
    SFP_ERROR_DESCRIPTION_POWER_BUDGET_EXCEEDED     = 'Power budget exceeded'
    SFP_ERROR_DESCRIPTION_I2C_STUCK                 = 'Bus stuck (I2C data or clock shorted)'
    SFP_ERROR_DESCRIPTION_BAD_EEPROM                = 'Bad or unsupported EEPROM'
    SFP_ERROR_DESCRIPTION_UNSUPPORTED_CABLE         = 'Unsupported cable'
    SFP_ERROR_DESCRIPTION_HIGH_TEMP                 = 'High temperature'
    SFP_ERROR_DESCRIPTION_BAD_CABLE                 = 'Bad cable (module/cable is shorted)'

    # SFP status
    SFP_STATUS_BIT_REMOVED                = 0x00000000
    SFP_STATUS_BIT_INSERTED               = 0x00000001
    # SFP error status
    SFP_ERROR_BIT_BLOCKING                = 0x00000002
    SFP_ERROR_BIT_POWER_BUDGET_EXCEEDED   = 0x00000004
    SFP_ERROR_BIT_I2C_STUCK               = 0x00000008
    SFP_ERROR_BIT_BAD_EEPROM              = 0x00000010
    SFP_ERROR_BIT_UNSUPPORTED_CABLE       = 0x00000020
    SFP_ERROR_BIT_HIGH_TEMP               = 0x00000040
    SFP_ERROR_BIT_BAD_CABLE               = 0x00000080

    SFP_ERROR_BIT_TO_DESCRIPTION_DICT = {
        SFP_ERROR_BIT_BLOCKING:                SFP_ERROR_DESCRIPTION_BLOCKING,
        SFP_ERROR_BIT_POWER_BUDGET_EXCEEDED:   SFP_ERROR_DESCRIPTION_POWER_BUDGET_EXCEEDED,
        SFP_ERROR_BIT_I2C_STUCK:               SFP_ERROR_DESCRIPTION_I2C_STUCK,
        SFP_ERROR_BIT_BAD_EEPROM:              SFP_ERROR_DESCRIPTION_BAD_EEPROM,
        SFP_ERROR_BIT_UNSUPPORTED_CABLE:       SFP_ERROR_DESCRIPTION_UNSUPPORTED_CABLE,
        SFP_ERROR_BIT_HIGH_TEMP:               SFP_ERROR_DESCRIPTION_HIGH_TEMP,
        SFP_ERROR_BIT_BAD_CABLE:               SFP_ERROR_DESCRIPTION_BAD_CABLE
    }

    # Port types that are used by the chassis API ChassisBase.get_port_or_cage_type()
    # It's possible that multiple types are supported on one port.
    # In that case, the result will be logical OR of all the supported types
    # Check example in ChassisBase.get_port_or_cage_type()
    SFP_PORT_TYPE_BIT_RJ45                = 0x00000001
    SFP_PORT_TYPE_BIT_SFP                 = 0x00000002
    SFP_PORT_TYPE_BIT_XFP                 = 0x00000004
    SFP_PORT_TYPE_BIT_SFP_PLUS            = 0x00000008
    SFP_PORT_TYPE_BIT_QSFP                = 0x00000010
    SFP_PORT_TYPE_BIT_CFP                 = 0x00000020
    SFP_PORT_TYPE_BIT_QSFP_PLUS           = 0x00000040
    SFP_PORT_TYPE_BIT_QSFP28              = 0x00000080
    SFP_PORT_TYPE_BIT_SFP28               = 0x00000100
    SFP_PORT_TYPE_BIT_CFP2                = 0x00000200
    SFP_PORT_TYPE_BIT_QSFP56              = 0x00000400
    SFP_PORT_TYPE_BIT_QSFPDD              = 0x00000800
    SFP_PORT_TYPE_BIT_OSFP                = 0x00001000
    SFP_PORT_TYPE_BIT_SFP_DD              = 0x00002000
```
