//! **owrust** Rust library interfaces with owserver to use 1-wire devices
//!
//! This is a tool in the 1-wire file system **OWFS**
//!
//! This library is the central part of **owrust** -- the _rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## PURPOSE
//! lib.rs is the library code that actually performs the **owserver protocol**.
//! Communication with **owserver** is over TCP/IP (network) using an efficient well-documented protocol.
//!
//! Supported operations are read, write, dir, present and size, with some variations
//!
//! The main struct is OwMessage which holds all the configuration information.
//! Typically it is populated by the command line or configuration files
//!
//! ## EXAMPLES
//! ```
//! use owrust ; // basic library
//! use owrust::parse_args ; // configure from command line, file or OsString
//!
//! let mut owserver = owrust::new() ; // create an OwMessage struct
//!   // configure from command line and get 1-wire paths
//! let paths = parse_args::command_line( &mut owserver ) ;
//!   // Call any of the OwMessage functions like dir, read, write,...
//!   ```

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use md5::{Digest, Md5};
use rand::rngs::OsRng;
use rand::RngCore;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::message::Token;

pub(super) fn make_token() -> Token {
    let mut buffer: Vec<u8> = Vec::new();

    // add time
    buffer.extend_from_slice(
        &SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
            .to_le_bytes(),
    );

    // add pid
    buffer.extend_from_slice(&std::process::id().to_le_bytes());

    // add random salt
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    buffer.extend_from_slice(&salt);

    // MD5 hash
    let mut hasher = Md5::new();
    hasher.update(&buffer);

    // return 16 bytes
    let mut ret: Token = [0u8; 16];
    ret.copy_from_slice(&hasher.finalize());
    ret
}
