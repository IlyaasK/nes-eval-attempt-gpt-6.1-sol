; Bank1 select: ordinary MMC1 reads its own reset opcode at $8000;
; an unbanked SEROM board would still read $11 from the low PRG bank.
.segment "HEADER"
.byte "NES",$1a,2,1,$10,0
.res 8,0
.segment "BANK0"
.res $4000,$11
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
 lda #1
 sta $e000
 lda #0
 .repeat 4
 sta $e000
 .endrepeat
 lda $8000
 sta $4011
 tax
 lda #$3f
 sta $2006
 lda #0
 sta $2006
 txa
 sta $2007
 lda #$3f
 sta $2006
 lda #0
 sta $2006
forever:
 jmp forever
irq:
 rti
.segment "VECTORS"
.word irq,reset,irq
.segment "CHR"
.res $2000,0
