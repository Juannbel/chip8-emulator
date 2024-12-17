use std::{path::Path, thread::sleep};

// use chip::Chip;

// Reference docs: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
//
// mod chip;
// mod display;

// fn main() {
//     let mut args = std::env::args();
//     let rom_arg = args.nth(1).expect("Expected a ROM file");
//     let rom = Path::new(&rom_arg);

//     let mut chip = Chip::new();
//     if let Err(msg) = chip.load(rom) {
//         println!("Error loading the rom: {}", msg);
//         return;
//     }

use chip8_emulator::chip::{self, Chip};
//     loop {
//         chip.update();
//         std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
//     }
// }
use crossterm::{
    event::{self, poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Widget},
    Terminal,
};

fn main() -> Result<(), io::Error> {
    let mut args = std::env::args();
    let rom_arg = args.nth(1).expect("Expected a ROM file");
    let rom = Path::new(&rom_arg);

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // terminal.draw(|f| {
    //     let size = f.size();
    //     let block = Block::default().title("Block").borders(Borders::ALL);
    //     f.render_widget(block, size);
    // })?;

    let mut chip = Chip::new(&mut terminal);
    if let Err(msg) = chip.load(rom) {
        println!("Error loading the rom: {}", msg);
        return Ok(());
    }
    chip.run()?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
