// This file is part of zinc64.
// Copyright (c) 2016-2018 Sebastian Jastrzebski. All rights reserved.
// Licensed under the GPLv3. See LICENSE file in the project root for full license text.

use super::spec::Spec;

pub struct Config {
    pub border_color: u8,
    pub csel: bool,
    pub rsel: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            border_color: 0x0e,
            csel: true,
            rsel: true,
        }
    }

    pub fn reset(&mut self) {
        self.border_color = 0x0e;
        self.csel = true;
        self.rsel = true;
    }
}

pub struct BorderUnit {
    spec: Spec,
    pub config: Config,
    main_flop: bool,
    vertical_flop: bool,
}

impl BorderUnit {
    pub fn new(spec: Spec) -> Self {
        BorderUnit {
            spec,
            config: Config::new(),
            main_flop: false,
            vertical_flop: false,
        }
    }

    fn map_sprite_to_screen(&self, x: u16) -> u16 {
        match self.spec.first_x_coord {
            0x194 => {
                match x {
                    0x000...0x193 => x + 0x64, // 0x1f7 - 0x193
                    0x194...0x1ff => x - 0x194,
                    _ => panic!("invalid sprite coords {}", x),
                }
            }
            0x19c => {
                match x {
                    0x000...0x19b => x + 0x64, // 0x1ff - 0x19b
                    0x19c...0x1ff => x - 0x19c,
                    _ => panic!("invalid sprite coords {}", x),
                }
            }
            _ => panic!("invalid sprite coords {}", x),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.main_flop || self.vertical_flop
    }

    pub fn output(&self) -> u8 {
        self.config.border_color
    }

    pub fn reset(&mut self) {
        self.config.reset();
        self.main_flop = false;
        self.vertical_flop = false;
    }

    /*
           |   CSEL=0   |   CSEL=1
     ------+------------+-----------
     Left  |  31 ($1f)  |  24 ($18)
     Right | 335 ($14f) | 344 ($158)

            |   RSEL=0  |  RSEL=1
     -------+-----------+----------
     Top    |  55 ($37) |  51 ($33)
     Bottom | 247 ($f7) | 251 ($fb)
    */

    pub fn update_main_flop(&mut self, x: u16, y: u16, den: bool) {
        /*
        Section: 3.9. The border unit
        1. If the X coordinate reaches the right comparison value, the main border
           flip flop is set.
        4. If the X coordinate reaches the left comparison value and the Y
           coordinate reaches the bottom one, the vertical border flip flop is set.
        5. If the X coordinate reaches the left comparison value and the Y
           coordinate reaches the top one and the DEN bit in register $d011 is set,
           the vertical border flip flop is reset.
        6. If the X coordinate reaches the left comparison value and the vertical
           border flip flop is not set, the main flip flop is reset.
        */
        if self.config.csel {
            if x == self.map_sprite_to_screen(0x18) {
                self.update_vertical_flop(y, den);
                if !self.vertical_flop {
                    self.main_flop = false;
                }
            } else if x == self.map_sprite_to_screen(0x158) {
                self.main_flop = true;
            }
        } else {
            if x == self.map_sprite_to_screen(0x1f) {
                self.update_vertical_flop(y, den);
                if !self.vertical_flop {
                    self.main_flop = false;
                }
            } else if x == self.map_sprite_to_screen(0x14f) {
                self.main_flop = true;
            }
        }
    }

    pub fn update_vertical_flop(&mut self, y: u16, den: bool) {
        /*
        Section: 3.9. The border unit
        2. If the Y coordinate reaches the bottom comparison value in cycle 63, the
           vertical border flip flop is set.
        3. If the Y coordinate reaches the top comparison value in cycle 63 and the
           DEN bit in register $d011 is set, the vertical border flip flop is
           reset.
        */
        if self.config.rsel {
            if y == 51 && den {
                self.vertical_flop = false;
            } else if y == 251 {
                self.vertical_flop = true;
            }
        } else {
            if y == 55 && den {
                self.vertical_flop = false;
            } else if y == 247 {
                self.vertical_flop = true;
            }
        }
    }
}
