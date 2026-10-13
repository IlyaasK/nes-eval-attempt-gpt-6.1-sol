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
 ldx #0
wait:
 bit $2002
 bpl wait
 inx
 cpx #5
 bne wait
 lda #$60
 sta $4011
 ldx #0
w2:
 bit $2002
 bpl w2
 inx
 cpx #5
 bne w2
 lda #0
 sta $4011
 lda #1
 sta $4015
 lda #$bf
 sta $4000
 lda #$ff
 sta $4002
 lda #0
 sta $4003
loop:
 jmp loop
nmi:
 rti
.segment "VECTORS"
.word nmi,reset,nmi
