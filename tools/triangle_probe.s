.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 lda #$04
 sta $4015
 lda #$ff
 sta $4008
 lda #$70
 sta $400a
 lda #$18
 sta $400b
loop:
 jmp loop
nmi:
 rti
.segment "VECTORS"
.word nmi,reset,nmi
