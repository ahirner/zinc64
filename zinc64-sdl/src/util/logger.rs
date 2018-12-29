// This file is part of zinc64.
// Copyright (c) 2016-2019 Sebastian Jastrzebski. All rights reserved.
// Licensed under the GPLv3. See LICENSE file in the project root for full license text.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::result::Result;
use std::str::FromStr;

use log::{LogLevel, LogMetadata, LogRecord};

pub struct Logger {
    level: LogLevel,
    targets: HashMap<String, LogLevel>,
}

impl Logger {
    pub fn build(level: &str) -> Result<Logger, String> {
        let loglevel =
            LogLevel::from_str(&level).map_err(|_| format!("invalid log level {}", level))?;
        let mut logger = Logger {
            level: loglevel,
            targets: HashMap::new(),
        };
        let path = Path::new("logger.conf");
        logger.load_config(path)?;
        Ok(logger)
    }

    pub fn enable(logger: Logger) -> Result<(), String> {
        log::set_logger(|max_log_level| {
            max_log_level.set(logger.get_level().to_log_level_filter());
            Box::new(logger)
        })
        .map_err(|_| "cannot initialize logging".to_string())
    }

    pub fn add_target(&mut self, target: String, level: &str) -> Result<(), String> {
        let loglevel = LogLevel::from_str(level)
            .map_err(|_| format!("invalid log level {} for target {}", level, &target))?;
        self.targets.insert(target, loglevel);
        Ok(())
    }

    pub fn get_level(&self) -> LogLevel {
        self.level
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::explicit_counter_loop))]
    pub fn load_config(&mut self, path: &Path) -> Result<(), String> {
        let file = File::open(path)
            .map_err(|_| format!("failed to open file {}", path.to_str().unwrap()))?;
        let reader = BufReader::new(file);
        let lines: Vec<_> = reader.lines().collect();
        let mut line_num = 0;
        for l in lines {
            line_num += 1;
            let line = l.unwrap();
            if let Some(equals) = line.find('=') {
                let (target, level) = line.split_at(equals);
                self.add_target(target.to_string(), &level[1..])?;
            } else {
                return Err(format!("invalid logger config line {}", line_num));
            }
        }
        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &LogMetadata<'_>) -> bool {
        if let Some(target_level) = self.targets.get(metadata.target()) {
            metadata.level() <= (*target_level)
        } else {
            metadata.level() <= self.level
        }
    }

    fn log(&self, record: &LogRecord<'_>) {
        if self.enabled(record.metadata()) {
            println!(
                "{} [{}] - {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }
}
