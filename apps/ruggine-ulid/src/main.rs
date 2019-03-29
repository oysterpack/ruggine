/*
 * Copyright 2019 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! OysterPack ruggine projects make heavy use of [ULIDs](https://github.com/ulid/spec).
//! This project provides a simple command line tool to generate and parse a ULID.
//!
//! ## Installation
//! ```
//! cargo install ruggine-ulid
//! ```
//!
//! ## CLI
//! <pre>
//! ruggine-ulid 0.1.0
//! Alfio Zappala <oysterpack.inc@gmail.com>
//! Command line utility to generate and parse a ULID (https://github.com/ulid/spec).
//! Output format: `ulid_str u128 (u64, u64) ulid_timestamp_rfc3339`
//!
//! USAGE:
//!     ruggine-ulid <SUBCOMMAND>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! SUBCOMMANDS:
//!     generate    generate new ULID
//!     help        Prints this message or the help of the given subcommand(s)
//!     parse       parse ULID represented as either a string, u128 number, or (u64, u64) tuple - ULID strings are
//!                 leniently parsed as specified in Crockford Base32 Encoding (https://crockford.com/wrmg/base32.html)
//! </pre>
//!
//! ## Examples
//!
//! ### generate new ULID
//! ```
//! > ruggine-ulid generate
//! 01D6989F6P0TGQ8NH3K64EH6TD 1877390914292581084991991368823380813 (101773565394028193, 8382926898730670925) 2019-03-18T20:36:06.486Z
//! ```
//!
//! ### parse a ULID string
//! ```
//! > ruggine-ulid parse 01D6989F6P0TGQ8NH3K64EH6TD
//! 01D6989F6P0TGQ8NH3K64EH6TD 1877390914292581084991991368823380813 (101773565394028193, 8382926898730670925) 2019-03-18T20:36:06.486Z
//! ```
//!
//! ### parse a ULID represented as u128
//! ```
//! > ruggine-ulid parse 1877390914292581084991991368823380813
//! 01D6989F6P0TGQ8NH3K64EH6TD 1877390914292581084991991368823380813 (101773565394028193, 8382926898730670925) 2019-03-18T20:36:06.486Z
//! ```
//!
//! ### parse a ULID represented as (u64, u64)
//! ```
//! > ruggine-ulid parse "(101773565394028193, 8382926898730670925)"
//! 01D6989F6P0TGQ8NH3K64EH6TD 1877390914292581084991991368823380813 (101773565394028193, 8382926898730670925) 2019-03-18T20:36:06.486Z
//! ```

mod utils;

use exitfailure::ExitFailure;
use rusty_ulid::Ulid;
use structopt::StructOpt;
use utils::parse_ulid;

fn main() -> Result<(), ExitFailure> {
    Command::from_args().execute()?;
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ruggine-ulid",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// Command line utility to generate and parse a ULID (https://github.com/ulid/spec).
///
/// Output format: `ulid_str u128 (u64, u64) ulid_timestamp_rfc3339`
enum Command {
    #[structopt(name = "generate")]
    /// generate new ULID
    Generate,
    #[structopt(name = "parse")]
    /// parse ULID represented as either a string, u128 number, or (u64, u64) tuple
    /// - ULID strings are leniently parsed as specified in Crockford Base32 Encoding (https://crockford.com/wrmg/base32.html)
    Parse { ulid: String },
}

impl Command {
    fn execute(self) -> Result<(), failure::Error> {
        match self {
            Command::Generate => {
                print_ulid(Ulid::generate());
                Ok(())
            }
            Command::Parse { ulid } => parse_ulid(&ulid).map(print_ulid),
        }
    }
}

/// format: "ULID", u128, (u64, u64), {ULID timestamp in rfc3339 format}, e.g.,
/// `01D68MY5VYJ46TP3BNDYMBTG5R 1877366381619605767429357207335813304 (101772235475161357, 12325636877615710392) 2019-03-18T14:57:53.534Z`
fn print_ulid(ulid: Ulid) {
    let ulid_u128: u128 = ulid.into();
    let ulid_64_tuple: (u64, u64) = ulid.into();
    println!(
        "{} {} {:?} {:?}",
        ulid.to_string(),
        ulid_u128,
        ulid_64_tuple,
        ulid.datetime()
    );
}
