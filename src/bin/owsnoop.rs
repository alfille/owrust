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
//! ## EXAMPLES
//! ### Snoop console
//! ```
//! owsnoop -s localhost:4304 -p localhost:14304
//! ```
//! ### Directory list
//! ```
//! owdir -s localhost:14304
//! ```
//! Snoop console:
//! ```text
//! Query Message incoming Version: 10001 tokens=1
//! DIRALL /
//! Flags: C mbar f.i net   alias  bus
//! Payload:2 Size:0 Offset:0
//! 
//! Response Message Incoming Version: 0
//! Return code = 0
//! Flags: C mbar f.i net   alias  bus
//! Payload:113 Size:112 Offset:32770
//!
//! ```
//! ### Temperature read
//! ```
//! owread -s localhost:14304 /10.67c6697351ff/temperature
//! ```
//! Snoop console:
//! ```text
//! Query Message incoming Version: 10001 tokens=1
//! READ /10.67c6697351ff/temperature
//! Flags: C mbar f.i net   alias  bus
//! Payload:29 Size:65536 Offset:0
//! 
//! Response Message Incoming Version: 0
//! Return code = 12
//! Flags: C mbar f.i net   alias  bus
//! Payload:12 Size:12 Offset:0
//! 
//! ```

//! ### {c} 2025 Paul H Alfille -- MIT Licence

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"

use owrust::parse_args::{OwSnoop, Parser};

fn main() {
    let mut owserver = owrust::new(); // create structure for owserver communication
    let prog = OwSnoop;

    // configure and get paths
    match prog.command_line(&mut owserver) {
        Ok(paths) => {
            if !paths.is_empty() {
                // Path not supported in owsnoop
                eprintln!("Path not supported in onsnoop, only -p and -s)");
                return;
            }
            match owserver.listen() {
                Ok(_x) => (),
                Err(e) => {
                    eprintln!("No listening address given (e.g. -p localhost:14304) {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("owsnoop parameter trouble {}", e);
        }
    }
}
