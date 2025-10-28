use std::ffi::CString ;

struct Header {
	be:[i32; 6] ,
}

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

const SENDVERSION: u32 = 0 ;

pub const DEVICE_F_I:  u32 = 0x00000000 ;
pub const DEVICE_FI:   u32 = 0x01000000 ;
pub const DEVICE_F_I_C:u32 = 0x02000000 ;
pub const DEVICE_F_IC: u32 = 0x03000000 ;
pub const DEVICE_FI_C: u32 = 0x04000000 ;
pub const DEVICE_FIC:  u32 = 0x05000000 ;

pub const TEMPERATURE_C: u32 = 0x00000000 ;
pub const TEMPERATURE_F: u32 = 0x00010000 ;
pub const TEMPERATURE_K: u32 = 0x00020000 ;
pub const TEMPERATURE_R: u32 = 0x00030000 ;

pub const PRESSURE_MBAR: u32 = 0x00000000 ;
pub const PRESSURE_ATM:  u32 = 0x00040000 ;
pub const PRESSURE_MMHG: u32 = 0x00080000 ;
pub const PRESSURE_INHG: u32 = 0x000C0000 ;
pub const PRESSURE_PSI:  u32 = 0x00100000 ;
pub const PRESSURE_PA:   u32 = 0x00140000 ;

pub const OWNET_FLAG:  u32 = 0x00000100 ;
pub const UNCACHED:    u32 = 0x00000020 ;
pub const SAFEMODE:    u32 = 0x00000010 ;
pub const ALIAS:       u32 = 0x00000008 ;
pub const PERSISTENCE: u32 = 0x00000004 ;
pub const BUS_RET:     u32 = 0x00000002 ;

pub struct SendMessage {
	version: u32,
	payload: u32,
	mtype:   u32,
	flags:   u32,
	size:    u32,
	offset:  u32,
	content: Vec<u8>,
	any_content: bool,
}
impl SendMessage {
	fn setup( &mut self ) {
		self.version = SENDVERSION ;
		self.payload = 0 ;
		self.mtype = NOP ;
		self.flags = DEVICE_F_I | TEMPERATURE_C | PRESSURE_MBAR ;
		self.size = 0 ;
		self.offset = 0 ;
		self.content = CString::new("").into_bytes_with_nul() ;
	}
	
	fn load_header( &self ) -> Vec<u8> {
		[ self.version, self.payload, self.mtype, self.flags, self.size, self.offset ]
		.iter()
		.flat_map( |&u| u.to_be_bytes() )
		.collect()
	}
	
	fn add_content( &mut self, dir: String ) -> Result<(),std::ffi::NulError> {
		self.content = match CString::new(dir) {
			Ok(s)=>s.into_bytes_with_nul(),
			Err(e)=>return Err(e),
		} ;
		self.payload = self.content.len() as u32 ;
		self.any_content = self.payload > 0 ; 
		Ok(())
	}
		
}

pub struct ReceiveMessage {
	version: u32,
	payload: u32,
	ret:     u32,
	flags:   u32,
	size:    u32,
	offset:  u32,
	content: Vec<u8>,
}

impl ReceiveMessage {
	fn TestVersion()->bool {
		true
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

enum ToMessage {
	Nop,
	Read{ 
		filename: String, 
		size: u32,
	},
	Write{
		filename: String,
		value: String,
	},
	Dir {
		dirname: String,
	},
	Size {
		filename: String,
	},
	Present {
		filename: String,
	},
	DirAll {
		dirname: String,
	},
	Get {
		dirname: String,
	},
	DirAllSlash {
		dirname: String,
	},
	GetSlash {
		dirname: String,
	}
}

enum FromMessage {
	Nop,
	Read{
		value: String,
		ret: bool, 
	},
	Write{
		ret: bool,
	},
	Dir {
		dirname: String,
		eol: bool
	},
	Size {
		size: u32,
		ret: bool,
	},
	Present {
		ret: bool,
	},
	DirAll {
		dirname: String,
	},
	Get {
		dirname: String,
	},
	DirAllSlash {
		dirname: String,
	},
	GetSlash {
		dirname: String,
	},
	Ping,
}

