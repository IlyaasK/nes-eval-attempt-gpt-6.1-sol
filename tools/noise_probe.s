.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 lda #$08
 sta $4015
 lda #$3f
 sta $400c
 lda #$03
 sta $400e
 lda #$18
 sta $400f
loop:
 jmp loop
nmi:
 rti
.segment "VECTORS"
.word nmi,reset,nmi
