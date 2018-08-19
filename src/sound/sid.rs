// This file is part of zinc64.
// Copyright (c) 2016-2018 Sebastian Jastrzebski. All rights reserved.
// Licensed under the GPLv3. See LICENSE file in the project root for full license text.

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use core::{Chip, Clock, SidModel, SoundOutput};
use log::LogLevel;
use resid;

// TODO sound: add sid output sample rate test cases

#[derive(Clone, Copy, PartialEq)]
pub enum SamplingMethod {
    Fast,
    Interpolate,
    Resample,
    ResampleFast,
}

pub struct Sid {
    // Dependencies
    system_clock: Rc<Clock>,
    sound_buffer: Arc<Mutex<dyn SoundOutput>>,
    // Functional Units
    resid: resid::Sid,
    // Runtime State
    cycles: u64,
}

impl Sid {
    pub fn new(
        chip_model: SidModel,
        system_clock: Rc<Clock>,
        sound_buffer: Arc<Mutex<dyn SoundOutput>>,
    ) -> Self {
        info!(target: "sound", "Initializing SID");
        let resid_model = match chip_model {
            SidModel::Mos6581 => resid::ChipModel::Mos6581,
            SidModel::Mos8580 => resid::ChipModel::Mos8580,
        };
        let resid = resid::Sid::new(resid_model);
        Sid {
            system_clock,
            sound_buffer,
            resid,
            cycles: 0,
        }
    }

    pub fn enable_filter(&mut self, enabled: bool) {
        self.resid.enable_filter(enabled);
    }

    pub fn set_sampling_parameters(
        &mut self,
        sampling_method: SamplingMethod,
        clock_freq: u32,
        sample_freq: u32,
    ) {
        let resid_sampling_method = match sampling_method {
            SamplingMethod::Fast => resid::SamplingMethod::Fast,
            SamplingMethod::Interpolate => resid::SamplingMethod::Interpolate,
            SamplingMethod::Resample => resid::SamplingMethod::Resample,
            SamplingMethod::ResampleFast => resid::SamplingMethod::ResampleFast,
        };
        self.resid
            .set_sampling_parameters(resid_sampling_method, clock_freq, sample_freq);
    }

    fn sync(&mut self) {
        if self.cycles != self.system_clock.get() {
            let delta = (self.system_clock.get() - self.cycles) as u32;
            self.clock_delta(delta);
        }
    }
}

impl Chip for Sid {
    fn clock(&mut self) {
        self.resid.clock();
        self.cycles = self.cycles.wrapping_add(1);
    }

    fn clock_delta(&mut self, cycles: u32) {
        if cycles > 0 {
            let mut buffer = [0i16; 8192]; // TODO sound: magic value
            let buffer_length = buffer.len();
            let mut samples = 0;
            let mut delta = cycles;
            while delta > 0 {
                let (read, next_delta) =
                    self.resid
                        .sample(delta, &mut buffer[samples..], buffer_length - samples, 1);
                samples += read as usize;
                delta = next_delta;
            }
            let mut output = self.sound_buffer.lock().unwrap();
            for i in 0..samples {
                output.write(buffer[i]);
            }
            self.cycles = self.cycles.wrapping_add(cycles as u64);
        }
    }

    fn process_vsync(&mut self) {
        self.sync();
    }

    fn reset(&mut self) {
        self.resid.reset();
        self.cycles = self.system_clock.get();
    }

    // I/O

    fn read(&mut self, reg: u8) -> u8 {
        self.sync();
        self.resid.read(reg)
    }

    fn write(&mut self, reg: u8, value: u8) {
        if log_enabled!(LogLevel::Trace) {
            trace!(target: "sid::reg", "Write 0x{:02x} = 0x{:02x}", reg, value);
        }
        self.sync();
        self.resid.write(reg, value);
    }
}

#[cfg(test)]
mod tests {
    /*
    use super::*;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    static SID_DATA: [u16; 51] = [
        25, 177, 250, 28, 214, 250,
        25, 177, 250, 25, 177, 250,
        25, 177, 125, 28, 214, 125,
        32, 94, 750, 25, 177, 250,
        28, 214, 250, 19, 63, 250,
        19, 63, 250, 19, 63, 250,
        21, 154, 63, 24, 63, 63,
        25, 177, 250, 24, 63, 125,
        19, 63, 250,
    ];

    fn setup_sid(clock: Rc<Clock>) -> Sid {
        let sound_buffer = Arc::new(Mutex::new(CircularBuffer::new(8192)));
        let mut sid = Sid::new(SidModel::Mos6581, clock, sound_buffer);
        sid.reset();
        sid
    }

    #[test]
    fn test_sid_output() {
        let clock = Rc::new(Clock::new());
        let mut sid = setup_sid(clock.clone());
        sid.write(0x05, 0x09); // AD1
        sid.write(0x06, 0x00); // SR1
        sid.write(0x18, 0x0f); // MODVOL
        let mut i = 0;
        while i < SID_DATA.len() {
            sid.write(0x01, SID_DATA[i + 0] as u8); // FREQHI1
            sid.write(0x00, SID_DATA[i + 1] as u8); // FREQLO1
            sid.write(0x00, 0x21); // CR1
            for _j in 0..SID_DATA[i + 2] {
                sid.clock_delta(22);
                clock.tick_delta(22);
            }
            sid.write(0x00, 0x20); // CR1
            for _j in 0..50 {
                sid.clock_delta(22);
                clock.tick_delta(22);
            }
            i += 3;
        }
        let buffer = sid.sound_buffer.lock().unwrap();
        assert_eq!(clock.get() * 44100 / 985248, buffer.len() as u64);
    }
    */
}
