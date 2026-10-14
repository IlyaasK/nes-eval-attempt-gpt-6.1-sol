#[derive(Clone)]
pub struct Cart {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub ram: Vec<u8>,
    pub chr_ram: bool,
    pub mapper: u16,
    pub mirror: u8,
    pub four: bool,
    bank: u8,
    shift: u8,
    control: u8,
    chr0: u8,
    chr1: u8,
    chr_upper: bool,
    prgb: u8,
    last_write: u64,
    select: u8,
    regs: [u8; 8],
    pub irq: bool,
    irq_enable: bool,
    irq_reload: bool,
    irq_latch: u8,
    irq_counter: u8,
    a12: bool,
    low_since: u64,
    ram_enable: bool,
    ram_protect: bool,
}
impl Cart {
    pub fn load(b: &[u8]) -> Option<Self> {
        if b.len() < 16 || &b[..4] != b"NES\x1a" { return None }
        let nes2 = b[7] & 12 == 8;
        let mapper =
            (b[6] as u16 >> 4) | (b[7] as u16 & 0xf0) |
                if nes2 { (b[8] as u16 & 15) << 8 } else { 0 };
        if ![0, 1, 2, 3, 4].contains(&mapper) { return None }
        fn size(lo: u8, hi: u8, unit: usize) -> Option<usize> {
            if hi == 15 {
                (1usize.checked_shl((lo >> 2) as
                                    u32)?).checked_mul((lo as usize & 3) * 2 + 1)
            } else { Some(((hi as usize) << 8 | lo as usize) * unit) }
        }
        let ps = size(b[4], if nes2 { b[9] & 15 } else { 0 }, 16384)?;
        let cs = size(b[5], if nes2 { b[9] >> 4 } else { 0 }, 8192)?;
        let offset: usize = 16 + if b[6] & 4 != 0 { 512 } else { 0 };
        // The reference rejects PRG images smaller than one 16 KiB bank,
        // including NES2 exponent sizes. Such images cannot supply the
        // mapper windows assumed below and must fail before CPU startup.
        if ps < 16384 || (mapper == 3 && cs == 0)
            || offset.checked_add(ps)?.checked_add(cs)? > b.len() {
            return None
        }
        let ram_size =
            if nes2 {
                let a = b[10] & 15;
                let c = b[10] >> 4;
                (if a > 0 { 64usize << a } else { 0 }) +
                    (if c > 0 { 64usize << c } else { 0 })
            } else { b[8].max(1) as usize * 8192 };
        let chr_size =
            if cs != 0 {
                cs
            } else if nes2 {
                let a = b[11] & 15;
                let c = b[11] >> 4;
                ((if a > 0 { 64usize << a } else { 0 }) +
                            (if c > 0 { 64usize << c } else { 0 })).max(8192)
            } else { 8192 };
        let cart =
            Self {
                prg: b[offset..offset + ps].to_vec(),
                chr: if cs > 0 {
                    b[offset + ps..offset + ps + cs].to_vec()
                } else { vec![0;chr_size] },
                ram: vec![0;if
                    mapper==1||mapper==4{ram_size.max(8192)}else{ram_size}],
                chr_ram: cs == 0,
                mapper,
                mirror: if mapper == 1 { 2 } else { b[6] & 1 },
                four: b[6] & 8 != 0,
                bank: 0,
                shift: 16,
                control: 12,
                chr0: 0,
                chr1: 0,
                chr_upper: false,
                prgb: 0,
                last_write: u64::MAX,
                select: 0,
                regs: [0; 8],
                irq: false,
                irq_enable: false,
                irq_reload: false,
                irq_latch: 0,
                irq_counter: 0,
                a12: false,
                low_since: 0,
                ram_enable: true,
                ram_protect: false,
            };
        Some(cart)
    }
    pub fn read(&self, a: u16, open: u8) -> u8 {
        if a < 0x6000 { return open }
        if a < 0x8000 {
            return if !self.ram.is_empty() && self.ram_enable &&
                        self.mmc1_ram_enabled() {
                    self.ram[self.ram_index(a)]
                } else { open }
        }
        let off = a as usize & 0x1fff;
        let slot = (a as usize - 0x8000) / 8192;
        let n = self.prg.len() / 8192;
        let bank =
            match self.mapper {
                0 | 3 => (a as usize - 0x8000) / 8192,
                2 =>
                    if slot < 2 {
                        self.bank as usize * 2 + slot
                    } else { n - 2 + slot % 2 },
                1 => {
                    let n16 = (n / 2).min(16);
                    let outer =
                        if n >= 64 && self.chr.len() == 8192 {
                            (self.mmc1_chr() as usize & 16) * 2
                        } else { 0 };
                    let s = slot / 2;
                    let b =
                        match (self.control >> 2) & 3 {
                            0 | 1 => (self.prgb as usize & 14) + s,
                            2 => if s == 0 { 0 } else { self.prgb as usize & 15 },
                            _ => if s == 0 { self.prgb as usize & 15 } else { n16 - 1 },
                        };
                    outer + b * 2 + slot % 2
                }
                4 => {
                    let b6 = self.regs[6] as usize & 63;
                    let b7 = self.regs[7] as usize & 63;
                    match slot {
                        0 => if self.select & 64 != 0 { n - 2 } else { b6 },
                        1 => b7,
                        2 => if self.select & 64 != 0 { b6 } else { n - 2 },
                        _ => n - 1,
                    }
                }
                _ => 0,
            };
        self.prg[(bank % n) * 8192 + off]
    }
    pub fn write(&mut self, a: u16, v: u8, cycle: u64) {
        if a < 0x6000 { return }
        if a < 0x8000 {
            if !self.ram.is_empty() && self.ram_enable && !self.ram_protect &&
                    self.mmc1_ram_enabled() {
                let i = self.ram_index(a);
                self.ram[i] = v
            }
            return
        }
        match self.mapper {
            1 => {
                if self.last_write != u64::MAX && cycle == self.last_write + 1
                    {
                    return
                }
                self.last_write = cycle;
                if v & 128 != 0 {
                    self.shift = 16;
                    self.control |= 12
                } else {
                    let done = self.shift & 1 != 0;
                    self.shift = (self.shift >> 1) | ((v & 1) << 4);
                    if done {
                        match (a >> 13) & 3 {
                            0 => self.control = self.shift,
                            1 => self.chr0 = self.shift,
                            2 => self.chr1 = self.shift,
                            _ => self.prgb = self.shift,
                        };
                        self.shift = 16
                    }
                }
                self.mirror =
                    match self.control & 3 { 0 => 2, 1 => 3, 2 => 1, _ => 0, };
            }
            2 => self.bank = v,
            // CNROM's register and PRG ROM drive the bus together. The
            // reference applies conflicts even to NES2 submapper1, while
            // its UxROM implementation accepts the CPU value directly.
            3 => self.bank = v & self.read(a, 0),
            4 =>
                match a & 0xe001 {
                    0x8000 => self.select = v,
                    0x8001 => self.regs[(self.select & 7) as usize] = v,
                    0xa000 =>
                        if !self.four {
                            self.mirror = if v & 1 == 0 { 1 } else { 0 }
                        },
                    0xa001 => {
                        self.ram_enable = v & 128 != 0;
                        self.ram_protect = v & 64 != 0
                    }
                    0xc000 => self.irq_latch = v,
                    0xc001 => self.irq_reload = true,
                    0xe000 => { self.irq_enable = false; self.irq = false }
                    0xe001 => self.irq_enable = true,
                    _ => {}
                },
            _ => {}
        }
    }
    fn mmc1_chr(&self) -> u8 {
        if self.control & 16 != 0 && self.chr_upper { self.chr1 } else { self.chr0 }
    }
    fn mmc1_ram_enabled(&self) -> bool {
        if self.mapper != 1 { return true; }
        let snrom_disabled = self.chr_ram && self.chr.len() == 8192
            && self.prg.len() <= 262144 && self.ram.len() == 8192
            && self.mmc1_chr() & 16 != 0;
        self.prgb & 16 == 0 && !snrom_disabled
    }
    fn ram_index(&self, a: u16) -> usize {
        let bank =
            if self.mapper == 1 && self.chr.len() == 8192 {
                if self.ram.len() >= 32768 {
                    (self.mmc1_chr() as usize >> 2) & 3
                } else if self.ram.len() >= 16384 {
                    (self.mmc1_chr() as usize >> 3) & 1
                } else { 0 }
            } else { 0 };
        (bank * 8192 + (a as usize - 0x6000)) % self.ram.len()
    }
    fn chr_index(&self, a: u16) -> usize {
        let slot = a as usize / 1024;
        let bank =
            match self.mapper {
                1 =>
                    if self.control & 16 != 0 {
                        (if a < 4096 { self.chr0 } else { self.chr1 }) as usize * 4
                            + slot % 4
                    } else { (self.chr0 as usize & 30) * 4 + slot },
                3 => self.bank as usize * 8 + slot,
                4 => {
                    let s = slot ^ if self.select & 128 != 0 { 4 } else { 0 };
                    match s {
                        0 => self.regs[0] as usize & 254,
                        1 => (self.regs[0] as usize & 254) + 1,
                        2 => self.regs[1] as usize & 254,
                        3 => (self.regs[1] as usize & 254) + 1,
                        _ => self.regs[s - 2] as usize,
                    }
                }
                _ => slot,
            };
        (bank * 1024 + (a as usize & 1023)) % self.chr.len()
    }
    pub fn chr_read(&self, a: u16) -> u8 { self.chr[self.chr_index(a)] }
    pub fn chr_write(&mut self, a: u16, v: u8) {
        if self.chr_ram { let i = self.chr_index(a); self.chr[i] = v }
    }
    pub fn nt_index(&self, a: u16) -> usize {
        let a = (a as usize - 0x2000) & 4095;
        let table = a / 1024;
        let t =
            if self.four {
                table
            } else {
                match self.mirror {
                    0 => table / 2,
                    1 => table % 2,
                    2 => 0,
                    _ => 1,
                }
            };
        t * 1024 + (a & 1023)
    }
    pub fn address(&mut self, a: u16, dot: u64) {
        if self.mapper == 1 {
            // SxROM boards reuse the currently selected CHR address lines
            // for PRG ROM/RAM. In 4 KiB mode the PPU chooses the register.
            self.chr_upper = a & 0x1000 != 0;
            return;
        }
        if self.mapper != 4 { return }
        let high = a & 0x1000 != 0;
        if !high {
            if self.a12 { self.low_since = dot }
        } else if !self.a12 && dot.saturating_sub(self.low_since) >= 8 {
            if self.irq_counter == 0 || self.irq_reload {
                self.irq_counter = self.irq_latch
            } else { self.irq_counter -= 1 }
            self.irq_reload = false;
            if self.irq_counter == 0 && self.irq_enable { self.irq = true }
        }
        self.a12 = high;
    }
}

#[cfg(test)]
mod tests {
    use super::Cart;
    fn image(mapper: u8, prg_banks: u8, chr_banks: u8) -> Vec<u8> {
        let mut b =
            vec![0;16+prg_banks as usize*16384+chr_banks as usize*8192];
        b[..4].copy_from_slice(b"NES\x1a");
        b[4] = prg_banks;
        b[5] = chr_banks;
        b[6] = mapper << 4;
        for (i, bank) in
            b[16..16 +
                                prg_banks as usize * 16384].chunks_mut(8192).enumerate() {
            bank.fill(i as u8)
        }
        for (i, bank) in
            b[16 + prg_banks as usize * 16384..].chunks_mut(1024).enumerate()
            {
            bank.fill(i as u8)
        }
        b
    }
    fn serial(c: &mut Cart, a: u16, v: u8, cycle: &mut u64) {
        for bit in 0..5 { c.write(a, (v >> bit) & 1, *cycle); *cycle += 2 }
    }
    #[test]
    fn nrom_16k_mirrors_and_ram_starts_zero() {
        let c = Cart::load(&image(0, 1, 0)).unwrap();
        assert_eq!(c.read(0x8000,0),0);
        assert_eq!(c.read(0xc000,0),0);
        assert_eq!(c.read(0xffff,0),1);
        assert!(c.ram.iter().all(|&b|b==0));
        assert!(c.chr.iter().all(|&b|b==0));
    }
    #[test]
    fn uxrom_keeps_last_bank_fixed() {
        let mut c = Cart::load(&image(2, 4, 0)).unwrap();
        c.write(0x9000, 2, 0);
        assert_eq!(c.read(0x8000,0),4);
        assert_eq!(c.read(0xbfff,0),5);
        assert_eq!(c.read(0xc000,0),6);
        assert_eq!(c.read(0xffff,0),7);
    }
    #[test]
    fn cnrom_chr_banks() {
        let mut c = Cart::load(&image(3, 2, 4)).unwrap();
        c.prg[0] = 255;
        c.write(0x8000, 3, 0);
        assert_eq!(c.chr_read(0),24);
        assert_eq!(c.chr_read(0x1fff),31);
    }
    #[test]
    fn cnrom_conflicts_use_the_rom_byte_at_the_written_address() {
        let mut c = Cart::load(&image(3, 2, 4)).unwrap();
        c.prg[0] = 1;
        c.prg[1] = 2;
        c.write(0x8000, 3, 0);
        assert_eq!(c.chr_read(0), 8);
        c.write(0x8001, 3, 1);
        assert_eq!(c.chr_read(0), 16);
        c.write(0xc000, 1, 2);
        assert_eq!(c.chr_read(0), 0);
    }
    #[test]
    fn mmc1_serial_write_and_rmw_filter() {
        let mut c = Cart::load(&image(1, 4, 2)).unwrap();
        let mut cycle = 0;
        serial(&mut c, 0xe000, 2, &mut cycle);
        assert_eq!(c.read(0x8000,0),4);
        assert_eq!(c.read(0xc000,0),6);
        c.write(0x8000, 128, cycle);
        c.write(0x8000, 1, cycle + 1);
        cycle += 3;
        serial(&mut c, 0x8000, 0x1a, &mut cycle);
        serial(&mut c, 0xa000, 1, &mut cycle);
        serial(&mut c, 0xc000, 2, &mut cycle);
        assert_eq!(c.read(0x8000,0),0);
        assert_eq!(c.read(0xc000,0),4);
        assert_eq!(c.chr_read(0),4);
        assert_eq!(c.chr_read(0x1000),8);
        assert_eq!(c.nt_index(0x2000),0);
        assert_eq!(c.nt_index(0x2400),1024);
        assert_eq!(c.nt_index(0x2800),0);
    }
    #[test]
    fn mmc1_512k_outer_bank() {
        let mut c = Cart::load(&image(1, 32, 0)).unwrap();
        let mut cycle = 0;
        assert_eq!(c.read(0xc000,0),30);
        serial(&mut c, 0xa000, 16, &mut cycle);
        serial(&mut c, 0xe000, 3, &mut cycle);
        assert_eq!(c.read(0x8000,0),38);
        assert_eq!(c.read(0xc000,0),62);
    }
    #[test]
    fn mmc1_outer_and_ram_banks_follow_the_ppu_in_four_kib_mode() {
        let mut rom = image(1, 32, 0);
        rom[8] = 4;
        let mut c = Cart::load(&rom).unwrap();
        let mut cycle = 0;
        serial(&mut c, 0x8000, 28, &mut cycle);
        serial(&mut c, 0xa000, 4, &mut cycle);
        serial(&mut c, 0xc000, 28, &mut cycle);
        serial(&mut c, 0xe000, 2, &mut cycle);
        c.address(0, 0);
        c.write(0x6000, 11, cycle);
        assert_eq!(c.read(0x8000, 0), 4);
        c.address(0x1000, 1);
        c.write(0x6000, 22, cycle);
        assert_eq!(c.read(0x8000, 0), 36);
        assert_eq!(c.read(0x6000, 0), 22);
        c.address(0, 2);
        assert_eq!(c.read(0x6000, 0), 11);
        serial(&mut c, 0x8000, 12, &mut cycle);
        c.address(0x1000, 3);
        assert_eq!(c.read(0x8000, 0), 4);
        assert_eq!(c.read(0x6000, 0), 11);
    }
    #[test]
    fn snrom_chr_ram_disable_preserves_data_and_returns_open_bus() {
        let mut c = Cart::load(&image(1, 4, 0)).unwrap();
        let mut cycle = 0;
        c.write(0x6000, 37, cycle);
        serial(&mut c, 0x8000, 28, &mut cycle);
        serial(&mut c, 0xc000, 16, &mut cycle);
        c.address(0x1000, 0);
        assert_eq!(c.read(0x6000, 0x60), 0x60);
        c.write(0x6000, 99, cycle);
        c.address(0, 1);
        assert_eq!(c.read(0x6000, 0x60), 37);

        // The reference does not apply this extra enable to CHR ROM boards.
        let mut c = Cart::load(&image(1, 4, 1)).unwrap();
        c.write(0x6000, 37, cycle);
        serial(&mut c, 0xa000, 16, &mut cycle);
        assert_eq!(c.read(0x6000, 0x60), 37);
    }
    #[test]
    fn mmc1_expansion_lines_are_not_used_with_larger_chr_rom() {
        let mut rom = image(1, 32, 8);
        rom[8] = 2;
        let mut c = Cart::load(&rom).unwrap();
        let mut cycle = 0;
        c.write(0x6000, 37, cycle);
        serial(&mut c, 0xa000, 24, &mut cycle);
        serial(&mut c, 0xe000, 2, &mut cycle);
        assert_eq!(c.read(0x8000, 0), 4);
        assert_eq!(c.read(0x6000, 0), 37);
    }
    #[test]
    fn mmc3_prg_swap_chr_inversion_and_protected_ram() {
        let mut c = Cart::load(&image(4, 4, 4)).unwrap();
        c.write(0x8000, 6, 0);
        c.write(0x8001, 3, 1);
        assert_eq!(c.read(0x8000,0),3);
        assert_eq!(c.read(0xc000,0),6);
        c.write(0x8000, 0x46, 2);
        assert_eq!(c.read(0x8000,0),6);
        assert_eq!(c.read(0xc000,0),3);
        c.write(0x8000, 0, 3);
        c.write(0x8001, 7, 4);
        assert_eq!(c.chr_read(0),6);
        assert_eq!(c.chr_read(1024),7);
        c.write(0x8000, 0x80, 5);
        assert_eq!(c.chr_read(0x1000),6);
        c.write(0x6000, 45, 6);
        c.write(0xa001, 0xc0, 7);
        c.write(0x6000, 23, 8);
        assert_eq!(c.read(0x6000,0),45);
        c.write(0xa001, 0, 9);
        assert_eq!(c.read(0x6000,0xa5),0xa5);
    }
    #[test]
    fn nes2_requires_mmc_ram_even_if_none_declared() {
        for mapper in [0, 1, 4] {
            let mut b = image(mapper, 2, 0);
            b[7] = 8;
            let c = Cart::load(&b).unwrap();
            assert_eq!(c.ram.len(),if mapper==0{0}else{8192});
            assert_eq!(c.read(0x6000,0xaa),if mapper==0{0xaa}else{0});
        }
    }
    #[test]
    fn malformed_exponent_sizes_fail_without_panicking() {
        let mut b = image(0, 1, 0);
        b[7] = 8;
        b[9] = 255;
        b[4] = 252;
        b[5] = 252;
        assert!(Cart::load(&b).is_none());
    }
    #[test]
    fn nes2_small_prg_images_are_rejected_before_reset_vector_reads() {
        for exponent in 10..14 {
            let mut b = vec![0; 16 + (1 << exponent)];
            b[..4].copy_from_slice(b"NES\x1a");
            b[4] = exponent << 2;
            b[7] = 8;
            b[9] = 15;
            assert!(Cart::load(&b).is_none());
        }
        let c = Cart::load(&image(2, 1, 0)).unwrap();
        assert_eq!(c.read(0xfffc, 0), 1);
        assert_eq!(c.read(0xc000, 0), 0);
    }
    #[test]
    fn cnrom_requires_chr_rom() {
        assert!(Cart::load(&image(3, 1, 0)).is_none());
        assert!(Cart::load(&image(3, 1, 1)).is_some());
    }
}
