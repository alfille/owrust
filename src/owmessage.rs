// owrust project
// https://github.com/alfille/owrust
//
// This is a rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

//! owmessage abstracts the actual tcp packet format (to some extent)

pub struct OwMessageSend {
	version: u32,
	payload: u32,
	mtype:   u32,
	flags:   u32,
	size:    u32,
	offset:  u32,
	content: Vec<u8>,
}

pub impl OwMessageSend {
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

pub struct OwMessageReceive {
	version: u32,
	payload: i32,
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
			payload: u32::from_be_bytes(buffer[ 4.. 8].try_into().unwrap()) as i32,
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
