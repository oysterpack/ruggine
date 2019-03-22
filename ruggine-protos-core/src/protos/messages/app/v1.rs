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

// TODO: App!()
// - generate a lazy static App
// - use procedural macro

/// PackageId constructor that is initialized via [cargo provided environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates):
/// - `CARGO_PKG_NAME`
/// - `CARGO_PKG_VERSION`
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

/// Package constructor that is initialized via [cargo provided environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates):
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_id_macro() {
        let id = PackageId {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        println!("{:?}", id);

        let id_2 = package_id!();
        println!("{:?}", id_2);
        assert_eq!(id, id_2);
    }

    #[test]
    fn package_macro() {
        let pkg = package!();
        println!("{:#?}", pkg);

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
}
