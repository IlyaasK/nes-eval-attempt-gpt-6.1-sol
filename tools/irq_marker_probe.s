.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$00
 sta $4017
 ldx #$ff
 txs
 cli
 lda #$00
 jmp forever
.ifdef PAGE_CROSS
 .align 256
 .res 252,$ea
.endif
forever:
.ifdef BRANCH_LOOP
 cmp $01
 beq forever
.else
 jmp forever
.endif
irq:
 pha
 lda $00
 eor #96
 sta $00
 sta $4011
 lda $4015
 pla
 rti
.segment "VECTORS"
.word irq,reset,irq
