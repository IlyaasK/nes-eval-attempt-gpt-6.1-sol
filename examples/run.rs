use nes_emu::{cart::Cart, cpu::Nes};
use std::{env, fs};
fn main() {
    let args: Vec<_> = env::args().collect();
    let rom = fs::read(&args[1]).unwrap();
    let mut n = Nes::new(Cart::load(&rom).unwrap());

    #[cfg(feature="trace")]
    if let Ok(seed) = env::var("NES_NOISE_SEED") {
        n.bus.borrow_mut().apu.debug_noise(seed.parse().unwrap(),
            env::var("NES_NOISE_TIMER").unwrap_or("0".into()).parse().unwrap());
    }

    #[cfg(feature="trace")]
    if let Ok(mask) = env::var("NES_MUTE") {
        n.bus.borrow_mut().apu.debug_mute(mask.parse().unwrap());
    }
    let frames: usize = args[2].parse().unwrap();
    let out = args.get(3).map(String::as_str).unwrap_or("/tmp/ours");
    fs::create_dir_all(out).unwrap();
    let mut changes = Vec::new();
    if let Some(p) = args.get(4) {
        for line in fs::read_to_string(p).unwrap().lines() {
            let line = line.split('#').next().unwrap();
            let w: Vec<_> = line.split_whitespace().collect();
            if w.len() == 2 {
                changes.push((w[0].parse::<usize>().unwrap(),
                        u8::from_str_radix(w[1].trim_start_matches("0x"),
                                16).unwrap()))
            }
        }
    }
    let mut audio = Vec::new();
    for frame in 0..frames {
        for &(f, k) in &changes { if f == frame { n.set_keys(k) } }
        n.run_frame();
        let mut bytes = Vec::with_capacity(122880);
        for &p in &n.bus.borrow().ppu.fb {
            bytes.extend_from_slice(&p.to_le_bytes())
        }
        if env::var_os("LAST_ONLY").is_none() || frame == frames - 1 {
            fs::write(format!("{out}/frame_{frame:05}.idx"), bytes).unwrap();
        }
        n.bus.borrow_mut().apu.drain();
        for &p in &n.bus.borrow().apu.drained {
            audio.extend_from_slice(&p.to_le_bytes())
        }
    }
    fs::write(format!("{out}/audio.raw"), audio).unwrap();
    let bus = n.bus.borrow();
    let ram = &bus.cart.ram;
    let status = ram.first().copied().unwrap_or(0);
    let signature = ram.get(1..4).unwrap_or(&[]);
    let log = ram.get(4..ram.len().min(1024)).unwrap_or(&[]);
    println!("frames={} cycles={} pc={:04x} status={} signature={:02x?} log={}",
        frames, bus.cycles, n.pc(), status, signature,
        String::from_utf8_lossy(log).split('\0').next().unwrap());
}
