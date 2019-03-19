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

use assert_cmd::prelude::*;

use lazy_static::lazy_static;
use std::process::{Command, Output};

lazy_static! {
    static ref REGEX_OUTPUT: regex::Regex = regex::Regex::new(
        r"^[[:alnum:]]{26} \d+ \(\d+, \d+\) \d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{3}Z$"
    )
    .unwrap();
}

fn check_output_format(output: &Output) {
    let output = String::from_utf8_lossy(&output.stdout);
    println!("[{}]", output.as_ref().trim());
    assert!(
        REGEX_OUTPUT.is_match(output.as_ref().trim()),
        "output did not match regex"
    );
}

#[test]
fn no_args() {
    let mut cmd = Command::cargo_bin("ruggine-ulid").unwrap();
    let output = cmd.output().unwrap();
    println!("{:#?}", output);
    assert!(!output.status.success());
}

#[test]
fn help() {
    let mut cmd = Command::cargo_bin("ruggine-ulid").unwrap();
    cmd.arg("help");
    let output = cmd.output().unwrap();
    println!("{:#?}", output);
    assert!(output.status.success());
}

#[test]
fn generate() {
    let mut cmd = Command::cargo_bin("ruggine-ulid").unwrap();
    cmd.arg("generate");
    let output = cmd.output().unwrap();
    println!("{:?}", output);
    assert!(output.status.success());
    check_output_format(&output);
}

#[test]
fn parse_ulid_str() {
    let ulid = rusty_ulid::Ulid::generate();

    let mut cmd = Command::cargo_bin("ruggine-ulid").unwrap();
    cmd.args(&["parse", ulid.to_string().as_str()]);
    let output = cmd.output().unwrap();
    println!("{:?}", output);
    assert!(output.status.success());
    check_output_format(&output);
}

#[test]
fn parse_ulid_u128() {
    let ulid = rusty_ulid::Ulid::generate();
    let ulid: u128 = ulid.into();
    let ulid = format!("{}", ulid);
    println!("parse_ulid_u128(): {}", ulid);

    let mut cmd = Command::cargo_bin("ruggine-ulid").unwrap();
    cmd.args(&["parse", ulid.as_str()]);
    let output = cmd.output().unwrap();
    println!("{:?}", output);
    check_output_format(&output);
}

#[test]
fn parse_ulid_u64_u64() {
    let ulid = rusty_ulid::Ulid::generate();
    let ulid: (u64, u64) = ulid.into();
    let ulid = format!("{:?}", ulid);
    println!("parse_ulid_u64_u64(): {}", ulid);

    let mut cmd = Command::cargo_bin("ruggine-ulid").unwrap();
    cmd.args(&["parse", ulid.as_str()]);
    let output = cmd.output().unwrap();
    println!("{:?}", output);
    check_output_format(&output);
}
