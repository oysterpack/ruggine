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
    "/oysterpack.ruggine.protos.core.messages.app.v1.rs"
));

/// [App](protos/messages/app/v1/struct.App.html) constructor, which assigns a unique instance ID.
///
/// ## Example
/// ```
/// # use ruggine_protos_core::app;
/// let app = app!();
/// ```
#[macro_export]
macro_rules! app {
    () => {
        $crate::protos::messages::app::v1::App {
            package: Some($crate::package!()),
            instance_id: Some(rusty_ulid::Ulid::generate().into()),
        }
    };
}

/// [PackageId](protos/messages/app/v1/struct.PackageId.html) constructor that is initialized via [cargo provided environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates):
/// - CARGO_PKG_NAME
/// - CARGO_PKG_VERSION
///
/// ## Example
/// ```
/// # use ruggine_protos_core::package_id;
/// let pkg_id = package_id!();
/// ```
#[macro_export]
macro_rules! package_id {
    () => {
        $crate::protos::messages::app::v1::PackageId {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    };
}

/// [Package](protos/messages/app/v1/struct.Package.html) constructor that is initialized via [cargo provided environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates):
/// - CARGO_PKG_NAME
/// - CARGO_PKG_VERSION
/// - CARGO_PKG_AUTHORS
/// - CARGO_PKG_DESCRIPTION
/// - CARGO_PKG_HOMEPAGE
/// - CARGO_PKG_REPOSITORY
///
/// ## Example
/// ```
/// # use ruggine_protos_core::package;
/// let pkg = package!();
/// ```
#[macro_export]
macro_rules! package {
    () => {{
        let authors = env!("CARGO_PKG_AUTHORS").to_string();
        let authors = authors
            .as_str()
            .split(':')
            .map(|author| author.to_string())
            .collect();
        $crate::protos::messages::app::v1::Package {
            id: Some($crate::package_id!()),
            authors,
            description: env!("CARGO_PKG_DESCRIPTION").to_string(),
            homepage: env!("CARGO_PKG_HOMEPAGE").to_string(),
            repository: env!("CARGO_PKG_REPOSITORY").to_string(),
        }
    }};
}

lazy_static::lazy_static! {
    /// Defines a `PROCESS` lazy static for [Process](protos/messages/app/v1/struct.Process.html).
    /// A unique instance ID is assigned. The lazy static should be defined at application startup
    static ref PROCESS: Process = Process {
        pid: std::process::id(),
        start_time: Some(chrono::Utc::now().into()),
    };
}

/// The process is assigned a unique instance id.
/// - this will lazily load and cache the Process. Thus, in order for the process start time to be
///   accurate, this should be the first function invoked in the app's main()
pub fn process() -> Process {
    *PROCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::*;
    use std::panic::catch_unwind;

    #[test]
    fn package_id_macro() {
        let _ = catch_unwind(env_logger::init);

        let id = PackageId {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        info!("{:?}", id);

        let id_2 = package_id!();
        info!("{:?}", id_2);
        assert_eq!(id, id_2);
    }

    #[test]
    fn package_macro() {
        let _ = catch_unwind(env_logger::init);

        let pkg = package!();
        info!("{:#?}", pkg);

        let id = PackageId {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        assert_eq!(pkg.id.unwrap(), id);
        assert_eq!(pkg.authors, vec![env!("CARGO_PKG_AUTHORS").to_string()]);
        assert_eq!(pkg.description, env!("CARGO_PKG_DESCRIPTION").to_string());
        assert_eq!(pkg.homepage, env!("CARGO_PKG_HOMEPAGE").to_string());
        assert_eq!(pkg.repository, env!("CARGO_PKG_REPOSITORY").to_string());
    }

    #[test]
    fn app_macro() {
        let _ = catch_unwind(env_logger::init);

        let app = app!();
        info!("{:#?}", app);

        let pkg = app.package.expect("package is not set");
        for pkg_id in pkg.id.as_ref() {
            let id = PackageId {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            };
            assert_eq!(pkg_id, &id);
        }

        let _ = app.instance_id.expect("instance_id is not set");
    }

    #[test]
    fn process_macro() {
        let _ = catch_unwind(env_logger::init);

        let now = chrono::Utc::now();
        let process = process();
        info!("{:#?}", process);
        assert!(process.pid > 0);
        let start_time = process.start_time.expect("start_time is not set");
        let start_time: chrono::DateTime<chrono::Utc> = start_time.into();
        assert!(start_time >= now);
    }
}
