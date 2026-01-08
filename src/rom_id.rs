//! Rom Id code
//! Each 1-wire device has a unique address
//! * 1 byte "family ocde" -- chip type
//! * 6 byte unique id
//! * 1 byte check byte (crc8)

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use std::ops::Deref;
use std::convert::TryInto;

pub struct RomId( [u8;8] ) ;

impl Deref for RomId {
    type Target = [u8;8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// generic crc8 calculator for a given byte sequence
pub fn crc8( bytes: &[u8] ) -> u8 {
	let mut crc = 0u8;

	for &byte in bytes {
		let mut temp_byte = byte;
		for _ in 0..8 {
			let mix = (crc ^ temp_byte) & 0x01;
			crc >>= 1;
			if mix != 0 {
				crc ^= 0x8C; // This is the reflected polynomial 0x31
			}
			temp_byte >>= 1;
		}
	}
	crc
}

impl RomId {
	/// create a blank RomId (all zeros, won't even pass CRC8 test)
	pub fn new<B>(data: B) -> Self 
	where
		B: AsRef<[u8]>,
	{
		let mut rom = [0u8;8] ;
		let bytes = data.as_ref() ;
		if bytes.len()>7 {
			rom.copy_from_slice(&bytes[0..8]);
		} else if bytes.len() == 7 {
			rom.copy_from_slice(&bytes[0..7]);
			rom[7]=crc8(&bytes);
		}
		Self( rom )
	}
	/// Get family code (first byte)
	pub fn family( &self ) -> u8 {
		self[0]
	}
	/// id is the middle 6 bytes (excluding family code and crc8 -- it's the unique part
	pub fn id( &self) -> [u8;6] {
		self[1..7].try_into().unwrap()
	}
	/// Get crc8 check byte
	pub fn crc8( &self ) -> u8 {
		self[7]
	}
	/// Check that crc8 byte is correct
	pub fn test_crc8( &self ) -> bool {
		crc8(&self.0)==0u8
	}
	pub fn make_crc8( &self ) -> u8 {
		crc8(&self[0..7])
	}
}
	
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn t_rom8() {
		let data = [ 0x10, 0x67, 0xc6, 0x69, 0x73, 0x51, 0xff, 0x8d ] ;
		let rom = RomId::new(data) ;
		assert_eq!(rom.family(), 0x10 );
		assert_eq!(rom.crc8(), 0x8d);
		assert!(rom.test_crc8());
		assert_eq!(rom.id(), [0x67, 0xc6, 0x69, 0x73, 0x51, 0xff]);
		assert_eq!(rom.crc8(), 0x8e); // should fail
    }
}
