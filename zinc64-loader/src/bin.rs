// This file is part of zinc64.
// Copyright (c) 2016-2019 Sebastian Jastrzebski. All rights reserved.
// Licensed under the GPLv3. See LICENSE file in the project root for full license text.

use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;
use std::result::Result;

use zinc64::system::{AutostartMethod, Image, C64};

use super::Loader;

struct BinImage {
    data: Vec<u8>,
    offset: u16,
}

impl Image for BinImage {
    fn mount(&mut self, c64: &mut C64) {
        info!(target: "loader", "Mounting BIN image");
        c64.get_cpu_mut().write(0x0001, 0);
        c64.load(&self.data, self.offset);
        c64.get_cpu_mut().set_pc(self.offset);
    }

    fn unmount(&mut self, _c64: &mut C64) {}
}

pub struct BinLoader {
    offset: u16,
}

impl BinLoader {
    pub fn new(offset: u16) -> Self {
        Self { offset }
    }
}

impl Loader for BinLoader {
    fn autostart(&self, path: &Path) -> Result<AutostartMethod, io::Error> {
        let image = self.load(path)?;
        Ok(AutostartMethod::WithBinImage(image))
    }

    fn load(&self, path: &Path) -> Result<Box<dyn Image>, io::Error> {
        info!(target: "loader", "Loading BIN {}", path.to_str().unwrap());
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Box::new(BinImage {
            data,
            offset: self.offset,
        }))
    }
}
