use crate::{cart::Cart, ppu::Ppu, apu::Apu};
const C: u8 = 1;
const Z: u8 = 2;
const I: u8 = 4;
const D: u8 = 8;
const V: u8 = 64;
const N: u8 = 128;
#[derive(Clone,Copy,PartialEq)]
enum Op {
    BRK,
    ORA,
    KIL,
    SLO,
    NOP,
    ASL,
    PHP,
    ANC,
    BPL,
    CLC,
    JSR,
    AND,
    RLA,
    BIT,
    ROL,
    PLP,
    BMI,
    SEC,
    RTI,
    EOR,
    SRE,
    LSR,
    PHA,
    ALR,
    JMP,
    BVC,
    CLI,
    RTS,
    ADC,
    RRA,
    ROR,
    PLA,
    ARR,
    BVS,
    SEI,
    STA,
    SAX,
    STY,
    STX,
    DEY,
    TXA,
    XAA,
    BCC,
    AHX,
    TYA,
    TXS,
    TAS,
    SHY,
    SHX,
    LDY,
    LDA,
    LDX,
    LAX,
    TAY,
    TAX,
    BCS,
    CLV,
    TSX,
    LAS,
    CPY,
    CMP,
    DCP,
    DEC,
    INY,
    DEX,
    AXS,
    BNE,
    CLD,
    CPX,
    SBC,
    ISC,
    INC,
    INX,
    BEQ,
    SED,
}
#[derive(Clone,Copy,PartialEq)]
enum Mode { Imp, Acc, Imm, Zp, Zpx, Zpy, Abs, Abx, Aby, Ind, Izx, Izy, Rel, }
fn decode(code: u8) -> (Op, Mode) {
    use Op::*;
    use Mode::*;
    const O: [Op; 256] =
        [BRK, ORA, KIL, SLO, NOP, ORA, ASL, SLO, PHP, ORA, ASL, ANC, NOP, ORA,
                ASL, SLO, BPL, ORA, KIL, SLO, NOP, ORA, ASL, SLO, CLC, ORA,
                NOP, SLO, NOP, ORA, ASL, SLO, JSR, AND, KIL, RLA, BIT, AND,
                ROL, RLA, PLP, AND, ROL, ANC, BIT, AND, ROL, RLA, BMI, AND,
                KIL, RLA, NOP, AND, ROL, RLA, SEC, AND, NOP, RLA, NOP, AND,
                ROL, RLA, RTI, EOR, KIL, SRE, NOP, EOR, LSR, SRE, PHA, EOR,
                LSR, ALR, JMP, EOR, LSR, SRE, BVC, EOR, KIL, SRE, NOP, EOR,
                LSR, SRE, CLI, EOR, NOP, SRE, NOP, EOR, LSR, SRE, RTS, ADC,
                KIL, RRA, NOP, ADC, ROR, RRA, PLA, ADC, ROR, ARR, JMP, ADC,
                ROR, RRA, BVS, ADC, KIL, RRA, NOP, ADC, ROR, RRA, SEI, ADC,
                NOP, RRA, NOP, ADC, ROR, RRA, NOP, STA, NOP, SAX, STY, STA,
                STX, SAX, DEY, NOP, TXA, XAA, STY, STA, STX, SAX, BCC, STA,
                KIL, AHX, STY, STA, STX, SAX, TYA, STA, TXS, TAS, SHY, STA,
                SHX, AHX, LDY, LDA, LDX, LAX, LDY, LDA, LDX, LAX, TAY, LDA,
                TAX, LAX, LDY, LDA, LDX, LAX, BCS, LDA, KIL, LAX, LDY, LDA,
                LDX, LAX, CLV, LDA, TSX, LAS, LDY, LDA, LDX, LAX, CPY, CMP,
                NOP, DCP, CPY, CMP, DEC, DCP, INY, CMP, DEX, AXS, CPY, CMP,
                DEC, DCP, BNE, CMP, KIL, DCP, NOP, CMP, DEC, DCP, CLD, CMP,
                NOP, DCP, NOP, CMP, DEC, DCP, CPX, SBC, NOP, ISC, CPX, SBC,
                INC, ISC, INX, SBC, NOP, SBC, CPX, SBC, INC, ISC, BEQ, SBC,
                KIL, ISC, NOP, SBC, INC, ISC, SED, SBC, NOP, ISC, NOP, SBC,
                INC, ISC];
    const M: [Mode; 256] =
        [Imp, Izx, Imp, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Acc, Imm, Abs, Abs,
                Abs, Abs, Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby,
                Imp, Aby, Abx, Abx, Abx, Abx, Abs, Izx, Imp, Izx, Zp, Zp, Zp,
                Zp, Imp, Imm, Acc, Imm, Abs, Abs, Abs, Abs, Rel, Izy, Imp,
                Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx,
                Abx, Imp, Izx, Imp, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Acc, Imm,
                Abs, Abs, Abs, Abs, Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx,
                Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx, Imp, Izx, Imp, Izx,
                Zp, Zp, Zp, Zp, Imp, Imm, Acc, Imm, Ind, Abs, Abs, Abs, Rel,
                Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx,
                Abx, Abx, Abx, Imm, Izx, Imm, Izx, Zp, Zp, Zp, Zp, Imp, Imm,
                Imp, Imm, Abs, Abs, Abs, Abs, Rel, Izy, Imp, Izy, Zpx, Zpx,
                Zpy, Zpy, Imp, Aby, Imp, Aby, Abx, Abx, Aby, Aby, Imm, Izx,
                Imm, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Imp, Imm, Abs, Abs, Abs,
                Abs, Rel, Izy, Imp, Izy, Zpx, Zpx, Zpy, Zpy, Imp, Aby, Imp,
                Aby, Abx, Abx, Aby, Aby, Imm, Izx, Imm, Izx, Zp, Zp, Zp, Zp,
                Imp, Imm, Imp, Imm, Abs, Abs, Abs, Abs, Rel, Izy, Imp, Izy,
                Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
                Imm, Izx, Imm, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Imp, Imm, Abs,
                Abs, Abs, Abs, Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp,
                Aby, Imp, Aby, Abx, Abx, Abx, Abx];
    (O[code as usize], M[code as usize])
}

use std::{
    cell::RefCell, rc::Rc, future::Future, pin::Pin,
    task::{Context, Poll, Waker},
};
#[derive(Clone,Copy)]
enum CycleOp { Read(u16), Write(u16, u8), Oam(u8), }
pub struct Bus {
    pub cart: Cart,
    pub ppu: Ppu,
    pub apu: Apu,
    pub ram: [u8; 2048],
    pub cycles: u64,
    open: u8,
    keys: u8,
    shift: [u8; 2],
    strobe: bool,
    dma: Option<u8>,
    nmi: bool,
    last_irq: bool,
    last_nmi: bool,
    nmi_sample: bool,
    oam_active: bool,
    dmc_active: bool,
    last_dma_cycle: bool,
    request: Option<CycleOp>,
    result: Option<u8>,
}
impl Bus {
    fn new(cart: Cart) -> Self {
        Self {
            cart,
            ppu: Ppu::new(),
            apu: Apu::new(),
            ram: [0; 2048],
            cycles: 8,
            open: 0,
            keys: 0,
            shift: [0; 2],
            strobe: false,
            dma: None,
            nmi: false,
            last_irq: false,
            last_nmi: false,
            nmi_sample: false,
            oam_active: false,
            dmc_active: false,
            last_dma_cycle: false,
            request: None,
            result: None,
        }
    }
    fn execute(&mut self) {
        let op = self.request.take().expect("CPU bus request");
        self.last_dma_cycle = self.oam_active || self.dmc_active;
        self.tick();
        let value =
            match op {
                CycleOp::Read(a) => self.raw_read(a),
                CycleOp::Write(a, v) => { self.raw_write(a, v); 0 }
                CycleOp::Oam(v) => {
                    let a = self.ppu.oam_addr;
                    self.ppu.oam[a as usize] = v;
                    self.ppu.oam_addr = a.wrapping_add(1);
                    0
                }
            };
        self.result = Some(value);
    }
    fn tick(&mut self) {
        self.last_nmi = self.nmi;
        self.last_irq =
            self.cart.irq || self.apu.frame_irq || self.apu.dmc_irq;
        self.cycles += 1;
        self.apu.tick();
        self.ppu.tick(&mut self.cart);
        if self.ppu.nmi_line && !self.nmi_sample { self.nmi = true }
        self.nmi_sample = self.ppu.nmi_line;
        for _ in 0..2 { self.ppu.tick(&mut self.cart) }
    }
    fn raw_read(&mut self, a: u16) -> u8 {
        let v =
            match a {
                0..=0x1fff => self.ram[a as usize & 2047],
                0x2000..=0x3fff => self.ppu.read_reg(&mut self.cart, a),
                0x4015 => self.apu.status(self.open),
                0x4016 | 0x4017 => {
                    let i = (a & 1) as usize;
                    let b =
                        if self.strobe {
                            if i == 0 { self.keys & 1 } else { 0 }
                        } else {
                            let b = self.shift[i] & 1;
                            self.shift[i] = (self.shift[i] >> 1) | 128;
                            b
                        };
                    (self.open & 224) | b
                }
                0x4000..=0x401f => self.open,
                _ => self.cart.read(a, self.open),
            };
        self.open = v;
        v
    }
    fn raw_write(&mut self, a: u16, v: u8) {

        #[cfg(feature="trace")]
        if (0x4000..=0x4017).contains(&a) {
            eprintln!("WRITE {} {:04x} {}",self.cycles,a,v)
        }
        self.open = v;
        match a {
            0..=0x1fff => self.ram[a as usize & 2047] = v,
            0x2000..=0x3fff =>
                self.ppu.write_reg(&mut self.cart, a, v, self.cycles),
            0x4014 => self.dma = Some(v),
            0x4016 => {
                if v & 1 != 0 || self.strobe { self.shift = [self.keys, 0] }
                self.strobe = v & 1 != 0
            }
            0x4000..=0x4017 => self.apu.write(a, v),
            0x4018..=0x401f => {}
            _ => self.cart.write(a, v, self.cycles),
        }
    }
}

struct Access {
    bus: Rc<RefCell<Bus>>,
    op: CycleOp,
}
impl Future for Access {
    type Output = u8;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<u8> {
        let mut bus = self.bus.borrow_mut();
        if let Some(v) = bus.result.take() {
            Poll::Ready(v)
        } else {
            assert!(bus.request.is_none());
            bus.request = Some(self.op);
            Poll::Pending
        }
    }
}
#[derive(Clone)]
struct Cpu {
    bus: Rc<RefCell<Bus>>,
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: u8,
    pc: u16,
    base_addr: u16,
    irq_pending: bool,
    nmi_pending: bool,
    jammed: bool,
}
impl Cpu {
    async fn access(&mut self, op: CycleOp) -> u8 {
        Access { bus: self.bus.clone(), op }.await
    }
    async fn dmc_dma(&mut self, a: u16) {
        let need = self.bus.borrow().apu.dmc_need();
        if need {
            self.bus.borrow_mut().dmc_active = true;
            self.access(CycleOp::Read(a)).await;
            self.access(CycleOp::Read(a)).await;
            if self.bus.borrow().cycles & 1 == 0 {
                self.access(CycleOp::Read(a)).await;
            }
            let addr = self.bus.borrow().apu.dmc_addr;
            let v = self.access(CycleOp::Read(addr)).await;
            self.bus.borrow_mut().apu.dmc_supply(v);
            self.bus.borrow_mut().dmc_active = false;
        }
    }
    async fn read(&mut self, a: u16) -> u8 {
        self.dmc_dma(a).await;
        self.access(CycleOp::Read(a)).await
    }
    async fn write(&mut self, a: u16, v: u8) {
        self.access(CycleOp::Write(a, v)).await;
    }
    async fn fetch(&mut self) -> u8 {
        let v = self.read(self.pc).await;
        self.pc = self.pc.wrapping_add(1);
        v
    }
    async fn word(&mut self) -> u16 {
        let l = self.fetch().await as u16;
        l | ((self.fetch().await as u16) << 8)
    }
    async fn push(&mut self, v: u8) {
        self.write(0x100 | self.s as u16, v).await;
        self.s = self.s.wrapping_sub(1)
    }
    async fn pop(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read(0x100 | self.s as u16).await
    }
    fn flag(&mut self, f: u8, on: bool) {
        if on { self.p |= f } else { self.p &= !f }
    }
    fn nz(&mut self, v: u8) {
        self.flag(Z, v == 0);
        self.flag(N, v & 128 != 0)
    }
    fn adc(&mut self, v: u8) {
        let sum = self.a as u16 + v as u16 + (self.p & C) as u16;
        let r = sum as u8;
        self.flag(C, sum > 255);
        self.flag(V, (!(self.a ^ v) & (self.a ^ r) & 128) != 0);
        self.a = r;
        self.nz(r)
    }
    fn cmp(&mut self, a: u8, v: u8) {
        self.flag(C, a >= v);
        self.nz(a.wrapping_sub(v))
    }
    async fn interrupt(&mut self, nmi: bool) {
        self.read(self.pc).await;
        self.read(self.pc).await;
        self.push((self.pc >> 8) as u8).await;
        self.push(self.pc as u8).await;
        self.push((self.p & !16) | 32).await;
        self.p |= I;
        let vector =
            if nmi || self.bus.borrow().nmi {
                self.bus.borrow_mut().nmi = false;
                0xfffa
            } else { 0xfffe };
        let l = self.read(vector).await as u16;
        self.pc = l | ((self.read(vector + 1).await as u16) << 8);
        self.irq_pending = false;
    }
    async fn address(&mut self, m: Mode, store: bool) -> u16 {
        use Mode::*;
        match m {
            Imm => { let a = self.pc; self.pc = self.pc.wrapping_add(1); a }
            Zp => self.fetch().await as u16,
            Zpx | Zpy => {
                let a = self.fetch().await;
                self.read(a as u16).await;
                a.wrapping_add(if m == Zpx { self.x } else { self.y }) as u16
            }
            Abs => self.word().await,
            Abx | Aby => {
                let base = self.word().await;
                self.base_addr = base;
                let a =
                    base.wrapping_add(if m == Abx { self.x } else { self.y } as
                            u16);
                if store || (base ^ a) & 0xff00 != 0 {
                    self.read((base & 0xff00) | (a & 255)).await;
                }
                a
            }
            Izx => {
                let b = self.fetch().await;
                self.read(b as u16).await;
                let b = b.wrapping_add(self.x);
                let l = self.read(b as u16).await as u16;
                l | ((self.read(b.wrapping_add(1) as u16).await as u16) << 8)
            }
            Izy => {
                let b = self.fetch().await;
                let l = self.read(b as u16).await as u16;
                let base =
                    l |
                        ((self.read(b.wrapping_add(1) as u16).await as u16) << 8);
                self.base_addr = base;
                let a = base.wrapping_add(self.y as u16);
                if store || (base ^ a) & 0xff00 != 0 {
                    self.read((base & 0xff00) | (a & 255)).await;
                }
                a
            }
            _ => 0,
        }
    }
    async fn step(&mut self) {
        use Op::*;
        use Mode::*;
        let dma = self.bus.borrow_mut().dma.take();
        if let Some(page) = dma {
            self.bus.borrow_mut().oam_active = true;
            self.read(self.pc).await;
            if self.bus.borrow().cycles & 1 == 0 { self.read(self.pc).await; }
            for i in 0..256 {
                let v = self.read((page as u16) << 8 | i).await;
                self.access(CycleOp::Oam(v)).await;
            }
            self.bus.borrow_mut().oam_active = false;
            return
        }
        if self.jammed { self.read(self.pc).await; return }
        if self.nmi_pending {
            self.bus.borrow_mut().nmi = false;
            self.nmi_pending = false;
            self.interrupt(true).await;
            return
        }
        if self.irq_pending { self.interrupt(false).await; return }
        let old_i = self.p & I;
        let mut branch_poll = None;
        let code = self.fetch().await;
        let (op, mode) = decode(code);
        match op {
            BRK => {
                self.fetch().await;
                self.push((self.pc >> 8) as u8).await;
                self.push(self.pc as u8).await;
                self.push(self.p | 48).await;
                self.p |= I;
                let vec =
                    if self.bus.borrow().nmi {
                        self.bus.borrow_mut().nmi = false;
                        0xfffa
                    } else { 0xfffe };
                let l = self.read(vec).await as u16;
                self.pc = l | ((self.read(vec + 1).await as u16) << 8);
            }
            JSR => {
                let l = self.fetch().await;
                self.read(0x100 | self.s as u16).await;
                self.push((self.pc >> 8) as u8).await;
                self.push(self.pc as u8).await;
                let h = self.fetch().await;
                self.pc = l as u16 | ((h as u16) << 8)
            }
            JMP => {
                let a = self.word().await;
                self.pc =
                    if mode == Ind {
                        let l = self.read(a).await as u16;
                        l |
                            ((self.read((a & 0xff00) | (a.wrapping_add(1) & 255)).await
                                            as u16) << 8)
                    } else { a }
            }
            RTS => {
                self.read(self.pc).await;
                self.read(0x100 | self.s as u16).await;
                let l = self.pop().await as u16;
                self.pc = l | ((self.pop().await as u16) << 8);
                self.read(self.pc).await;
                self.pc = self.pc.wrapping_add(1)
            }
            RTI => {
                self.read(self.pc).await;
                self.read(0x100 | self.s as u16).await;
                self.p = (self.pop().await & !16) | 32;
                let l = self.pop().await as u16;
                self.pc = l | ((self.pop().await as u16) << 8)
            }
            PHP | PHA => {
                self.read(self.pc).await;
                self.push(if op == PHP { self.p | 48 } else { self.a }).await
            }
            PLP | PLA => {
                self.read(self.pc).await;
                self.read(0x100 | self.s as u16).await;
                let v = self.pop().await;
                if op == PLP {
                    self.p = (v & !16) | 32
                } else { self.a = v; self.nz(v) }
            }
            BPL | BMI | BVC | BVS | BCC | BCS | BNE | BEQ => {
                let branch_irq = self.bus.borrow().last_irq;
                let delta = self.fetch().await as i8;
                branch_poll = Some((branch_irq, self.bus.borrow().nmi));
                let take =
                    match op {
                        BPL => self.p & N == 0,
                        BMI => self.p & N != 0,
                        BVC => self.p & V == 0,
                        BVS => self.p & V != 0,
                        BCC => self.p & C == 0,
                        BCS => self.p & C != 0,
                        BNE => self.p & Z == 0,
                        _ => self.p & Z != 0,
                    };
                if take {
                    let old = self.pc;
                    self.read(old).await;
                    self.pc = self.pc.wrapping_add(delta as u16);
                    if (old ^ self.pc) & 0xff00 != 0 {
                        self.read((old & 0xff00) | (self.pc & 255)).await;
                        let (irq, nmi) = branch_poll.unwrap();
                        branch_poll = Some((irq || self.bus.borrow().last_irq, nmi));
                    }
                }
            }
            _ => {
                let rmw =
                    matches!(op,ASL|LSR|ROL|ROR|INC|DEC|SLO|SRE|RLA|RRA|DCP|ISC);
                let store = matches!(op,STA|STX|STY|SAX|AHX|SHX|SHY|TAS);
                if mode == Imp {
                    self.read(self.pc).await;
                    match op {
                        CLC => self.p &= !C,
                        SEC => self.p |= C,
                        CLI => self.p &= !I,
                        SEI => self.p |= I,
                        CLD => self.p &= !D,
                        SED => self.p |= D,
                        CLV => self.p &= !V,
                        DEY => { self.y = self.y.wrapping_sub(1); self.nz(self.y) }
                        INY => { self.y = self.y.wrapping_add(1); self.nz(self.y) }
                        DEX => { self.x = self.x.wrapping_sub(1); self.nz(self.x) }
                        INX => { self.x = self.x.wrapping_add(1); self.nz(self.x) }
                        TXA => { self.a = self.x; self.nz(self.a) }
                        TYA => { self.a = self.y; self.nz(self.a) }
                        TAX => { self.x = self.a; self.nz(self.x) }
                        TAY => { self.y = self.a; self.nz(self.y) }
                        TSX => { self.x = self.s; self.nz(self.x) }
                        TXS => self.s = self.x,
                        KIL => self.jammed = true,
                        _ => {}
                    }
                } else if mode == Acc {
                    self.read(self.pc).await;
                    self.a = self.modify(op, self.a)
                } else {
                    let addr = self.address(mode, store || rmw).await;
                    if store {
                        let high = ((self.base_addr >> 8) as u8).wrapping_add(1);
                        let v =
                            match op {
                                STA => self.a,
                                STX => self.x,
                                STY => self.y,
                                SAX => self.a & self.x,
                                SHX => self.x & high,
                                SHY => self.y & high,
                                TAS => { self.s = self.a & self.x; self.s & high }
                                _ => self.a & self.x & high,
                            };
                        // These unstable stores can also replace the address high byte
                        // when the index carries out of the operand's low byte.
                        let target = if matches!(op, AHX | SHX | SHY | TAS) &&
                            (self.base_addr ^ addr) & 0xff00 != 0 {
                            ((v as u16) << 8) | (addr & 255)
                        } else { addr };
                        self.write(target, v).await
                    } else {
                        let mut v = self.read(addr).await;
                        if rmw {
                            self.write(addr, v).await;
                            v = self.modify(op, v);
                            self.write(addr, v).await
                        } else {
                            match op {
                                ORA => { self.a |= v; self.nz(self.a) }
                                AND => { self.a &= v; self.nz(self.a) }
                                EOR => { self.a ^= v; self.nz(self.a) }
                                ADC => self.adc(v),
                                SBC => self.adc(!v),
                                LDA => { self.a = v; self.nz(v) }
                                LDX => { self.x = v; self.nz(v) }
                                LDY => { self.y = v; self.nz(v) }
                                LAX => { self.a = v; self.x = v; self.nz(v) }
                                LAS => {
                                    self.a = v & self.s;
                                    self.x = self.a;
                                    self.s = self.a;
                                    self.nz(self.a)
                                }
                                CMP => self.cmp(self.a, v),
                                CPX => self.cmp(self.x, v),
                                CPY => self.cmp(self.y, v),
                                BIT => {
                                    self.flag(Z, self.a & v == 0);
                                    self.p = (self.p & !192) | (v & 192)
                                }
                                ANC => {
                                    self.a &= v;
                                    self.nz(self.a);
                                    self.flag(C, self.a & 128 != 0)
                                }
                                ALR => { self.a &= v; self.a = self.modify(LSR, self.a) }
                                ARR => {
                                    self.a &= v;
                                    self.a = (self.a >> 1) | ((self.p & C) << 7);
                                    self.nz(self.a);
                                    self.flag(C, self.a & 64 != 0);
                                    self.flag(V, ((self.a >> 6) ^ (self.a >> 5)) & 1 != 0)
                                }
                                AXS => {
                                    let a = self.a & self.x;
                                    self.x = a.wrapping_sub(v);
                                    self.flag(C, a >= v);
                                    self.nz(self.x)
                                }
                                XAA => { self.a = self.x & v; self.nz(self.a) }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        let i = if matches!(op,CLI|SEI|PLP) { old_i } else { self.p & I };
        let (irq, nmi) =
            branch_poll.unwrap_or((self.bus.borrow().last_irq,
                    self.bus.borrow().nmi));
        self.irq_pending = irq && i == 0;
        self.nmi_pending = nmi;
    }
    fn modify(&mut self, op: Op, v: u8) -> u8 {
        use Op::*;
        let carry = self.p & C;
        let r =
            match op {
                ASL | SLO => { self.flag(C, v & 128 != 0); v << 1 }
                LSR | SRE => { self.flag(C, v & 1 != 0); v >> 1 }
                ROL | RLA => { self.flag(C, v & 128 != 0); (v << 1) | carry }
                ROR | RRA => {
                    self.flag(C, v & 1 != 0);
                    (v >> 1) | (carry << 7)
                }
                INC | ISC => v.wrapping_add(1),
                DEC | DCP => v.wrapping_sub(1),
                _ => v,
            };
        match op {
            SLO => { self.a |= r; self.nz(self.a) }
            SRE => { self.a ^= r; self.nz(self.a) }
            RLA => { self.a &= r; self.nz(self.a) }
            RRA => self.adc(r),
            DCP => self.cmp(self.a, r),
            ISC => self.adc(!r),
            _ => self.nz(r),
        }
        r
    }
}

/// A bus-cycle scheduler keeps instructions suspended at frame boundaries.
/// Controller changes therefore take effect before the next CPU bus cycle.
pub struct Nes {
    pub bus: Rc<RefCell<Bus>>,
    cpu: Cpu,
    pending: Option<Pin<Box<dyn Future<Output = Cpu>>>>,
    first_frame: bool,
}
impl Nes {
    pub fn new(cart: Cart) -> Self {
        let pc =
            cart.read(0xfffc, 0) as u16 |
                ((cart.read(0xfffd, 0) as u16) << 8);
        let bus = Rc::new(RefCell::new(Bus::new(cart)));
        let cpu =
            Cpu {
                bus: bus.clone(),
                a: 0,
                x: 0,
                y: 0,
                s: 253,
                p: 36,
                pc,
                base_addr: 0,
                irq_pending: false,
                nmi_pending: false,
                jammed: false,
            };
        Self { bus, cpu, pending: None, first_frame: true }
    }
    pub fn set_keys(&mut self, k: u8) {
        let mut b = self.bus.borrow_mut();
        b.keys = k;
        if b.strobe { b.shift[0] = k }
    }
    pub fn pc(&self) -> u16 { self.cpu.pc }
    pub fn clock(&mut self) {
        let mut cx = Context::from_waker(Waker::noop());
        if self.pending.is_none() {
            let mut cpu = self.cpu.clone();
            self.pending =
                Some(Box::pin(async move { cpu.step().await; cpu }));
            let result =
                self.pending.as_mut().unwrap().as_mut().poll(&mut cx);
            assert!(result.is_pending());
        }
        self.bus.borrow_mut().execute();
        if let Poll::Ready(cpu) =
                self.pending.as_mut().unwrap().as_mut().poll(&mut cx) {
            self.cpu = cpu;
            self.pending = None;
        }
    }
    pub fn run_frame(&mut self) {
        if self.first_frame { self.first_frame = false; return }
        let frame = self.bus.borrow().ppu.frame;
        while self.bus.borrow().ppu.frame == frame ||
            (self.pending.is_some() && !self.bus.borrow().last_dma_cycle) {
            self.clock()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oam_dma_suspends_at_the_frame_boundary_and_resumes() {
        let mut rom = vec![0;16+32768];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4]=2;
        let mut code=vec![0x78,0xa9,1,0x8d,0x16,0x40];
        for _ in 0..23{code.extend_from_slice(&[0xa2,255,0xca,0xd0,0xfd]);}
        code.extend(std::iter::repeat_n(0xea,200));
        code.extend_from_slice(&[0xa9,2,0x8d,0x14,0x40]);
        let end=0x8000+code.len() as u16;
        code.extend_from_slice(&[0x4c,end as u8,(end>>8) as u8]);
        rom[16..16+code.len()].copy_from_slice(&code);
        rom[16+32764..16+32766].copy_from_slice(&0x8000u16.to_le_bytes());
        let mut nes=Nes::new(Cart::load(&rom).unwrap());
        for i in 0..256{nes.bus.borrow_mut().ram[512+i]=i as u8;}
        nes.run_frame();
        assert_eq!(nes.bus.borrow().cycles,8);
        nes.run_frame();
        assert!(nes.bus.borrow().oam_active);
        assert_eq!(nes.bus.borrow().ppu.oam[255],0);
        nes.run_frame();
        assert!(!nes.bus.borrow().oam_active);
        assert_eq!(nes.bus.borrow().ppu.oam[255],255);
    }
}

#[cfg(test)]
mod unofficial_tests {
    use super::*;
    #[test]
    fn shx_crossing_a_page_masks_data_and_the_destination_address() {
        let mut rom=vec![0;16+32768];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4]=2;
        rom[16..24].copy_from_slice(&[0xa2,3,0xa0,1,0x9e,0xff,0x60,0x02]);
        rom[16+32764..16+32766].copy_from_slice(&0x8000u16.to_le_bytes());
        let mut nes=Nes::new(Cart::load(&rom).unwrap());
        nes.run_frame();
        nes.run_frame();
        assert_eq!(nes.bus.borrow().ram[0x100],1);
        assert_eq!(nes.bus.borrow().cart.ram[0x100],0);
    }
}

#[cfg(test)]
mod dma_timing_tests {
    use super::*;

    #[test]
    fn oam_dma_gets_have_even_parity_and_stall_for_513_or_514_cycles() {
        for (prefix, expected_stall) in [(vec![], 513), (vec![0x24, 0], 514)] {
            let mut rom = vec![0; 16 + 32768];
            rom[..4].copy_from_slice(b"NES\x1a");
            rom[4] = 2;
            let mut code = prefix;
            code.extend_from_slice(&[0xa9, 2, 0x8d, 0x14, 0x40, 0x02]);
            rom[16..16 + code.len()].copy_from_slice(&code);
            rom[16 + 32764..16 + 32766].copy_from_slice(&0x8000u16.to_le_bytes());
            let mut nes = Nes::new(Cart::load(&rom).unwrap());
            for i in 0..256 { nes.bus.borrow_mut().ram[512 + i] = i as u8; }
            while nes.bus.borrow().dma.is_none() { nes.clock(); }
            let start = nes.bus.borrow().cycles;
            nes.clock();
            let mut writes = 0;
            while nes.bus.borrow().oam_active {
                let address = nes.bus.borrow().ppu.oam_addr;
                nes.clock();
                let bus = nes.bus.borrow();
                if bus.ppu.oam_addr != address {
                    // Each PUT follows an even GET, so the PUT is odd.
                    assert_eq!(bus.cycles & 1, 1);
                    writes += 1;
                }
            }
            assert_eq!(writes, 256);
            let bus = nes.bus.borrow();
            assert_eq!(bus.cycles - start, expected_stall);
            for i in 0..256 { assert_eq!(bus.ppu.oam[i], i as u8); }
        }
    }
}

#[cfg(test)]
mod branch_interrupt_tests {
    use super::*;

    #[test]
    fn nmi_during_the_last_taken_branch_cycle_waits_for_the_next_instruction() {
        for (address, branch_cycles) in [(0x8000u16, 3), (0x80fc, 4)] {
            let mut rom = vec![0; 16 + 32768];
            rom[..4].copy_from_slice(b"NES\x1a");
            rom[4] = 2;
            let offset = 16 + (address - 0x8000) as usize;
            rom[offset..offset + 4].copy_from_slice(&[0xc5, 0, 0xf0, 0xfc]);
            rom[16 + 32764..16 + 32766].copy_from_slice(&address.to_le_bytes());
            let mut nes = Nes::new(Cart::load(&rom).unwrap());
            for _ in 0..3 + branch_cycles - 1 { nes.clock(); }
            nes.bus.borrow_mut().nmi = true;
            nes.clock();
            assert_eq!(nes.pc(), address);
            assert!(!nes.cpu.nmi_pending);
            for _ in 0..3 { nes.clock(); }
            assert!(nes.cpu.nmi_pending);
        }
    }
}
