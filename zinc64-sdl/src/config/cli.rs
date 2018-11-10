// This file is part of zinc64.
// Copyright (c) 2016-2018 Sebastian Jastrzebski. All rights reserved.
// Licensed under the GPLv3. See LICENSE file in the project root for full license text.

use std::net::SocketAddr;
use std::path::Path;
use std::result::Result;

use getopts;
use zinc64::core::SystemModel;
use zinc64::device;
use zinc64::system::{Config, C64};
use zinc64_loader::{BinLoader, Loader, Loaders};

use super::{JamAction, Options};

static NAME: &'static str = "zinc64";
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct Cli;

impl Cli {
    pub fn parse_args(args: &[String]) -> Result<getopts::Matches, String> {
        let opts = Cli::build_options();
        let matches = opts
            .parse(&args[1..])
            .map_err(|f| format!("Invalid options\n{}", f))?;
        Ok(matches)
    }

    pub fn parse_app_options(matches: &getopts::Matches) -> Result<Options, String> {
        let width = matches
            .opt_str("width")
            .map(|s| s.parse::<u32>().unwrap())
            .unwrap_or(800);
        let height = matches
            .opt_str("height")
            .map(|s| s.parse::<u32>().unwrap())
            .unwrap_or(600);
        let options = Options {
            fullscreen: matches.opt_present("fullscreen"),
            window_size: (width, height),
            speed: matches
                .opt_str("speed")
                .map(|s| s.parse::<u8>().unwrap())
                .unwrap_or(100),
            warp_mode: matches.opt_present("warp"),
            debug: matches.opt_present("debug"),
            dbg_address: matches.opt_str("debugaddress").map(|s| {
                let addr: SocketAddr = s.parse().unwrap();
                addr
            }),
            jam_action: matches
                .opt_str("jamaction")
                .map(|s| JamAction::from(&s))
                .unwrap_or(JamAction::Continue),
            rap_address: matches.opt_str("rap").map(|s| {
                let addr: SocketAddr = s.parse().unwrap();
                addr
            }),
        };
        Ok(options)
    }

    pub fn parse_system_config(matches: &getopts::Matches) -> Result<Config, String> {
        let model = SystemModel::from(
            &matches
                .opt_str("model")
                .unwrap_or_else(|| String::from("pal")),
        );
        let mut config = Config::new(model);
        Cli::parse_device_config(&mut config, matches)?;
        Cli::parse_sound_config(&mut config, matches)?;
        Ok(config)
    }

    pub fn print_help() {
        let opts = Cli::build_options();
        println!("{} {}", NAME, VERSION);
        println!();
        println!("Usage:");
        print!("{}", opts.usage("C64 rustified emulator"));
    }

    pub fn print_version() {
        println!("{} {}", NAME, VERSION);
    }

    pub fn set_c64_options(c64: &mut C64, matches: &getopts::Matches) -> Result<(), String> {
        Cli::set_debug_options(c64, matches)?;
        Cli::set_autostart_options(c64, matches)?;
        Ok(())
    }

    fn build_options() -> getopts::Options {
        let mut opts = getopts::Options::new();
        opts.optopt("", "model", "set NTSC or PAL variants", "[ntsc|pal]")
            // Autostart
            .optopt("", "autostart", "attach and autostart image", "path")
            .optopt("", "binary", "load binary into memory", "path")
            .optopt("", "offset", "offset at which to load binary", "address")
            // App
            .optflag("", "console", "start in console mode")
            .optflag("f", "fullscreen", "enable fullscreen")
            .optopt("", "width", "window width", "800")
            .optopt("", "height", "window height", "600")
            .optopt("", "speed", "set speed of the emulator", "number")
            .optflag("", "warp", "enable wrap mode")
            // Device
            .optopt("", "joydev1", "set device for joystick 1", "none")
            .optopt("", "joydev2", "set device for joystick 2", "numpad")
            // Sound
            .optflag("", "nosound", "disable sound playback")
            .optflag("", "nosidfilters", "disable SID filters")
            .optopt(
                "",
                "soundbufsize",
                "set sound buffer size in samples",
                "4096",
            )
            .optopt("", "soundrate", "set sound sample rate in Hz", "44100")
            // Debug
            .optmulti("", "bp", "set breakpoint at this address", "address")
            .optflag("d", "debug", "start debugger")
            .optopt(
                "",
                "debugaddress",
                "start debugger bound to the specified address",
                "127.0.0.1:9999",
            )
            .optopt(
                "",
                "jamaction",
                "set cpu jam handling",
                "[continue|quit|reset]",
            )
            .optopt(
                "",
                "rap",
                "start rap server bound to the specified address",
                "127.0.0.1:9999",
            )
            // Logging
            .optopt(
                "",
                "loglevel",
                "set log level",
                "[error|warn|info|debug|trace]",
            )
            .optmulti("", "log", "set log level for a target", "target=level")
            // Help
            .optflag("h", "help", "display this help")
            .optflag("V", "version", "display this version");
        opts
    }

    fn parse_device_config(config: &mut Config, matches: &getopts::Matches) -> Result<(), String> {
        if let Some(joydev) = matches.opt_str("joydev1") {
            config.joystick.joystick_1 = device::joystick::Mode::from(&joydev);
        }
        if let Some(joydev) = matches.opt_str("joydev2") {
            config.joystick.joystick_2 = device::joystick::Mode::from(&joydev);
        } else {
            config.joystick.joystick_2 = device::joystick::Mode::Numpad;
        }
        Ok(())
    }

    fn parse_sound_config(config: &mut Config, matches: &getopts::Matches) -> Result<(), String> {
        config.sound.enable = !matches.opt_present("nosound");
        config.sound.buffer_size = matches
            .opt_str("soundbufsize")
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(4096);
        config.sound.sample_rate = matches
            .opt_str("soundrate")
            .map(|s| s.parse::<u32>().unwrap())
            .unwrap_or(44100);
        config.sound.sid_filters = !matches.opt_present("nosidfilters");
        Ok(())
    }

    fn set_autostart_options(c64: &mut C64, matches: &getopts::Matches) -> Result<(), String> {
        match matches.opt_str("autostart") {
            Some(image_path) => {
                let path = Path::new(&image_path);
                let loader = Loaders::from_path(path);
                let mut autostart = loader.autostart(path).map_err(|err| format!("{}", err))?;
                autostart.execute(c64);
            }
            None => {
                if let Some(binary_path) = matches.opt_str("binary") {
                    let offset = matches
                        .opt_str("offset")
                        .map(|s| s.parse::<u16>().unwrap())
                        .unwrap_or(0);
                    let path = Path::new(&binary_path);
                    let loader = BinLoader::new(offset);
                    let mut image = loader.load(path).map_err(|err| format!("{}", err))?;
                    image.mount(c64);
                }
            }
        }
        Ok(())
    }

    fn set_debug_options(c64: &mut C64, matches: &getopts::Matches) -> Result<(), String> {
        let bps_strs = matches.opt_strs("bp");
        let bps = bps_strs.iter().map(|s| s.parse::<u16>().unwrap());
        for bp in bps {
            c64.get_bpm_mut().set(bp, false);
        }
        Ok(())
    }
}
