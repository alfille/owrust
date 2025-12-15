//! **owread** -- _Rust version_
//!
//! ## Read a value from owserver ( from a 1-wire device )
//!
//! **owread** is a tool in the 1-wire file system **OWFS**
//!
//! This Rust version of **owread** is part of **owrust** -- the _Rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## SYNTAX
//! ```
//! owread [OPTIONS] PATH
//! ```
//!
//! ## PURPOSE
//! Read the value of a device property
//! * Often a sensor reading like `10.4323424342/temperature`
//! * can also be informational like `10.4323424342/type`
//!
//! ## OPTIONS
//! * `-s IP:port` (default `localhost:4304`)
//! * `--hex       show the value in hexidecimal
//! * `--size n    return only n bytes
//! * `--offset m  start return at byte m
//! * -h           for full list of options
//!
//! ## PATH
//! * 1-wire path to a file
//! * No Default
//! * More than one path can be given
//!
//! **owread** only works on files, not directories. Use **owget** to read both files and directories.
//!
//! ## USAGE
//! * owserver must be running in a network-accessible location
//! * `owread` is a command line program
//! * output to stdout
//! * errors to stderr
//!
//! ## EXAMPLE
//! Read a temperature
//! ```
//! owread /10.67C6697351FF/temperature
//! ```
//! ```text
//!     85.7961
//! ```
//! Read temperature in hex
//! ```
//! owread /10.67C6697351FF/temperature --hex
//! ```
//! ```text
//! 20 20 20 20 20 37 36 2E 31 35 38 35
//! ```
//! {c} 2025 Paul H Alfille -- MIT Licence

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use owrust::parse_args;

fn main() {
    let mut owserver = owrust::new(); // create structure for owserver communication

    // configure and get paths
    match parse_args::command_line(&mut owserver) {
        Ok(paths) => {
            if paths.is_empty() {
                // No path
                eprintln!("No 1-wire path, so no readings");
            } else {
                // for each pathon command line
                for path in paths.into_iter() {
                    from_path(&mut owserver, path);
                }
            }
        }
        Err(e) => {
            eprintln!("owread trouble {}", e);
        }
    }
}

// print 1-wire file contents (e.g. a sensor reading)
fn from_path(owserver: &mut owrust::OwClient, path: String) {
    match owserver.read(&path) {
        Ok(values) => match owserver.show_result(values) {
            Ok(s) => {
                println!("{}", s);
            }
            Err(e) => {
                eprintln!("Reading error {}", e);
            }
        },
        Err(e) => {
            eprintln!("Trouble with path {} Error {}", path, e);
        }
    }
}
