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

use std::ffi ;
use std::fmt ;
use std::io::{Read,Write} ;
use std::net::TcpStream ;
use std::time::Duration ;
use std::str ;

pub mod parse_args ;

/// ### new
/// Creates a new OwClient
/// * configure flags and server address before using
/// * use public OwClient methods to manage owserver communication
pub fn new() -> OwClient {
    OwClient::new()
}

#[derive(Debug,PartialEq,Clone)]
/// ### Temperature scale
/// sent to owserver in the flag parameter since only the original 1-wire 
/// program in the chain knows the type of value being sought
pub enum Temperature {
    CELSIUS,
    FARENHEIT,
    KELVIN,
    RANKINE,
    DEFAULT,
}

#[derive(Debug,PartialEq,Clone)]
/// ### Pressure scale
/// sent to owserver in the flag parameter since only the original 1-wire 
/// program in the chain knows the type of value being sought
pub enum Pressure {
    MMHG,
    INHG,
    PA,
    PSI,
    ATM,
    MBAR,
    DEFAULT,
}

#[derive(Debug,PartialEq,Clone)]
/// ### 1-wire ID format
/// has components:
///  F family code (1 byte)
///  I unique serial number (6 bytes)
///  C checksum (1-byte)
pub enum Format {
    FI,
    FdI,
    FIC,
    FIdC,
    FdIC,
    FdIdC,
    DEFAULT,
}

#[derive(Debug,Clone)]
/// ### OwClient
/// structure that manages the connection to owserver
/// * Stores configuration settings
/// * has public fuction for each message type to owserver
///   * read
///   * write
///   * dir
/// * convenience functions for printing results
/// ### Creation
/// ```
/// let mut owserver = owrust::new() ;
/// ```
pub struct OwClient {
    owserver:    String,
    temperature: Temperature,
    pressure:    Pressure,
    format:      Format,
    size:        u32,
    offset:      u32,
    slash:       bool,
    hex:         bool,
    bare:        bool,
    persistence: bool,
    debug:       u32,
    flags:       u32,
}

impl OwClient {
    // Flag for types
    // -- Format flags (mutually exclusive)
    const FORMAT_F_I:  u32 = 0x00000000 ;
    const FORMAT_FI:   u32 = 0x01000000 ;
    const FORMAT_F_I_C:u32 = 0x02000000 ;
    const FORMAT_F_IC: u32 = 0x03000000 ;
    const FORMAT_FI_C: u32 = 0x04000000 ;
    const FORMAT_FIC:  u32 = 0x05000000 ;
    // -- Temperature flags (mutually exclusive)
    const TEMPERATURE_C: u32 = 0x00000000 ;
    const TEMPERATURE_F: u32 = 0x00010000 ;
    const TEMPERATURE_K: u32 = 0x00020000 ;
    const TEMPERATURE_R: u32 = 0x00030000 ;
    // -- Pressure flags (mutually exclusive)
    const PRESSURE_MBAR: u32 = 0x00000000 ;
    const PRESSURE_ATM:  u32 = 0x00040000 ;
    const PRESSURE_MMHG: u32 = 0x00080000 ;
    const PRESSURE_INHG: u32 = 0x000C0000 ;
    const PRESSURE_PSI:  u32 = 0x00100000 ;
    const PRESSURE_PA:   u32 = 0x00140000 ;
    // -- Other independent flags
    const OWNET_FLAG:  u32 = 0x00000100 ;
    const UNCACHED:    u32 = 0x00000020 ;
    const SAFEMODE:    u32 = 0x00000010 ;
    const ALIAS:       u32 = 0x00000008 ;
    const PERSISTENCE: u32 = 0x00000004 ;
    const BUS_RET:     u32 = 0x00000002 ;

    fn new() -> Self {
        let mut owc = OwClient {
            owserver: String::from("localhost:4304"),
            temperature: Temperature::DEFAULT,
            pressure: Pressure::DEFAULT,
            format: Format::DEFAULT,
            size: 0,
            offset: 0,
            slash: false,
            hex: false,
            bare: false,
            persistence: false,
            debug: 0,
            flags: 0,
        } ;
        owc.make_flags() ;
        owc
    }
        
    // make the owserver flag field based on configuration settings
    fn make_flags( &mut self ) {
        let mut flags = 0 ;
        if ! self.bare {
            flags |= OwClient::BUS_RET ;
        }
        if self.persistence {
            flags |= OwClient::PERSISTENCE ;
        }
        flags |= match self.temperature {
            Temperature::CELSIUS   => OwClient::TEMPERATURE_C,
            Temperature::FARENHEIT => OwClient::TEMPERATURE_F,
            Temperature::KELVIN    => OwClient::TEMPERATURE_K,
            Temperature::RANKINE   => OwClient::TEMPERATURE_R,
            Temperature::DEFAULT   => OwClient::TEMPERATURE_C,
        } ;
        
        flags |= match self.pressure {
            Pressure::MBAR => OwClient::PRESSURE_MBAR,
            Pressure::MMHG => OwClient::PRESSURE_MMHG,
            Pressure::INHG => OwClient::PRESSURE_INHG,
            Pressure::ATM  => OwClient::PRESSURE_ATM ,
            Pressure::PA   => OwClient::PRESSURE_PA,
            Pressure::PSI  => OwClient::PRESSURE_PSI,
            Pressure::DEFAULT => OwClient::PRESSURE_MBAR,
        };
        
        flags |= match self.format {
            Format::FI => OwClient::FORMAT_FI,
            Format::FdI => OwClient::FORMAT_F_I,
            Format::FIC => OwClient::FORMAT_FIC,
            Format::FIdC => OwClient::FORMAT_FI_C,
            Format::FdIC=> OwClient::FORMAT_F_IC,
            Format::FdIdC => OwClient::FORMAT_F_I_C,
            Format::DEFAULT => OwClient::FORMAT_F_I,
        } ;
        self.flags = flags
    }
    
    fn param1( &self, text: &str, mtype: u32 ) -> OwEResult<OwMessageSend> {
        let mut msg = OwMessageSend::new(self.flags) ;
        if self.debug > 1 {
            eprintln!( "Type {} with text {} being prepared for sending", OwMessageSend::message_name(mtype), text ) ;
        }
        msg.mtype = mtype ;
        msg.add_path( text ) ? ;
		Ok(msg)
    }
    
    fn make_write( &self, text: &str, value: &[u8] ) -> OwEResult<OwMessageSend> {
        let mut msg = OwMessageSend::new(self.flags) ;
        msg.mtype = OwMessageSend::WRITE ;
        msg.add_path( text ) ? ;
        msg.add_data( value ) ;
        Ok(msg)
    }

    fn make_read( &self, text: &str ) -> OwEResult<OwMessageSend> {
        self.param1( text, OwMessageSend::READ )
    }
    fn make_dir( &self, text: &str ) -> OwEResult<OwMessageSend> {
        self.param1( text, OwMessageSend::DIR )
    }
    fn make_size( &self, text: &str ) -> OwEResult<OwMessageSend> {
        self.param1( text, OwMessageSend::SIZE )
    }
    fn make_present( &self, text: &str ) -> OwEResult<OwMessageSend> {
        self.param1( text, OwMessageSend::PRESENT )
    }
    fn make_dirall( &self, text: &str ) -> OwEResult<OwMessageSend> {
        self.param1( text, OwMessageSend::DIRALL )
    }
    fn make_get( &self, text: &str ) -> OwEResult<OwMessageSend> {
        self.param1( text, OwMessageSend::GET )
    }
    fn make_dirallslash( &self, text: &str ) -> OwEResult<OwMessageSend> {
        self.param1( text, OwMessageSend::DIRALLSLASH )
    }
    fn make_getslash( &self, text: &str ) -> OwEResult<OwMessageSend> {
        self.param1( text, OwMessageSend::GETSLASH )
    }
    
    fn send_get_single( &self, send: OwMessageSend ) -> OwEResult<OwMessageReceive> {
        let stream = self.send_packet( send ) ? ;       
        self.get_msg_single( stream )
    }

    fn send_get_many( &self, send: OwMessageSend ) -> OwEResult<OwMessageReceive> {
        let stream = self.send_packet( send ) ? ;       
        self.get_msg_many( stream )
    }

    fn get_msg_single( &self, stream: TcpStream ) -> OwEResult<OwMessageReceive> {
        // Set timeout
        self.set_timeout( &stream ) ? ;
        self.get_packet( &stream )
    }
    
    fn set_timeout( &self, stream: &TcpStream ) -> OwEResult<()> {
        // Set timeout
        stream.set_read_timeout( Some(Duration::from_secs(5))) ? ;
        Ok(())
    }
    
    // Loop through getting packets until payload empty
    // for directories
    fn get_msg_many( &self, stream: TcpStream ) -> OwEResult<OwMessageReceive> {
        // Set timeout
        self.set_timeout( &stream ) ? ;
        
        let mut full_rcv = self.get_packet( &stream ) ? ;

        if full_rcv.payload == 0 {
            return Ok(full_rcv) ;
        }
        
        loop {
            // get more packets and add content to first one, adjusting payload size
            let mut rcv = self.get_packet( &stream ) ? ;
            if self.debug > 0 {
                eprintln!("Another packet");
            }
            if rcv.payload == 0 {
                return Ok(full_rcv) ;
            }
            full_rcv.content[(full_rcv.payload-1) as usize] = b',' ; // trailing null -> comma
            full_rcv.content.append( &mut rcv.content ) ; // add this packet's info
            full_rcv.payload += rcv.payload ;
        }
    }
    
    fn send_packet( &self, send: OwMessageSend ) -> OwEResult<TcpStream> {
        let mut msg:Vec<u8> = 
            [ send.version, send.payload, send.mtype, send.flags, send.size, send.offset ]
            .iter()
            .flat_map( |&u| u.to_be_bytes() )
            .collect() ;
        if send.payload > 0 {
            msg.extend_from_slice(&send.content) ;
        }

        // Write to network
        if self.debug > 1 {
            eprintln!("about to connect");
        }
        let mut stream = TcpStream::connect( &self.owserver ) ? ;
		stream.write_all( &msg ) ? ;
        Ok(stream)
    }

    fn get_packet( &self, mut stream: &TcpStream ) -> OwEResult<OwMessageReceive> {
        // get a single non-ping message.
        // May need multiple for directories
        static HSIZE: usize = 24 ;
        let mut buffer: [u8; HSIZE ] = [ 0 ; HSIZE ];
        
        loop {
            stream.read_exact( &mut buffer ) ? ;
            let mut rcv = OwMessageReceive::new(buffer);
            
            if self.debug > 0 {
                rcv.tell() ;
            }
            
            if (rcv.payload as i32) < 0 {
                // ping
                if self.debug > 1 {
                    eprintln!("Ping");
                }
                continue ;
            }
            if rcv.payload > 0 {
				// create Vec with just the right size (based on payload)
                rcv.content = Vec::with_capacity(rcv.payload as usize) ;
                rcv.content.resize(rcv.payload as usize,0);
                
                stream.read_exact(&mut rcv.content ) ? ;
            }
            return Ok(rcv) ;
        }
    }
    
    fn get_value( &self, path: &str, f: fn(&OwClient, &str)->OwEResult<OwMessageSend>) -> OwEResult< Vec<u8>> {
        let msg = f( self, path ) ? ;
        let rcv = self.send_get_single( msg ) ? ;
        if rcv.payload > 0 {
            let v: Vec<u8> = rcv.content ;
            return Ok( v ) ;
        }
        Ok(Vec::new())
    }
    fn get_bare_value( &self, path: &str, f: fn(&OwClient, &str)->OwEResult<OwMessageSend>) -> OwEResult< Vec<u8>> {
        let msg = f( self, path ) ? ;
        let mut rcv = self.send_get_single( msg ) ? ;
        if rcv.payload > 0 {
			if self.bare {
				rcv.bare_filter() ? ;
			}
            let v: Vec<u8> = rcv.content ;
            return Ok( v ) ;
        }
        Ok(Vec::new())
    }
    
    /// ### read
    /// reads a value from a 1-wire file
    /// * path is the 1-wire address of the file 
    ///   * (e.g. /10.112233445566/temperature)
    /// * returns a `Vec<u8>` or error
    /// * result can be displayed with **show_result**
    pub fn read( &self, path: &str ) -> OwEResult<Vec<u8>> {
        self.get_value( path, OwClient::make_read)
    }
    /// ### write
    /// write a value to a 1-wire file
    /// * path is the 1-wire address of the file
    /// * value is a `Vec<u8>` byte sequence to write 
    ///   * (e.g. /10.112233445566/temperature)
    /// * returns () or error
    pub fn write( &self, path: &str, value: &[u8] ) -> OwEResult<()> {
        let msg = OwClient::make_write( self, path, value ) ? ;
        let rcv = self.send_get_single( msg ) ? ;
        if rcv.ret == 0 {
            Ok( () )
        } else {
            Err(OwError::Output(
                format!("Return code from owserver is error {}",rcv.ret)
                ))
        }
    }

    /// ### dirall
    /// returns the path directory listing
    /// * uses a separate message for each entry
    /// * honors the _--dir_ command line option
    /// * honors the _--bare_ command line option
    /// * returns `Vec<u8>` or error
    /// * result can be displayed with **show_text**
    pub fn dir( &self, path: &str ) -> OwEResult<Vec<u8>> {
        let msg = self.make_dir( path ) ? ;
        let mut rcv = self.send_get_many( msg ) ? ;
        if rcv.payload > 0 {
			if self.bare {
				rcv.bare_filter() ? ;
			}
            let v: Vec<u8> = rcv.content ;
            return Ok( v ) ;
        }
        Ok(Vec::new())
    }

    /// ### present
    /// returns the existence of a 1-wire device
    /// * Rarely used function
    /// * path is the 1-wire address of the the device
    /// * returns bool or error
    pub fn present( &self, path: &str ) -> OwEResult<bool> {
        let msg = self.make_present( path ) ? ;
        let rcv = self.send_get_single( msg ) ? ;
        Ok(rcv.ret==0)
    }

    /// ### size
    /// returns the length of read response
    /// * Rarely used function
    /// * path is the 1-wire address of the the device property
    /// * returns `i32` or error
    pub fn size( &self, path: &str ) -> OwEResult<i32> {
        let msg = self.make_size( path ) ? ;
        let rcv = self.send_get_single( msg ) ? ;
        let ret = rcv.ret;
        if ret < 0 {
            Err(OwError::Output(format!("Return code from owserver is error {}",rcv.ret)))
        } else {
            Ok(ret)
        }
    }
    /// ### dirall
    /// returns the path directory listing
    /// * efficiently uses a single message
    /// * honors the _--dir_ command line option
    /// * honors the _--bare_ command line option
    /// * returns `Vec<u8>` or error
    /// * result can be displayed with **show_text**
    pub fn dirall( &self, path: &str ) -> OwEResult<Vec<u8>> {
        match self.slash {
            true => self.get_bare_value(path,OwClient::make_dirallslash),
            _ => self.get_bare_value(path,OwClient::make_dirall),
        }
    }
    /// ### get
    /// combines **dir** and **read** functionality
    /// * _read_ if path is a file
    /// * _dir_ if path is a directory
    /// * honors the _--dir_ command line option
    /// * honors the _--hex_ command line option
    /// * honors the _--bare_ command line option
    /// * returns `Vec<u8>` or error
    /// * result can be displayed with **show_result**
    pub fn get( &self, path: &str ) -> OwEResult<Vec<u8>> {
        match self.slash {
            true => self.get_bare_value( path, OwClient::make_getslash),
            _ => self.get_bare_value( path, OwClient::make_get),
        }
    }

    /// ### show_result 
    /// prints the result of an owserver query
    /// * honors the hex setting
    /// * good for **read** and **get**
    pub fn show_result( &self, v: Vec<u8> ) -> OwEResult<String> {
        if self.hex {
            Ok(v.iter().map(|b| format!("{:02X}",b)).collect::<Vec<String>>().join(" "))
        } else {
            self.show_text(v)
        }
    }

    /// ### show_test 
    /// prints the result of an owserver query
    /// * ignores the hex setting
    /// * good for **dir**
    pub fn show_text( &self, v: Vec<u8> ) -> OwEResult<String> {
        let s = str::from_utf8(&v) ? ;
        Ok(s.to_string())
    }
    
    /// ### input_to_write
    /// take the value string for **owwrite**
    /// * if not --hex, use str as bytes directly, else
    /// * read as a hex string
    pub fn input_to_write( &self, s: &str ) -> OwEResult<Vec<u8>> {
    if ! self.hex {
        return Ok(s.as_bytes().to_vec()) ;
    }
    // hex
    if ! s.len().is_multiple_of(2) {
        return Err(OwError::Numeric("Hex string should be an even length".into())) ;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            match u8::from_str_radix(&s[i..i+2], 16) {
                Ok(byte) => Ok(byte),
                Err(e) => Err(OwError::Numeric(
                    format!("Bad hex characters {}",e)
                    )
                ),
            }}
        )
        .collect()
    }
}


struct OwMessageSend {
    version: u32,
    payload: u32,
    mtype:   u32,
    flags:   u32,
    size:    u32,
    offset:  u32,
    content: Vec<u8>,
}

impl OwMessageSend {
    // Default owserver version (to owserver)
    const SENDVERSION: u32 = 0 ;

    // Maximum make_size of returned data (pretty arbitrary but matches C implementation)
    const DEFAULTSIZE: u32 = 65536 ;

    // Message types
    const NOP:         u32 = 1 ;
    const READ:        u32 = 2 ;
    const WRITE:       u32 = 3 ;
    const DIR:         u32 = 4 ;
    const SIZE:        u32 = 5 ;
    const PRESENT:     u32 = 6 ;
    const DIRALL:      u32 = 7 ;
    const GET:         u32 = 8 ;
    const DIRALLSLASH: u32 = 9 ;
    const GETSLASH:    u32 = 10 ;

    fn new(flag: u32)-> OwMessageSend {
        OwMessageSend {
            version: OwMessageSend::SENDVERSION,
            payload: 0,
            mtype:   OwMessageSend::NOP,
            flags:   flag,
            size:    OwMessageSend::DEFAULTSIZE,
            offset:  0,
            content: [].to_vec(),
        }
    }

    fn message_name( mtype: u32 ) -> &'static str {
        match mtype {
            OwMessageSend::NOP => "NOP",
            OwMessageSend::READ => "READ",
            OwMessageSend::WRITE => "WRITE",
            OwMessageSend::DIR => "DIR",
            OwMessageSend::SIZE => "SIZE",
            OwMessageSend::PRESENT => "PRESENT",
            OwMessageSend::DIRALL => "DIRALL",
            OwMessageSend::GET => "GET",
            OwMessageSend::DIRALLSLASH => "DIRALLSLASH",
            OwMessageSend::GETSLASH => "GETSLASH",
            _ => "UNKNOWN",
        }
    }

    fn add_path( &mut self, path: &str ) -> OwEResult<()> {
        // Add nul-terminated path (and includes null in payload size)
        let s = ffi::CString::new(path) ? ;
        self.content = s.as_bytes().to_vec() ;
        self.payload = self.content.len() as u32 ;
        Ok(())
    }
    
    fn add_data( &mut self, data: &[u8] ) {
        // Add data after path without nul
        self.content.extend_from_slice(data) ;
        self.size = data.len() as u32 ;
        self.payload += self.size ;
    }
}

struct OwMessageReceive {
    version: u32,
    payload: u32,
    ret:     i32,
    flags:   u32,
    size:    u32,
    offset:  u32,
    content: Vec<u8>,
}
impl OwMessageReceive {
    const HSIZE: usize = 24 ;
    fn new( buffer: [u8;OwMessageReceive::HSIZE] ) -> Self {
        OwMessageReceive {          
            version: u32::from_be_bytes(buffer[ 0.. 4].try_into().unwrap()),
            payload: u32::from_be_bytes(buffer[ 4.. 8].try_into().unwrap()),
            ret:     u32::from_be_bytes(buffer[ 8..12].try_into().unwrap()) as i32,
            flags:   u32::from_be_bytes(buffer[12..16].try_into().unwrap()),
            size:    u32::from_be_bytes(buffer[16..20].try_into().unwrap()),
            offset:  u32::from_be_bytes(buffer[20..24].try_into().unwrap()),
            content: [].to_vec(),
        }
    }
    fn tell( &self) {
        eprintln!( "ver {:X}, pay {}, ret {}, flg {:X}, siz {}, off {}",self.version,self.payload,self.ret,self.flags,self.size,self.offset);
    }

    fn bare_filter( &mut self ) -> OwEResult<()> {
		let s = String::from_utf8(self.content.clone()) ? ;
		self.content = s.split(',')
			.filter( |s| ! OwMessageReceive::is_bad_bare(s) )
			.collect::<Vec<&str>>()
			.join(",")
			.as_bytes()
			.to_vec();
		self.payload = self.content.len() as u32 ;
		Ok(())
    }

    // filter devices properties that are less interesting
    fn is_bad_bare(path: &str) -> bool {
        let bare_bad: Vec<&str> = vec![
            "address",
            "crc8",
            "family",
            "id",
            "locator",
            "r_address",
            "r_id",
            "r_locator",
            "type",
            ];
        match path.split('/').rev().find(|s| !s.is_empty()) {
            Some(s) => bare_bad.contains(&s),
            _ => false,
        }
    }
}

/// ### OwEResult
///
/// type alias for Result<_,OwError> to reduce boilerplate
/// `OwEResult<String>` is equivalent to `Result<String,OwError>`
pub type OwEResult<T> = std::result::Result<T,OwError> ;

#[derive(Debug)]
/// ### OwError 
/// the **owrust**-specific error type
///
/// details field is a String with error details
pub enum OwError {
	General(String),
	Input(String),
	Output(String),
    Io(std::io::Error),
    Args(pico_args::Error),
    Numeric(String),
    Text(String),
}

impl fmt::Display for OwError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			OwError::General(e) => write!(f,"An error: {}",e),
			OwError::Input(e) => write!(f,"Input error: {}",e),
			OwError::Output(e) => write!(f,"Output error: {}",e),
			OwError::Io(e) => write!(f,"IO error: {}",e),
			OwError::Args(e) => write!(f,"Args error: {}",e),
			OwError::Text(e) => write!(f,"Text conversion error: {}",e),
			OwError::Numeric(e) => write!(f,"Non-numeric characters: {}",e),
		}
    }
}
impl std::error::Error for OwError {
    fn source( &self ) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			OwError::Io(e) => Some(e),
			OwError::Args(e) => Some(e),
			_ => None,
		}
    }
}

impl From<std::io::Error> for OwError {
	fn from(e: std::io::Error) -> Self {
		OwError::Io(e)
	}
}
impl From<pico_args::Error> for OwError {
	fn from(e: pico_args::Error) -> Self {
		OwError::Args(e)
	}
}
impl From<std::str::Utf8Error> for OwError {
	fn from(_e: std::str::Utf8Error) -> Self {
		OwError::Text("Utf8 Error".into())
	}
}
impl From<std::string::FromUtf8Error> for OwError {
	fn from(_e: std::string::FromUtf8Error) -> Self {
		OwError::Text("FromUTF8Error".into())
	}
}
impl From<std::ffi::NulError> for OwError {
	fn from(_e: std::ffi::NulError) -> Self {
		OwError::Text("Nul Error".into())
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_client() {
        let owc = OwClient::new();
        assert_eq!(owc.temperature, Temperature::DEFAULT);
        assert_eq!(owc.pressure, Pressure::DEFAULT);
        assert_eq!(owc.format, Format::DEFAULT);
    }
    
    #[test]
    fn printable_test() {
        let mut owc = OwClient::new();
        // Regular
        owc.hex = false ;
        let v :Vec<u8> = vec!(72,101,108,108,111);
        let x = owc.show_result(v).unwrap() ;
        assert_eq!(x,"Hello");

        // Hex
        owc.hex = true ;
        let v :Vec<u8> = vec!(72,101,108,108,111);
        let x = owc.show_result(v).unwrap() ;
        assert_eq!(x,"48 65 6C 6C 6F");
    }
}
