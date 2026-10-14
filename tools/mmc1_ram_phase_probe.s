; Sample $6000 while rendering switches SOROM between two RAM banks. DAC
; levels $11/$22 expose PPU address-pin timing without changing CPU cycles.
.ifndef CONTROL
CONTROL = 28
.endif
.ifndef PPU_CONTROL
PPU_CONTROL = $10
.endif
.ifndef READ_DELAY
READ_DELAY = 0
.endif
.ifndef CHR_HIGH
CHR_HIGH = 8
.endif
.ifndef RAM_SHIFT
RAM_SHIFT = 8
.endif
.macro serial addr,value
 .repeat 5,bit
 lda #((value >> bit) & 1)
 sta addr
 .endrepeat
.endmacro
.segment "HEADER"
.byte "NES",$1a,4,0,$10,8,0,0,RAM_SHIFT,0,0,0,0,0
.segment "BANK0"
.res $4000,0
.segment "BANK1"
.res $4000,0
.segment "BANK2"
.res $4000,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 ldy #24
outer:
 ldx #255
inner:
 dex
 bne inner
 dey
 bne outer
 serial $8000,28
 serial $a000,0
 serial $c000,CHR_HIGH
 lda #0
 sta $2006
 sta $2006
 lda #$11
 sta $6000
 lda #$10
 sta $2006
 lda #0
 sta $2006
 lda #$22
 sta $6000
 lda #0
 sta $2006
 sta $2006
 serial $8000,CONTROL
 lda #PPU_CONTROL
 sta $2000
 lda #8
 sta $2001
 .repeat READ_DELAY
 nop
 .endrepeat
forever:
 lda $6000
 sta $4011
 jmp forever
irq:
 rti
.segment "VECTORS"
.word irq,reset,irq
