.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 lda #$20
 sta $4011
 lda #0
 sta $4014
 lda #$60
 sta $4011
loop:
 jmp loop
nmi:
 rti
.segment "VECTORS"
.word nmi,reset,nmi
