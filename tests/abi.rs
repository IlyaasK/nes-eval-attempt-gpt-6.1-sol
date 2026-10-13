use nes_emu::*;
fn run(frames: usize) -> (Vec<u16>, Vec<i16>) {
    let mut audio = Vec::new();
    for _ in 0..frames {
        emu_run_frame();
        let len = emu_audio_samples() as usize;
        if len > 0 {
            unsafe {
                audio.extend_from_slice(std::slice::from_raw_parts(emu_audio_buffer(),
                        len));
            }
        }
        assert_eq!(emu_audio_samples(),0,"audio must drain exactly once");
    }
    let fb =
        unsafe {
            std::slice::from_raw_parts(emu_framebuffer(), 256 * 240).to_vec()
        };
    (fb, audio)
}

#[test]
fn exported_abi_load_drain_and_cold_reset_are_deterministic() {
    let rom = include_bytes!("../dev-roms/robotfindskitten.nes");
    assert_eq!(emu_init(),1);
    assert_eq!(emu_audio_rate(),44100);
    let ptr = emu_rom_buffer();
    unsafe { std::ptr::copy_nonoverlapping(rom.as_ptr(), ptr, rom.len()); }
    assert_eq!(emu_load_rom(rom.len() as i32),1);
    let first = run(120);
    assert!(first.0.iter().any(|&p|p!=0));
    assert!(first.1.iter().any(|&s|s!=0));
    emu_set_keys(0x88);
    run(20);
    assert_eq!(emu_reset(),1);
    assert_eq!(emu_rom_buffer(),ptr,"ROM upload allocation must remain stable");
    assert_eq!(run(120),first,"cold reset must reproduce both output streams");
    assert_eq!(emu_load_rom(-1),0);
    assert_eq!(emu_load_rom(8*1024*1024+1),0);
}
