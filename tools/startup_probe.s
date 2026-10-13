.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #1
 sta $4016
 lda #0
 sta $4016
 lda $4016
 and #1
 sta $00
 ldx #0
wait:
 bit $2002
 bpl wait
 inx
 cpx #3
 bne wait
 lda #$3f
 sta $2006
 lda #0
 sta $2006
 lda $00
 clc
 adc #$20
 sta $2007
 lda #0
 sta $2006
 sta $2006
loop:
 jmp loop
nmi:
 rti
.segment "VECTORS"
.word nmi,reset,nmi
