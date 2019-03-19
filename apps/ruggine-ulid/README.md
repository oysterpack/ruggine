# ruggine-ulid

OysterPack ruggine projects make heavy use of [ULIDs](https://github.com/ulid/spec).
This project provides a simple command line tool to generate and parse a ULID.

## Installation
```
cargo install ruggine-ulid
```

## CLI
<pre>
ruggine-ulid 0.1.0
Alfio Zappala <oysterpack.inc@gmail.com>
Command line utility to generate and parse a ULID (https://github.com/ulid/spec).

Output format: `ulid_str u128 (u64, u64) ulid_timestamp_rfc3339`

USAGE:
    ruggine-ulid <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    generate    generate new ULID
    help        Prints this message or the help of the given subcommand(s)
    parse       parse ULID represented as either a string, u128 number, or (u64, u64) tuple - ULID strings are
                leniently parsed as specified in Crockford Base32 Encoding (https://crockford.com/wrmg/base32.html)

</pre>

## Examples
```
# generate new ULID
> ruggine-ulid generate
01D6989F6P0TGQ8NH3K64EH6TD 1877390914292581084991991368823380813 (101773565394028193, 8382926898730670925) 2019-03-18T20:36:06.486Z

# parse a ULID string
> ruggine-ulid parse 01D6989F6P0TGQ8NH3K64EH6TD
01D6989F6P0TGQ8NH3K64EH6TD 1877390914292581084991991368823380813 (101773565394028193, 8382926898730670925) 2019-03-18T20:36:06.486Z

# parse a ULID represented as u128
> ruggine-ulid parse 1877390914292581084991991368823380813
01D6989F6P0TGQ8NH3K64EH6TD 1877390914292581084991991368823380813 (101773565394028193, 8382926898730670925) 2019-03-18T20:36:06.486Z

# parse a ULID represented as (u64, u64)
> ruggine-ulid parse 01D6989F6P0TGQ8NH3K64EH6TD
01D6989F6P0TGQ8NH3K64EH6TD 1877390914292581084991991368823380813 (101773565394028193, 8382926898730670925) 2019-03-18T20:36:06.486Z
```
