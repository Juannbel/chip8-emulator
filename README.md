# Chip8 Emulator

![Blinky running on the emulator](./images/blinky.png)

This is a Chip8 emulator written in Rust, capable of running most of the Chip8 games.

## Usage

To run the emulator you need to have Rust installed in your system. You can install it by following the instructions in the [Rust website](https://www.rust-lang.org/tools/install).
You also need to have the SDL2 library installed in your system. In Ubuntu you can install it by running:

```bash
sudo apt-get install libsdl2-dev
```

After you have installed Rust and SDL2, clone this repository and run the emulator with:

```bash
cargo run --release -- <path_to_rom>
```

Where `<path_to_rom>` is the path to the Chip8 rom you want to run. There are some roms in the `roms` directory that you can use.

## References

The primary technical reference used to implement this emulator is the Cowgod's Chip-8 Technical Reference v1.0. You can find it [here](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM).

The tests roms used to test the emulator are from the [Chip8 test suit repository](https://github.com/Timendus/chip8-test-suite)
