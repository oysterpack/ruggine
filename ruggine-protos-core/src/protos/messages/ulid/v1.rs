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

//! ulid v1

include!(concat!(
    env!("OUT_DIR"),
    "/oysterpack.ruggine.protos.core.messages.ulid.v1.rs"
));

use std::fmt;

impl From<rusty_ulid::Ulid> for Ulid {
    fn from(ulid: rusty_ulid::Ulid) -> Ulid {
        let (u64_0, u64_1): (u64, u64) = ulid.into();
        Ulid { u64_0, u64_1 }
    }
}

impl From<Ulid> for rusty_ulid::Ulid {
    fn from(ulid: Ulid) -> rusty_ulid::Ulid {
        (ulid.u64_0, ulid.u64_1).into()
    }
}

impl fmt::Display for Ulid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ulid: rusty_ulid::Ulid = (self.u64_0, self.u64_1).into();
        f.write_str(ulid.to_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_ulid() {
        let ulid = rusty_ulid::Ulid::generate();
        let ulid_proto = Ulid::from(ulid);
        println!("{}", ulid_proto);
        let (u64_0, u64_1): (u64, u64) = ulid.into();
        assert_eq!(ulid_proto.u64_0, u64_0);
        assert_eq!(ulid_proto.u64_1, u64_1);
        assert_eq!(ulid, ulid_proto.into());
    }
}
