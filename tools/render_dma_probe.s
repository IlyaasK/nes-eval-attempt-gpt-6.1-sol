; OAM DMA writes while rendering is active. The final palette encodes
; whether the initial zero OAM byte survived those writes. DIRECT_WRITE
; substitutes a CPU $2004 store for DMA, exposing the reference's two paths.
.segment "HEADER"
.byte "NES",$1a,2,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 ldx #0
 lda #$aa
fill_page:
 sta $0200,x
 inx
 bne fill_page
 ; Pass the warm-up interval with a counted loop.
 ldy #24
wait_outer:
 ldx #255
wait_inner:
 dex
 bne wait_inner
 dey
 bne wait_outer
 lda #8
 sta $2001
.ifdef DIRECT_WRITE
 lda #$aa
 sta $2004
.else
 lda #2
 sta $4014
.endif
 lda #0
 sta $2001
 sta $2003
 lda $2004
 sta $6000
 and #$20
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
