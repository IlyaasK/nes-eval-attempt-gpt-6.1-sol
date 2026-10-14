; Vary the PRG bus drive and selected CHR bank to identify conflict behavior.
.ifndef ROM_DRIVE
ROM_DRIVE = 0
.endif
.ifndef BANK_VALUE
BANK_VALUE = 3
.endif
.ifndef STREAM
STREAM = 0
.endif
.segment "HEADER"
.byte "NES",$1a,2,4,$30,0
.res 8,0
.segment "BANK0"
.if STREAM
.repeat 256,I
.byte (I >> 3) & 3
.endrepeat
.res $3f00,0
.else
.byte ROM_DRIVE
.res $3fff,0
.endif
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
.if STREAM
 ldy #0
stream:
 lda #BANK_VALUE
 sta $8000,y
 lda #0
 sta $2006
 sta $2006
 lda $2007
 lda $2007
 sta $4011
 iny
 jmp stream
.else
 lda #BANK_VALUE
 sta $8000
 lda #0
 sta $2006
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
.endif
irq:
 rti
.segment "VECTORS"
.word irq,reset,irq
.segment "CHR"
.res $2000,0
.res $2000,$11
.res $2000,$22
.res $2000,$33
