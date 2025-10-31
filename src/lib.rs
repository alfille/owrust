use std::ffi ;

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

pub struct OwMessage {
    version: u32,
    payload: u32,
    mtype:   u32,
    flags:   u32,
    size:    u32,
    offset:  u32,
    content: Vec<u8>,
    any_content: bool,
    ret:     i32,
}
impl OwMessage {
    // Default owserver version (to owserver)
    const SENDVERSION: u32 = 0 ;

    // Maximum size of returned data (pretty arbitrary but matches C implementation)
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

    fn nop()-> Result<Self,()> {
        Ok(
        Self {
            version: OwMessage::SENDVERSION,
            payload: 0,
            mtype:   OwMessage::NOP,
            flags:   DEVICE_F_I | TEMPERATURE_C | PRESSURE_MBAR,
            size:    OwMessage::DEFAULTSIZE,
            offset:  0,
            content: [].to_vec(),
            any_content: false,
            ret:     0,
        }
        )
    }
    
    fn param1( text: String, mtype: u32, msg_name: &str ) -> Result<Self,String> {
        let mut msg = Self::nop().unwrap() ;
        msg.mtype = mtype ;
        if msg.add_path( text ) {
            Ok(msg)
        } else {
            let e: String = format!("Trouble creating {} message",msg_name) ;
            Err(e)
        }
    }
    
    fn read( dir: String ) -> Result<Self,String> {
        OwMessage::param1( dir, OwMessage::READ, "READ" )
    }
    fn write( dir: String, value: String ) -> Result<Self,String> {
        let mut msg = Self::nop().unwrap() ;
        msg.mtype = OwMessage::WRITE ;
        if msg.add_path( dir ) && msg.add_data( value ) {
            Ok(msg)
        } else {
            let e = String::from("Trouble creating WRITE message") ;
            Err(e)
        }
    }
    fn dir( dir: String ) -> Result<Self,String> {
        OwMessage::param1( dir, OwMessage::DIR, "DIR" )
    }
    fn size( dir: String ) -> Result<Self,String> {
        OwMessage::param1( dir, OwMessage::PRESENT, "PRESENT" )
    }
    fn present( dir: String ) -> Result<Self,String> {
        OwMessage::param1( dir, OwMessage::PRESENT, "PRESENT" )
    }
    fn dirall( dir: String ) -> Result<Self,String> {
        OwMessage::param1( dir, OwMessage::DIRALL, "DIRALL" )
    }
    fn get( dir: String ) -> Result<Self,String> {
        OwMessage::param1( dir, OwMessage::GET, "GET" )
    }
    fn dirallslash( dir: String ) -> Result<Self,String> {
        OwMessage::param1( dir, OwMessage::DIRALLSLASH, "DIRALLSLASH" )
    }
    fn getslash( dir: String ) -> Result<Self,String> {
        OwMessage::param1( dir, OwMessage::GETSLASH, "GETSLASH" )
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
        self.version = u32::from_be_bytes(fm[ 0.. 4].try_into().unwrap()) ;
        self.payload = u32::from_be_bytes(fm[ 4.. 8].try_into().unwrap()) ;
        self.mtype   = u32::from_be_bytes(fm[ 8..12].try_into().unwrap()) ;
        self.flags   = u32::from_be_bytes(fm[12..16].try_into().unwrap()) ;
        self.size    = u32::from_be_bytes(fm[16..20].try_into().unwrap()) ;
        self.offset  = u32::from_be_bytes(fm[20..24].try_into().unwrap()) ;
        self.ret     = self.mtype as i32 ; // allows negative return codes as C errors
        if self.payload > 0 {
            self.content = fm[24..(24+self.payload as usize)].to_vec() ;
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
            Err(_e)=>return false,
        } ;
        self.payload = self.content.len() as u32 ;
        self.any_content = self.payload > 0 ; 
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
