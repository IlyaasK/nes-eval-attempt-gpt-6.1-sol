.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 lda #$02
 sta $4015
 lda #$7a
 sta $4004
 lda #$53
 sta $4006
 lda #$18
 sta $4007
loop:
 jmp loop
nmi:
 rti
.segment "VECTORS"
.word nmi,reset,nmi
