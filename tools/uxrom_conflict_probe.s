; Bank zero drives zero during the write. Bus conflicts select zero;
; an unrestricted bank write selects bank two ($22). Code runs in fixed PRG.
.segment "HEADER"
.byte "NES",$1a,4,0,$20,0
.res 8,0
.segment "BANK0"
.res $4000,0
.segment "BANK1"
.res $4000,$11
.segment "BANK2"
.res $4000,$22
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
 lda #2
 sta $8000
 lda $8000
 sta $6000
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
