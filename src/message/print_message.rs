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

pub use crate::error::OwEResult;
use crate::message::OwQuery;

/// ### PrintMessage trait
/// Trait for displaying content of "snooped" messages
/// * covers OwResponse and OwQuery
/// * uses "getter" functions for struct fields
/// * able to navigate the different interpretation of the ret / mtype field
/// * 4 lines
///   * Title and version
///   * Message type or return code and contents
///   * Flag details
///   * Size and offset
/// * could also be used for client messages
pub trait PrintMessage {
    // Getters
    fn version(&self) -> u32;
    fn flags(&self) -> u32;
    fn payload(&self) -> i32;
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
    fn print_all(&self, title: &str) -> [String; 5] {
        [
            self.version_line_1(title),
            self.line_2(),
            self.flags_line_3(),
            self.sizes_line_4(),
            "".to_string(),
        ]
    }

    fn version_line_1(&self,title: &str) -> String {
        format!("{} Version: {}", title, self.string_version())
    }
    fn line_2(&self) -> String ;
    fn mtype_line_2(&self) -> String {
        self.string_type()
    }
    fn return_line_2(&self) -> String {
        format!("Return code = {}", self.ret())
    }
    fn flags_line_3(&self) -> String {
            format!(
                "Flags: {}",
                crate::message::OwMessage::flag_string(self.flags())
			)
	}
    fn sizes_line_4(&self) -> String {
        format!(
            "Payload:{} Size:{} Offset:{}",
            self.string_payload(),
            self.string_size(),
            self.string_offset()
        )
    }
    fn string_path(&self) -> String {
        String::from_utf8_lossy(self.content()).to_string()
    }
    fn string_path_pair(&self) -> (String, String) {
        let path_len: usize = (self.payload() - (self.size() as i32)) as usize;
        let first: String = String::from_utf8_lossy(&self.content()[..path_len]).to_string();
        let second: String = self.content()[path_len..self.payload() as usize]
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join(" ");
        (first, second)
    }
    fn string_version(&self) -> String {
        if (self.version() & crate::message::SERVERMESSAGE) == crate::message::SERVERMESSAGE {
            format!(
                "{:X} tokens={}",
                self.version(),
                self.version() & crate::message::SERVERTOKENS
            )
        } else {
            format!("{:X}", self.version())
        }
    }
    /*
        fn string_ret(&self) -> String {
            format!("{}", self.ret())
        }
    */
    fn string_type(&self) -> String {
        match self.mtype() {
            OwQuery::NOP => "NOP".to_string(),
            OwQuery::READ => format!("READ {}", self.string_path()),
            OwQuery::WRITE => {
                let w = self.string_path_pair();
                format!("WRITE {} => {}", w.0, w.1)
            }
            OwQuery::DIR => format!("DIR {}", self.string_path()),
            OwQuery::SIZE => "SIZE".to_string(),
            OwQuery::PRESENT => "PRESENT".to_string(),
            OwQuery::DIRALL => format!("DIRALL {}", self.string_path()),
            OwQuery::GET => format!("GET {}", self.string_path()),
            OwQuery::DIRALLSLASH => format!("DIRALLSLASH {}", self.string_path()),
            OwQuery::GETSLASH => format!("GETSLASH {}", self.string_path()),
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

