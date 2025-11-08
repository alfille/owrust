/*
owrust library is a rust module that communicates with owserver (http://owfs.org)
This allows Dallas 1-wire devices to be used easily from rust code
*/

use std::ffi ;
use std::io::{self,Read,Write,ErrorKind} ;
use std::net::TcpStream ;
use std::time::Duration ;

use clap::Parser ;
use std::path::PathBuf;

pub fn new() -> OwClient {
	Cli::parse() ;
	OwClient::new()
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
	/// Optional config file		
	#[arg(short, long, global = true, default_value = "owfs.conf")]
	config: PathBuf,
	
	/// Optional toml file		
	#[arg(short, long, global = true, default_value = "owfs.toml")]
	toml: PathBuf,
	
	
}

pub struct OwClient {
	owserver:    String,
	temperature: String,
	pressure:    String,
	device:      String,
	flag:        u32,
}

impl OwClient {
	// Flag for types
	// -- Device format flags (mutually exclusive)
	const DEVICE_F_I:  u32 = 0x00000000 ;
	const DEVICE_FI:   u32 = 0x01000000 ;
	const DEVICE_F_I_C:u32 = 0x02000000 ;
	const DEVICE_F_IC: u32 = 0x03000000 ;
	const DEVICE_FI_C: u32 = 0x04000000 ;
	const DEVICE_FIC:  u32 = 0x05000000 ;
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
		let mut ls = OwClient {
			owserver: String::from("localhost:3504"),
			temperature: "C".to_string(),
			pressure: "mmHg".to_string(),
			device: "f_i".to_string(),
			flag:   0,
		} ;
		ls.make_flag() ;
		ls
	}
	
	fn make_flag( &mut self ) {
		self.flag = 0 ;
		if let Some(first) = self.temperature.chars().next() {
			self.flag |= match first {
				'F' | 'f' => OwClient::TEMPERATURE_F,
				'K' | 'k' => OwClient::TEMPERATURE_K,
				'R' | 'r' => OwClient::TEMPERATURE_R,
				_ => OwClient::TEMPERATURE_C,
			}
		}
		
		if self.pressure != "" {
			self.flag |= match &self.pressure.to_lowercase() as &str {
				"mbar" => OwClient::PRESSURE_MBAR,
				"atm"  => OwClient::PRESSURE_ATM ,
				"mmhg" | "torr" => OwClient::PRESSURE_MMHG,
				"inhg" => OwClient::PRESSURE_INHG,
				"psi"  => OwClient::PRESSURE_PSI ,
				"pa"   => OwClient::PRESSURE_PA  ,
				_      => OwClient::PRESSURE_MBAR,
			}
		}
				
	}

	fn make_nop(&self)-> Result<OwMessage,io::Error> {
		Ok(
		OwMessage {
			version: OwMessage::SENDVERSION,
			payload: 0,
			mtype:   OwMessage::NOP,
			flags:   self.flag,
			size:    OwMessage::DEFAULTSIZE,
			offset:  0,
			content: [].to_vec(),
		}
		)
	}
	
	fn param1( &self, text: &str, mtype: u32, msg_name: &str ) -> Result<OwMessage,io::Error> {
		let mut msg = self.make_nop().unwrap() ;
		msg.mtype = mtype ;
		if msg.add_path( text ) {
			Ok(msg)
		} else {
			let explain: String = format!("Trouble creating {} message",msg_name) ;
			Err(OwMessage::string_error(&explain))
		}
	}
	
	fn make_write( &self, text: &str, value: &str ) -> Result<OwMessage,io::Error> {
		let mut msg = self.make_nop().unwrap() ;
		msg.mtype = OwMessage::WRITE ;
		if msg.add_path( text ) && msg.add_data( value ) {
			Ok(msg)
		} else {
			let explain: String = format!("Trouble creating {} message","WRITE") ;
			Err(OwMessage::string_error(&explain))
		}
	}

	fn make_read( &self, text: &str ) -> Result<OwMessage,io::Error> {
		self.param1( text, OwMessage::READ, "READ" )
	}
	fn make_dir( &self, text: &str ) -> Result<OwMessage,io::Error> {
		self.param1( text, OwMessage::DIR, "DIR" )
	}
	fn make_size( &self, text: &str ) -> Result<OwMessage,io::Error> {
		self.param1( text, OwMessage::PRESENT, "PRESENT" )
	}
	fn make_present( &self, text: &str ) -> Result<OwMessage,io::Error> {
		self.param1( text, OwMessage::PRESENT, "PRESENT" )
	}
	fn make_dirall( &self, text: &str ) -> Result<OwMessage,io::Error> {
		self.param1( text, OwMessage::DIRALL, "DIRALL" )
	}
	fn make_get( &self, text: &str ) -> Result<OwMessage,io::Error> {
		self.param1( text, OwMessage::GET, "GET" )
	}
	fn make_dirallslash( &self, text: &str ) -> Result<OwMessage,io::Error> {
		self.param1( text, OwMessage::DIRALLSLASH, "DIRALLSLASH" )
	}
	fn make_getslash( &self, text: &str ) -> Result<OwMessage,io::Error> {
		self.param1( text, OwMessage::GETSLASH, "GETSLASH" )
	}
	
	fn from_message( &self, mut stream: TcpStream ) -> Result<OwMessage,io::Error> {
		let mut rcv = self.make_nop() ? ;
		static HSIZE: usize = 24 ;
		let mut buffer: [u8; HSIZE ] = [ 0 ; HSIZE ];
		
		// Set timeout
		stream.set_read_timeout( Some(Duration::from_secs(5))) ? ;
		
		loop {
			stream.read_exact( &mut buffer ) ? ;
			rcv.version = u32::from_be_bytes(buffer[ 0.. 4].try_into().unwrap()) ;
			rcv.payload = u32::from_be_bytes(buffer[ 4.. 8].try_into().unwrap()) ;
			rcv.mtype   = u32::from_be_bytes(buffer[ 8..12].try_into().unwrap()) ;
			rcv.flags   = u32::from_be_bytes(buffer[12..16].try_into().unwrap()) ;
			rcv.size    = u32::from_be_bytes(buffer[16..20].try_into().unwrap()) ;
			rcv.offset  = u32::from_be_bytes(buffer[20..24].try_into().unwrap()) ;
			
			let length = rcv.payload as i32 ;
			if length < 0 {
				// ping
				continue ;
			}
			if length > 0 {
				let mut chunk = stream.take( length as u64 ) ;
				rcv.content.clear() ;
				let c = chunk.read_to_end(&mut rcv.content ) ? ;
				if c != length as usize {
					return Err(OwMessage::string_error("Receive bad payload length")) ;
				}
			}
			break ;
		}
		return Ok(rcv) ;
	}
	
	fn to_message( &self, send: OwMessage ) -> Result<OwMessage,io::Error> {
		let mut msg:Vec<u8> = 
			[ send.version, send.payload, send.mtype, send.flags, send.size, send.offset ]
			.iter()
			.flat_map( |&u| u.to_be_bytes() )
			.collect() ;
		if send.content_length() > 0 {
			msg.extend_from_slice(&send.content) ;
		}

		// Write to network
		let mut stream = TcpStream::connect( &self.owserver ) ? ;
		stream.write_all( &msg ) ? ;
		
		let rcv = self.from_message( stream ) ? ;
		
		Ok(rcv)
	}
	fn retrieve_1_value( &self, path: &str, f: fn(&OwClient, &str)->Result<OwMessage,io::Error>) -> Result< Vec<u8>, io::Error> {
		let msg = f( self, path ) ? ;
		let rcv = self.to_message( msg ) ? ;
		if rcv.content_length() > 0 {
			let v: Vec<u8> = rcv.content.clone() ;
			return Ok( v ) ;
		}
		Ok("".as_bytes().to_vec())
	}
	
	pub fn read( &self, path: &str ) -> Result<&Vec<u8>,io::Error> {
		let v = self.retrieve_1_value( path, OwClient::make_read) ? ;
		Ok(&v.clone())
	}
	pub fn dir( &self, path: &str ) -> Result<&Vec<u8>,io::Error> {
		let v: Vec<u8> = self.retrieve_1_value( path, OwClient::make_dirall) ? ;
		Ok(&v)
	}
	pub fn present( &self, path: &str ) -> Result<bool,io::Error> {
		let msg = self.make_present( path ) ? ;
		let rcv = self.to_message( msg ) ? ;
		Ok(rcv.ret_code()==0)
	}
	pub fn size( &self, path: &str ) -> Result<i32,io::Error> {
		let msg = self.make_size( path ) ? ;
		let rcv = self.to_message( msg ) ? ;
		let ret = rcv.ret_code();
		if ret < 0 {
			return Err(OwMessage::string_error("Bad size"));
		} else {
			return Ok(ret) ;
		}
	}
	pub fn dirall( &self, path: &str ) -> Result<&Vec<u8>,io::Error> {
		let v = self.retrieve_1_value( path, OwClient::make_dirall) ? ;
		Ok(&v)
	}
	pub fn dirallslash( &self, path: &str ) -> Result<&Vec<u8>,io::Error> {
		let v = self.retrieve_1_value( path, OwClient::make_dirallslash) ? ;
		Ok(&v)
	}
	pub fn get( &self, path: &str ) -> Result<&Vec<u8>,io::Error> {
		let v = self.retrieve_1_value( path, OwClient::make_get) ? ;
		Ok(&v)
	}
	pub fn getslash( &self, path: &str ) -> Result<&Vec<u8>,io::Error> {
		let v = self.retrieve_1_value( path, OwClient::make_getslash) ? ;
		Ok(&v)
	}
}

struct OwMessage {
	version: u32,
	payload: u32,
	mtype:   u32,
	flags:   u32,
	size:    u32,
	offset:  u32,
	content: Vec<u8>,
}
impl OwMessage {
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

	fn string_error(e: &str) ->io::Error {
		io::Error::new(ErrorKind::InvalidInput, e )
	}
	
	fn ret_code( &self ) -> i32 {
		self.mtype as i32
	}
	fn content_length( &self ) -> usize {
		self.payload as usize
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
	
	fn val_to_string( &self ) -> Result<&str,io::Error> {
		if self.payload as i32 > 0 {
			return match str::from_utf8( &self.content ) {
				Ok(s) => Ok(s),
				Err(_)=>Err(OwMessage::string_error("Bad characters"))
			} ;
		}
		Err(OwMessage::string_error("No payload"))
	}
	
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
