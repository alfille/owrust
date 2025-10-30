use std::ffi ;

struct Header {
    be:[i32; 6] ,
}
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

// Default owserver version (to owserver)
const SENDVERSION: u32 = 0 ;

// Maximum size of returned data (pretty arbitrary but matches C implementation)
const DEFAULTSIZE: u32 = 65536 ;

// Flag for types
// -- Device format flags (mutually exclusive)
pub const DEVICE_F_I:  u32 = 0x00000000 ;
pub const DEVICE_FI:   u32 = 0x01000000 ;
pub const DEVICE_F_I_C:u32 = 0x02000000 ;
pub const DEVICE_F_IC: u32 = 0x03000000 ;
pub const DEVICE_FI_C: u32 = 0x04000000 ;
pub const DEVICE_FIC:  u32 = 0x05000000 ;
// -- Temperature flags (mutually exclusive)
pub const TEMPERATURE_C: u32 = 0x00000000 ;
pub const TEMPERATURE_F: u32 = 0x00010000 ;
pub const TEMPERATURE_K: u32 = 0x00020000 ;
pub const TEMPERATURE_R: u32 = 0x00030000 ;
// -- Pressure flags (mutually exclusive)
pub const PRESSURE_MBAR: u32 = 0x00000000 ;
pub const PRESSURE_ATM:  u32 = 0x00040000 ;
pub const PRESSURE_MMHG: u32 = 0x00080000 ;
pub const PRESSURE_INHG: u32 = 0x000C0000 ;
pub const PRESSURE_PSI:  u32 = 0x00100000 ;
pub const PRESSURE_PA:   u32 = 0x00140000 ;
// -- Other independent flags
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
    fn nop()-> Result<Self,()> {
        Ok(
        Self {
            version: SENDVERSION,
            payload: 0,
            mtype:   NOP,
            flags:   DEVICE_F_I | TEMPERATURE_C | PRESSURE_MBAR,
            size:    DEFAULTSIZE,
            offset:  0,
            content: [].to_vec(),
            any_content: false,
        }
        )
    }
    
    fn read( dir: String ) -> Result<Self,String> {
        let mut read = Self::nop().unwrap() ;
        read.mtype = READ ;
        if read.add_path( dir ) {
            Ok(read)
        } else {
            Err(String::from("Trouble adding content to Read message"))
        }
    }
    
    fn to_message( &self ) -> Vec<u8> {
        let mut ret:Vec<u8> = 
            [ self.version, self.payload, self.mtype, self.flags, self.size, self.offset ]
            .iter()
            .flat_map( |&u| u.to_be_bytes() )
            .collect() ;
        if self.any_content {
            ret.extend_from_slice(&self.content) ;
        }
        ret
    }
    
    fn from_message( &mut self, fm: Vec<u8> ) -> bool {
		self.version = u32::from_be_bytes(fm[0..3].try_into().unwrap()) ;
		self.payload = u32::from_be_bytes(fm[4..7].try_into().unwrap()) ;
		self.mtype   = u32::from_be_bytes(fm[8..11].try_into().unwrap()) ;
		self.flags   = u32::from_be_bytes(fm[12..15].try_into().unwrap()) ;
		self.size    = u32::from_be_bytes(fm[16..19].try_into().unwrap()) ;
		self.offset  = u32::from_be_bytes(fm[20..23].try_into().unwrap()) ;
		if self.payload > 0 {
			self.content = fm[25..(24+self.payload as usize)].to_vec() ;
			self.any_content = true ;
		} else {
			self.any_content = false ;
		}
		true
	}
    
    fn add_path( &mut self, path: String ) -> bool {
        // Add nul-terminated path (and includes null in payload size)
        self.content = match ffi::CString::new(path) {
            Ok(s)=>s.into_bytes_with_nul(),
            Err(e)=>return false,
        } ;
        self.payload = self.content.len() as u32 ;
        self.any_content = self.payload > 0 ; 
        true
    }
    
    fn add_data( &mut self, data: String ) -> bool {
        // Add data after path without nul
        let mut dbytes = match ffi::CString::new( data ) {
            Ok(s)=>s.into_bytes(),
            Err(e)=>return false,
        } ;
        self.content.extend_from_slice(&dbytes) ;
        self.size = dbytes.len() as u32 ;
        self.payload += self.size ;
        true
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
    fn test_version()->bool {
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

