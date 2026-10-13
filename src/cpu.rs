use crate::{cart::Cart,ppu::Ppu,apu::Apu};
const C:u8=1;const Z:u8=2;const I:u8=4;const D:u8=8;const V:u8=64;const N:u8=128;
#[derive(Clone,Copy,PartialEq)]
enum Op{BRK,ORA,KIL,SLO,NOP,ASL,PHP,ANC,BPL,CLC,JSR,AND,RLA,BIT,ROL,PLP,BMI,SEC,RTI,EOR,SRE,LSR,PHA,ALR,JMP,BVC,CLI,RTS,ADC,RRA,ROR,PLA,ARR,BVS,SEI,STA,SAX,STY,STX,DEY,TXA,XAA,BCC,AHX,TYA,TXS,TAS,SHY,SHX,LDY,LDA,LDX,LAX,TAY,TAX,BCS,CLV,TSX,LAS,CPY,CMP,DCP,DEC,INY,DEX,AXS,BNE,CLD,CPX,SBC,ISC,INC,INX,BEQ,SED}
#[derive(Clone,Copy,PartialEq)]
enum Mode{Imp,Acc,Imm,Zp,Zpx,Zpy,Abs,Abx,Aby,Ind,Izx,Izy,Rel}
fn decode(code:u8)->(Op,Mode){use Op::*;use Mode::*;
 const O:[Op;256]=[
 BRK,ORA,KIL,SLO,NOP,ORA,ASL,SLO,PHP,ORA,ASL,ANC,NOP,ORA,ASL,SLO,
 BPL,ORA,KIL,SLO,NOP,ORA,ASL,SLO,CLC,ORA,NOP,SLO,NOP,ORA,ASL,SLO,
 JSR,AND,KIL,RLA,BIT,AND,ROL,RLA,PLP,AND,ROL,ANC,BIT,AND,ROL,RLA,
 BMI,AND,KIL,RLA,NOP,AND,ROL,RLA,SEC,AND,NOP,RLA,NOP,AND,ROL,RLA,
 RTI,EOR,KIL,SRE,NOP,EOR,LSR,SRE,PHA,EOR,LSR,ALR,JMP,EOR,LSR,SRE,
 BVC,EOR,KIL,SRE,NOP,EOR,LSR,SRE,CLI,EOR,NOP,SRE,NOP,EOR,LSR,SRE,
 RTS,ADC,KIL,RRA,NOP,ADC,ROR,RRA,PLA,ADC,ROR,ARR,JMP,ADC,ROR,RRA,
 BVS,ADC,KIL,RRA,NOP,ADC,ROR,RRA,SEI,ADC,NOP,RRA,NOP,ADC,ROR,RRA,
 NOP,STA,NOP,SAX,STY,STA,STX,SAX,DEY,NOP,TXA,XAA,STY,STA,STX,SAX,
 BCC,STA,KIL,AHX,STY,STA,STX,SAX,TYA,STA,TXS,TAS,SHY,STA,SHX,AHX,
 LDY,LDA,LDX,LAX,LDY,LDA,LDX,LAX,TAY,LDA,TAX,LAX,LDY,LDA,LDX,LAX,
 BCS,LDA,KIL,LAX,LDY,LDA,LDX,LAX,CLV,LDA,TSX,LAS,LDY,LDA,LDX,LAX,
 CPY,CMP,NOP,DCP,CPY,CMP,DEC,DCP,INY,CMP,DEX,AXS,CPY,CMP,DEC,DCP,
 BNE,CMP,KIL,DCP,NOP,CMP,DEC,DCP,CLD,CMP,NOP,DCP,NOP,CMP,DEC,DCP,
 CPX,SBC,NOP,ISC,CPX,SBC,INC,ISC,INX,SBC,NOP,SBC,CPX,SBC,INC,ISC,
 BEQ,SBC,KIL,ISC,NOP,SBC,INC,ISC,SED,SBC,NOP,ISC,NOP,SBC,INC,ISC];
 const M:[Mode;256]=[
 Imp,Izx,Imp,Izx,Zp,Zp,Zp,Zp,Imp,Imm,Acc,Imm,Abs,Abs,Abs,Abs,
 Rel,Izy,Imp,Izy,Zpx,Zpx,Zpx,Zpx,Imp,Aby,Imp,Aby,Abx,Abx,Abx,Abx,
 Abs,Izx,Imp,Izx,Zp,Zp,Zp,Zp,Imp,Imm,Acc,Imm,Abs,Abs,Abs,Abs,
 Rel,Izy,Imp,Izy,Zpx,Zpx,Zpx,Zpx,Imp,Aby,Imp,Aby,Abx,Abx,Abx,Abx,
 Imp,Izx,Imp,Izx,Zp,Zp,Zp,Zp,Imp,Imm,Acc,Imm,Abs,Abs,Abs,Abs,
 Rel,Izy,Imp,Izy,Zpx,Zpx,Zpx,Zpx,Imp,Aby,Imp,Aby,Abx,Abx,Abx,Abx,
 Imp,Izx,Imp,Izx,Zp,Zp,Zp,Zp,Imp,Imm,Acc,Imm,Ind,Abs,Abs,Abs,
 Rel,Izy,Imp,Izy,Zpx,Zpx,Zpx,Zpx,Imp,Aby,Imp,Aby,Abx,Abx,Abx,Abx,
 Imm,Izx,Imm,Izx,Zp,Zp,Zp,Zp,Imp,Imm,Imp,Imm,Abs,Abs,Abs,Abs,
 Rel,Izy,Imp,Izy,Zpx,Zpx,Zpy,Zpy,Imp,Aby,Imp,Aby,Abx,Abx,Aby,Aby,
 Imm,Izx,Imm,Izx,Zp,Zp,Zp,Zp,Imp,Imm,Imp,Imm,Abs,Abs,Abs,Abs,
 Rel,Izy,Imp,Izy,Zpx,Zpx,Zpy,Zpy,Imp,Aby,Imp,Aby,Abx,Abx,Aby,Aby,
 Imm,Izx,Imm,Izx,Zp,Zp,Zp,Zp,Imp,Imm,Imp,Imm,Abs,Abs,Abs,Abs,
 Rel,Izy,Imp,Izy,Zpx,Zpx,Zpx,Zpx,Imp,Aby,Imp,Aby,Abx,Abx,Abx,Abx,
 Imm,Izx,Imm,Izx,Zp,Zp,Zp,Zp,Imp,Imm,Imp,Imm,Abs,Abs,Abs,Abs,
 Rel,Izy,Imp,Izy,Zpx,Zpx,Zpx,Zpx,Imp,Aby,Imp,Aby,Abx,Abx,Abx,Abx];(O[code as usize],M[code as usize])}
pub struct Nes {
 pub cart:Cart,pub ppu:Ppu,pub apu:Apu,pub ram:[u8;2048],pub a:u8,pub x:u8,pub y:u8,pub s:u8,pub p:u8,pub pc:u16,pub cycles:u64,
 open:u8,keys:u8,shift:[u8;2],strobe:bool,dma:Option<u8>,nmi:bool,irq_pending:bool,last_irq:bool,jammed:bool,nmi_sample:bool,nmi_pending:bool,last_nmi:bool,
}
impl Nes {
 pub fn new(cart:Cart)->Self{let pc=cart.read(0xfffc,0) as u16|((cart.read(0xfffd,0) as u16)<<8);Self{cart,ppu:Ppu::new(),apu:Apu::new(),ram:[0;2048],a:0,x:0,y:0,s:253,p:36,pc,cycles:8,open:0,keys:0,shift:[0;2],strobe:false,dma:None,nmi:false,irq_pending:false,last_irq:false,jammed:false,nmi_sample:false,nmi_pending:false,last_nmi:false}}
 pub fn set_keys(&mut self,k:u8){self.keys=k;if self.strobe{self.shift[0]=k}}
 fn tick(&mut self){self.last_nmi=self.nmi;self.last_irq=self.cart.irq||self.apu.frame_irq||self.apu.dmc_irq;self.cycles+=1;self.apu.tick();self.ppu.tick(&mut self.cart);if self.ppu.nmi_line&&!self.nmi_sample{self.nmi=true}self.nmi_sample=self.ppu.nmi_line;for _ in 0..2{self.ppu.tick(&mut self.cart)}self.ppu.nmi_edge=false}
 fn raw_read(&mut self,a:u16)->u8{let v=match a{0..=0x1fff=>self.ram[a as usize&2047],0x2000..=0x3fff=>self.ppu.read_reg(&mut self.cart,a),0x4015=>self.apu.status(self.open),0x4016|0x4017=>{let i=(a&1) as usize;let b=if self.strobe{if i==0{self.keys&1}else{0}}else{let b=self.shift[i]&1;self.shift[i]=(self.shift[i]>>1)|128;b};(self.open&224)|b},0x4000..=0x401f=>self.open,_=>self.cart.read(a,self.open)};self.open=v;v}
 fn raw_write(&mut self,a:u16,v:u8){self.open=v;match a{0..=0x1fff=>self.ram[a as usize&2047]=v,0x2000..=0x3fff=>self.ppu.write_reg(&mut self.cart,a,v,self.cycles),0x4014=>self.dma=Some(v),0x4016=>{if v&1!=0||self.strobe{self.shift=[self.keys,0]}self.strobe=v&1!=0},0x4000..=0x4017=>self.apu.write(a,v),0x4018..=0x401f=>{},_=>self.cart.write(a,v,self.cycles)}}
 fn dmc_dma(&mut self,a:u16){if self.apu.dmc_need(){self.tick();self.raw_read(a);self.tick();self.raw_read(a);if self.cycles&1==0{self.tick();self.raw_read(a);}self.tick();let addr=self.apu.dmc_addr;let v=self.raw_read(addr);self.apu.dmc_supply(v)}}
 fn read(&mut self,a:u16)->u8{self.dmc_dma(a);self.tick();self.raw_read(a)}
 fn write(&mut self,a:u16,v:u8){self.tick();self.raw_write(a,v)}
 fn fetch(&mut self)->u8{let v=self.read(self.pc);self.pc=self.pc.wrapping_add(1);v}
 fn word(&mut self)->u16{let l=self.fetch() as u16;l|((self.fetch() as u16)<<8)}
 fn push(&mut self,v:u8){self.write(0x100|self.s as u16,v);self.s=self.s.wrapping_sub(1)}
 fn pop(&mut self)->u8{self.s=self.s.wrapping_add(1);self.read(0x100|self.s as u16)}
 fn flag(&mut self,f:u8,on:bool){if on{self.p|=f}else{self.p&=!f}}
 fn nz(&mut self,v:u8){self.flag(Z,v==0);self.flag(N,v&128!=0)}
 fn adc(&mut self,v:u8){let sum=self.a as u16+v as u16+(self.p&C) as u16;let r=sum as u8;self.flag(C,sum>255);self.flag(V,(!(self.a^v)&(self.a^r)&128)!=0);self.a=r;self.nz(r)}
 fn cmp(&mut self,a:u8,v:u8){self.flag(C,a>=v);self.nz(a.wrapping_sub(v))}
 fn interrupt(&mut self,nmi:bool){self.read(self.pc);self.read(self.pc);self.push((self.pc>>8) as u8);self.push(self.pc as u8);let vector=if nmi||self.nmi{self.nmi=false;0xfffa}else{0xfffe};self.push((self.p&!16)|32);self.p|=I;let l=self.read(vector) as u16;self.pc=l|((self.read(vector+1) as u16)<<8);self.irq_pending=false;}
 fn address(&mut self,m:Mode,store:bool)->u16{use Mode::*;match m{Imm=>{let a=self.pc;self.pc=self.pc.wrapping_add(1);a},Zp=>self.fetch() as u16,Zpx|Zpy=>{let a=self.fetch();self.read(a as u16);a.wrapping_add(if m==Zpx{self.x}else{self.y}) as u16},Abs=>self.word(),Abx|Aby=>{let base=self.word();let a=base.wrapping_add(if m==Abx{self.x}else{self.y} as u16);if store||(base^a)&0xff00!=0{self.read((base&0xff00)|(a&255));}a},Izx=>{let b=self.fetch();self.read(b as u16);let b=b.wrapping_add(self.x);let l=self.read(b as u16) as u16;l|((self.read(b.wrapping_add(1) as u16) as u16)<<8)},Izy=>{let b=self.fetch();let l=self.read(b as u16) as u16;let base=l|((self.read(b.wrapping_add(1) as u16) as u16)<<8);let a=base.wrapping_add(self.y as u16);if store||(base^a)&0xff00!=0{self.read((base&0xff00)|(a&255));}a},_=>0}}
 pub fn step(&mut self){use Op::*;use Mode::*;
  if let Some(page)=self.dma.take(){self.read(self.pc);if self.cycles&1!=0{self.read(self.pc);}for i in 0..256{let v=self.read((page as u16)<<8|i);self.tick();let a=self.ppu.oam_addr;self.ppu.oam[a as usize]=v;self.ppu.oam_addr=a.wrapping_add(1)}return}
  if self.jammed{self.read(self.pc);return}
  if self.nmi_pending{self.nmi=false;self.nmi_pending=false;self.interrupt(true);return}if self.irq_pending{self.interrupt(false);return}
  let old_i=self.p&I;let mut branch_poll=None;let code=self.fetch();let (op,mode)=decode(code);
  match op {
   BRK=>{self.fetch();self.push((self.pc>>8) as u8);self.push(self.pc as u8);let vec=if self.nmi{self.nmi=false;0xfffa}else{0xfffe};self.push(self.p|48);self.p|=I;let l=self.read(vec) as u16;self.pc=l|((self.read(vec+1) as u16)<<8);},
   JSR=>{let l=self.fetch();self.read(0x100|self.s as u16);self.push((self.pc>>8) as u8);self.push(self.pc as u8);let h=self.fetch();self.pc=l as u16|((h as u16)<<8)},
   JMP=>{let a=self.word();self.pc=if mode==Ind{let l=self.read(a) as u16;l|((self.read((a&0xff00)|(a.wrapping_add(1)&255)) as u16)<<8)}else{a}},
   RTS=>{self.read(self.pc);self.read(0x100|self.s as u16);let l=self.pop() as u16;self.pc=l|((self.pop() as u16)<<8);self.read(self.pc);self.pc=self.pc.wrapping_add(1)},
   RTI=>{self.read(self.pc);self.read(0x100|self.s as u16);self.p=(self.pop()&!16)|32;let l=self.pop() as u16;self.pc=l|((self.pop() as u16)<<8)},
   PHP|PHA=>{self.read(self.pc);self.push(if op==PHP{self.p|48}else{self.a})},
   PLP|PLA=>{self.read(self.pc);self.read(0x100|self.s as u16);let v=self.pop();if op==PLP{self.p=(v&!16)|32}else{self.a=v;self.nz(v)}},
   BPL|BMI|BVC|BVS|BCC|BCS|BNE|BEQ=>{let delta=self.fetch() as i8;branch_poll=Some((self.last_irq,self.last_nmi));let take=match op{BPL=>self.p&N==0,BMI=>self.p&N!=0,BVC=>self.p&V==0,BVS=>self.p&V!=0,BCC=>self.p&C==0,BCS=>self.p&C!=0,BNE=>self.p&Z==0,_=>self.p&Z!=0};if take{let old=self.pc;self.read(old);self.pc=self.pc.wrapping_add(delta as u16);if (old^self.pc)&0xff00!=0{self.read((old&0xff00)|(self.pc&255));let (irq,nmi)=branch_poll.unwrap();branch_poll=Some((irq||self.last_irq,nmi||self.last_nmi));}}},
   _=>{
    let rmw=matches!(op,ASL|LSR|ROL|ROR|INC|DEC|SLO|SRE|RLA|RRA|DCP|ISC);
    let store=matches!(op,STA|STX|STY|SAX|AHX|SHX|SHY|TAS);
    if mode==Imp{self.read(self.pc);match op{CLC=>self.p&=!C,SEC=>self.p|=C,CLI=>self.p&=!I,SEI=>self.p|=I,CLD=>self.p&=!D,SED=>self.p|=D,CLV=>self.p&=!V,DEY=>{self.y=self.y.wrapping_sub(1);self.nz(self.y)},INY=>{self.y=self.y.wrapping_add(1);self.nz(self.y)},DEX=>{self.x=self.x.wrapping_sub(1);self.nz(self.x)},INX=>{self.x=self.x.wrapping_add(1);self.nz(self.x)},TXA=>{self.a=self.x;self.nz(self.a)},TYA=>{self.a=self.y;self.nz(self.a)},TAX=>{self.x=self.a;self.nz(self.x)},TAY=>{self.y=self.a;self.nz(self.y)},TSX=>{self.x=self.s;self.nz(self.x)},TXS=>self.s=self.x,KIL=>self.jammed=true,_=>{}}}
    else if mode==Acc{self.read(self.pc);self.a=self.modify(op,self.a)}
    else{let addr=self.address(mode,store||rmw);if store{let high=((addr>>8) as u8).wrapping_add(1);let v=match op{STA=>self.a,STX=>self.x,STY=>self.y,SAX=>self.a&self.x,SHX=>self.x&high,SHY=>self.y&high,TAS=>{self.s=self.a&self.x;self.s&high},_=>self.a&self.x&high};self.write(addr,v)}else{let mut v=self.read(addr);if rmw{self.write(addr,v);v=self.modify(op,v);self.write(addr,v)}else{match op{ORA=>{self.a|=v;self.nz(self.a)},AND=>{self.a&=v;self.nz(self.a)},EOR=>{self.a^=v;self.nz(self.a)},ADC=>self.adc(v),SBC=>self.adc(!v),LDA=>{self.a=v;self.nz(v)},LDX=>{self.x=v;self.nz(v)},LDY=>{self.y=v;self.nz(v)},LAX=>{self.a=v;self.x=v;self.nz(v)},LAS=>{self.a=v&self.s;self.x=self.a;self.s=self.a;self.nz(self.a)},CMP=>self.cmp(self.a,v),CPX=>self.cmp(self.x,v),CPY=>self.cmp(self.y,v),BIT=>{self.flag(Z,self.a&v==0);self.p=(self.p&!192)|(v&192)},ANC=>{self.a&=v;self.nz(self.a);self.flag(C,self.a&128!=0)},ALR=>{self.a&=v;self.a=self.modify(LSR,self.a)},ARR=>{self.a&=v;self.a=(self.a>>1)|((self.p&C)<<7);self.nz(self.a);self.flag(C,self.a&64!=0);self.flag(V,((self.a>>6)^(self.a>>5))&1!=0)},AXS=>{let a=self.a&self.x;self.x=a.wrapping_sub(v);self.flag(C,a>=v);self.nz(self.x)},XAA=>{self.a=self.x&v;self.nz(self.a)},_=>{}}}}}
   }
  }
  let i=if matches!(op,CLI|SEI|PLP){old_i}else{self.p&I};let (irq,nmi)=branch_poll.unwrap_or((self.last_irq,self.last_nmi));self.irq_pending=irq&&i==0;self.nmi_pending=nmi;
 }
 fn modify(&mut self,op:Op,v:u8)->u8{use Op::*;let carry=self.p&C;let r=match op{ASL|SLO=>{self.flag(C,v&128!=0);v<<1},LSR|SRE=>{self.flag(C,v&1!=0);v>>1},ROL|RLA=>{self.flag(C,v&128!=0);(v<<1)|carry},ROR|RRA=>{self.flag(C,v&1!=0);(v>>1)|(carry<<7)},INC|ISC=>v.wrapping_add(1),DEC|DCP=>v.wrapping_sub(1),_=>v};match op{SLO=>{self.a|=r;self.nz(self.a)},SRE=>{self.a^=r;self.nz(self.a)},RLA=>{self.a&=r;self.nz(self.a)},RRA=>self.adc(r),DCP=>self.cmp(self.a,r),ISC=>self.adc(!r),_=>self.nz(r)}r}
 pub fn run_frame(&mut self){let frame=self.ppu.frame;while self.ppu.frame==frame{self.step()}}
}
