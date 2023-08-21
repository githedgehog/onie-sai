/**********************************************************************
 *
 * Copyright 2021 Broadcom
 *
 **********************************************************************
 *
 * @filename   port_locator.c
 *
 * @purpose    A generic firmware for the port locator
 *
 * @author     Dante (Kuo-Jung) Su
 *
 * @end
 *
 *********************************************************************/
#include <shared/cmicfw/cmicx_led_public.h>

/*
 * CMICX LED Memory Map
 *
 * 0x0000 - 0x37ff: linkscan.bin: TEXT + DATA
 * 0x3800 - 0x3ffb: custom_led.bin (2044 bytes)
 * 0x4000 - 0x7fff: linkscan.bin: BSS + STACK
 * 0x9000 - 0x9fff: ACCU RAM (16-bits word)
 * 0xa000 - 0xafff: PATT RAM (16-bits word)
 * 0xb000 - 0xb0b7: CPU-Accessible Registers (e.g. SRAM_CTRL, CNFG_LINK...)
 */

#define LED_PORTS_NUM           128
#define LED_INTFS_NUM           1
#define LED_LOCATOR_F           0x8000

#define LED_SOLID_ON            3
#define LED_SOLID_OFF           0
#define LED_BLINK               2

#define ACCU_MEM16(ctrl, idx)   *(uint16 *)((ctrl)->accu_ram_base + (((idx) - 1) << 2))
#define PATT_MEM16(ctrl, idx)   *(uint16 *)((ctrl)->pat_ram_base  + (((idx) - 1) << 2))
#define LED_SMEM16(ctrl, idx)   *(uint16 *)((ctrl)->pat_ram_base +  ((511 + (idx)) << 2))

void custom_led_handler(soc_led_custom_handler_ctrl_t *ctrl, uint32 activities)
{
    int i, phy;

    for (phy = 1; phy <= LED_PORTS_NUM; ++phy)
    {
        /* BLINK if the locator mode is set */
        if (LED_SMEM16(ctrl, phy) & LED_LOCATOR_F)
        {
            PATT_MEM16(ctrl, phy) = LED_BLINK;
        }
        /* SOLID ON if the link is up */
        else if (ACCU_MEM16(ctrl, phy) & LED_HW_LINK_UP)
        {
            PATT_MEM16(ctrl, phy) = LED_SOLID_ON;
        }
        /* SOLID OFF if the link is down */
        else
        {
            PATT_MEM16(ctrl, phy) = LED_SOLID_OFF;
        }
    }

    for (i = 0; i < LED_INTFS_NUM; ++i)
    {
        ctrl->intf_ctrl[i].start_row = 0;
        ctrl->intf_ctrl[i].end_row   = LED_PORTS_NUM - 1;
        ctrl->intf_ctrl[i].pat_width = 2;
        ctrl->intf_ctrl[i].valid     = 1;
    }
}
