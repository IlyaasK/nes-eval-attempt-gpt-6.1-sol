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
enum CycleOp { Read(u16), Write(u16, u8), }
#[derive(Clone, Copy)]
enum DmcDmaStage { Dummy, Get }
#[derive(Clone, Copy)]
enum DmaAction { Hold, OamGet(u16), OamPut(u8), DmcGet(u16) }
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
    pad_oe: Option<u16>,
    pad_bit: u8,
    dma: Option<u8>,
    nmi: bool,
    last_irq: bool,
    nmi_sample: bool,
    oam_active: bool,
    dmc_active: bool,
    last_dma_cycle: bool,
    dma_halt_addr: u16,
    oam_page: u8,
    oam_index: u16,
    oam_latch: u8,
    oam_have_data: bool,
    dmc_pending_at: Option<u64>,
    dmc_stage: Option<DmcDmaStage>,
    last_dmc_get: Option<u64>,
    dmc_register_selector: Option<u8>,
    dmc_open_before_get: u8,
    dmc_forced: bool,
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
            pad_oe: None,
            pad_bit: 0,
            dma: None,
            nmi: false,
            last_irq: false,
            nmi_sample: false,
            oam_active: false,
            dmc_active: false,
            last_dma_cycle: false,
            dma_halt_addr: 0,
            oam_page: 0,
            oam_index: 0,
            oam_latch: 0,
            oam_have_data: false,
            dmc_pending_at: None,
            dmc_stage: None,
            last_dmc_get: None,
            dmc_register_selector: None,
            dmc_open_before_get: 0,
            dmc_forced: false,
            request: None,
            result: None,
        }
    }
    /// Advance one CPU clock. A DMA cycle retains the CPU's read request
    /// and leaves its future suspended until the bus is returned to it.
    fn execute(&mut self) -> bool {
        let op = self.request.expect("CPU bus request");
        // Timer/reader requests become visible in this clock before DMA
        // arbitrates the CPU's queued bus operation.
        self.tick();
        let next_cycle = self.cycles;
        let get_cycle = next_cycle & 1 == 0;
        let cpu_read = matches!(op, CycleOp::Read(_));
        let aborted_reload = self.apu.dmc_abort_at == Some(next_cycle);
        if aborted_reload { self.apu.dmc_abort_at = None; }
        let was_halted = self.oam_active || self.dmc_active;
        let mut oam_started = false;
        let mut dmc_started = false;
        if !self.apu.dmc_need() && !self.dmc_active && !self.dmc_forced {
            self.dmc_pending_at = None;
        }
        if cpu_read {
            if let Some(page) = self.dma.take() {
                self.oam_active = true;
                self.oam_page = page;
                self.oam_index = 0;
                self.oam_have_data = false;
                oam_started = true;
            }
            if !self.dmc_active && self.last_dmc_get != Some(next_cycle - 1)
                && self.dmc_pending_at.is_some_and(|t| t <= next_cycle) {
                self.dmc_pending_at = None;
                self.dmc_active = true;
                self.dmc_stage = Some(DmcDmaStage::Dummy);
                dmc_started = true;
            }
        }
        if !was_halted && (oam_started || dmc_started) {
            if let CycleOp::Read(a) = op { self.dma_halt_addr = a; }
        }
        #[cfg(feature="trace")]
        if dmc_started {
            eprintln!("DMC_HALT {} {:04x}", self.cycles, self.dma_halt_addr);
        }
        self.last_dma_cycle = self.oam_active || self.dmc_active;
        if aborted_reload && cpu_read && !self.last_dma_cycle {
            // This request has only its halt clock, with no dummy or get.
            // A coincident CPU write cancels it; an OAM DMA can absorb it.
            if let CycleOp::Read(a) = op { self.dma_halt_addr = a; }
            self.last_dma_cycle = true;
            let open = self.open;
            self.raw_read(self.dma_halt_addr);
            self.open = open;
            return false;
        }
        if self.last_dma_cycle {
            let dmc_get = !dmc_started && get_cycle &&
                matches!(self.dmc_stage, Some(DmcDmaStage::Get));
            let action = if dmc_get {
                DmaAction::DmcGet(self.apu.dmc_addr)
            } else if self.oam_active && !oam_started {
                if get_cycle && !self.oam_have_data {
                    DmaAction::OamGet((self.oam_page as u16) << 8 | self.oam_index)
                } else if !get_cycle && self.oam_have_data {
                    DmaAction::OamPut(self.oam_latch)
                } else { DmaAction::Hold }
            } else { DmaAction::Hold };
            match action {
                DmaAction::Hold => {
                    // A repeated read can affect peripherals, but a DMA
                    // setup/alignment clock does not replace the bus latch.
                    let open = self.open;
                    self.raw_read(self.dma_halt_addr);
                    self.open = open;
                }
                DmaAction::OamGet(a) => {
                    self.oam_latch = self.raw_dma_read(a);
                    self.oam_have_data = true;
                }
                DmaAction::OamPut(v) => {
                    self.pad_oe = None;
                    let a = self.ppu.oam_addr;
                    self.ppu.oam[a as usize] = v;
                    self.ppu.oam_addr = a.wrapping_add(1);
                    self.oam_have_data = false;
                    self.oam_index += 1;
                    if self.oam_index == 256 { self.oam_active = false; }
                }
                DmaAction::DmcGet(a) => {
                    self.dmc_open_before_get = self.open;
                    let v = self.raw_dma_read(a);
                    self.last_dmc_get = Some(self.cycles);
                    self.dmc_register_selector = Some(self.dmc_open_before_get);
                    #[cfg(feature="trace")]
                    eprintln!("DMC_GET {} {:04x} {:04x} {}", self.cycles,
                        a, self.dma_halt_addr, v);
                    self.apu.dmc_supply(v);
                    if self.apu.dmc_need() {
                        self.dmc_pending_at = Some(self.cycles + 1);
                    }
                    self.dmc_active = false;
                    self.dmc_forced = false;
                    self.dmc_stage = None;
                }
            }
            if !dmc_started && matches!(self.dmc_stage, Some(DmcDmaStage::Dummy)) {
                self.dmc_stage = Some(DmcDmaStage::Get);
            }
            return false;
        }
        self.request = None;
        let value = match op {
            CycleOp::Read(a) => {
                // A DMC transfer retains the pre-get bus byte as a register
                // selector. The next CPU I/O read consumes it; RAM, ROM,
                // operand fetches and writes do not. Matching a canonical
                // register repeats its read without adding a CPU clock.
                // Mirrored PPU ports consume the latch but do not repeat.
                let selector = if (0x2000..0x4020).contains(&a) {
                    self.dmc_register_selector.take()
                } else { None };
                let repeat_read = ((0x2000..=0x2007).contains(&a) ||
                    (0x4000..0x4020).contains(&a)) && selector == Some(a as u8);
                if repeat_read {
                    self.raw_read(a);
                    self.pad_oe = None;
                }
                self.raw_read(a)
            },
            CycleOp::Write(a, v) => { self.raw_write(a, v); 0 }
        };
        self.result = Some(value);
        true
    }
    fn tick(&mut self) {
        self.last_irq =
            self.cart.irq || self.apu.frame_irq || self.apu.dmc_irq;
        self.cycles += 1;
        let dmc_needed = self.apu.dmc_need();
        self.apu.tick();
        if self.apu.dmc_extra_at.is_some_and(|t| t <= self.cycles) {
            let t = self.apu.dmc_extra_at.take().unwrap();
            self.dmc_pending_at = Some(t);
            self.dmc_forced = true;
        }
        if self.apu.dmc_need() && !self.dmc_active && self.dmc_pending_at.is_none()
            && (!dmc_needed || self.apu.dmc_output_clock == Some(self.cycles))
            && self.last_dmc_get != Some(self.cycles - 1) {
            // The reader requests RDY in the PUT phase which transfers the
            // sample buffer to the output shifter, including this clock.
            // The first clock after a get cannot start a second request;
            // an empty packet retries on its next output boundary.
            self.dmc_pending_at = Some(self.cycles + if self.cycles & 1 == 0 { 1 } else { 0 });
        }
        self.ppu.tick(&mut self.cart);
        if self.ppu.nmi_line && !self.nmi_sample { self.nmi = true }
        self.nmi_sample = self.ppu.nmi_line;
        for _ in 0..2 { self.ppu.tick(&mut self.cart) }
    }
    fn raw_read(&mut self, a: u16) -> u8 {
        if a != 0x4016 && a != 0x4017 { self.pad_oe = None; }
        let v =
            match a {
                0..=0x1fff => self.ram[a as usize & 2047],
                0x2000..=0x3fff => self.ppu.read_reg(&mut self.cart, a),
                0x4015 => {
                    // The status register's floating bit retains the bus
                    // from its stalled access rather than the DMC ROM
                    // byte. Controller upper bits still see that ROM byte.
                    let open = if self.dma_halt_addr == 0x4015 &&
                        self.last_dmc_get == Some(self.cycles - 1) {
                        self.dmc_open_before_get
                    } else { self.open };
                    self.apu.status(open)
                },
                0x4016 | 0x4017 => {
                    let i = (a & 1) as usize;
                    let b =
                        if self.strobe {
                            if i == 0 { self.keys & 1 } else { 0 }
                        } else if self.pad_oe == Some(a) {
                            self.pad_bit
                        } else {
                            let b = self.shift[i] & 1;
                            self.shift[i] = (self.shift[i] >> 1) | 128;
                            b
                        };
                    self.pad_oe = Some(a);
                    self.pad_bit = b;
                    (self.open & 224) | b
                }
                0x4000..=0x401f => self.open,
                _ => self.cart.read(a, self.open),
            };
        #[cfg(feature="trace")]
        if (0x2000..0x4000).contains(&a) && a & 7 == 7 {
            eprintln!("PPU_READ {} {:04x} {}", self.cycles, a, v);
        }
        #[cfg(feature="trace")]
        if a == 0x6000 && self.cart.mapper == 1 && self.cart.ram.len() > 8192 {
            eprintln!("RAM_READ {} {} {} {} {}", self.cycles, self.ppu.frame,
                self.ppu.scan, self.ppu.dot, v);
        }
        self.open = v;
        v
    }
    fn raw_dma_read(&mut self, a: u16) -> u8 {
        // External RAM, cartridge and PPU decode the DMA address. The
        // internal 2A03 register decoder instead keeps the halted CPU's
        // high address bits and takes only the DMA address's low five bits.
        let previous_oe = self.pad_oe;
        let previous_bit = self.pad_bit;
        let external = if (0x4000..=0x401f).contains(&a) {
            self.open
        } else { self.raw_read(a) };
        let internal = (self.dma_halt_addr & 0xffe0) | (a & 31);
        self.open = external;
        let value = if internal == 0x4016 || internal == 0x4017 {
            // A shadow read of the same controller keeps /OE asserted even
            // while the cartridge is driving the external DMA bus.
            self.pad_oe = previous_oe;
            self.pad_bit = previous_bit;
            self.raw_read(internal)
        } else {
            self.pad_oe = None;
            if internal == 0x4015 { self.raw_read(internal) } else { external }
        };
        self.open = value;
        value
    }

    fn raw_write(&mut self, a: u16, v: u8) {
        self.pad_oe = None;

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
            0x4000..=0x4017 => {
                let needed = self.apu.dmc_need();
                self.apu.write(a, v);
                if a == 0x4015 {
                    if !self.apu.dmc_need() {
                        self.dmc_pending_at = None;
                    } else if !needed {
                        // The first load halts on GET in the second following
                        // APU cycle; later reloads use the PUT phase.
                        self.dmc_pending_at = Some(self.cycles + if self.cycles & 1 == 0 { 4 } else { 3 });
                    }
                }
            }
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
    async fn read(&mut self, a: u16) -> u8 {
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
            Abs => {
                let a = self.word().await;
                a
            },
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
                let a = l | ((self.read(b.wrapping_add(1) as u16).await as u16) << 8);
                a
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
                // Branches poll IRQ before the operand fetch. NMI's edge
                // latch is sampled with that fetch; page fixup adds an IRQ
                // poll but does not add a second NMI poll in the reference.
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
        if !self.bus.borrow_mut().execute() { return }
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
        self.bus.borrow_mut().ppu.publish_next_pixel();
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
    fn dmc_initial_load_straddling_output_boundary_loses_or_accelerates_a_reload() {
        for (length,enable,last_get,address,left,stalls) in [
            (1u8,802u64,808,0xc801,16,3),
            (1,804,814,0xc802,15,6),
            (1,806,812,0xc801,16,3),
            (0,804,810,0xc801,0,3),
        ] {
            let mut rom = vec![0;16+32768];
            rom[..4].copy_from_slice(b"NES\x1a");
            rom[4] = 2;
            let mut code = vec![0x78,0xa9,64,0x8d,0x17,0x40,
                0xa9,15,0x8d,0x10,0x40,0xa9,64,0x8d,0x11,0x40,
                0xa9,32,0x8d,0x12,0x40,0xa9,length,0x8d,0x13,0x40];
            code.extend(std::iter::repeat_n(0xea,((enable-46)/2) as usize));
            code.extend_from_slice(&[0xa9,16,0x8d,0x15,0x40]);
            let end = 0x8000+code.len() as u16;
            code.extend_from_slice(&[0x4c,end as u8,(end>>8) as u8]);
            rom[16..16+code.len()].copy_from_slice(&code);
            rom[16+0x4800] = 255;
            rom[16+32764..16+32766].copy_from_slice(&0x8000u16.to_le_bytes());
            let mut nes = Nes::new(Cart::load(&rom).unwrap());
            let mut halted = 0;
            while nes.bus.borrow().cycles < 819 {
                nes.clock();
                if nes.bus.borrow().last_dma_cycle { halted += 1; }
            }
            assert_eq!(halted,stalls);
            let b = nes.bus.borrow();
            assert_eq!(b.last_dmc_get,Some(last_get));
            assert_eq!(b.apu.dmc_addr,address);
            assert_eq!(b.apu.dmc_left,left);
        }
    }
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

#[cfg(test)]
mod dmc_dma_tests {
    use super::*;

    fn fixture(delay: Option<usize>) -> (Nes, u16) {
        let mut rom = vec![0; 16 + 32768];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 2;
        let mut code = vec![
            0x78,
            0xa9, 0x40, 0x8d, 0x17, 0x40,
            0xa9, 0x4f, 0x8d, 0x10, 0x40,
            0xa9, 0x00, 0x8d, 0x11, 0x40,
            0xa9, 0x20, 0x8d, 0x12, 0x40,
            0xa9, 0x01, 0x8d, 0x13, 0x40,
            0xa9, 0x10, 0x8d, 0x15, 0x40,
            0xa9, 0x7f, 0x8d, 0x11, 0x40,
        ];
        if let Some(delay) = delay {
            let mut count = (delay - 1) / 5;
            let mut remainder = delay - (5 * count + 1);
            if remainder == 1 { count -= 1; remainder += 5; }
            code.extend_from_slice(&[0xa2, count as u8, 0xca, 0xd0, 0xfd]);
            if remainder & 1 != 0 {
                code.extend_from_slice(&[0x24, 0]);
                remainder -= 3;
            }
            code.extend(std::iter::repeat_n(0xea, remainder / 2));
            code.extend_from_slice(&[0xa9, 2, 0x8d, 0x14, 0x40,
                0xa9, 0x7e, 0x8d, 0x11, 0x40]);
        }
        let end = 0x8000 + code.len() as u16;
        code.extend_from_slice(&[0x4c, end as u8, (end >> 8) as u8]);
        rom[16..16 + code.len()].copy_from_slice(&code);
        rom[16 + 0x4800..16 + 0x4811].fill(255);
        rom[16 + 32764..16 + 32766].copy_from_slice(&0x8000u16.to_le_bytes());
        (Nes::new(Cart::load(&rom).unwrap()), end)
    }

    #[test]
    fn first_dmc_load_waits_for_get_in_the_second_following_apu_cycle() {
        let (mut nes, end) = fixture(None);
        while nes.bus.borrow().cycles < 49 { nes.clock(); }
        assert_eq!(nes.bus.borrow().apu.dmc_left, 17);
        assert!(!nes.bus.borrow().dmc_active);
        nes.clock();
        assert!(nes.bus.borrow().dmc_active);
        nes.clock();
        assert_eq!(nes.bus.borrow().apu.dmc_left, 17);
        nes.clock();
        assert_eq!(nes.bus.borrow().cycles, 52);
        assert_eq!(nes.bus.borrow().apu.dmc_left, 16);
        assert_eq!(nes.bus.borrow().apu.dmc_addr, 0xc801);
        while nes.bus.borrow().cycles < 55 { nes.clock(); }
        assert_eq!(nes.pc(), end);
        assert_eq!(nes.bus.borrow().open, 127);
    }

    #[test]
    fn reload_overlaps_oam_setup_and_preserves_get_put_parity() {
        let (mut nes, _) = fixture(Some(401));
        for i in 0..256 { nes.bus.borrow_mut().ram[512 + i] = i as u8; }
        while nes.bus.borrow().dma.is_none() { nes.clock(); }
        let start = nes.bus.borrow().cycles;
        assert_eq!(start, 462);
        nes.clock();
        let mut puts = 0;
        while nes.bus.borrow().oam_active {
            let address = nes.bus.borrow().ppu.oam_addr;
            nes.clock();
            let b = nes.bus.borrow();
            if b.ppu.oam_addr != address {
                assert_eq!(b.cycles & 1, 1);
                puts += 1;
            }
        }
        assert_eq!(puts, 256);
        assert_eq!(nes.bus.borrow().cycles - start, 515);
        for i in 0..256 { assert_eq!(nes.bus.borrow().ppu.oam[i], i as u8); }
    }

    #[test]
    fn a_controller_clocks_once_per_contiguous_output_enable() {
        let (nes, _) = fixture(None);
        let mut b = nes.bus.borrow_mut();
        b.keys = 2;
        b.raw_write(0x4016, 1);
        b.raw_write(0x4016, 0);
        assert_eq!(b.raw_read(0x4016) & 1, 0);
        assert_eq!(b.raw_read(0x4016) & 1, 0);
        assert_eq!(b.raw_read(0x4016) & 1, 0);
        b.raw_read(0x8000);
        assert_eq!(b.raw_read(0x4016) & 1, 1);
    }

    #[test]
    fn dmc_address_fetch_conflicts_repeat_ports_and_respect_page_crossing() {
        fn run(setup: &[u8], access: &[u8], frame_irq: bool) -> u8 {
            let mut rom = vec![0; 16 + 32768];
            rom[..4].copy_from_slice(b"NES\x1a");
            rom[4] = 2;
            let mut code = vec![0x78, 0xa9,0x4f,0x8d,0x10,0x40,
                0xa9,0x20,0x8d,0x12,0x40, 0xa9,1,0x8d,0x13,0x40,
                0xa9,1,0x8d,0x16,0x40, 0xa9,0,0x8d,0x16,0x40];
            code.extend_from_slice(setup);
            code.extend_from_slice(&[0xa9,16,0x8d,0x15,0x40]);
            code.extend_from_slice(access);
            code.extend_from_slice(&[0x8d,0,0x60,0x02]);
            rom[16..16 + code.len()].copy_from_slice(&code);
            rom[16 + 0x4800..16 + 0x4811].fill(255);
            rom[16 + 32764..16 + 32766].copy_from_slice(&0x8000u16.to_le_bytes());
            let mut nes = Nes::new(Cart::load(&rom).unwrap());
            nes.set_keys(2); // A=0, B=1 makes the deleted bit observable.
            {
                let mut b = nes.bus.borrow_mut();
                b.apu.frame_irq = frame_irq;
                for (a, v) in [(0,17), (1,34), (2,51)] { b.cart.chr_write(a, v); }
            }
            while nes.bus.borrow().cycles < 200 { nes.clock(); }
            let value = nes.bus.borrow().cart.ram[0];
            value
        }
        // An odd enable interrupts the last absolute operand fetch.
        assert_eq!(run(&[0x24,0], &[0xad,0x16,0x40,0x29,1], false), 1);
        assert_eq!(run(&[0x24,0], &[0xad,7,0x20], false), 17);
        assert_eq!(run(&[0x24,0], &[0xad,0x15,0x40], true), 16);
        // If DMA halts the port itself, its existing repeated reads suffice;
        // $4015's undriven bit must not pick up the DMC ROM's $FF.
        assert_eq!(run(&[], &[0xad,0x15,0x40], true), 16);
        // Indirect address reads also carry the conflict to the data read.
        assert_eq!(run(&[0xa9,0x16,0x85,0x10,0xa9,0x40,0x85,0x11,0xa0,0],
            &[0xb1,0x10,0x29,1], false), 1);
        // The page-crossing dummy read consumes it before accessing the pad.
        assert_eq!(run(&[0xa2,23], &[0xbd,0xff,0x3f,0x29,1], false), 0);
        // An indexed operand's low byte need not equal the final register.
        // Matching the instruction phase alone would repeat these wrongly.
        assert_eq!(run(&[0xa2,7,0x24,0], &[0xbd,0,0x20], false), 0);
        assert_eq!(run(&[0x24,0], &[0xad,0x0f,0x20], false), 0);
    }

    #[test]
    fn a_dmc_bus_selector_survives_ram_reads_but_is_consumed_by_io_reads() {
        for (intervening, expected) in [
            (&[][..],34), (&[0x24,0][..],34), (&[0x2c,2,0x20][..],17),
        ] {
            let mut code = vec![0xa9,7,0x8d,1,0x60,0xca,0xd0,0];
            code.extend_from_slice(intervening);
            code.extend_from_slice(&[0xad,7,0x20,0x8d,0,0x60,0x02]);
            let mut rom = vec![0; 16 + 32768];
            rom[..4].copy_from_slice(b"NES\x1a");
            rom[4] = 2;
            rom[16..16 + code.len()].copy_from_slice(&code);
            rom[16 + 0x4800..16 + 0x4811].fill(255);
            rom[16 + 32764..16 + 32766].copy_from_slice(&0x8000u16.to_le_bytes());
            let mut nes = Nes::new(Cart::load(&rom).unwrap());
            {
                let mut b = nes.bus.borrow_mut();
                for (a, v) in [(0,17), (1,34), (2,51)] { b.cart.chr_write(a, v); }
                let Bus { ppu, cart, .. } = &mut *b;
                ppu.read_reg(cart, 0x2007);
                // The request coincides with STA's write clock, so it waits
                // for DEX's opcode fetch. The CPU bus is still the written7.
                b.apu.dmc_addr = 0xc800;
                b.apu.dmc_left = 17;
                b.dmc_pending_at = Some(14);
            }
            while nes.bus.borrow().cycles < 100 { nes.clock(); }
            assert_eq!(nes.bus.borrow().cart.ram[0], expected);
        }
    }

    #[test]
    fn stopping_dmc_retains_aborted_and_already_scheduled_reloads() {
        // Original probe stops at807, two clocks before buffer transfer809.
        // Its DAC marker normally writes at827; the aborted halt moves it
        // to828. Moving the stop to809 retains a full DMA and writes at832.
        let original = [
            0x78,0xa9,0x40,0x8d,0x17,0x40,0xa9,0x0f,0x8d,0x10,0x40,
            0xa9,0x40,0x8d,0x11,0x40,0xa9,0x20,0x8d,0x12,0x40,
            0xa9,1,0x8d,0x13,0x40,0xa9,0x10,0x8d,0x15,0x40,
            0xa2,0x95,0xca,0xd0,0xfd,0xa2,1,0xca,0xd0,0xfd,
            0xa9,0,0x8d,0x15,0x40,0xa2,3,0xca,0xd0,0xfd,0x8d,0x11,0x40,
        ];
        for (shift, marker) in [(false,828), (true,832)] {
            let mut code = original.to_vec();
            if shift { code.insert(41, 0xea); }
            let end = 0x8000 + code.len() as u16;
            code.extend_from_slice(&[0x4c,end as u8,(end >> 8) as u8]);
            let mut rom = vec![0; 16 + 32768];
            rom[..4].copy_from_slice(b"NES\x1a");
            rom[4] = 2;
            rom[16..16 + code.len()].copy_from_slice(&code);
            rom[16 + 0x4800..16 + 0x4900].fill(255);
            rom[16 + 32764..16 + 32766].copy_from_slice(&0x8000u16.to_le_bytes());
            let mut nes = Nes::new(Cart::load(&rom).unwrap());
            while !matches!(nes.bus.borrow().request,
                Some(CycleOp::Write(0x4011,0))) { nes.clock(); }
            nes.clock();
            assert_eq!(nes.bus.borrow().cycles, marker);
            assert_eq!(nes.bus.borrow().apu.dmc_left, 0);
        }
    }

    #[test]
    fn a_one_byte_load_can_abort_a_reload_but_does_not_fetch_a_later_chip_extra_byte() {
        // Load at800 ends at806, three clocks before the shifter transfer.
        // An extra halt at809 moves its marker from823 to824. Loading two
        // clocks later ends at808, and the reference has no full extra DMA.
        let original = [0x78,0xa9,0x40,0x8d,0x17,0x40,
            0xa9,0x0f,0x8d,0x10,0x40,0xa9,0x40,0x8d,0x11,0x40,
            0xa9,0x20,0x8d,0x12,0x40,0xa9,0,0x8d,0x13,0x40,
            0xa2,0x96,0xca,0xd0,0xfd,0x24,0,0xa9,0x10,0x8d,0x15,0x40,
            0xa2,2,0xca,0xd0,0xfd,0x24,0,0xa9,0,0x8d,0x11,0x40];
        for (shift, marker) in [(false,824), (true,825)] {
            let mut code = original.to_vec();
            if shift { code.insert(33, 0xea); }
            let end = 0x8000 + code.len() as u16;
            code.extend_from_slice(&[0x4c,end as u8,(end >> 8) as u8]);
            let mut rom = vec![0; 16 + 32768];
            rom[..4].copy_from_slice(b"NES\x1a");
            rom[4] = 2;
            rom[16..16 + code.len()].copy_from_slice(&code);
            rom[16 + 0x4800] = 255;
            rom[16 + 32764..16 + 32766].copy_from_slice(&0x8000u16.to_le_bytes());
            let mut nes = Nes::new(Cart::load(&rom).unwrap());
            while !matches!(nes.bus.borrow().request,
                Some(CycleOp::Write(0x4011,0))) { nes.clock(); }
            nes.clock();
            assert_eq!(nes.bus.borrow().cycles, marker);
            assert_eq!(nes.bus.borrow().apu.dmc_left, 0);
        }
    }

    #[test]
    fn a_dmc_shadow_read_keeps_the_same_controller_output_enable() {
        let (nes, _) = fixture(None);
        let mut b = nes.bus.borrow_mut();
        b.keys = 2;
        b.raw_write(0x4016, 1);
        b.raw_write(0x4016, 0);
        assert_eq!(b.raw_read(0x4016) & 1, 0);
        b.dma_halt_addr = 0x4016;
        b.cart.prg[0x4816] = 255;
        // The original shadow-read probe also confirms $E0 as the DMC
        // sample: the reference audio drops five bits then rises three.
        assert_eq!(b.raw_dma_read(0xc816), 0xe0);
        assert_eq!(b.raw_read(0x4016) & 1, 0);
        b.raw_read(0x8000);
        assert_eq!(b.raw_read(0x4016) & 1, 1);
    }

    #[test]
    fn oam_io_decode_and_alignment_preserve_the_observed_bus_data() {
        let (mut nes, end) = fixture(Some(401));
        let mut b = nes.bus.borrow_mut();
        let pos = b.cart.prg.windows(5)
            .position(|bytes| bytes == [0xa9, 2, 0x8d, 0x14, 0x40]).unwrap();
        b.cart.prg[pos + 1] = 0x40;
        b.keys = 1;
        b.raw_write(0x4016, 1);
        drop(b);
        while nes.pc() != end { nes.clock(); }
        let b = nes.bus.borrow();
        // The held CPU is fetching from ROM, so $4015/$4016/$4017 do not
        // decode during DMA from page $40. The prior $4014 data floats.
        assert_eq!(&b.ppu.oam[..32], &[0x40; 32]);
        // A DMC fetch drives $FF later in the transfer. The following OAM
        // alignment read must not replace it with the next CPU opcode.
        assert_eq!(b.ppu.oam[255], 255);
    }

    #[test]
    fn dma_tail_timings_match_the_measured_reference_markers() {
        for (delay, expected) in [
            (231, 815), (232, 817), (233, 817), (234, 818), (235, 818),
            (236, 818), (237, 818), (238, 821), (239, 821), (240, 823),
        ] {
            let (mut nes, end) = fixture(Some(delay));
            while nes.pc() != end { nes.clock(); }
            assert_eq!(nes.bus.borrow().cycles, expected, "delay {delay}");
            assert_eq!(nes.bus.borrow().open, 126);
        }
    }
}
