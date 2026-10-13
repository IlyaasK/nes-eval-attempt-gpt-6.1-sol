.segment "HEADER"
.byte "NES",$1a,1,1,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 bit $2002
wait1:
 bit $2002
 bpl wait1
wait2:
 bit $2002
 bpl wait2
 lda #$20
 sta $2006
 lda #0
 sta $2006
 ldy #4
 ldx #0
 lda #1
fill_tiles:
 sta $2007
 dex
 bne fill_tiles
 dey
 bne fill_tiles
 lda #$23
 sta $2006
 lda #$c0
 sta $2006
 lda #0
 ldx #64
fill_attributes:
 sta $2007
 dex
 bne fill_attributes
 lda #$3f
 sta $2006
 lda #0
 sta $2006
 ldx #0
fill_palette:
 lda palette,x
 sta $2007
 inx
 cpx #32
 bne fill_palette
 lda #0
 sta $2003
 lda #255
 ldx #0
hide_sprites:
 sta $2004
 dex
 bne hide_sprites
 lda #0
 sta $2003
 ldx #0
show_sprites:
 lda sprites,x
 sta $2004
 inx
 cpx #16
 bne show_sprites
 lda #0
 sta $2005
 sta $2005
 sta $2000
 lda #$1e
 sta $2001
forever:
 jmp forever
irq:
 rti
palette:
.byte 15,17,15,15,15,17,15,15,15,17,15,15,15,17,15,15
.byte 15,15,36,15,15,15,36,15,15,15,36,15,15,15,36,15
sprites:
 .ifdef OVERLAP
.byte 20,2,0,248,24,2,0,0,60,2,0,247,80,2,0,0
 .else
 .ifdef COUNT_DROP
.byte 24,2,0,0,20,2,0,255,60,2,0,247,80,2,0,0
 .else
.byte 20,2,0,248,40,2,0,255,60,2,0,247,80,2,0,0
 .endif
 .endif
.segment "VECTORS"
.word irq,reset,irq
.segment "CHR"
.res 16,0
.res 8,255
.res 8,0
.res 8,0
.res 8,255
.res 8192-48,0
