.segment "HEADER"
.byte "NES",$1a,2,0,0,0
.res 8,0
.segment "CODE"
reset:
 sei
 lda #$40
 sta $4017
 lda #$4f
 sta $4010
 lda #$40
 sta $4011
 lda #$20
 sta $4012
 lda #$01
 sta $4013
 lda #$10
 sta $4015
 lda #$60
 sta $4011
.ifdef OAM_DMA
 lda #2
 sta $4014
 lda #$20
 sta $4011
.endif
loop:
 jmp loop
irq:
 rti
.segment "SAMPLE"
.res 17,$aa
.segment "VECTORS"
.word irq,reset,irq
