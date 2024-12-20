use sdl2::EventPump;
use sdl2::{render::WindowCanvas, AudioSubsystem};
use std::{
    io::{self, BufReader, Read},
    path::Path,
    thread::sleep,
    time::Duration,
};

use rand::Rng;

use crate::config::Config;
use crate::{display::Display, keypad::Keypad, speaker::Speaker};

#[derive(Debug)]
struct Instruction {
    parts: (u8, u8, u8, u8),
    nnn: u16,
    n: u8,
    x: u8,
    y: u8,
    kk: u8,
}

impl Instruction {
    fn new(instruction: u16) -> Instruction {
        let parts = (
            (instruction & 0xF000).checked_shr(12).unwrap_or(0) as u8,
            (instruction & 0x0F00).checked_shr(8).unwrap_or(0) as u8,
            (instruction & 0x00F0).checked_shr(4).unwrap_or(0) as u8,
            (instruction & 0x000F) as u8,
        );

        // nnn or addr - A 12-bit value, the lowest 12 bits of the instruction
        // n or nibble - A 4-bit value, the lowest 4 bits of the instruction
        // x - A 4-bit value, the lower 4 bits of the high byte of the instruction
        // y - A 4-bit value, the upper 4 bits of the low byte of the instruction
        // kk or byte - An 8-bit value, the lowest 8 bits of the instruction

        Instruction {
            parts,
            nnn: instruction & 0x0FFF,
            n: parts.3,
            x: parts.1,
            y: parts.2,
            kk: (instruction & 0x00FF) as u8,
        }
    }
}

pub struct Chip {
    config: Config,
    waiting_key: bool,
    drew_on_frame: bool,
    rng: rand::rngs::ThreadRng,
    keep_running: bool,
    ram: [u8; Chip::RAM_SIZE],
    stack: [u16; Chip::STACK_SIZE],
    regs: [u8; Chip::GENERAL_REGISTERS],
    // original u16
    i_reg: usize,
    delay_reg: u8,
    sound_reg: u8,
    // original u16 and u8 registers, usize to simplify indexing
    pc_reg: usize,
    sp_reg: usize,
    display: Display,
    keypad: Keypad,
    speaker: Speaker,
}

impl std::fmt::Debug for Chip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Registers: ")?;
        writeln!(
            f,
            "PC : {:04X} | SP : {:02X} | I: {:04X} | Delay: {:02X} | Sound : {:02X}",
            self.pc_reg, self.sp_reg, self.i_reg, self.delay_reg, self.sound_reg
        )?;

        for b in self.regs {
            write!(f, "{:02X} ", b)?;
        }
        writeln!(f)?;

        writeln!(f, "Stack: ")?;
        for b in self.stack {
            write!(f, "{:04X} ", b)?;
        }
        writeln!(f)?;

        writeln!(f, "Ram: ")?;
        for (i, b) in self.ram.iter().enumerate() {
            if i % 32 == 0 {
                writeln!(f)?;
                write!(f, "{} | ", i)?;
            }
            write!(f, "{:02X} ", b)?;
        }
        Ok(())
    }
}

impl Chip {
    const RAM_SIZE: usize = 4096;
    const STACK_SIZE: usize = 16;
    const GENERAL_REGISTERS: usize = 16;
    const PROGRAM_START: usize = 512;
    const BYTES_PER_SPRITE: u8 = 5;

    pub fn new(
        canvas: WindowCanvas,
        event_queue: EventPump,
        audio_subsystem: AudioSubsystem,
    ) -> Chip {
        let mut chip = Chip {
            config: Config::new(),
            waiting_key: false,
            drew_on_frame: false,
            keep_running: true,
            rng: rand::thread_rng(),
            ram: [0; Chip::RAM_SIZE],
            stack: [0; Chip::STACK_SIZE],
            regs: [0; Chip::GENERAL_REGISTERS],
            i_reg: 0,
            delay_reg: 0,
            sound_reg: 0,
            pc_reg: Chip::PROGRAM_START,
            sp_reg: 0,
            display: Display::new(canvas),
            keypad: Keypad::new(event_queue),
            speaker: Speaker::new(audio_subsystem),
        };

        let sprites = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        chip.ram[0..sprites.len()].copy_from_slice(&sprites);

        chip
    }

    pub fn load(&mut self, rom_path: &Path) -> Result<String, String> {
        let file = std::fs::File::open(rom_path);
        if let Err(_) = file {
            return Err(String::from("Error opening ROM file"));
        }

        let mut file_reader = BufReader::new(file.unwrap());
        let mut buffer = Vec::new();
        if let Err(_) = file_reader.read_to_end(&mut buffer) {
            return Err(String::from("Error reading ROM file"));
        }

        self.ram[Chip::PROGRAM_START..(Chip::PROGRAM_START + buffer.len())]
            .copy_from_slice(&buffer);

        self.config.adjust_to_rom(&buffer);

        Ok(String::from("ROM Loaded on memory"))
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        while self.keep_running {
            if !self.waiting_key {
                self.keep_running = self.keypad.handle_events(&mut self.config);
            }

            if self.sound_reg > 0 {
                self.speaker.start();
                self.sound_reg -= 1;
            } else {
                self.speaker.stop();
            }

            if self.delay_reg > 0 {
                self.delay_reg -= 1;
            }

            for _ in 0..self.config.ipf {
                self.update();
                if self.config.vblank && self.drew_on_frame {
                    break;
                }
            }

            self.drew_on_frame = false;

            self.display.render();
            sleep(Duration::from_millis(1000 / self.config.rate));
        }
        Ok(())
    }

    fn fetch_and_decode(&mut self) -> Instruction {
        let raw_instruction =
            ((self.ram[self.pc_reg] as u16) << 8) | (self.ram[self.pc_reg + 1] as u16);

        self.pc_reg += 2;
        Instruction::new(raw_instruction)
    }

    pub fn update(&mut self) {
        let instruction = self.fetch_and_decode();

        match instruction.parts {
            (0x0, 0x0, 0xE, 0x0) => self.cls(),
            (0x0, 0x0, 0xE, 0xE) => self.ret(),
            (0x1, _, _, _) => self.jump(instruction.nnn),
            (0x2, _, _, _) => self.call(instruction.nnn),
            (0x3, _, _, _) => self.skip_if_equal_byte(instruction.x, instruction.kk),
            (0x4, _, _, _) => self.skip_if_not_equal_byte(instruction.x, instruction.kk),
            (0x5, _, _, _) => self.skip_if_equal_registers(instruction.x, instruction.y),
            (0x6, _, _, _) => self.load_byte_to_reg(instruction.x, instruction.kk),
            (0x7, _, _, _) => self.add_byte_to_reg(instruction.x, instruction.kk),
            (0x8, _, _, 0x0) => self.load_reg_to_reg(instruction.x, instruction.y),
            (0x8, _, _, 0x1) => self.or_reg_reg(instruction.x, instruction.y),
            (0x8, _, _, 0x2) => self.and_reg_reg(instruction.x, instruction.y),
            (0x8, _, _, 0x3) => self.xor_reg_reg(instruction.x, instruction.y),
            (0x8, _, _, 0x4) => self.add_reg_reg(instruction.x, instruction.y),
            (0x8, _, _, 0x5) => self.sub_reg_reg(instruction.x, instruction.y),
            (0x8, _, _, 0x6) => self.shift_right(instruction.x, instruction.y),
            (0x8, _, _, 0x7) => self.subn_reg_reg(instruction.x, instruction.y),
            (0x8, _, _, 0xE) => self.shift_left(instruction.x, instruction.y),
            (0x9, _, _, 0x0) => self.skip_if_not_equal_registers(instruction.x, instruction.y),
            (0xA, _, _, _) => self.load_to_i_reg(instruction.nnn),
            (0xB, _, _, _) => self.jump_with_offset(instruction.x, instruction.nnn),
            (0xC, _, _, _) => self.rand(instruction.x, instruction.kk),
            (0xD, _, _, _) => self.draw(instruction.x, instruction.y, instruction.n),
            (0xE, _, 0x9, 0xE) => self.skip_if_key(instruction.x),
            (0xE, _, 0xA, 0x1) => self.skip_if_not_key(instruction.x),
            (0xF, _, 0x0, 0x7) => self.set_reg_from_delay_timer(instruction.x),
            (0xF, _, 0x0, 0xA) => self.wait_key(instruction.x),
            (0xF, _, 0x1, 0x5) => self.set_delay_timer_from_reg(instruction.x),
            (0xF, _, 0x1, 0x8) => self.set_sound_timer_from_reg(instruction.x),
            (0xF, _, 0x1, 0xE) => self.add_to_i(instruction.x),
            (0xF, _, 0x2, 0x9) => self.i_to_digit_sprite(instruction.x),
            (0xF, _, 0x3, 0x3) => self.decimal_reg_to_memory(instruction.x),
            (0xF, _, 0x5, 0x5) => self.write_regs_to_mem(instruction.x),
            (0xF, _, 0x6, 0x5) => self.read_regs_from_mem(instruction.x),
            _ => {}
        }
    }

    // 0nnn - SYS addr
    // Jump to a machine code routine at nnn.
    // This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters.

    // 00E0 - CLS
    // Clear the display.
    fn cls(&mut self) {
        self.display.clear();
        self.display.render();
    }

    // 00EE - RET
    // Return from a subroutine.
    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn ret(&mut self) {
        self.sp_reg -= 1;
        self.pc_reg = self.stack[self.sp_reg as usize] as usize;
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    fn jump(&mut self, addr: u16) {
        self.pc_reg = addr as usize
    }

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    fn call(&mut self, addr: u16) {
        self.stack[self.sp_reg as usize] = self.pc_reg as u16;
        self.sp_reg += 1;
        self.pc_reg = addr as usize;
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn skip_if_equal_byte(&mut self, x: u8, kk: u8) {
        if self.regs[x as usize] == kk {
            self.pc_reg += 2;
        }
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn skip_if_not_equal_byte(&mut self, x: u8, kk: u8) {
        if self.regs[x as usize] != kk {
            self.pc_reg += 2;
        }
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn skip_if_equal_registers(&mut self, x: u8, y: u8) {
        if self.regs[x as usize] == self.regs[y as usize] {
            self.pc_reg += 2;
        }
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    // The interpreter puts the value kk into register Vx.
    fn load_byte_to_reg(&mut self, x: u8, kk: u8) {
        self.regs[x as usize] = kk;
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn add_byte_to_reg(&mut self, x: u8, kk: u8) {
        self.regs[x as usize] = self.regs[x as usize].wrapping_add(kk);
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    fn load_reg_to_reg(&mut self, x: u8, y: u8) {
        self.regs[x as usize] = self.regs[y as usize];
    }

    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy.
    // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
    fn or_reg_reg(&mut self, x: u8, y: u8) {
        self.regs[x as usize] |= self.regs[y as usize];
        if self.config.logic {
            self.regs[0xF] = 0;
        }
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise AND compares the corrseponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
    fn and_reg_reg(&mut self, x: u8, y: u8) {
        self.regs[x as usize] &= self.regs[y as usize];
        if self.config.logic {
            self.regs[0xF] = 0;
        }
    }

    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    fn xor_reg_reg(&mut self, x: u8, y: u8) {
        self.regs[x as usize] ^= self.regs[y as usize];
        if self.config.logic {
            self.regs[0xF] = 0;
        }
    }

    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry.
    // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn add_reg_reg(&mut self, x: u8, y: u8) {
        let (res, overflow) = self.regs[x as usize].overflowing_add(self.regs[y as usize]);
        self.regs[x as usize] = res;
        self.regs[0xF] = overflow as u8;
    }

    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    fn sub_reg_reg(&mut self, x: u8, y: u8) {
        let (res, overflow) = self.regs[x as usize].overflowing_sub(self.regs[y as usize]);
        self.regs[x as usize] = res;
        self.regs[0xF] = !overflow as u8;
    }

    // 8xy6 - SHR Vx {, Vy}
    // Set Vx = Vx SHR 1.
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    fn shift_right(&mut self, x: u8, y: u8) {
        let source = if self.config.shift { x } else { y };
        let lsb = self.regs[source as usize] & 0x1;
        self.regs[x as usize] = self.regs[source as usize].checked_shr(1).unwrap_or(0);
        self.regs[0xF] = lsb;
    }

    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    fn subn_reg_reg(&mut self, x: u8, y: u8) {
        let (res, overflow) = self.regs[y as usize].overflowing_sub(self.regs[x as usize]);
        self.regs[x as usize] = res;
        self.regs[0xF] = if overflow { 0 } else { 1 };
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    fn shift_left(&mut self, x: u8, y: u8) {
        let source = if self.config.shift { x } else { y };
        let msb = self.regs[source as usize].checked_shr(7).unwrap_or(0);
        self.regs[x as usize] = self.regs[source as usize].checked_shl(1).unwrap_or(0);
        self.regs[0xF] = msb;
    }

    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    fn skip_if_not_equal_registers(&mut self, x: u8, y: u8) {
        if self.regs[x as usize] != self.regs[y as usize] {
            self.pc_reg += 2;
        }
    }

    // Annn - LD I, addr
    // Set I = nnn.
    // The value of register I is set to nnn.
    fn load_to_i_reg(&mut self, addr: u16) {
        self.i_reg = addr as usize;
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0.
    // The program counter is set to nnn plus the value of V0.
    fn jump_with_offset(&mut self, x: u8, addr: u16) {
        self.pc_reg =
            addr as usize + self.regs[if self.config.jump { x } else { 0 } as usize] as usize;
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk.
    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
    fn rand(&mut self, x: u8, kk: u8) {
        self.regs[x as usize] = self.rng.gen::<u8>() & kk;
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    fn draw(&mut self, x: u8, y: u8, n: u8) {
        self.drew_on_frame = true;
        let mut collision = 0;

        // independent of wrap quirk on or off (this would do the trick to draw the sprite if it's entirely off the screen with wrap off)
        let dx = self.regs[x as usize] as usize % Display::WIDTH;
        let dy = self.regs[y as usize] as usize % Display::HEIGHT;

        for i in 0..n {
            if self.display.draw(
                dx as u8,
                dy as u8 + i,
                self.ram[self.i_reg + i as usize],
                self.config.wrap,
            ) {
                collision = 1;
            }
        }

        self.regs[0xF] = collision;
        self.display.render();
    }

    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
    fn skip_if_key(&mut self, x: u8) {
        if self.keypad.is_pressed(self.regs[x as usize]) {
            self.pc_reg += 2;
        }
    }

    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
    fn skip_if_not_key(&mut self, x: u8) {
        if !self.keypad.is_pressed(self.regs[x as usize]) {
            self.pc_reg += 2;
        }
    }

    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value.
    // The value of DT is placed into Vx.
    fn set_reg_from_delay_timer(&mut self, x: u8) {
        self.regs[x as usize] = self.delay_reg;
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    // All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn wait_key(&mut self, x: u8) {
        self.waiting_key = true;
        if let Some(key) = self
            .keypad
            .block_read(&mut self.keep_running, &mut self.config)
        {
            self.regs[x as usize] = key as u8;
            self.waiting_key = false;
        } else {
            self.pc_reg -= 2;
        }
    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx.
    // DT is set equal to the value of Vx.
    fn set_delay_timer_from_reg(&mut self, x: u8) {
        self.delay_reg = self.regs[x as usize];
    }

    // Fx18 - LD ST, Vx
    // Set sound timer = Vx.
    // ST is set equal to the value of Vx.
    fn set_sound_timer_from_reg(&mut self, x: u8) {
        self.sound_reg = self.regs[x as usize];
    }

    // Fx1E - ADD I, Vx
    // Set I = I + Vx.
    // The values of I and Vx are added, and the results are stored in I.
    fn add_to_i(&mut self, x: u8) {
        self.i_reg += self.regs[x as usize] as usize;
    }

    // Fx29 - LD F, Vx
    // Set I = location of sprite for digit Vx.
    // The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
    fn i_to_digit_sprite(&mut self, x: u8) {
        self.i_reg = Chip::BYTES_PER_SPRITE as usize * self.regs[x as usize] as usize;
    }

    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    fn decimal_reg_to_memory(&mut self, x: u8) {
        let value = self.regs[x as usize];
        self.ram[self.i_reg] = value / 100;
        self.ram[self.i_reg + 1] = (value / 10) % 10;
        self.ram[self.i_reg + 2] = value % 10;
    }

    // Fx55 - LD [I], Vx
    // Store registers V0 through Vx in memory starting at location I.
    // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    fn write_regs_to_mem(&mut self, x: u8) {
        for i in 0..=x {
            self.ram[self.i_reg + i as usize] = self.regs[i as usize]
        }

        if self.config.memory_leave_i_unchanged {
            return;
        }

        self.i_reg += x as usize;
        if !self.config.memory_increment_by_x {
            self.i_reg += 1;
        }
    }

    // Fx65 - LD Vx, [I]
    // Read registers V0 through Vx from memory starting at location I.
    // The interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn read_regs_from_mem(&mut self, x: u8) {
        for i in 0..=x {
            self.regs[i as usize] = self.ram[self.i_reg + i as usize];
        }

        if self.config.memory_leave_i_unchanged {
            return;
        }

        self.i_reg += x as usize;
        if !self.config.memory_increment_by_x {
            self.i_reg += 1;
        }
    }
}

// testing
//
#[cfg(test)]
mod tests {
    use super::*;

    fn init_chip() -> Chip {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let window = video_subsystem
            .window("Chip8 Emulator", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .unwrap();
        let canvas: WindowCanvas = window.into_canvas().build().unwrap();
        let event_queue = sdl_context.event_pump().unwrap();
        Chip::new(canvas, event_queue, audio_subsystem)
    }

    #[test]
    fn add_byte_to_reg() {
        let mut chip = init_chip();
        chip.regs[0x3] = 0x2;

        chip.add_byte_to_reg(0x3, 0x1);
        assert_eq!(chip.regs[0x3], 0x3);

        chip.add_byte_to_reg(0x5, 0x9);
        assert_eq!(chip.regs[0x5], 0x9);

        chip.regs[0x8] = 0xFF;
        chip.add_byte_to_reg(0x8, 0x1);
        assert_eq!(chip.regs[0x8], 0x0);
    }

    #[test]
    fn add_reg_reg() {
        let mut chip = init_chip();

        chip.regs[0x0] = 0xFF;
        chip.regs[0x1] = 0x01;

        chip.add_reg_reg(0x0, 0x1);
        assert_eq!(chip.regs[0x0], 0x0);
        assert_eq!(chip.regs[0xF], 1);

        chip.regs[0x2] = 0x09;

        chip.add_reg_reg(0x0, 0x2);
        assert_eq!(chip.regs[0x0], 0x9);
        assert_eq!(chip.regs[0xF], 0);
    }
}
