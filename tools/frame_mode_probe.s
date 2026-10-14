; Change frame sequencer mode just before its IRQ/half-frame edge.
; WRITE_CYCLE is even; the $4015 read completes four cycles later.
.ifndef WRITE_CYCLE
WRITE_CYCLE = 29848
.endif
.ifndef MODE
MODE = $80
.endif
.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 nop
 lda #$40
 sta $4017 ; cycle 18; delayed reset at 22, final half frame at 29851
 lda #MODE
 .repeat (WRITE_CYCLE-24)/2
 nop
 .endrepeat
 sta $4017
 lda $4015
 sta $6000
 and #$40
 lsr a
 ora #$0f
 tax
 lda #$3f
 sta $2006
 lda #0
 sta $2006
 txa
 sta $2007
 lda #$3f
 sta $2006
 lda #0
 sta $2006
forever:
 jmp forever
irq:
 rti
.segment "VECTORS"
.word irq,reset,irq
