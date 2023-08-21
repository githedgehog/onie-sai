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
#define LED_MAGIC               0x504c /* i.e. PL */
#define LED_INTFS_NUM           LED_HW_INTF_MAX_NUM
#define LED_PORTS_NUM           512
#define LED_SHMEM_BASE          0x39f0 /* i.e. 0x3800 + 0x1f0(496 bytes) */
#define LED_TICKS_SHIFT         5

#define ACCU_MEM16(ctrl, idx)   *(uint16 *)((ctrl)->accu_ram_base + ((idx) << 2))
#define PATT_MEM16(ctrl, idx)   *(uint16 *)((ctrl)->pat_ram_base  + ((idx) << 2))

/* Physical Port ID --> LED Pattern Index */
#define LED_PMAP16(ctrl, idx)   *(uint16 *)((ctrl)->pat_ram_base +  ((512 + (idx)) << 2))

typedef struct pl_conf_s
{
    unsigned tail     : 10; /* Tail/end of the LED port (BIT 0 - BIT 9) */
    unsigned head     : 10; /* Head/1st of the LED port (BIT10 - BIT19) */
    unsigned rsvd     : 6;  /* Reserved                 (BIT20 - BIT25) */
    unsigned bits     : 5;  /* Number of Bits Per Port  (BIT26 - BIT30) */
    unsigned valid    : 1;  /* Enable                   (BIT31) */
} pl_conf_t __attribute__ ((aligned (4)));

typedef struct pl_patt_s {
    uint16 led_on;  /* LED ON */
    uint16 led_off; /* LED OFF */
} pl_patt_t __attribute__ ((aligned (4)));

typedef struct pl_ctrl_s
{
    uint16    magic;
    uint16    length;
    pl_conf_t conf[LED_INTFS_NUM];
    pl_patt_t patt[LED_INTFS_NUM];
} pl_ctrl_t __attribute__ ((aligned (4)));

typedef struct pl_dbg_s
{
    uint16    magic;
    uint16    length;
    uint32    ctrl_base;
    uint32    activities;
    uint32    rsvd;
} pl_dbg_t __attribute__ ((aligned (16)));

static pl_dbg_t *led_dbg = (void *)(LED_SHMEM_BASE);
static pl_ctrl_t *led_ctrl = (void *)(LED_SHMEM_BASE + sizeof(pl_dbg_t));

void custom_led_handler(soc_led_custom_handler_ctrl_t *ctrl, uint32 activities)
{
    int i = 0, tick = activities >> LED_TICKS_SHIFT;

    led_dbg->magic = LED_MAGIC;
    led_dbg->length = sizeof(*led_dbg);
    led_dbg->ctrl_base = (uint32)ctrl;
    led_dbg->activities = activities;
    led_dbg->rsvd = 0;

    if ((led_ctrl->magic != LED_MAGIC) || (led_ctrl->length < sizeof(pl_ctrl_t)))
    {
        PATT_MEM16(ctrl, 0) = 0xdead;
        PATT_MEM16(ctrl, 1) = 0xbeef;
        for (i = 0; i < LED_INTFS_NUM; ++i)
        {
            ctrl->intf_ctrl[i].valid = 0;
        }
    }
    else
    {
        /* Both 'led' and 'phy' begin with 1 */
        int led, phy, pid, blink;
        uint16 lit, off;

        /* Turn OFF the LED of all ports */
        for (phy = 1; phy <= LED_PORTS_NUM; ++phy)
        {
            led = LED_PMAP16(ctrl, phy - 1) & 0x03ff;      /* BIT  0-9  (LED Port ID) */
            pid = (LED_PMAP16(ctrl, phy - 1) >> 10) & 0x3; /* BIT 10-12 (LED Pattern ID) */
            if (led == 0)
            {
                continue;
            }

            PATT_MEM16(ctrl, led - 1) = led_ctrl->patt[pid].led_off;
        }

        /* Turn on the LED if the link is up */
        for (phy = 1; phy <= LED_PORTS_NUM; ++phy)
        {
            led = LED_PMAP16(ctrl, phy - 1) & 0x03ff;      /* BIT  0-9  (LED Port ID) */
            pid = (LED_PMAP16(ctrl, phy - 1) >> 10) & 0x3; /* BIT 10-12 (LED Pattern ID) */
            blink = LED_PMAP16(ctrl, phy - 1) & 0x8000;    /* BIT 15    (LED Blink) */
            if (led == 0)
            {
                continue;
            }

            lit = led_ctrl->patt[pid].led_on;
            off = led_ctrl->patt[pid].led_off;
            if (ACCU_MEM16(ctrl, phy - 1) & LED_HW_LINK_UP)
            {
                if (blink > 0)
                {
                    PATT_MEM16(ctrl, led - 1) = (tick & 0x01) ? lit : off;
                }
                else
                {
                    PATT_MEM16(ctrl, led - 1) = lit;
                }
            }
        }

        for (i = 0; i < LED_INTFS_NUM; ++i)
        {
            ctrl->intf_ctrl[i].start_row = led_ctrl->conf[i].head;
            ctrl->intf_ctrl[i].end_row   = led_ctrl->conf[i].tail;
            ctrl->intf_ctrl[i].pat_width = led_ctrl->conf[i].bits;
            ctrl->intf_ctrl[i].valid     = led_ctrl->conf[i].valid;
        }
    }

}

