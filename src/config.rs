use std::fs;

use serde_json::Value;
use sha1::Digest;

pub struct Config {
    pub rate: u64,
    pub ipf: u64,
    // quirks
    pub shift: bool,
    pub memory_increment_by_x: bool,
    pub memory_leave_i_unchanged: bool,
    pub wrap: bool,
    pub jump: bool,
    pub vblank: bool,
    pub logic: bool,
}

impl Config {
    const DEFAULT_CONFIG: Config = Config {
        rate: 60,
        ipf: 12,
        shift: false,
        memory_increment_by_x: false,
        memory_leave_i_unchanged: false,
        wrap: false,
        jump: false,
        vblank: true,
        logic: true,
    };

    pub fn new() -> Config {
        Config::DEFAULT_CONFIG
    }

    pub fn adjust_to_rom(&mut self, rom: &Vec<u8>) {
        if let Some(platform) = self.get_platform(&rom) {
            println!("{}", platform);
            let quirks = match &*platform {
                "originalChip8" | "hybridVIP" | "chip8x" => {
                    (false, false, false, false, false, true, true)
                }
                "modernChip8" => (false, false, false, false, false, false, false),
                "chip48" => (true, true, false, false, true, false, false),
                "superchip1" => (true, true, false, false, true, false, false),
                "superchip" => (true, false, true, false, true, false, false),
                _ => (
                    self.shift,
                    self.memory_increment_by_x,
                    self.memory_leave_i_unchanged,
                    self.wrap,
                    self.jump,
                    self.vblank,
                    self.logic,
                ),
            };

            (
                self.shift,
                self.memory_increment_by_x,
                self.memory_leave_i_unchanged,
                self.wrap,
                self.jump,
                self.vblank,
                self.logic,
            ) = quirks;
        }
    }

    fn get_platform(&self, rom: &Vec<u8>) -> Option<String> {
        let hash = self.get_sha1(rom);

        let data = fs::read_to_string("./db/sha1-hashes.json").ok()?;
        let hashes: Value = serde_json::from_str(&data).ok()?;

        let program_index = hashes.get(&hash)?.as_u64()? as usize;

        println!("{}", program_index);

        let data = fs::read_to_string("./db/programs.json").ok()?;
        let programs: Value = serde_json::from_str(&data).ok()?;

        let platform = programs
            .get(program_index)?
            .get("roms")?
            .get(&hash)?
            .get("platforms")?
            .get(0)?
            .as_str()?;

        Some(platform.to_string())
    }

    fn get_sha1(&self, rom: &Vec<u8>) -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update(&rom);
        let result = hasher.finalize();
        result.iter().map(|byte| format!("{:02x}", byte)).collect()
    }
}
