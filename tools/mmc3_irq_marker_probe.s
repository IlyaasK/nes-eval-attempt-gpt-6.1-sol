.segment "HEADER"
.byte "NES",$1a,2,1,$40,0
.res 8,0
.segment "PAD"
.res $6000,0
.segment "CODE"
.ifndef CTRL
CTRL = $08
.endif
.ifndef LATCH
LATCH = 8
.endif
reset:
 sei
 lda #$40
 sta $4017
 lda #$00
 sta $2000
 sta $2001
 bit $2002
wait1:
 bit $2002
 bpl wait1
wait2:
 bit $2002
 bpl wait2
 .ifdef BOTTOM_SPRITES
 ; Fill all OAM with out-of-range sprites, then populate the last visible
 ; line with eight even-bank 8x16 sprites. Pre-render must clear them.
 lda #0
 sta $2003
 lda #255
 ldx #0
fill_oam:
 sta $2004
 inx
 bne fill_oam
 lda #0
 sta $2003
 ldx #8
bottom_oam:
 lda #232
 sta $2004
 lda #0
 sta $2004
 sta $2004
 sta $2004
 dex
 bne bottom_oam
 .endif
 lda #CTRL
 sta $2000
 .ifdef BOTTOM_SPRITES
 lda #$18
 .else
 lda #$08
 .endif
 sta $2001
 lda #LATCH
 sta $c000
 sta $c001
 sta $e001
 cli
 lda #$00
 jmp forever
.ifdef PAGE_CROSS
 .align 256
 .res 252,$ea
.endif
forever:
.ifdef BRANCH_LOOP
 cmp $01
 beq forever
.else
 jmp forever
.endif
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
