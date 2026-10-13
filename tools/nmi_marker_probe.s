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
 lda #$80
 sta $2000
forever:
 jmp forever
nmi:
 pha
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
