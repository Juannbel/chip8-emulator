// Reference docs: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
//
use chip8_emulator::chip::Chip;
use sdl2::render::WindowCanvas;
use std::io;
use std::path::Path;

fn main() -> Result<(), io::Error> {
    let mut args = std::env::args();
    let rom_arg = args.nth(1).expect("Expected a ROM file");
    let rom = Path::new(&rom_arg);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Chip8 Emulator", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas: WindowCanvas = window.into_canvas().build().unwrap();
    let mut event_queue = sdl_context.event_pump().unwrap();

    let mut chip = Chip::new(&mut canvas, &mut event_queue);
    if let Err(msg) = chip.load(rom) {
        println!("Error loading the rom: {}", msg);
        return Ok(());
    }
    chip.run()?;

    Ok(())
}
