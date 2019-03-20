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

//! ULID protobuf utility methods.

impl From<rusty_ulid::Ulid> for crate::protos::messages::ulid::Ulid {
    fn from(ulid: rusty_ulid::Ulid) -> crate::protos::messages::ulid::Ulid {
        let mut proto = crate::protos::messages::ulid::Ulid::new();
        let (u64_1, u64_2): (u64, u64) = ulid.into();
        proto.u64_1 = u64_1;
        proto.u64_2 = u64_2;
        proto
    }
}

impl From<crate::protos::messages::ulid::Ulid> for rusty_ulid::Ulid {
    fn from(ulid: crate::protos::messages::ulid::Ulid) -> rusty_ulid::Ulid {
        (ulid.u64_1, ulid.u64_2).into()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn from_ulid() {
        let ulid = rusty_ulid::Ulid::generate();
        let ulid_proto = crate::protos::messages::ulid::Ulid::from(ulid);
        let (u64_1, u64_2): (u64, u64) = ulid.into();
        assert_eq!(ulid_proto.u64_1, u64_1);
        assert_eq!(ulid_proto.u64_2, u64_2);
        assert_eq!(ulid, ulid_proto.into());
    }
}