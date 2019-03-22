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

//! app v1 protobuf messages

include!(concat!(
    env!("OUT_DIR"),
    "/oysterpack.ruggine.protos.core.messages.time.v1.rs"
));

use std::fmt;

impl<Tz: chrono::TimeZone> From<chrono::DateTime<Tz>> for Timestamp {
    fn from(ts: chrono::DateTime<Tz>) -> Self {
        Self {
            seconds: ts.timestamp(),
            nanos: ts.timestamp_subsec_nanos(),
        }
    }
}

impl From<Timestamp> for chrono::DateTime<chrono::Utc> {
    fn from(ts: Timestamp) -> Self {
        use chrono::TimeZone;
        chrono::Utc.timestamp(ts.seconds, ts.nanos)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use chrono::TimeZone;
        f.write_str(
            chrono::Utc
                .timestamp(self.seconds, self.nanos)
                .to_rfc3339()
                .as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datetime_conversions() {
        let now = chrono::Utc::now();
        let ts = Timestamp::from(now);
        let datetime: chrono::DateTime<chrono::Utc> = ts.clone().into();
        println!("now: {}", now);
        println!("ts: {}", ts);
        println!("datetime: {}", datetime);
        assert_eq!(now, datetime);
    }
}
