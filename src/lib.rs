/*
owrust library is a rust module that communicates with owserver (http://owfs.org)
This allows Dallas 1-wire devices to be used easily from rust code
*/

mod owrust {

	use std::ffi ;
	use std::io::{self,Read,Write,ErrorKind} ;
	use std::net::TcpStream ;
	use std::time::Duration ;

	use clap::Parser ;

	pub fn new() -> OwClient {
		OwClient::new()
	}

	#[derive(Parser, Debug)]
	#[command(version, about, long_about = None)]
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
				owserver: String::from("localhost:3404"),
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
		
		fn param1( &self, text: String, mtype: u32, msg_name: &str ) -> Result<OwMessage,io::Error> {
			let mut msg = self.make_nop().unwrap() ;
			msg.mtype = mtype ;
			if msg.add_path( text ) {
				Ok(msg)
			} else {
				Err(OwMessage::string_error(format!("Trouble creating {} message",msg_name)))
			}
		}
		
		fn make_write( &self, text: String, value: String ) -> Result<OwMessage,io::Error> {
			let mut msg = self.make_nop().unwrap() ;
			msg.mtype = OwMessage::WRITE ;
			if msg.add_path( text ) && msg.add_data( value ) {
				Ok(msg)
			} else {
				Err(OwMessage::string_error(format!("Trouble creating {} message","WRITE")))
			}
		}

		fn make_read( &self, text: String ) -> Result<OwMessage,io::Error> {
			self.param1( text, OwMessage::READ, "READ" )
		}
		fn make_dir( &self, text: String ) -> Result<OwMessage,io::Error> {
			self.param1( text, OwMessage::DIR, "DIR" )
		}
		fn make_size( &self, text: String ) -> Result<OwMessage,io::Error> {
			self.param1( text, OwMessage::PRESENT, "PRESENT" )
		}
		fn make_present( &self, text: String ) -> Result<OwMessage,io::Error> {
			self.param1( text, OwMessage::PRESENT, "PRESENT" )
		}
		fn make_dirall( &self, text: String ) -> Result<OwMessage,io::Error> {
			self.param1( text, OwMessage::DIRALL, "DIRALL" )
		}
		fn make_get( &self, text: String ) -> Result<OwMessage,io::Error> {
			self.param1( text, OwMessage::GET, "GET" )
		}
		fn make_dirallslash( &self, text: String ) -> Result<OwMessage,io::Error> {
			self.param1( text, OwMessage::DIRALLSLASH, "DIRALLSLASH" )
		}
		fn make_getslash( &self, text: String ) -> Result<OwMessage,io::Error> {
			self.param1( text, OwMessage::GETSLASH, "GETSLASH" )
		}
		
		fn from_message( &self, mut stream: TcpStream ) -> Result<OwMessage,io::Error> {
			let mut rcv = self.make_nop() ? ;
			static HSIZE: usize = 24 ;
			let mut buffer: [u8; HSIZE ] = [ 0 ; HSIZE ];
			
			// Set timeout
			stream.set_read_timeout( Some(Duration::from_secs(5))) ? ;
			
			loop {
				let n = stream.read( &mut buffer ) ? ;
				if n == HSIZE {
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
						let c = stream.read( &mut rcv.content ) ? ;
						if c != length as usize {
							return Err(OwMessage::string_error(String::from("Receive bad payload length"))) ;
						}
					}
					break ;
				}
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
		fn retrieve_1_value( &self, path: String, f: fn(&OwClient, String)->Result<OwMessage,io::Error>) -> Result< String, io::Error> {
			let msg = f( self, path ) ? ;
			let rcv = self.to_message( msg ) ? ;
			let s = rcv.val_to_string() ? ;
			Ok(s)
		}
		
		pub fn read( &self, path:String ) -> Result<String,io::Error> {
			self.retrieve_1_value( path, OwClient::make_read)
		}
		pub fn dir( &self, path:String ) -> Result<String,io::Error> {
			self.retrieve_1_value( path, OwClient::make_dirall)
		}
		pub fn present( &self, path:String ) -> Result<bool,io::Error> {
			let msg = self.make_present( path ) ? ;
			let rcv = self.to_message( msg ) ? ;
			Ok(rcv.ret_code()==0)
		}
		pub fn size( &self, path:String ) -> Result<i32,io::Error> {
			let msg = self.make_size( path ) ? ;
			let rcv = self.to_message( msg ) ? ;
			let ret = rcv.ret_code();
			if ret < 0 {
				return Err(OwMessage::string_error(String::from("Bad size")));
			} else {
				return Ok(ret) ;
			}
		}
		pub fn dirall( &self, path:String ) -> Result<String,io::Error> {
			self.retrieve_1_value( path, OwClient::make_dirall)
		}
		pub fn dirallslash( &self, path:String ) -> Result<String,io::Error> {
			self.retrieve_1_value( path, OwClient::make_dirallslash)
		}
		pub fn get( &self, path:String ) -> Result<String,io::Error> {
			self.retrieve_1_value( path, OwClient::make_get)
		}
		pub fn getslash( &self, path:String ) -> Result<String,io::Error> {
			self.retrieve_1_value( path, OwClient::make_getslash)
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

		fn string_error(e: String) ->io::Error {
			io::Error::new(ErrorKind::InvalidInput, e )
		}
		
		fn ret_code( &self ) -> i32 {
			self.mtype as i32
		}
		fn content_length( &self ) -> usize {
			self.payload as usize
		}
		
		fn add_path( &mut self, path: String ) -> bool {
			// Add nul-terminated path (and includes null in payload size)
			self.content = match ffi::CString::new(path) {
				Ok(s)=>s.into_bytes_with_nul(),
				Err(_e)=>return false,
			} ;
			self.payload = self.content.len() as u32 ;
			true
		}
		
		fn add_data( &mut self, data: String ) -> bool {
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
		
		fn val_to_string( &self ) -> Result<String,io::Error> {
			if self.payload as i32 > 0 {
				match String::from_utf8(self.content.clone()) {
					Ok(s) => return Ok(s),
					Err(_) => return Err(OwMessage::string_error(String::from("Value conversiion error (to String)"))),
				}
			}
			Err(OwMessage::string_error(String::from("No payload")))
		}
		
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
