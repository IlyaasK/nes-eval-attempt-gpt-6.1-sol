.ifndef INDEX
INDEX = 0
.endif
.ifndef PAGE
PAGE = $40
.endif
.segment "HEADER"
.ifdef DMC
.byte "NES",$1a,2,0,0,0
.else
.byte "NES",$1a,1,0,0,0
.endif
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 .ifdef DMC
 lda #$4f
 sta $4010
 lda #127
 sta $4011
 lda #$20
 sta $4012
 lda #1
 sta $4013
 lda #16
 sta $4015
 .endif
 lda #1
 sta $4016
 .ifdef DMC
 ldx #120
delay_dma:
 dex
 bne delay_dma
 .endif
 lda #PAGE
 sta $4014
.ifdef HALT_NOP
 nop
.endif
 lda #INDEX
 sta $2003
 lda $2004
 sta $00
 bit $2002
wait1:
 bit $2002
 bpl wait1
wait2:
 bit $2002
 bpl wait2
 lda $00
 and #$c0
 sta $2001
 lda #$3f
 sta $2006
 lda #0
 sta $2006
 lda $00
 and #$3f
 sta $2007
 lda #$3f
 sta $2006
 lda #0
 sta $2006
forever:
 jmp forever
irq:
 rti
.ifdef DMC
.segment "SAMPLE"
.res 17,$ff
.endif
.segment "VECTORS"
.word irq,reset,irq
