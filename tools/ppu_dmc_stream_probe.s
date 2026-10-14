; Repeated PPUDATA reads during looping DMC playback exercise all reload
; alignments. CHR bytes0..127 stream into the DAC, exposing skipped reads.
.ifndef EXTRA_NOPS
EXTRA_NOPS = 0
.endif
.segment "HEADER"
.byte "NES",$1a,2,1,0,0
.res 8,0
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
 lda #0
 sta $2006
 sta $2006
 lda #$4f
 sta $4010
 lda #$20
 sta $4012
 lda #1
 sta $4013
 lda #16
 sta $4015
restart:
 ldx #32
stream:
 lda $2007
 sta $4011
 .repeat EXTRA_NOPS
 nop
 .endrepeat
 dex
 bne stream
 lda #0
 sta $2006
 sta $2006
 jmp restart
irq:
 rti
.segment "SAMPLE"
.res 17,$ff
.segment "VECTORS"
.word irq,reset,irq
.segment "CHR"
.repeat $2000,I
.byte I & 127
.endrepeat
