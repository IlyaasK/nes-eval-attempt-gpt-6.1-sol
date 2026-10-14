; CPU PPUADDR/PPUDATA accesses drive MMC3 A12 with rendering disabled.
.ifndef MODE
MODE=0
.endif
.ifndef LATCH
LATCH=1
.endif
.ifndef WAIT
WAIT=0
.endif
.segment "HEADER"
.byte "NES",$1a,2,1,$40,0
.res 8,0
.segment "PAD"
.res $6000,0
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
 lda #0
 sta $2000
 sta $2001
 bit $2002
 lda #LATCH
 sta $c000
 lda #0
 sta $c001
 sta $e001
 cli
forever:
.if MODE=0 .or MODE=3
 lda #0
 sta $2006
 sta $2006
 .repeat WAIT
 nop
 .endrepeat
 .if MODE=0
 lda #$10
 .else
 lda #$3f
 .endif
 sta $2006
 lda #0
 sta $2006
 .if MODE=3
 lda $2007
 lda $2007
 .endif
.else
 .if MODE=1
 lda #$0f
 .else
 lda #$2f
 .endif
 sta $2006
 lda #$ff
 sta $2006
 .repeat WAIT
 nop
 .endrepeat
 .if MODE=4
 lda #0
 sta $2007
 .else
 lda $2007
 .endif
.endif
 jmp forever
irq:
 pha
 lda $00
 eor #96
 sta $00
 sta $4011
 lda #0
 sta $e000
 sta $c001
 sta $e001
 pla
 rti
.segment "VECTORS"
.word irq,reset,irq
.segment "CHR"
.res $2000,0
