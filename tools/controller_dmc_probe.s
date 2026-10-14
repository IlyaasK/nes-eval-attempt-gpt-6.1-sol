.ifndef EXTRA_DELAY
EXTRA_DELAY = 0
.endif
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
 lda #127
 sta $4011
 lda #$20
 sta $4012
 .ifdef SHADOW
 lda #2
 .else
 lda #1
 .endif
 sta $4013
 lda #16
 sta $4015
 lda #1
 sta $4016
 lda #0
 sta $4016
 .ifdef SHADOW
 ldy #7
delay_outer:
 ldx #255
delay_inner:
 dex
 bne delay_inner
 dey
 bne delay_outer
 ldx #152
 .else
 ldx #148
 .endif
delay_read:
 dex
 bne delay_read
 bit $00
.repeat EXTRA_DELAY
 nop
.endrepeat
 lda $4016
 and #1
 sta $00
 lda $4016
 and #1
 asl a
 ora $00
 sta $00
 bit $2002
wait1:
 bit $2002
 bpl wait1
wait2:
 bit $2002
 bpl wait2
 lda #$3f
 sta $2006
 lda #0
 sta $2006
 lda $00
 clc
 adc #$20
 sta $2007
 lda #$3f
 sta $2006
 lda #0
 sta $2006
forever:
 jmp forever
irq:
 rti
.segment "SAMPLE"
.ifdef SHADOW
.res 33,$ff
.else
.res 17,$ff
.endif
.segment "VECTORS"
.word irq,reset,irq
