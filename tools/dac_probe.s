.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 lda #0
 sta $4015
loop:
 ldx #255
wait:
 dex
 bne wait
 eor #$60
 sta $4011
 jmp loop
nmi:
 rti
.segment "VECTORS"
.word nmi,reset,nmi
