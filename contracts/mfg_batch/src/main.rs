//MFG_BATCH

// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


// Fun fact https://crates.io/crates/diesel (Rust ORM)
#[macro_use]
extern crate cfg_if;
extern crate grid_sdk;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[macro_use]
        extern crate clap;
        #[macro_use]
        extern crate log;
        use std::process;
        use log::LogLevelFilter;
        use log4rs::append::console::ConsoleAppender;
        use log4rs::config::{Appender, Config, Root};
        use log4rs::encode::pattern::PatternEncoder;
        use sawtooth_sdk::processor::TransactionProcessor;
        // Load the MfgBatch transaction handler
        use crate::handler::MfgBatchTransactionHandler;
    } else {
        #[macro_use]
        extern crate sabre_sdk;
    }
}

pub mod handler;
mod payload;
pub mod permissions;
mod state;
mod validation;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let matches = clap_app!(intkey =>
        (version: crate_version!())
        (about: "Grid Manufactured Batch Processor (Rust)")
        (@arg connect: -C --connect +takes_value
         "connection endpoint for validator")
        (@arg verbose: -v --verbose +multiple
         "increase output verbosity"))
    .get_matches();

    let endpoint = matches
        .value_of("connect")
        // Attach WASM to Sabre validator 
        .unwrap_or("tcp://localhost:4004");

    let console_log_level;
    match matches.occurrences_of("verbose") {
        0 => console_log_level = LogLevelFilter::Warn,
        1 => console_log_level = LogLevelFilter::Info,
        2 => console_log_level = LogLevelFilter::Debug,
        _ => console_log_level = LogLevelFilter::Trace,
    }

    // Format and log messages
    let stdout = ConsoleAppender::builder()
        // Create a heap reference object
        .encoder(Box::new(PatternEncoder::new(
            "{h({l:5.5})} | {({M}:{L}):20.20} | {m}{n}",
        )))
        .build();

    let config = match Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(console_log_level))
    {
        Ok(x) => x,
        Err(_) => process::exit(1),
    };

    match log4rs::init_config(config) {
        Ok(_) => (),
        Err(_) => process::exit(1),
    }
    // Assign the batch handler to the Sabre validator
    let handler = MfgBatchTransactionHandler::new();
    let mut processor = TransactionProcessor::new(endpoint);

    info!("Console logging level: {}", console_log_level);

    processor.add_handler(&handler);
    processor.start();
}

#[cfg(target_arch = "wasm32")]
fn main() {}
