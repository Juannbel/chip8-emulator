use std::path::Path;

use chip::Chip;

// Reference docs: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
//
mod chip;
mod display;

fn main() {
    let mut args = std::env::args();
    let rom_arg = args.nth(1).expect("Expected a ROM file");
    let rom = Path::new(&rom_arg);

    let mut chip = Chip::new();
    if let Err(msg) = chip.load(rom) {
        println!("Error loading the rom: {}", msg);
        return;
    }

    loop {
        chip.update();
        std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
    }
}
