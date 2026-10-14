; An explicit NES2 CHR-RAM size may mirror $1000 onto $0000.
.ifndef CHR_SHIFT
CHR_SHIFT = 6
.endif
.segment "HEADER"
.byte "NES",$1a,1,0,0,8,0,0,0,0,CHR_SHIFT,0,0,0
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
 lda #$22
 sta $2007
 lda #$10
 sta $2006
 lda #0
 sta $2006
 lda $2007
 lda $2007
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
