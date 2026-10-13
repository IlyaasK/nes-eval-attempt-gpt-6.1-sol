.segment "HEADER"
.byte "NES",$1a,2,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 lda #$4f
 sta $4010
 lda #$00
 sta $4011
 lda #$20
 sta $4012
 lda #$01
 sta $4013
 lda #$10
 sta $4015
 lda #$7f
 sta $4011
.ifdef OAM_DMA
.ifndef DELAY_CYCLES
DELAY_CYCLES = 401
.endif
RAW_COUNT = (DELAY_CYCLES-1)/5
RAW_REMAINDER = DELAY_CYCLES-(5*RAW_COUNT+1)
.if RAW_REMAINDER = 1
DELAY_COUNT = RAW_COUNT-1
DELAY_REMAINDER = 6
.else
DELAY_COUNT = RAW_COUNT
DELAY_REMAINDER = RAW_REMAINDER
.endif
 ldx #DELAY_COUNT
delay_oam:
 dex
 bne delay_oam
.if DELAY_REMAINDER & 1
 bit $00
 .repeat (DELAY_REMAINDER-3)/2
 nop
 .endrepeat
.else
 .repeat DELAY_REMAINDER/2
 nop
 .endrepeat
.endif
 lda #2
 sta $4014
 lda #$7e
 sta $4011
.endif
loop:
 jmp loop
irq:
 rti
.segment "SAMPLE"
.res 17,$ff
.segment "VECTORS"
.word irq,reset,irq
