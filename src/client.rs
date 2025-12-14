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

use std::io::{Read,Write} ;
use std::net::{TcpStream,TcpListener} ;
use std::time::Duration ;
use std::str ;
use ::std::thread ;

mod receive ;
use receive::OwMessageReceive ;

mod send ;
use send::OwMessageSend ;

pub use crate::error::{OwError,OwEResult};

pub mod parse_args ;

/// Type for server tokens to prevent owserver network loops
pub type Token = [u8;16] ;
mod token ;
use token::make_token ;

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

#[derive(Debug)]
struct Stream {
    stream: Option<TcpStream>,
}
impl Clone for Stream {
    fn clone( &self ) -> Self {
        Stream{ stream: None }
    }
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
    listener: Option<String> ,
    token: Token ,
    temperature: Temperature,
    pressure:    Pressure,
    format:      Format,
    size:        u32,
    offset:      u32,
    slash:       bool,
    hex:         bool,
    bare:        bool,
    prune:       bool,
    persistence: bool,
    stream:      Stream,
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
    #[allow(unused)]
    const OWNET_FLAG:  u32 = 0x00000100 ;

    #[allow(unused)]
    const UNCACHED:    u32 = 0x00000020 ;

    #[allow(unused)]
    const SAFEMODE:    u32 = 0x00000010 ;

    #[allow(unused)]
    const ALIAS:       u32 = 0x00000008 ;

    const PERSISTENCE: u32 = 0x00000004 ;

    #[allow(unused)]
    const BUS_RET:     u32 = 0x00000002 ;

    fn new() -> Self {
        let mut owc = OwClient {
            owserver: String::from("localhost:4304"),
            listener: None ,
            token: make_token() ,
            temperature: Temperature::DEFAULT,
            pressure: Pressure::DEFAULT,
            format: Format::DEFAULT,
            size: 0,
            offset: 0,
            slash: false,
            hex: false,
            bare: false,
            prune: false,
            persistence: false,
            stream: Stream{ stream:None },
            debug: 0,
            flags: 0,
        } ;
        owc.make_flags() ;
        owc
    }
        
    // make the owserver flag field based on configuration settings
    pub fn make_flags( &mut self ) {
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
    
    fn make_write( &self, text: &str, value: &[u8] ) -> OwEResult<OwMessageSend> {
        OwMessageSend::new( self.flags, OwMessageSend::WRITE, Some(text), Some(value) )
    }
    fn make_read( &self, text: &str ) -> OwEResult<OwMessageSend> {
        OwMessageSend::new( self.flags, OwMessageSend::READ, Some(text), None )
    }
    fn make_dir( &self, text: &str ) -> OwEResult<OwMessageSend> {
        OwMessageSend::new( self.flags, OwMessageSend::DIR, Some(text), None )
    }
    fn make_size( &self, text: &str ) -> OwEResult<OwMessageSend> {
        OwMessageSend::new( self.flags, OwMessageSend::SIZE, Some(text), None )
    }
    fn make_present( &self, text: &str ) -> OwEResult<OwMessageSend> {
        OwMessageSend::new( self.flags, OwMessageSend::PRESENT, Some(text), None )
    }
    fn make_dirall( &self, text: &str ) -> OwEResult<OwMessageSend> {
        OwMessageSend::new( self.flags, OwMessageSend::DIRALL, Some(text), None )
    }
    fn make_get( &self, text: &str ) -> OwEResult<OwMessageSend> {
        OwMessageSend::new( self.flags, OwMessageSend::GET, Some(text), None )
    }
    fn make_dirallslash( &self, text: &str ) -> OwEResult<OwMessageSend> {
        OwMessageSend::new( self.flags, OwMessageSend::DIRALLSLASH, Some(text), None )
    }
    fn make_getslash( &self, text: &str ) -> OwEResult<OwMessageSend> {
        OwMessageSend::new( self.flags, OwMessageSend::GETSLASH, Some(text), None )
    }
    
    fn send_get_single( &mut self, send: OwMessageSend ) -> OwEResult<OwMessageReceive> {
        self.send_packet( send ) ? ;       
        self.get_msg_single()
    }

    fn send_get_many( &mut self, send: OwMessageSend ) -> OwEResult<OwMessageReceive> {
        self.send_packet( send ) ? ;       
        self.get_msg_many()
    }

    fn get_msg_single( &mut self ) -> OwEResult<OwMessageReceive> {
        // Set timeout
        self.set_timeout() ? ;
        let stream = match self.stream.stream.as_mut() {
            Some(s) => s ,
            None => {
                return Err(OwError::General("No Tcp stream defined".to_string()));
            },
        } ;
        let rcv = OwMessageReceive::get_packet(stream,None) ? ;
        Ok(rcv)
    }
    
    fn set_timeout( &mut self ) -> OwEResult<()> {
        match self.stream.stream.as_mut() {
            Some(s) => {
                // Set timeout
                s.set_read_timeout( Some(Duration::from_secs(5))) ? ;
            },
            None => {
                return Err(OwError::General("No Tcp stream defined".to_string()));
            },
        }
        Ok(())
    }
    
    // Loop through getting packets until payload empty
    // for directories
    fn get_msg_many( &mut self ) -> OwEResult<OwMessageReceive> {
        // Set timeout
        self.set_timeout() ? ;
        
        let stream = match self.stream.stream.as_mut() {
            Some(s) => s ,
            None => {
                return Err(OwError::General("No Tcp stream defined".to_string()));
            },
        } ;
        let mut full_rcv = OwMessageReceive::get_packet(stream,None) ?;

        if full_rcv.payload == 0 {
            return Ok(full_rcv) ;
        }
        
        loop {
            // get more packets and add content to first one, adjusting payload size
            let mut rcv = OwMessageReceive::get_packet( stream,None ) ? ;
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
    
    fn connect( &mut self ) -> OwEResult<()> {
        let stream = TcpStream::connect( &self.owserver ) ? ;
        self.stream.stream = Some(stream) ;
        Ok(())
    }

    fn send_packet( &mut self, mut msg: OwMessageSend ) -> OwEResult<()> {
        // Write to network
        if self.debug > 1 {
            eprintln!("about to connect");
        }
        
        // Create or reuse a connection
        if self.persistence {
            match self.stream.stream.as_mut() {
                None => {
                    // need initial connection
                    self.connect() ? ;
                },
                Some(s) => {
                    // test existing connection
                    match s.write_all(&[]) {
                        Ok(()) => {},
                        Err(_) => {
                            // recreate
                            self.connect() ? ;
                        },
                    }
                },
            } ;
        } else {
            // No persistence, make new connection
            self.connect() ? ;
        }
        if let Some(stream) = &mut self.stream.stream {
            msg.send(stream) ? ;
        }
        Ok(())
    }

    fn get_value( &mut self, path: &str, f: fn(&OwClient, &str)->OwEResult<OwMessageSend>) -> OwEResult< Vec<u8>> {
        let msg = f( self, path ) ? ;
        let rcv = self.send_get_single( msg ) ? ;
        if rcv.payload > 0 {
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
    pub fn read( &mut self, path: &str ) -> OwEResult<Vec<u8>> {
        self.get_value( path, OwClient::make_read)
    }
    /// ### write
    /// write a value to a 1-wire file
    /// * path is the 1-wire address of the file
    /// * value is a `Vec<u8>` byte sequence to write 
    ///   * (e.g. /10.112233445566/temperature)
    /// * returns () or error
    pub fn write( &mut self, path: &str, value: &[u8] ) -> OwEResult<()> {
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
    pub fn dir( &mut self, path: &str ) -> OwEResult<Vec<String>> {
        let msg = self.make_dir( path ) ? ;
        let mut rcv = self.send_get_many( msg ) ? ;
        self.dirboth( &mut rcv.content )
    }

    /// ### present
    /// returns the existence of a 1-wire device
    /// * Rarely used function
    /// * path is the 1-wire address of the the device
    /// * returns bool or error
    pub fn present( &mut self, path: &str ) -> OwEResult<bool> {
        let msg = self.make_present( path ) ? ;
        let rcv = self.send_get_single( msg ) ? ;
        Ok(rcv.ret==0)
    }

    /// ### size
    /// returns the length of read response
    /// * Rarely used function
    /// * path is the 1-wire address of the the device property
    /// * returns `i32` or error
    pub fn size( &mut self, path: &str ) -> OwEResult<i32> {
        let msg = self.make_size( path ) ? ;
        let rcv = self.send_get_single( msg ) ? ;
        let ret = rcv.ret;
        if ret < 0 {
            Err(OwError::Output(format!("Return code from owserver is error {}",rcv.ret)))
        } else {
            Ok(ret)
        }
    }

    // Get last base of "filename" excluding blank or blank
    fn basename( path: &str ) -> String {
        let copy = path.to_string() ;
        for n in copy.split('/').rev() {
            if ! n.is_empty() {
                n.to_string() ;
                let m: Vec<&str> = n.split('.').collect();
                return m[0].to_string() ;
            }
        }
        "".to_string()
    }
    // dirboth prunes nulls and possibly the prunelist if --prune specified
    pub fn dirboth( &self, raw_dir: &mut Vec<u8>  ) -> OwEResult<Vec<String>> {
        raw_dir.retain( |&b| b!=0 ) ;
        let mut s : Vec<&str> = str::from_utf8( raw_dir )? .split(',').collect();
        if self.prune {
            let prune_list: Vec<&str> = vec![
                "address",
                "crc8",
                "family",
                "id",
                "locator",
                "r_address",
                "r_id",
                "r_locator",
                "type",
                "bus",
                ];
            s.retain( |&x| !prune_list.contains(&OwClient::basename(x).as_str()) ) ;
        }
        Ok(s.into_iter().map( String::from ).collect())
    }
    /// ### dirall
    /// returns the path directory listing
    /// * efficiently uses a single message
    /// * honors the _--dir_ command line option
    /// * honors the _--bare_ command line option
    /// * removes some stray null bytes erroneously added by original owserver to file names
    /// * returns `Vec<String>` or error
    pub fn dirall( &mut self, path: &str ) -> OwEResult<Vec<String>> {
        let mut d: Vec<u8> = match self.slash {
            true => self.get_value(path,OwClient::make_dirallslash),
            _ => self.get_value(path,OwClient::make_dirall),
        } ? ;
        self.dirboth( &mut d )
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
    pub fn get( &mut self, path: &str ) -> OwEResult<Vec<u8>> {
        match self.slash {
            true => self.get_value( path, OwClient::make_getslash),
            _ => self.get_value( path, OwClient::make_get),
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
            let s = str::from_utf8(&v) ? ;
            Ok(s.to_string())
        }
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
    
    /// ### listen
    /// start an owserver (that forwards packets with some processing)
    /// * Uses threads
    pub fn listen( &self ) -> OwEResult<()> {
        if let Some(address) = &self.listener {
            let listen_stream = TcpListener::bind(address) ? ;
            for stream in listen_stream.incoming() {
                match stream {
                    Ok(stream) => {
                        let instance = OwServerInstance::new( self.clone(), stream ) ;
                        thread::spawn( move || {
                            instance.handle_query() ;
                        });
                    },
                    Err(e) => {
                        eprintln!("Bad server query connection. {}",e) ;
                    },
                }
            }
        } else {
            eprintln!("No --port address given.");
            return Err(OwError::General("No address given to listen on (--port)".to_string() ) ) ;
        }
        Ok(())
    }
}

struct OwServerInstance {
    client: crate::OwClient,
    stream: TcpStream,
}
impl OwServerInstance {
    fn new(client: crate::OwClient, stream: TcpStream) -> OwServerInstance {
        OwServerInstance {
            client,
            stream,
        }
    }
    fn handle_query( &self ) {
        // Set timeout
        match self.stream.set_read_timeout( Some(Duration::from_secs(5))) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Cannot set timeout to server query {}",e);
                return ;
            },
        }
        
        // get header
        static HSIZE: usize = 24 ;
        let mut buffer: [u8; HSIZE ] = [ 0 ; HSIZE ];

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
    #[test]
    fn bn_test() {
        let xs = vec!(
        ("basename", "basename".to_string()),
        ("basename.0","basename".to_string()),
        ("basename.1/","basename".to_string()),
        ("/dir/basename","basename".to_string()),
        ("dir/basename/","basename".to_string()),
        ("/root/dir/basename.2.3","basename".to_string()),
        );
        for x in xs {
            let s = OwClient::basename(x.0);
            assert_eq!(s,x.1);
        }
    }
}

