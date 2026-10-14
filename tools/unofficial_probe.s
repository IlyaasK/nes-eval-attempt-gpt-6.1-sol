; Encode A, X, or P after an unofficial immediate opcode into palette and
; emphasis bits. The framebuffer index holds the entire sampled byte.
.ifndef OPCODE
OPCODE = $8b
.endif
.ifndef A_VALUE
A_VALUE = 0
.endif
.ifndef X_VALUE
X_VALUE = 255
.endif
.ifndef IMMEDIATE
IMMEDIATE = 255
.endif
.ifndef FLAGS
FLAGS = $24
.endif
.ifndef RESULT
RESULT = 0
.endif
.segment "HEADER"
.byte "NES",$1a,1,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 ldy #24
outer:
 ldx #255
inner:
 dex
 bne inner
 dey
 bne outer
 lda #FLAGS
 pha
 lda #A_VALUE
 ldx #X_VALUE
 plp
 .byte OPCODE,IMMEDIATE
 sta $00
 stx $01
 php
 pla
 sta $02
 lda $00+RESULT
 sta $4011
 and #$c0
 lsr a
 sta $2001
 lda #$3f
 sta $2006
 lda #0
 sta $2006
 lda $00+RESULT
 and #$3f
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
