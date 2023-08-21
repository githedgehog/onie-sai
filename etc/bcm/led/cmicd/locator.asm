; 2-bits per port, and 32 ports per LEDuP
LED_BITS  equ   64
NUM_PORT  equ   32

; variables in memory buffer
port_st   equ   0xA0

; code
start:
	ld  a, 0

loop:
	ld  b, port_st
	add b, a
	tst (b), 6        ; locator=BIT6, have the LED BLINK if set
	jc  led_blink
	tst (b), 0        ; link-up=BIT0, have the LED ON if set
	jc  led_on
	jnc led_off

next:
	inc a
	cmp a, NUM_PORT
	jc  loop          ; if a < NUM_PORT, goto 'loop'

done:
	send LED_BITS

; LED_ON=11
led_on:
	stc
	push cy
	pack
	push cy
	pack
	jmp next

; LED_OFF=00
led_off:
	clc
	push cy
	pack
	push cy
	pack
	jmp next

; LED_OFF=10
led_blink:
	clc
	push cy
	pack
	stc
	push cy
	pack
	jmp next
