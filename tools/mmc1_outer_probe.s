; Mirror this 16 KiB fixed bank in banks 15 and 31 of a 512 KiB MMC1 ROM.
; Other banks contain their bank index. The palette reveals the selected
; switchable PRG bank after differing CHR register outer bits and a PPU access.
.ifndef CHR0
CHR0 = 0
.endif
.ifndef CHR1
CHR1 = 16
.endif
.ifndef CONTROL
CONTROL = 28
.endif
.ifndef PPU_HIGH
PPU_HIGH = $10
.endif
.macro serial addr,value
 .repeat 5,bit
 lda #((value >> bit) & 1)
 sta addr
 .endrepeat
.endmacro
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
.ifdef RAM_DISABLE_TEST
 lda #37
 sta $6000
.endif
 serial $8000,CONTROL
 serial $a000,CHR0
 serial $c000,CHR1
 serial $e000,2
 lda #PPU_HIGH
 sta $2006
 lda #0
 sta $2006
 lda $2007
.ifdef RAM_TEST
 lda #37
 sta $6000
.ifdef RAM_SWITCH
 serial $a000,RAM_SWITCH
.else
 lda #0
 sta $2006
 sta $2006
 lda $2007
.endif
 lda $6000
.else
.ifdef RAM_DISABLE_TEST
 lda $6000
.else
 lda $8000
.endif
.endif
 sta $6000
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
