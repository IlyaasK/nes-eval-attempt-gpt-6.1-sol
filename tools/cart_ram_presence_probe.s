; Return $2a if cartridge RAM stores data; otherwise capture CPU open bus.
.ifndef MAPPER
MAPPER=0
.endif
.ifndef RAM_UNITS
RAM_UNITS=0
.endif
.ifndef BATTERY
BATTERY=0
.endif
.ifndef NES2
NES2=0
.endif
.ifndef RAM_SHIFT
RAM_SHIFT=0
.endif
.segment "HEADER"
.byte "NES",$1a,1,1,(MAPPER<<4)|BATTERY,NES2*8
.if NES2
.byte 0,0
.if BATTERY
.byte RAM_SHIFT<<4
.else
.byte RAM_SHIFT
.endif
.res 5,0
.else
.byte RAM_UNITS
.res 7,0
.endif
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
 lda #$2a
 sta $6000
 lda $6000
 sta $00
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
.segment "VECTORS"
.word irq,reset,irq
.segment "CHR"
.res $2000,0
