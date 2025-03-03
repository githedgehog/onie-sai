/**
 * Copyright (c) 2018 Microsoft Open Technologies, Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License"); you may
 *    not use this file except in compliance with the License. You may obtain
 *    a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 *    THIS CODE IS PROVIDED ON AN *AS IS* BASIS, WITHOUT WARRANTIES OR
 *    CONDITIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING WITHOUT
 *    LIMITATION ANY IMPLIED WARRANTIES OR CONDITIONS OF TITLE, FITNESS
 *    FOR A PARTICULAR PURPOSE, MERCHANTABILITY OR NON-INFRINGEMENT.
 *
 *    See the Apache Version 2.0 License for specific language governing
 *    permissions and limitations under the License.
 *
 *    Microsoft would like to thank the following companies for their review and
 *    assistance with these files: Intel Corporation, Mellanox Technologies Ltd,
 *    Dell Products, L.P., Facebook, Inc., Marvell International Ltd.
 *
 * @file    saiextensions.h
 *
 * @brief   This module defines extensions of the Switch Abstraction Interface (SAI)
 */

#ifndef __SAIEXTENSIONS_H_
#define __SAIEXTENSIONS_H_

#include <saitypes.h>

/* existing enum extensions */
#include "saitypesextensions.h"
#include "saiswitchextensions.h"
#include "saibridgeextensions.h"
#include "saiportextensions.h"
#include "saifdbextensions.h"
#include "saihostifextensions.h"
#include "sairouterinterfaceextensions.h"
#include "saivlanextensions.h"
#ifdef BRCM_SAI_ACL_ENHANCEMENTS_SUPPORT
#include "saiaclextensions.h"
#endif /* BRCM_SAI_ACL_ENHANCEMENTS_SUPPORT */
#ifdef BRCM_SAI_TELEMETRY_SUPPORT
#include "saitamextensions.h"
#endif /* BRCM_SAI_TELEMETRY_SUPPORT */
#include "saiipmcextensions.h"
#include "saiipmcgroupextensions.h"
#include "sainexthopextensions.h"
#include "sainexthopgroupextensions.h"

/* new experimental object type includes */
#include "saiexperimentalbmtor.h"
#include "saipolicerextensions.h"

#ifdef BRCM_SAI_DEBUG_ENHANCEMENTS_SUPPORT
#include "saidebugcounterextensions.h"
#endif /* BRCM_SAI_DEBUG_ENHANCEMENTS_SUPPORT */

#include "saibufferextensions.h"
#include "saiexperimentalvlanstack.h"

/**
 * @brief Extensions to SAI APIs
 *
 * @flags free
 */
typedef enum _sai_api_extensions_t
{
    SAI_API_EXTENSIONS_RANGE_START = SAI_API_MAX,

    SAI_API_BMTOR = SAI_API_EXTENSIONS_RANGE_START,

    SAI_API_VLAN_STACK,

    /* Add new experimental APIs above this line */

    SAI_API_EXTENSIONS_RANGE_END

} sai_api_extensions_t;

#endif /* __SAIEXTENSIONS_H_ */
