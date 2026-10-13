.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #1
 sta $4016
.repeat 23
 ldx #255
 .byte $ca,$d0,$fd
.endrepeat
.repeat 200
 nop
.endrepeat
 lda #$40
 sta $4014
 lda #22
 sta $2003
 lda $2004
 and #1
 sta $00
 ldx #0
wait:
 bit $2002
 bpl wait
 inx
 cpx #2
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
