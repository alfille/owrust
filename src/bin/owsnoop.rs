//! **owsnoop** -- _Rust version_
//!
//! ## Show owserver messages
//!
//! **owdir** a tool in the 1-wire file system **OWFS**
//!
//!```text
//! Changes:
//!
//!   Client  <----------------->  owserver
//!
//! To:
//!
//!   Client  <-->  owsnoop  <--> owserver
//!                    |
//!         console <--+--> console
//! ```
//! This Rust version of **owsnoop** is part of **owrust** -- the _Rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## SYNTAX
//! ```
//! owsnoop -p snoop_address -s owserver_address
//!
//! default owserver address: localhost:4304
//! ```
//! ## PURPOSE
//! __owsnoop__ shows a 1-wire owserver protocol messages
//! * requests from client (like owdir for a directory)
//! * responses from owserver
//!
//! ## USAGE
//! * owserver must be running in a network-accessible location
//! * `owsnoop` is a command line program
//! * output to stdout
//! * errors to stderr
//!
//! ### {c} 2025 Paul H Alfille -- MIT Licence

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"

use owrust::parse_args;

fn main() {
    let mut owserver = owrust::new(); // create structure for owserver communication

    // configure and get paths
    match parse_args::command_line(&mut owserver) {
        Ok(paths) => {
            if !paths.is_empty() {
                // Path not supported in owsnoop
                eprintln!("Path not supported in onsnoop, only -p and -s)");
                return;
            }
            owserver
                .listen()
                .expect("No listening address given (e.g. -p localhost:14304)");
        }
        Err(e) => {
            eprintln!("owsnoop parameter trouble {}", e);
        }
    }
}
