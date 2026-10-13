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
 lda #$08
 sta $2000
 sta $2001
 lda #8
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
