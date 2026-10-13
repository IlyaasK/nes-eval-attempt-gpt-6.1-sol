.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 ldx #$ff
 txs
 bit $2002
wait1:
 bit $2002
 bpl wait1
wait2:
 bit $2002
 bpl wait2
.ifdef RENDERING
 lda #$08
 sta $2001
.endif
 lda #$80
 sta $2000
.ifdef BRANCH_LOOP
 lda #$00
 jmp forever
.ifdef PAGE_CROSS
 .align 256
 .res 252,$ea
.endif
forever:
 cmp $01
 beq forever
.else
forever:
 jmp forever
.endif
nmi:
 pha
.ifdef OAM_DMA
 lda #$00
 sta $4014
.endif
 lda $00
 eor #96
 sta $00
 sta $4011
 pla
 rti
irq:
 rti
.segment "VECTORS"
.word nmi,reset,irq
