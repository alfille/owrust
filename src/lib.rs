// owrust project
// https://github.com/alfille/owrust
//
// This is a rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

//! lib.rs is the library code that actually performs the owserver protocol.
//! Supported operations are read, write, dir, present and size, with some variations

//! the main struct is OwClient which holds all the configuration information
//! typically is is populated by the command line or configuration files

use std::ffi ;
use std::io::{Read,Write} ;
use std::net::TcpStream ;
use std::time::Duration ;
use std::str ;

pub mod parse_args ;

pub fn new() -> OwClient {
	OwClient::new()
}

#[derive(Debug,PartialEq)]
/// Temperature scale
/// sent to owserver in the flag parameter since only the original 1-wire 
///  program in the chain knows the type of value being sought
/// default is actually celsius
pub enum Temperature {
	CELSIUS,
	FARENHEIT,
	KELVIN,
	RANKINE,
	DEFAULT,
}

#[derive(Debug,PartialEq)]
/// Pressure scale
/// sent to owserver in the flag parameter since only the original 1-wire 
///  program in the chain knows the type of value being sought
pub enum Pressure {
	MMHG,
	INHG,
	PA,
	PSI,
	ATM,
	MBAR,
	DEFAULT,
}

#[derive(Debug,PartialEq)]
/// 1-wire ID format
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
pub struct OwClient {
	owserver:    String,
	temperature: Temperature,
	pressure:    Pressure,
	format:      Format,
	size:		 u32,
	offset:      u32,
	slash:       bool,
	hex:         bool,
	debug:	     u32,
	flag:        u32,
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
			debug: 0,
			flag:   0,
		} ;
		owc.make_flag() ;
		owc
	}
	
	/// set_temperature
	/// sets the temperature scale in the flags that is used for queries
	/// typically in the configuration step
	pub fn set_temperature( &mut self, temp: Temperature ) {
		self.temperature = temp ;
		self.make_flag() ;
	}
	pub fn get_temperature( &self ) -> &str {
		match self.temperature {
			Temperature::CELSIUS => "Celsius",
			Temperature::FARENHEIT => "Farenheit",
			Temperature::KELVIN => "Kelvin",
			Temperature::RANKINE => "Rankine",
			Temperature::DEFAULT => "Default",
		}
	}
	
	/// set_pressure
	/// sets the pressure scale in the flags that is used for queries
	/// typically in the configuration step
	pub fn set_pressure( &mut self, pres: Pressure ) {
		self.pressure = pres ;
		self.make_flag() ;
	}
	
	/// set_format
	/// sets the 1-wire device unique address format is used for display
	/// typically in the configuration step
	pub fn set_format( &mut self, dev: Format ) {
		self.format = dev ;
		self.make_flag() ;
	}
	
	/// set_server
	/// sets the newtwork address of the owserver being used
	/// should be in IP:port format
	/// example "127.0.0.0:4304"
	pub fn set_server( &mut self, srv: String ) {
		self.owserver = srv.clone() ;
	}	
	
	fn make_flag( &mut self ) {
		self.flag = OwClient::BUS_RET ;
		self.flag |= match self.temperature {
			Temperature::CELSIUS   => OwClient::TEMPERATURE_C,
			Temperature::FARENHEIT => OwClient::TEMPERATURE_F,
			Temperature::KELVIN    => OwClient::TEMPERATURE_K,
			Temperature::RANKINE   => OwClient::TEMPERATURE_R,
			Temperature::DEFAULT   => OwClient::TEMPERATURE_C,
		} ;
		
		self.flag |= match self.pressure {
			Pressure::MBAR => OwClient::PRESSURE_MBAR,
			Pressure::MMHG => OwClient::PRESSURE_MMHG,
			Pressure::INHG => OwClient::PRESSURE_INHG,
			Pressure::ATM  => OwClient::PRESSURE_ATM ,
			Pressure::PA   => OwClient::PRESSURE_PA,
			Pressure::PSI  => OwClient::PRESSURE_PSI,
			Pressure::DEFAULT => OwClient::PRESSURE_MBAR,
		};
		
		self.flag |= match self.format {
			Format::FI => OwClient::FORMAT_FI,
			Format::FdI => OwClient::FORMAT_F_I,
			Format::FIC => OwClient::FORMAT_FIC,
			Format::FIdC => OwClient::FORMAT_FI_C,
			Format::FdIC=> OwClient::FORMAT_F_IC,
			Format::FdIdC => OwClient::FORMAT_F_I_C,
			Format::DEFAULT => OwClient::FORMAT_F_I,
		} ;
		if self.debug > 1 {
			eprintln!("Flag now {:X}",self.flag) ;
		}
	}

	fn new_nop(&self)-> OwMessageSend {
		OwMessageSend {
			version: OwMessageSend::SENDVERSION,
			payload: 0,
			mtype:   OwMessageSend::NOP,
			flags:   self.flag,
			size:    OwMessageSend::DEFAULTSIZE,
			offset:  0,
			content: [].to_vec(),
		}
	}
	
	fn param1( &self, text: &str, mtype: u32 ) -> Result<OwMessageSend,OwError> {
		let mut msg = self.new_nop() ;
		if self.debug > 1 {
			eprintln!( "Type {} with text {} being prepared for sending", OwMessageSend::message_name(mtype), text ) ;
		}
		msg.mtype = mtype ;
		if msg.add_path( text ) {
			return Ok(msg);
		} else {
			eprintln!("Could not add path to sending message");
			return Err(OwError::TextError);
		}
	}
	
	fn make_write( &self, text: &str, value: &str ) -> Result<OwMessageSend,OwError> {
		let mut msg = self.new_nop() ;
		msg.mtype = OwMessageSend::WRITE ;
		if msg.add_path( text ) && msg.add_data( value ) {
			Ok(msg)
		} else {
			eprintln!("Could not add value to sending message");
			return Err(OwError::TextError);
		}
	}

	fn make_read( &self, text: &str ) -> Result<OwMessageSend,OwError> {
		self.param1( text, OwMessageSend::READ )
	}
	fn make_dir( &self, text: &str ) -> Result<OwMessageSend,OwError> {
		self.param1( text, OwMessageSend::DIR )
	}
	fn make_size( &self, text: &str ) -> Result<OwMessageSend,OwError> {
		self.param1( text, OwMessageSend::SIZE )
	}
	fn make_present( &self, text: &str ) -> Result<OwMessageSend,OwError> {
		self.param1( text, OwMessageSend::PRESENT )
	}
	fn make_dirall( &self, text: &str ) -> Result<OwMessageSend,OwError> {
		self.param1( text, OwMessageSend::DIRALL )
	}
	fn make_get( &self, text: &str ) -> Result<OwMessageSend,OwError> {
		self.param1( text, OwMessageSend::GET )
	}
	fn make_dirallslash( &self, text: &str ) -> Result<OwMessageSend,OwError> {
		self.param1( text, OwMessageSend::DIRALLSLASH )
	}
	fn make_getslash( &self, text: &str ) -> Result<OwMessageSend,OwError> {
		self.param1( text, OwMessageSend::GETSLASH )
	}
	
	fn send_get_single( &self, send: OwMessageSend ) -> Result<OwMessageReceive,OwError> {
		let stream = self.send_packet( send ) ? ;		
		self.get_msg_single( stream )
	}

	fn send_get_many( &self, send: OwMessageSend ) -> Result<OwMessageReceive,OwError> {
		let stream = self.send_packet( send ) ? ;		
		self.get_msg_many( stream )
	}

	fn get_msg_single( &self, stream: TcpStream ) -> Result<OwMessageReceive,OwError> {
		// Set timeout
		self.set_timeout( &stream ) ? ;
		self.get_packet( &stream )
	}
	
	fn set_timeout( &self, stream: &TcpStream ) -> Result<(),OwError> {
		// Set timeout
		match stream.set_read_timeout( Some(Duration::from_secs(5))) {
			Ok(_s)=>Ok(()),
			Err(e) => {
				eprintln!("Trouble setting timeout: {:?}",e);
				return Err(OwError::NetworkError);
			},
		}
	}
	
	// Loop through getting packets until payload empty
	// for directories
	fn get_msg_many( &self, stream: TcpStream ) -> Result<OwMessageReceive,OwError> {
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
	
	fn send_packet( &self, send: OwMessageSend ) -> Result<TcpStream,OwError> {
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
		let mut stream = match TcpStream::connect( &self.owserver ) {
			Ok(s)=> s,
			Err(e) => {
				eprintln!("Trouble connecting to owserver: {:?}",e) ;
				return Err(OwError::NetworkError) ;
			},
		};
			
		match stream.write_all( &msg ) {
			Ok(_s)=> (),
			Err(e) => {
				eprintln!("Trouble sending to owserver: {:?}",e) ;
				return Err(OwError::NetworkError) ;
			},
		} ;
		
		Ok(stream)
	}

	fn get_packet( &self, mut stream: &TcpStream ) -> Result<OwMessageReceive,OwError> {
		// get a single non-ping message.
		// May need multiple for directories
		static HSIZE: usize = 24 ;
		let mut buffer: [u8; HSIZE ] = [ 0 ; HSIZE ];
		
		loop {
			match stream.read_exact( &mut buffer ) {
				Ok(_s)=>(),
				Err(e) => {
					eprintln!("Trouble receiving header: {:?}",e);
					return Err(OwError::NetworkError);
				},
			};
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
				rcv.content = Vec::with_capacity(rcv.payload as usize) ;
				rcv.content.resize(rcv.payload as usize,0);
				
				match stream.read_exact(&mut rcv.content ) {
					Ok(_s)=>(),
					Err(e) => {
						eprintln!("Trouble receiving payload: {:?}",e);
						return Err(OwError::NetworkError);
					},
				} ;
			}
			return Ok(rcv) ;
		}
	}
	
	fn get_value( &self, path: &str, f: fn(&OwClient, &str)->Result<OwMessageSend,OwError>) -> Result< Vec<u8>, OwError> {
		let msg = f( self, path ) ? ;
		let rcv = self.send_get_single( msg ) ? ;
		if rcv.payload > 0 {
			let v: Vec<u8> = rcv.content ;
			return Ok( v ) ;
		}
		Ok(Vec::new())
	}
	
	pub fn read( &self, path: &str ) -> Result<Vec<u8>,OwError> {
		self.get_value( path, OwClient::make_read)
	}
	pub fn write( &self, path: &str, value: &str ) -> Result<(),OwError> {
		let msg = OwClient::make_write( self, path, value ) ? ;
		let rcv = self.send_get_single( msg ) ? ;
		if rcv.ret == 0 {
			return Ok( () ) ;
		}
		eprintln!("Return code from owserver is error {}",rcv.ret);
		return Err(OwError::OtherError);
	}
	pub fn dir( &self, path: &str ) -> Result<Vec<u8>,OwError> {
		let msg = self.make_dir( path ) ? ;
		let rcv = self.send_get_many( msg ) ? ;
		if rcv.payload > 0 {
			let v: Vec<u8> = rcv.content ;
			return Ok( v ) ;
		}
		Ok(Vec::new())
	}
	pub fn present( &self, path: &str ) -> Result<bool,OwError> {
		let msg = self.make_present( path ) ? ;
		let rcv = self.send_get_single( msg ) ? ;
		Ok(rcv.ret==0)
	}
	pub fn size( &self, path: &str ) -> Result<i32,OwError> {
		let msg = self.make_size( path ) ? ;
		let rcv = self.send_get_single( msg ) ? ;
		let ret = rcv.ret;
		if ret < 0 {
			eprintln!("Return code from owserver is error {}",rcv.ret);
			return Err(OwError::OtherError);
		} else {
			return Ok(ret) ;
		}
	}
	pub fn dirall( &self, path: &str ) -> Result<Vec<u8>,OwError> {
		match self.slash {
			true => self.get_value(path,OwClient::make_dirallslash),
			_ => self.get_value(path,OwClient::make_dirall),
		}
	}
	pub fn get( &self, path: &str ) -> Result<Vec<u8>,OwError> {
		match self.slash {
			true => self.get_value( path, OwClient::make_getslash),
			_ => self.get_value( path, OwClient::make_get),
		}
	}

	/// printable
	/// prints the data returned from
	///  dir, dirall
	pub fn printable( &self, v: Vec<u8> ) -> String {
		if self.hex {
			return v.iter().map(|b| format!("{:02X}",b)).collect::<Vec<String>>().join(" ") ;
		} else {
			return match str::from_utf8(&v) {
				Ok(s) => s.to_string() ,
				Err(_e) => "Unprintable characters".to_string(),
			} ;
		}
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

	fn add_path( &mut self, path: &str ) -> bool {
		// Add nul-terminated path (and includes null in payload size)
		self.content = match ffi::CString::new(path) {
			Ok(s)=>s.into_bytes_with_nul(),
			Err(_e)=>return false,
		} ;
		self.payload = self.content.len() as u32 ;
		true
	}
	
	fn add_data( &mut self, data: &str ) -> bool {
		// Add data after path without nul
		let dbytes = match ffi::CString::new( data ) {
			Ok(s)=>s.into_bytes(),
			Err(_e)=>return false,
		} ;
		self.content.extend_from_slice(&dbytes) ;
		self.size = dbytes.len() as u32 ;
		self.payload += self.size ;
		true
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
}

#[derive(Debug)]
pub enum OwError {
    TextError,
    NetworkError,
    ConfigError,
    OtherError,
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
		let x = owc.printable(v) ;
		assert_eq!(x,"Hello");

		// Hex
        owc.hex = true ;
		let v :Vec<u8> = vec!(72,101,108,108,111);
		let x = owc.printable(v) ;
		assert_eq!(x,"48 65 6C 6C 6F");
	}
}
