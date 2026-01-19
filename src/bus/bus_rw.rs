//! ### BusReadWrite struct
//! Low level commands to bus
//! * At reset, and byte level

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use anyhow::Result;

///pub trait BusReadWrite
pub trait BusReadWrite {
    fn read_bit(&mut self) -> Result<bool> {
        let b = self.read_byte()?;
        Ok((b & 0x01) != 0)
    }
    fn write_bit(&mut self, bit:bool) -> Result<()> {
        if bit {
            self.write_byte(0xFF)
        } else {
            self.write_byte(0x00)
        }
    }
    fn read_byte(&mut self) -> Result<u8> ;
    fn write_byte(&mut self, write:u8) -> Result<()> ;
    fn read_bytes(&mut self, n: u32) -> Result<Vec<u8>> {
        let mut v = Vec::new() ;
        for _ in 0..n {
            let b = self.read_byte() ?;
            v.push(b) ;
        }
        Ok(v)
    }
    fn write_bytes(&mut self, v: Vec<u8>) -> Result<()> {
        for b in v {
            self.write_byte(b)?;
        }
        Ok(())
    }
}
