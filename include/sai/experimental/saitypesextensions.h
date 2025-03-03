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
 * @file    saitypesextensions.h
 *
 * @brief   This module defines type extensions of the Switch Abstraction Interface (SAI)
 */

#ifndef __SAITYPESEXTENSIONS_H_
#define __SAITYPESEXTENSIONS_H_

#include <saitypes.h>

/**
 * @brief SAI object type extensions
 *
 * @flags free
 */
typedef enum _sai_object_type_extensions_t
{
    SAI_OBJECT_TYPE_EXTENSIONS_RANGE_START = SAI_OBJECT_TYPE_MAX,

    SAI_OBJECT_TYPE_TABLE_BITMAP_CLASSIFICATION_ENTRY = SAI_OBJECT_TYPE_EXTENSIONS_RANGE_START,

    SAI_OBJECT_TYPE_TABLE_BITMAP_ROUTER_ENTRY,

    SAI_OBJECT_TYPE_TABLE_META_TUNNEL_ENTRY,

    SAI_OBJECT_TYPE_VLAN_STACK,

    /* Add new experimental object types above this line */

    SAI_OBJECT_TYPE_EXTENSIONS_RANGE_END

} sai_object_type_extensions_t;

/**
 * @brief Attribute data for SAI_TAM_TABLE_ATTR_BIND_POINT extensions
 *
 * @flags free
 */
typedef enum _sai_tam_bind_point_type_extensions_t
{
    SAI_TAM_BIND_POINT_TYPE_EXTENSIONS_RANGE_START = SAI_TAM_BIND_POINT_TYPE_BSP,

    SAI_TAM_BIND_POINT_TYPE_DEVICE,

    SAI_TAM_BIND_POINT_TYPE_PORT_POOL,

    /* Add new experimental tam bind point types above this line */

    SAI_TAM_BIND_POINT_TYPE_EXTENSIONS_RANGE_END

} sai_tam_bind_point_type_extensions_t;

/**
 * @brief SAI ACL Stage extensions
 *
 * @flags free
 */
typedef enum _sai_acl_stage_extensions_t
{
    SAI_ACL_STAGE_EXTENSIONS_RANGE_START = SAI_ACL_STAGE_PRE_INGRESS + 0x1,

    SAI_ACL_STAGE_PRE_INGRESS_EM = SAI_ACL_STAGE_EXTENSIONS_RANGE_START,

    /* Add new experimental ACL stage above this line */

    SAI_ACL_STAGE_EXTENSIONS_RANGE_END

} sai_acl_stage_extensions_t;

#endif /* __SAITYPESEXTENSIONS_H_ */

