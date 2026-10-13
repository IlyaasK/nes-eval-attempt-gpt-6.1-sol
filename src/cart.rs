#[derive(Clone)]
pub struct Cart {
 pub prg: Vec<u8>, pub chr: Vec<u8>, pub ram: Vec<u8>, pub chr_ram: bool,
 pub mapper:u16, pub mirror:u8, pub four:bool,
 bank:u8, shift:u8, control:u8, chr0:u8, chr1:u8, prgb:u8, last_write:u64,
 select:u8, regs:[u8;8], pub irq:bool, irq_enable:bool, irq_reload:bool, irq_latch:u8, irq_counter:u8,
 a12:bool, low_since:u64, ram_enable:bool, ram_protect:bool,
}
impl Cart {
 pub fn load(b:&[u8])->Option<Self> {
  if b.len()<16 || &b[..4]!=b"NES\x1a" {return None}
  let nes2=b[7]&12==8;
  let mapper=(b[6] as u16>>4)|(b[7] as u16&0xf0)|if nes2 {(b[8] as u16&15)<<8}else{0};
  if ![0,1,2,3,4].contains(&mapper){return None}
  fn size(lo:u8,hi:u8,unit:usize)->Option<usize> {if hi==15{(1usize.checked_shl((lo>>2) as u32)?).checked_mul((lo as usize&3)*2+1)}else{Some(((hi as usize)<<8|lo as usize)*unit)}}
  let ps=size(b[4],if nes2{b[9]&15}else{0},16384)?;
  let cs=size(b[5],if nes2{b[9]>>4}else{0},8192)?;
  let offset=16+if b[6]&4!=0{512}else{0};
  if ps==0||offset+ps+cs>b.len(){return None}
  let ram_size=if nes2 {let a=b[10]&15;let c=b[10]>>4; (if a>0 {64usize<<a}else{0})+(if c>0{64usize<<c}else{0})}else{b[8].max(1) as usize*8192};
  let chr_size=if cs!=0{cs}else if nes2 {let a=b[11]&15;let c=b[11]>>4; ((if a>0{64usize<<a}else{0})+(if c>0{64usize<<c}else{0})).max(8192)}else{8192};
  let mut cart=Self{prg:b[offset..offset+ps].to_vec(),chr:if cs>0{b[offset+ps..offset+ps+cs].to_vec()}else{vec![0;chr_size]},ram:vec![0;ram_size.max(8192)],chr_ram:cs==0,mapper,mirror:b[6]&1,four:b[6]&8!=0,bank:0,shift:16,control:12,chr0:0,chr1:0,prgb:0,last_write:u64::MAX,select:0,regs:[0;8],irq:false,irq_enable:false,irq_reload:false,irq_latch:0,irq_counter:0,a12:false,low_since:0,ram_enable:true,ram_protect:false};
  if b[6]&4!=0{cart.ram[0x1000..0x1200].copy_from_slice(&b[16..528])}
  Some(cart)
 }
 pub fn read(&self,a:u16,open:u8)->u8 {
  if a<0x6000{return open}
  if a<0x8000 {return if self.ram_enable && (self.mapper!=1||self.prgb&16==0){self.ram[(a as usize-0x6000)%self.ram.len()]}else{open}}
  let off=a as usize&0x1fff;let slot=(a as usize-0x8000)/8192;let n=self.prg.len()/8192;
  let bank=match self.mapper{
   0|3=>(a as usize-0x8000)/8192,
   2=>if slot<2{self.bank as usize*2+slot}else{n-4+slot},
   1=>{let n16=n/2;let s=slot/2;let b=match (self.control>>2)&3{0|1=>(self.prgb as usize&14)+s,2=>if s==0{0}else{self.prgb as usize&15},_=>if s==0{self.prgb as usize&15}else{n16-1}};b*2+slot%2},
   4=>{let b6=self.regs[6] as usize&63;let b7=self.regs[7] as usize&63;match slot{0=>if self.select&64!=0{n-2}else{b6},1=>b7,2=>if self.select&64!=0{b6}else{n-2},_=>n-1}},_=>0};
  self.prg[(bank%n)*8192+off]
 }
 pub fn write(&mut self,a:u16,v:u8,cycle:u64){
  if a<0x6000{return}
  if a<0x8000 {if self.ram_enable&&!self.ram_protect&&(self.mapper!=1||self.prgb&16==0){let i=(a as usize-0x6000)%self.ram.len();self.ram[i]=v}return}
  match self.mapper {
   1=>{if self.last_write!=u64::MAX&&cycle==self.last_write+1{return}self.last_write=cycle;
    if v&128!=0{self.shift=16;self.control|=12}else{let done=self.shift&1!=0;self.shift=(self.shift>>1)|((v&1)<<4);if done{match (a>>13)&3{0=>self.control=self.shift,1=>self.chr0=self.shift,2=>self.chr1=self.shift,_=>self.prgb=self.shift};self.shift=16}}
    self.mirror=match self.control&3{0=>2,1=>3,2=>1,_=>0};
   },2|3=>self.bank=v,
   4=>match a&0xe001{0x8000=>self.select=v,0x8001=>self.regs[(self.select&7) as usize]=v,0xa000=>if !self.four{self.mirror=if v&1==0{1}else{0}},0xa001=>{self.ram_enable=v&128!=0;self.ram_protect=v&64!=0},0xc000=>self.irq_latch=v,0xc001=>self.irq_reload=true,0xe000=>{self.irq_enable=false;self.irq=false},0xe001=>self.irq_enable=true,_=>{}},_=>{}
  }
 }
 fn chr_index(&self,a:u16)->usize{
  let slot=a as usize/1024;let bank=match self.mapper{
   1=>if self.control&16!=0{(if a<4096{self.chr0}else{self.chr1}) as usize*4+slot%4}else{(self.chr0 as usize&30)*4+slot},
   3=>self.bank as usize*8+slot,
   4=>{let s=slot^if self.select&128!=0{4}else{0};match s{0=>self.regs[0] as usize&254,1=>(self.regs[0] as usize&254)+1,2=>self.regs[1] as usize&254,3=>(self.regs[1] as usize&254)+1,_=>self.regs[s-2] as usize}},_=>slot};(bank*1024+(a as usize&1023))%self.chr.len()
 }
 pub fn chr_read(&self,a:u16)->u8{self.chr[self.chr_index(a)]}
 pub fn chr_write(&mut self,a:u16,v:u8){if self.chr_ram{let i=self.chr_index(a);self.chr[i]=v}}
 pub fn nt_index(&self,a:u16)->usize {let a=(a as usize-0x2000)&4095;let table=a/1024;let t=if self.four{table}else{match self.mirror{0=>table/2,1=>table%2,2=>0,_=>1}};t*1024+(a&1023)}
 pub fn address(&mut self,a:u16,dot:u64){if self.mapper!=4{return}let high=a&0x1000!=0;if !high {if self.a12{self.low_since=dot}}else if !self.a12&&dot.saturating_sub(self.low_since)>=8{if self.irq_counter==0||self.irq_reload{self.irq_counter=self.irq_latch}else{self.irq_counter-=1}self.irq_reload=false;if self.irq_counter==0&&self.irq_enable{self.irq=true}}self.a12=high;}
}
