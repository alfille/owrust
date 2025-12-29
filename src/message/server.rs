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
//! use owrust::parse_args::{Parser,OwLib} ; // configure from command line, file or OsString
//!
//! let mut owserver = owrust::new() ; // create an OwMessage struct
//! let prog = OwLib ;
//!   // configure from command line and get 1-wire paths
//! let paths = prog.command_line( &mut owserver ) ;
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

use std::net::TcpStream;
use std::time::Duration;

use crate::message::query::OwQuery;

use crate::console::console_lines;
use crate::message::response::PrintMessage;

use crate::OwMessage;

pub(super) struct OwServerInstance {
    message: crate::OwMessage,
    stream_in: TcpStream,
}
impl OwServerInstance {
    pub(super) fn new(message: crate::OwMessage, stream_in: TcpStream) -> OwServerInstance {
        OwServerInstance { message, stream_in }
    }
    pub(super) fn handle_query(&mut self) {
        // Set timeout
        match self
            .stream_in
            .set_read_timeout(Some(Duration::from_secs(5)))
        {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Cannot set timeout to server query {}", e);
                return;
            }
        }

        // get Query
        let mut rcv = match OwQuery::get(&mut self.stream_in, self.message.token) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Could not read a packet. {}", e);
                return;
            }
        };

        // match persistence
        self.message
            .stream
            .set_persistence(rcv.flags() & OwMessage::PERSISTENCE != 0);

        // relay message on
        console_lines(rcv.print_all("Query Message incoming"));
        let _ = self.message.send_packet(&mut rcv);

        let old_dir_type = rcv.mtype == crate::message::query::OwQuery::DIR;

        loop {
            // wait for responses
            if let Ok(mut resp) = self.message.get_msg_any() {
                console_lines(resp.print_all("Response Message Incoming"));
                let _ = resp.send(&mut self.stream_in);
                if resp.payload < 0 {
                    // just a ping
                    continue;
                } else if resp.payload == 0 || !old_dir_type {
                    break;
                }
            }
        }
    }
}
