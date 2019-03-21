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

//! app util functions

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/ruggine_ulid/0.1.2")]
#![doc(hidden)]

use rusty_ulid::Ulid;

#[doc(hidden)]
/// Supported ULID formats are:
/// 1. ULID string
/// 2. u128
/// 3. (u64, u64)
pub fn parse_ulid(ulid: &str) -> Result<Ulid, failure::Error> {
    ulid.parse::<Ulid>()
        .or_else(|_| ulid.parse::<u128>().map(Ulid::from))
        .or_else(|_| parse_u64_tuple(&ulid))
}

/// expected format is (u64, u64), e.g., `(101772235475161357, 12325636877615710392)`
fn parse_u64_tuple(ulid: &str) -> Result<Ulid, failure::Error> {
    let re = regex::Regex::new(r"^\((\d+)\s*,\s*(\d+)\)$").unwrap();
    match re.captures(ulid) {
        Some(captures) => match captures.get(1).unwrap().as_str().parse::<u64>() {
            Ok(ulid_u64_0) => match captures.get(2).unwrap().as_str().parse::<u64>() {
                Ok(ulid_u64_1) => Ok(Ulid::from((ulid_u64_0, ulid_u64_1))),
                Err(err) => Err(failure::err_msg(format!(
                    "{} : {}",
                    err,
                    captures.get(2).unwrap().as_str()
                ))),
            },
            Err(err) => Err(failure::err_msg(format!(
                "{} : {}",
                err,
                captures.get(1).unwrap().as_str()
            ))),
        },
        None => Err(failure::err_msg(format!("Invalid format: {}", ulid))),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_ulid_str() {
        let ulid = Ulid::generate();
        let s = ulid.to_string();
        println!("parse_ulid({})", s);
        assert_eq!(parse_ulid(&s).unwrap(), ulid);
    }

    #[test]
    fn parse_ulid_str_invalid() {
        assert!(parse_ulid("invalid").is_err());
    }

    #[test]
    fn parse_ulid_u128() {
        let ulid = Ulid::generate();
        let ulid_u128: u128 = ulid.into();
        let s = format!("{}", ulid_u128);
        println!("parse_ulid({})", s);
        assert_eq!(parse_ulid(&s).unwrap(), ulid);
    }

    #[test]
    fn parse_ulid_u64_tuple() {
        let ulid = Ulid::generate();
        let ulid_u64_tuple: (u64, u64) = ulid.into();
        let s = format!("{:?}", ulid_u64_tuple);
        println!("parse_ulid({})", s);
        assert_eq!(parse_ulid(&s).unwrap(), ulid);
    }

    #[test]
    fn parse_ulid_u64_tuple_invalid() {
        let results = vec![
            parse_ulid("(101772465838646788, 181740289001541871298098098)"),
            parse_ulid("101772465838646788, 18174028900154187129"),
            parse_ulid("(101772465838646788, 18174028900154187129"),
            parse_ulid("101772465838646788, 18174028900154187129)"),
            parse_ulid("[101772465838646788, 18174028900154187129]"),
        ];
        results.iter().for_each(|result| println!("{:?}", result));

        assert!(results.iter().all(|result| result.is_err()));
    }

}
