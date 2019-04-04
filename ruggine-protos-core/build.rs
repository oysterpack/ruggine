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

fn main() {
    generate_tests_grpc_code();
    generate_message_protobufs();
}

fn generate_message_protobufs() {
    let mut config = prost_build::Config::new();
    config
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.app.v1.App",
            "#[derive(Eq)]",
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.app.v1.App",
            r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#,
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.app.v1.Package",
            "#[derive(Eq)]",
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.app.v1.Package",
            r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#,
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.app.v1.PackageId",
            "#[derive(Eq, Hash, Ord, PartialOrd)]",
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.app.v1.PackageId",
            r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#,
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.app.v1.Process",
            "#[derive(Eq, Hash, Ord, PartialOrd, Copy)]",
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.app.v1.Process",
            r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#,
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.ulid.v1.Ulid",
            "#[derive(Eq, Hash, Copy, Ord, PartialOrd)]",
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.ulid.v1.Ulid",
            r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#,
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.time.v1.Timestamp",
            "#[derive(Eq, Hash, Ord, PartialOrd, Copy)]",
        )
        .type_attribute(
            "oysterpack.ruggine.protos.core.messages.time.v1.Timestamp",
            r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#,
        );

    tower_grpc_build::Config::from_prost(config)
        .enable_client(false)
        .enable_server(false)
        .build(
            &[
                "ruggine-protos-core/protos/messages/app_v1.proto",
                "ruggine-protos-core/protos/messages/time_v1.proto",
                "ruggine-protos-core/protos/messages/ulid_v1.proto",
            ],
            &[".."],
        )
        .unwrap_or_else(|e| panic!("generate_message_protobufs() failed: {}", e));
}

fn generate_tests_grpc_code() {
    tower_grpc_build::Config::new()
        .enable_client(true)
        .enable_server(true)
        .build(&["tests/protos/foo.proto"], &["tests/protos"])
        .unwrap_or_else(|e| panic!("generate_tests_grpc_code() failed: {}", e));
}
