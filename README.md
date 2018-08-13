# Zinc64

[![Build Status](https://travis-ci.org/digitalstreamio/zinc64.svg?branch=master)](https://travis-ci.org/digitalstreamio/zinc64)
[![Crates.io](https://img.shields.io/crates/v/zinc64.svg?maxAge=2592000)](https://crates.io/crates/zinc64)

## Overview

zinc64 is a Commodore 64 emulator toolkit "with batteries included but
swappable". It is designed to be used as a standalone emulator or a library
used to build new emulators. The design philosophy allows for each component
to be swapped out and replaced by different implementation. Therefore,
special considerations were made to model interactions between chips
without coupling them together.

It implements MOS 6510 CPU, MOS 6526 CIA, MOS 6581 SID,
MOS 6567/6569 VIC chipset as well as various devices and peripherals available
with C64.

### Story

zinc64 was started as an exercise to learn Rust and explore Commodore 64
hardware in more detail. Somewhere around mid 2016 I needed to feed my 8-bit
nostalgia so I picked up a working Commodore 64 (physical version) and started
to assemble various accessories required to get software onto it. Soon enough I
had picked up a copy of C64 Programmer's Reference Guide and the rest is now
history.

## Design

### Extensibility

The emulator components may be swapped out by providing custom core::Factory trait
implementation. The default implementation of the core::Factory trait is done through
system::ChipFactory. The chip factory object is passed into system::C64 component
that provides core emulator functionality.

Here is an example how these components are used together:

        let config = Rc::new(Config::new(SystemModel::from("pal")));
        let chip_factory = Box::new(C64Factory::new(config.clone()));
        let mut c64 = C64::new(config.clone(), chip_factory).unwrap();
        c64.reset(false);

## Status

| Class    | Component     | Status      |
|----------|---------------|-------------|
| Chipset  | 6510 CPU      | Done
| Chipset  | Memory        | Done
| Chipset  | 6526 CIA      | Done
| Chipset  | 6581 SID      | Done
| Chipset  | 6567 VIC      | Done
| Device   | Cartridge     | Done
| Device   | Floppy        | Not Started
| Device   | Datassette    | Done
| Device   | Keyboard      | Done
| Device   | Joystick      | Done
| Debugger | Remote        | Done
| Debugger | Radare2       | Done
| Format   | Bin           | Done
| Format   | Crt           | Done
| Format   | D64           | Not Started
| Format   | P00           | Done
| Format   | Prg           | Done
| Format   | Tap           | Done
| Format   | T64           | Not Started

## Roadmap

- v0.7  - zinc64 ui
- v0.8  - floppy support

## Getting Started

1. Install Rust compiler or follow steps @ https://www.rust-lang.org/en-US/install.html.

        curl https://sh.rustup.rs -sSf | sh

2. Clone this repository.

        git clone https://github.com/digitalstreamio/zinc64

	or download as zip archive

		https://github.com/digitalstreamio/zinc64/archive/master.zip

3. Build the emulator.

        cd zinc64
        cargo build --release --all

4. Run the emulator.

        ./target/release/zinc64-sdl

    or start a program

    	./target/release/zinc64-sdl --autostart path

### Windows Considerations

1. Install [Microsoft Visual C++ Build Tools 2017](https://www.visualstudio.com/downloads/#build-tools-for-visual-studio-2017). Select Visual C++ build tools workload.

2. Install [SDL2 Development Libraries](http://www.libsdl.org/release/SDL2-devel-2.0.7-VC.zip).

3. Copy all SDL2 lib files from

		SDL2-devel-2.0.x-VC\SDL2-2.0.x\lib\x64\

	to 

		C:\Users\{Your Username}\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\lib

4. Copy SDL2.dll from

		SDL2-devel-2.0.x-VC\SDL2-2.0.x\lib\x64\

	to zinc64 project directory

## Debugger

To start the debugger, run the emulator with '-d' or '--debug' option. Optionally, you can specify '--debugaddress'
to bind to a specific address.

        ./target/release/zinc64-sdl --debug

To connect to the debugger, telnet to the address and port used by the debugger.

        telnet localhost 9999

Debugger commands and syntax are modeled after Vice emulator.  To see a list of available commands,
type in the debugging session:

        help

or to get help on a specific command:

        help <command>

### Radare2

Initial support for radare2 has been merged in version 0.3. To start the emulator with RAP server support, run

        ./target/release/zinc64-sdl --rap 127.0.0.1:9999

and connect with

        radare2 -a 6502 -d rap://localhost:9999/1

## Examples

I've included a number of examples from Kick Assembler that I've used to test various components of the emulator. They can be found in the bin folder of this repository and started with the emulator's autostart option.

        ./target/release/zinc64-sdl --autostart bin/SineAndGraphics.prg

| Program                  | Status  |
|--------------------------|---------|
| 6502_functional_test.bin | Pass    |
| FloydSteinberg.prg       | Pass    | 
| KoalaShower.prg          | Pass    |
| Message.prg              | Pass    |
| MusicIrq.prg             | Pass    |
| Scroll.prg               | Pass    |
| SID_Player.prg           | Pass    |
| SimpleSplits.prg         | Fails   |
| SineAndGraphics.prg      | Pass    |

### Tests

The cpu validation was performed with the help of [Klaus2m5 functional tests](https://github.com/Klaus2m5/6502_65C02_functional_tests) for the 6502 processor 

        ./target/release/zinc64-sdl --binary bin/6502_functional_test.bin --offset=1024 --console --loglevel trace

## Keyboard Shortcuts

| Shortcut  | Function          |
|-----------|-------------------|
| Alt-Enter | Toggle Full Screen
| Alt-F9    | Reset
| Alt-H     | Activate Debugger
| Alt-M     | Toggle Mute
| Alt-P     | Toggle Pause
| Alt-Q     | Quit
| Alt-W     | Warp Mode
| Ctrl-F1   | Tape Play/Stop
| NumPad-2  | Joystick Bottom
| NumPad-4  | Joystick Left
| NumPad-5  | Joystick Fire
| NumPad-6  | Joystick Right
| NumPad-8  | Joystick Top

## Credits

- Commodore folks for building an iconic 8-bit machine
- Rust developers for providing an incredible language to develop in
- Thanks to Rafal Wiosna for passing onto me some of his passion for 8-bit machines ;)
- Thanks to Klaus Dormann for his 6502_65C02_functional_tests, without which I would be lost
- Thanks to Dag Lem for his reSID implementation
- Thanks to Christian Bauer for his wonderful "The MOS 6567/6569 video controller (VIC-II) and its application in the Commodore 64" paper
- Thanks to Peter Schepers for his "Introduction to the various Emulator File Formats"
- Thanks to c64-wiki.com for my to go reference on various hardware components

