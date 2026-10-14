; Start DMC so its initial read may halt the final LDA $2007 bus cycle.
; Sequential CHR bytes expose extra/ignored PPUDATA reads during the halt.
.ifndef PARITY
PARITY = 0
.endif
.ifndef EXTRA_NOPS
EXTRA_NOPS = 0
.endif
.ifndef PRIME
PRIME = 0
.endif
.ifndef READ_REGISTER
READ_REGISTER = $2007
.endif
.ifndef FRAME_MODE
FRAME_MODE = $40
.endif
.ifndef CPU_OPCODE
CPU_OPCODE = $ad
.endif
.ifndef CPU_OPERAND
CPU_OPERAND = READ_REGISTER
.endif
.ifndef CPU_INDEX
CPU_INDEX = 0
.endif
.ifndef CONTROLLER_MASK
CONTROLLER_MASK = 1
.endif
.ifndef CAPTURE_FLAGS
CAPTURE_FLAGS = 0
.endif
.ifndef PPU_OPEN
PPU_OPEN = -1
.endif
.ifndef RELOAD_DELAY
RELOAD_DELAY = 0
.endif
.ifndef STORE_TARGET
STORE_TARGET = -1
.endif
.segment "HEADER"
.byte "NES",$1a,2,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #FRAME_MODE
 sta $4017
 ldy #24
outer:
 ldx #255
inner:
 dex
 bne inner
 dey
 bne outer
 lda #0
 sta $2006
 sta $2006
 .repeat 8,I
 lda #((I+1)*17)
 sta $2007
 .endrepeat
 lda #0
 sta $2006
 sta $2006
.if PRIME
 lda $2007
.endif
.if READ_REGISTER = $4016
 lda #1
 sta $4016
 lda #0
 sta $4016
.endif
 lda #$4f
 sta $4010
 lda #127
 sta $4011
 lda #$20
 sta $4012
 lda #1
 sta $4013
.if PPU_OPEN >= 0
 lda #PPU_OPEN
 sta $2000
.endif
.if CPU_OPCODE = $b1
 lda #<READ_REGISTER
 sta $10
 lda #>READ_REGISTER
 sta $11
.endif
 ldx #CPU_INDEX
.if CPU_OPCODE = $b1
 ldy #CPU_INDEX
.endif
 lda #16
.if PARITY
 bit $00
.endif
 sta $4015
.if RELOAD_DELAY
 ldx #RELOAD_DELAY
reload_wait:
 dex
 bne reload_wait
.endif
 .repeat EXTRA_NOPS
 nop
 .endrepeat
.if CPU_OPCODE = $b1
 .byte CPU_OPCODE,$10
.else
 .byte CPU_OPCODE,<CPU_OPERAND,>CPU_OPERAND
.endif
.if STORE_TARGET >= 0
 sta STORE_TARGET
 lda $2007
.endif
.if CAPTURE_FLAGS
 php
 pla
.endif
.if READ_REGISTER = $4016
 and #CONTROLLER_MASK
.endif
 sta $00
 lda #0
 sta $4015
 lda $00
 sta $4011
 and #$c0
 lsr a
 sta $2001
 lda #$3f
 sta $2006
 lda #0
 sta $2006
 lda $00
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
.segment "SAMPLE"
.res 17,$ff
.segment "VECTORS"
.word irq,reset,irq
