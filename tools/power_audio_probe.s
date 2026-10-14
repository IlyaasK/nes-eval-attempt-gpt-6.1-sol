.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
forever:
 jmp forever
irq:
 rti
.segment "VECTORS"
.word irq,reset,irq
