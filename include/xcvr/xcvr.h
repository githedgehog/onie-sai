#ifndef __XCVR_H_
#define __XCVR_H_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#define _In_
#define _Out_
#define _Inout_

/**
 * @brief All functions defined by this module have this return type.
 * 
 * Unless otherwise specified, 0 denotes success, and a negative integer
 * denotes an error whose error is defined in a list below, or a vendor
 * specific error code which is potentially not listed here.
*/
typedef int32_t xcvr_status_t;

/**
 * @brief Whenever we want to refer of a numbered/indexed port we use this type.
*/
typedef uint16_t idx_t;

/** Status success */
#define XCVR_STATUS_SUCCESS                         (0x00000000L)
/** Status error: General error */
#define XCVR_STATUS_ERROR_GENERAL                  (-0x00000001L)
/** Status error: Blocking EEPROM from being read */
#define XCVR_STATUS_ERROR_BLOCKING                 (-0x00000002L)
/** Status error: Power budget exceeded */
#define XCVR_STATUS_ERROR_POWER_BUDGET_EXCEEDED    (-0x00000004L)
/** Status error: Bus stuck (I2C data or clock shorted) */
#define XCVR_STATUS_ERROR_I2C_STUCK                (-0x00000008L)
/** Status error: Bad or unsupported EEPROM */
#define XCVR_STATUS_ERROR_BAD_EEPROM               (-0x00000010L)
/** Status error: Unsupported cable */
#define XCVR_STATUS_ERROR_UNSUPPORTED_CABLE        (-0x00000020L)
/** Status error: High temperature */
#define XCVR_STATUS_ERROR_HIGH_TEMP                (-0x00000040L)
/** Status error: Bad cable (module/cable is shorted) */
#define XCVR_STATUS_ERROR_BAD_CABLE                (-0x00000080L)
/** Status error: Unsupported platform */
#define XCVR_STATUS_ERROR_UNSUPPORTED_PLATFORM     (-0x00000100L)
/** Status error: Unimplemented */
#define XCVR_STATUS_ERROR_UNIMPLEMENTED            (-0x80000000L)

/**
 * @brief The port types of a transceiver. These map to the SONiC
 * SFP_PORT_TYPE_BIT_ types.
*/
typedef enum _xcvr_port_type_t {
    XCVR_PORT_TYPE_RJ45                = 0x00000001,
    XCVR_PORT_TYPE_SFP                 = 0x00000002,
    XCVR_PORT_TYPE_XFP                 = 0x00000004,
    XCVR_PORT_TYPE_SFP_PLUS            = 0x00000008,
    XCVR_PORT_TYPE_QSFP                = 0x00000010,
    XCVR_PORT_TYPE_CFP                 = 0x00000020,
    XCVR_PORT_TYPE_QSFP_PLUS           = 0x00000040,
    XCVR_PORT_TYPE_QSFP28              = 0x00000080,
    XCVR_PORT_TYPE_SFP28               = 0x00000100,
    XCVR_PORT_TYPE_CFP2                = 0x00000200,
    XCVR_PORT_TYPE_QSFP56              = 0x00000400,
    XCVR_PORT_TYPE_QSFPDD              = 0x00000800,
    XCVR_PORT_TYPE_OSFP                = 0x00001000,
    XCVR_PORT_TYPE_SFP_DD              = 0x00002000,
} xcvr_port_type_t;

/**
 * @brief The transceiver information
 * 
 * This datastructure has been ported from the SONiC python dictionary
 * suggestion. While this is not ideal to have such a large struct, we'll be
 * able to adjust this down the road.
*/
typedef struct _xcvr_transceiver_info_t {
    /** type of SFP */
    char type[255];
    /** type of SFP, abbreviated */
    char type_abbrv_name[255];
    /** hardware version of SFP */
    char hardware_rev[255];
    /** vendor revision of SFP */
    char vendor_rev[255];
    /** serial number of the SFP */
    char serial[255];
    /** SFP vendor name */
    char manufacturer[255];
    /** SFP model name */
    char model[255];
    /** connector information */
    char connector[255];
    /** encoding information */
    char encoding[255];
    /** extend identifier */
    char ext_identifier[255];
    /** extended rateSelect compliance */
    char ext_rateselect_compliance[255];
    /** cable length in m */
    uint32_t cable_length;
    /** nominal bit rate by 100Mbs */
    uint32_t nominal_bit_rate;
    /** specification compliance */
    char specification_compliance[255];
    /** vendor date */
    char vendor_date[255];
    /** vendor OUI */
    char vendor_oui[255];
    /** supported applications advertisement */
    char application_advertisement[255];
} xcvr_transceiver_info_t;

/**
 * @brief The transceiver status information
 * 
 * This datastructure has been ported from the SONiC python dictionary
 * suggestion. While this is not ideal to have such a large struct, we'll be
 * able to adjust this down the road.
*/
typedef struct _xcvr_transceiver_status_t {
        /** current module state (ModuleLowPwr, ModulePwrUp, ModuleReady, ModulePwrDn, Fault) */
        char module_state[255];
        /** reason of entering the module fault state */
        char module_fault_cause[255];
        /** datapath (DSP) firmware fault */
        bool datapath_firmware_fault;
        /** module firmware fault */
        bool module_firmware_fault;
        /** module state changed */
        bool module_state_changed;
        /** data path state indicator on host lane 1 */
        char datapath_hostlane1[255];
        /** data path state indicator on host lane 2 */
        char datapath_hostlane2[255];
        /** data path state indicator on host lane 3 */
        char datapath_hostlane3[255];
        /** data path state indicator on host lane 4 */
        char datapath_hostlane4[255];
        /** data path state indicator on host lane 5 */
        char datapath_hostlane5[255];
        /** data path state indicator on host lane 6 */
        char datapath_hostlane6[255];
        /** data path state indicator on host lane 7 */
        char datapath_hostlane7[255];
        /** data path state indicator on host lane 8 */
        char datapath_hostlane8[255];
        /** tx output status on media lane */
        bool txoutput_status;
        /** rx output status on host lane 1 */
        bool rxoutput_status_hostlane1;
        /** rx output status on host lane 2 */
        bool rxoutput_status_hostlane2;
        /** rx output status on host lane 3 */
        bool rxoutput_status_hostlane3;
        /** rx output status on host lane 4 */
        bool rxoutput_status_hostlane4;
        /** rx output status on host lane 5 */
        bool rxoutput_status_hostlane5;
        /** rx output status on host lane 6 */
        bool rxoutput_status_hostlane6;
        /** rx output status on host lane 7 */
        bool rxoutput_status_hostlane7;
        /** rx output status on host lane 8 */
        bool rxoutput_status_hostlane8;
        /** tx fault flag on media lane */
        bool txfault;
        /** tx loss of signal flag on host lane 1 */
        bool txlos_hostlane1;
        /** tx loss of signal flag on host lane 2 */
        bool txlos_hostlane2;
        /** tx loss of signal flag on host lane 3 */
        bool txlos_hostlane3;
        /** tx loss of signal flag on host lane 4 */
        bool txlos_hostlane4;
        /** tx loss of signal flag on host lane 5 */
        bool txlos_hostlane5;
        /** tx loss of signal flag on host lane 6 */
        bool txlos_hostlane6;
        /** tx loss of signal flag on host lane 7 */
        bool txlos_hostlane7;
        /** tx loss of signal flag on host lane 8 */
        bool txlos_hostlane8;
        /** tx clock and data recovery loss of lock on host lane 1 */
        bool txcdrlol_hostlane1;
        /** tx clock and data recovery loss of lock on host lane 2 */
        bool txcdrlol_hostlane2;
        /** tx clock and data recovery loss of lock on host lane 3 */
        bool txcdrlol_hostlane3;
        /** tx clock and data recovery loss of lock on host lane 4 */
        bool txcdrlol_hostlane4;
        /** tx clock and data recovery loss of lock on host lane 5 */
        bool txcdrlol_hostlane5;
        /** tx clock and data recovery loss of lock on host lane 6 */
        bool txcdrlol_hostlane6;
        /** tx clock and data recovery loss of lock on host lane 7 */
        bool txcdrlol_hostlane7;
        /** tx clock and data recovery loss of lock on host lane 8 */
        bool txcdrlol_hostlane8;
        /** rx loss of signal flag on media lane */
        bool rxlos;
        /** rx clock and data recovery loss of lock on media lane */
        bool rxcdrlol;
        /** configuration status for the data path of host line 1 */
        char config_state_hostlane1[255];
        /** configuration status for the data path of host line 2 */
        char config_state_hostlane2[255];
        /** configuration status for the data path of host line 3 */
        char config_state_hostlane3[255];
        /** configuration status for the data path of host line 4 */
        char config_state_hostlane4[255];
        /** configuration status for the data path of host line 5 */
        char config_state_hostlane5[255];
        /** configuration status for the data path of host line 6 */
        char config_state_hostlane6[255];
        /** configuration status for the data path of host line 7 */
        char config_state_hostlane7[255];
        /** configuration status for the data path of host line 8 */
        char config_state_hostlane8[255];
        /** data path configuration updated on host lane 1 */
        bool dpinit_pending_hostlane1;
        /** data path configuration updated on host lane 2 */
        bool dpinit_pending_hostlane2;
        /** data path configuration updated on host lane 3 */
        bool dpinit_pending_hostlane3;
        /** data path configuration updated on host lane 4 */
        bool dpinit_pending_hostlane4;
        /** data path configuration updated on host lane 5 */
        bool dpinit_pending_hostlane5;
        /** data path configuration updated on host lane 6 */
        bool dpinit_pending_hostlane6;
        /** data path configuration updated on host lane 7 */
        bool dpinit_pending_hostlane7;
        /** data path configuration updated on host lane 8 */
        bool dpinit_pending_hostlane8;
        /** temperature high alarm flag */
        bool temphighalarm_flag;
        /** temperature high warning flag */
        bool temphighwarning_flag;
        /** temperature low alarm flag */
        bool templowalarm_flag;
        /** temperature low warning flag */
        bool templowwarning_flag;
        /** vcc high alarm flag */
        bool vcchighalarm_flag;
        /** vcc high warning flag */
        bool vcchighwarning_flag;
        /** vcc low alarm flag */
        bool vcclowalarm_flag;
        /** vcc low warning flag */
        bool vcclowwarning_flag;
        /** tx power high alarm flag */
        bool txpowerhighalarm_flag;
        /** tx power low alarm flag */
        bool txpowerlowalarm_flag;
        /** tx power high warning flag */
        bool txpowerhighwarning_flag;
        /** tx power low alarm flag */
        bool txpowerlowwarning_flag;
        /** rx power high alarm flag */
        bool rxpowerhighalarm_flag;
        /** rx power low alarm flag */
        bool rxpowerlowalarm_flag;
        /** rx power high warning flag */
        bool rxpowerhighwarning_flag;
        /** rx power low warning flag */
        bool rxpowerlowwarning_flag;
        /** tx bias high alarm flag */
        bool txbiashighalarm_flag;
        /** tx bias low alarm flag */
        bool txbiaslowalarm_flag;
        /** tx bias high warning flag */
        bool txbiashighwarning_flag;
        /** tx bias low warning flag */
        bool txbiaslowwarning_flag;
        /** laser temperature high alarm flag */
        bool lasertemphighalarm_flag;
        /** laser temperature low alarm flag */
        bool lasertemplowalarm_flag;
        /** laser temperature high warning flag */
        bool lasertemphighwarning_flag;
        /** laser temperature low warning flag */
        bool lasertemplowwarning_flag;
        /** prefec ber high alarm flag */
        bool prefecberhighalarm_flag;
        /** prefec ber low alarm flag */
        bool prefecberlowalarm_flag;
        /** prefec ber high warning flag */
        bool prefecberhighwarning_flag;
        /** prefec ber low warning flag */
        bool prefecberlowwarning_flag;
        /** postfec ber high alarm flag */
        bool postfecberhighalarm_flag;
        /** postfec ber low alarm flag */
        bool postfecberlowalarm_flag;
        /** postfec ber high warning flag */
        bool postfecberhighwarning_flag;
        /** postfec ber low warning flag */
        bool postfecberlowwarning_flag;
} xcvr_transceiver_status_t;

/**
 * @brief Identifies the implementing library.
 * 
 * As multiple platforms could be supported by the same library this helps the
 * consumer of these libraries by preventing to load the same library multiple
 * times if not needed.
*/
const char *xcvr_library_name();

/**
 * @brief Checks if this library supports the platform "platform".
*/
bool xcvr_is_supported_platform(_In_ const char *platform);

/**
 * @brief Returns a list of all supported platforms by this library.
 * 
 * As multiple platforms could be supported by the same library, this helps the
 * consumer of these libraries by preventing to load the same library multiple
 * times if not needed.
*/
void xcvr_supported_platforms(
    _Out_ const char **supported_platforms,
    _Out_ size_t *supported_platforms_count
);

/**
 * @brief Returns the number of total physical ports for this platform.
 * 
 * This call makes no assumptions if modules are inserted or not, or if the
 * platform even has removable modules at all. It essentially should return
 * the total number of physical ports of the platform.
*/
xcvr_status_t xcvr_num_physical_ports(
    _In_ const char *platform,
    _Out_ idx_t *num
);

/**
 * @brief Tests a physical port if a transceiver is present/inserted or not.
 * 
 * This call makes no assumption if the transceiver is operational at all.
 * A cable might not even be plugged in, and the call would still return true
 * for as long as the module itself is inserted.
*/
xcvr_status_t xcvr_get_presence(
    _In_ const char *platform,
    _In_ idx_t index,
    _Out_ bool *is_present
);

/**
 * @brief Returns all supported transceiver port types for the physical port.
 * 
 * Note that the supported_port_types is a mask and will contain all supported
 * port types.
*/
xcvr_status_t xcvr_get_supported_port_types(
    _In_ const char *platform,
    _In_ idx_t index,
    _Out_ xcvr_port_type_t *supported_port_types
);

/**
 * @brief Returns the transceiver port type of the transceiver that is inserted
 * into the physical port right now.
 * 
 * Note that this will not return a mask like the supported types call.
*/
xcvr_status_t xcvr_get_inserted_port_type(
    _In_ const char *platform,
    _In_ idx_t index,
    _Out_ xcvr_port_type_t *supported_port_types
);

/**
 * @brief Returns the operational status of the transceiver.
 * 
 * This is from the view of the SFP module. This does not mean that the port
 * is functional.
*/
xcvr_status_t xcvr_get_oper_status(
    _In_ const char *platform,
    _In_ idx_t index,
    _Out_ bool *oper_status
);

/**
 * @brief Returns the reset status of the transceiver.
 * 
 * This is from the view of the SFP module. When reset status is true, then
 * operational status should be false and vice versa.
*/
xcvr_status_t xcvr_get_reset_status(
    _In_ const char *platform,
    _In_ idx_t index,
    _Out_ bool *reset_status
);

/**
 * @brief Performs a reset of the SFP module, and all settings will be reset
 * to driver defaults.
*/
xcvr_status_t xcvr_reset(
    _In_ const char *platform,
    _In_ idx_t index
);

/**
 * @brief Returns if the transceiver is running in low power mode
*/
xcvr_status_t xcvr_get_low_power_mode(
    _In_ const char *platform,
    _In_ idx_t index,
    _Out_ bool *low_power_mode
);

/**
 * @brief Sets the low power mode of the transceiver to on or off
*/
xcvr_status_t xcvr_set_low_power_mode(
    _In_ const char *platform,
    _In_ idx_t index,
    _In_ bool low_power_mode
);

/**
 * @brief Returns the transceiver info
*/
xcvr_status_t xcvr_get_transceiver_info(
    _In_ const char *platform,
    _In_ idx_t index,
    _Out_ xcvr_transceiver_info_t *transceiver_info
);

/**
 * @brief Returns the transceiver status
*/
xcvr_status_t xcvr_get_transceiver_status(
    _In_ const char *platform,
    _In_ idx_t index,
    _Out_ xcvr_transceiver_status_t *transceiver_status
);

#endif /* __XCVR_H_ */
