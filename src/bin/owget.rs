//! **owget** -- _Rust version_
//!
//! ## Read a value or directory from owserver
//! Combines the functions of **owread** and **owdir**
//!
//! **owget** is a tool in the 1-wire file system **OWFS**
//!
//! This Rust version of **owdir** is part of **owrust** -- the _Rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## SYNTAX
//! ```
//! owdir [OPTIONS] PATH
//! ```
//!
//! ## PURPOSE
//! __owget__ combines the functions of __owread__ and __owget__
//! * Useful for exploring the owfs structure without knowing if a file is a property or direcory
//! * __owtree__ is another way of visulaizing structure
//! * using *--dir--* for directory listing will distiguish files from directories
//!
//! ## OPTIONS
//! * `-s IP:port` (default `localhost:4304`)
//! * `--dir`      Add trailing **/** for directory elements
//! * `--bare`     Suppress non-device entries
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
//! ## USAGE
//! * owserver must be running in a network-accessible location
//! * `owget` is a command line program
//! * output to stdout
//! * errors to stderr
//!
//! ## EXAMPLE
//! Read a temperature
//! ```
//! owget /10.67C6697351FF/temperature
//! ```
//! ```text
//!     85.7961
//! ```
//! Get bare root directory
//! ```
//! owget --bare
//! ```
//! ```text
//! /10.67C6697351FF
//! /05.4AEC29CDBAAB
//! ```
//! {c} 2025 Paul H Alfille -- MIT Licence

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"

use owrust::parse_args;
use owrust::console::console_line;

fn main() {
    let mut owserver = owrust::new(); // create structure for owserver communication

    // configure and get paths
    match parse_args::command_line(&mut owserver) {
        Ok(paths) => {
            if paths.is_empty() {
                // No path -- assume root
                from_path(&mut owserver, "/".to_string());
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
fn from_path(owserver: &mut owrust::OwMessage, path: String) {
    match owserver.get(&path) {
        Ok(value) => {
            match String::from_utf8(value) {
                Ok(v) => {
                    console_line(v);
                }
                Err(e) => {
                    eprintln!("Unprintable string {}", e);
                }
            };
        }
        Err(e) => {
            eprintln!("Trouble with path {} Error {}", path, e);
        }
    }
}
