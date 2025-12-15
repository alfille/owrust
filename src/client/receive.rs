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
//! The main struct is OwClient which holds all the configuration information.
//! Typically it is populated by the command line or configuration files
//!
//! ## EXAMPLES
//! ```
//! use owrust ; // basic library
//! use owrust::parse_args ; // configure from command line, file or OsString
//!
//! let mut owserver = owrust::new() ; // create an OwClient struct
//!   // configure from command line and get 1-wire paths
//! let paths = parse_args::command_line( &mut owserver ) ;
//!   // Call any of the OwClient functions like dir, read, write,...
//!   ```

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use crate::client::OwMessageSend;
use crate::client::Token;
pub use crate::error::{OwEResult, OwError};
use std::io::{Read, Write};
use std::net::TcpStream;

const SERVERMESSAGE: u32 = 1 << 16;
const SERVERTOKENS: u32 = 0xFFFF;

/// message received back from owserver
/// * header (24 bytes) and content
pub(super) struct OwMessageReceive {
    pub(super) version: u32,
    pub(super) payload: u32,
    pub(super) ret: i32,
    pub(super) flags: u32,
    pub(super) size: u32,
    pub(super) offset: u32,
    pub(super) content: Vec<u8>,
    pub(super) tokenlist: Vec<Token>,
}
impl OwMessageReceive {
    const HSIZE: usize = 24;
    /// Take first 24 bytes of buffer to fill header
    pub(super) fn new(buffer: [u8; OwMessageReceive::HSIZE]) -> Self {
        OwMessageReceive {
            version: u32::from_be_bytes(buffer[0..4].try_into().unwrap()),
            payload: u32::from_be_bytes(buffer[4..8].try_into().unwrap()),
            ret: u32::from_be_bytes(buffer[8..12].try_into().unwrap()) as i32,
            flags: u32::from_be_bytes(buffer[12..16].try_into().unwrap()),
            size: u32::from_be_bytes(buffer[16..20].try_into().unwrap()),
            offset: u32::from_be_bytes(buffer[20..24].try_into().unwrap()),
            content: [].to_vec(),
            tokenlist: [].to_vec(),
        }
    }
    pub(super) fn tell(&self) {
        eprintln!(
            "ver {:X}, pay {}, ret {}, flg {:X}, siz {}, off {}",
            self.version, self.payload, self.ret, self.flags, self.size, self.offset
        );
    }

    /// ### get_packet
    /// Get a message from the network and parse it:
    /// * read header ( 6 words), translated from network order
    /// * read payload
    /// * read tokens
    /// * check for our token on list (==loop)
    /// * ignore pings
    pub fn get_packet(
        stream: &mut TcpStream,
        my_token: Option<Token>,
    ) -> OwEResult<OwMessageReceive> {
        // get a single non-ping message.
        // May need multiple for directories
        static HSIZE: usize = 24;
        let mut buffer: [u8; HSIZE] = [0; HSIZE];

        loop {
            stream.read_exact(&mut buffer)?;
            let mut rcv = OwMessageReceive::new(buffer);

            // read payload
            if rcv.payload > 0 {
                // create Vec with just the right size (based on payload)
                rcv.content = Vec::with_capacity(rcv.payload as usize);
                rcv.content.resize(rcv.payload as usize, 0);

                stream.read_exact(&mut rcv.content)?;
            }

            // read tokens
            if (rcv.version & SERVERMESSAGE) == SERVERMESSAGE {
                let toks = rcv.version & SERVERTOKENS;
                for _ in 0..toks {
                    let mut tok: Token = [0u8; 16];
                    stream.read_exact(&mut tok)?;
                    rcv.tokenlist.push(tok)
                }
            }

            // test token
            if let Some(t) = my_token
                && rcv.tokenlist.contains(&t)
            {
                return Err(OwError::General("Loop in owserver topology".to_string()));
            }

            // test ping message (ignore -- it's a keepalive)
            if (rcv.payload as i32) < 0 {
                // ping
                continue;
            }

            return Ok(rcv);
        }
    }
}

pub trait PrintMessage {
    // Header
    fn receive_title(&self) -> String;
    // Getters
    fn version(&self) -> u32;
    fn flags(&self) -> u32;
    fn payload(&self) -> u32;
    fn mtype(&self) -> u32 {
        self.ret() as u32
    }
    fn ret(&self) -> i32 {
        self.mtype() as i32
    }
    fn size(&self) -> u32;
    fn offset(&self) -> u32;
    fn content(&self) -> &Vec<u8>;
    //fn tokenlist( &self ) -> Vec<Token> ;

    /// ### print_all
    /// Shows message contents
    /// * connection from a listener port
    /// * message is from a client
    /// * program is functioning as a server
    /// * typically to show messages and forward them unchanged
    fn print_all(&self) {
        println!("{}", self.line1());
        println!("{}", self.line2());
        println!("{}", self.line3());
    }

    fn line1(&self) -> String {
        format!(
            "{} message. Version: {}",
            self.receive_title(),
            self.string_version()
        )
    }
    fn line2(&self) -> String {
        self.string_type()
    }
    fn alt_line2(&self) -> String {
        format!("Return code = {}", self.ret())
    }
    fn line3(&self) -> String {
        format!(
            "Flags:{} Payload:{} Size:{} Offset:{}",
            self.string_flags(),
            self.string_payload(),
            self.string_size(),
            self.string_offset()
        )
    }
    fn string_path(&self) -> String {
        String::from_utf8_lossy(self.content()).to_string()
    }
    fn string_path_pair(&self) -> (String, String) {
        let path_len: usize = ((self.payload() as i32) - (self.size() as i32)) as usize;
        let first: String = String::from_utf8_lossy(&self.content()[..path_len]).to_string();
        let second: String = self.content()[path_len..self.payload() as usize]
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join(" ");
        (first, second)
    }
    fn string_version(&self) -> String {
        if (self.version() & SERVERMESSAGE) == SERVERMESSAGE {
            format!(
                "{:X} tokens={}",
                self.version(),
                self.version() & SERVERTOKENS
            )
        } else {
            format!("{:X}", self.version())
        }
    }
    fn string_ret(&self) -> String {
        format!("{}", self.ret())
    }
    fn string_flags(&self) -> String {
        format!("{:X}", self.flags())
    }
    fn string_type(&self) -> String {
        match self.mtype() {
            OwMessageSend::NOP => "NOP".to_string(),
            OwMessageSend::READ => format!("READ {}", self.string_path()),
            OwMessageSend::WRITE => {
                let w = self.string_path_pair();
                format!("WRITE {} => {}", w.0, w.1)
            }
            OwMessageSend::DIR => format!("DIR {}", self.string_path()),
            OwMessageSend::SIZE => "SIZE".to_string(),
            OwMessageSend::PRESENT => "PRESENT".to_string(),
            OwMessageSend::DIRALL => format!("DIRALL {}", self.string_path()),
            OwMessageSend::GET => format!("GET {}", self.string_path()),
            OwMessageSend::DIRALLSLASH => format!("DIRALLSLASH {}", self.string_path()),
            OwMessageSend::GETSLASH => format!("GETSLASH {}", self.string_path()),
            _ => format!("UNKNOWN message number {}", self.mtype()),
        }
    }
    fn string_payload(&self) -> String {
        format!("{}", self.payload())
    }
    fn string_size(&self) -> String {
        format!("{}", self.size())
    }
    fn string_offset(&self) -> String {
        format!("{}", self.offset())
    }
}

impl PrintMessage for OwMessageReceive {
    fn receive_title(&self) -> String {
        "INCOMING from owserver ".to_string()
    }
    fn version(&self) -> u32 {
        self.version
    }
    fn ret(&self) -> i32 {
        self.ret
    }
    fn offset(&self) -> u32 {
        self.offset
    }
    fn payload(&self) -> u32 {
        self.payload
    }
    fn size(&self) -> u32 {
        self.size
    }
    fn flags(&self) -> u32 {
        self.flags
    }
    fn content(&self) -> &Vec<u8> {
        &self.content
    }
    /*
    fn tokenlist( &self ) -> u32 {
        self.tokenlist
    }
    */
}
