.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 lda #8
 sta $4015
 lda #$38
 sta $400c
 lda #15
 sta $400e
 lda #0
 sta $400f
forever:
 jmp forever
nmi:
 rti
.segment "VECTORS"
.word nmi,reset,nmi
