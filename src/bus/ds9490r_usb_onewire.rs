/// DS9490R/DS2490 USB 1-Wire Bus Master Implementation
/// 
/// The DS9490R uses the DS2490 chip which is a USB-to-1-Wire bridge.
/// It uses USB vendor-specific commands to control the 1-Wire bus.
/// 
/// USB IDs: VendorID=0x04FA, ProductID=0x2490

use rusb::{DeviceHandle, Context, UsbContext};
use std::time::Duration;

/// USB Vendor and Product IDs for DS2490/DS9490R
const DS2490_VENDOR_ID: u16 = 0x04FA;
const DS2490_PRODUCT_ID: u16 = 0x2490;

/// USB Control Transfer parameters
const USB_TIMEOUT: Duration = Duration::from_millis(1000);
const CONTROL_EP: u8 = 0; // Control endpoint

/// DS2490 Control Commands (bRequest values)
mod control_cmd {
    pub const COMM_CMD: u8 = 0x00;
    pub const MODE_CMD: u8 = 0x01;
    pub const TEST_CMD: u8 = 0x02;
}

/// DS2490 Communication Commands (wValue for COMM_CMD)
mod comm_cmd {
    pub const RESET: u16 = 0x0001;
    pub const BIT_IO: u16 = 0x0020;
    pub const BYTE_IO: u16 = 0x0080;
    pub const SEARCH: u16 = 0x00A2;
    pub const BLOCK_IO: u16 = 0x00E4;
}

/// DS2490 Mode Commands (wValue for MODE_CMD)
mod mode_cmd {
    pub const PULSE_EN: u16 = 0x0000;
    pub const SPEED_CHANGE_EN: u16 = 0x0001;
    pub const PROGRAM_PULSE: u16 = 0x0004;
    pub const STRONG_PULLUP: u16 = 0x0008;
}

/// Endpoint addresses
const EP_STATUS: u8 = 0x81;  // Bulk IN for status
const EP_DATA_IN: u8 = 0x83;  // Bulk IN for data
const EP_DATA_OUT: u8 = 0x02; // Bulk OUT for data

/// Status register structure
#[derive(Debug)]
struct StatusRegisters {
    enable_flags: u8,
    onewire_speed: u8,
    strong_pullup_duration: u8,
    programming_pulse_duration: u8,
    pulldown_slew_rate: u8,
    write1_low_time: u8,
    data_sample_offset: u8,
    write0_recovery_time: u8,
}

/// Represents a 64-bit 1-Wire device address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomId([u8; 8]);

impl RomId {
    pub fn new(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    pub fn family_code(&self) -> u8 {
        self.0[0]
    }

    pub fn serial_number(&self) -> [u8; 6] {
        let mut serial = [0u8; 6];
        serial.copy_from_slice(&self.0[1..7]);
        serial
    }

    pub fn crc(&self) -> u8 {
        self.0[7]
    }

    pub fn is_valid_crc(&self) -> bool {
        compute_crc8(&self.0[0..7]) == self.crc()
    }

    pub fn to_hex_string(&self) -> String {
        self.0.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }
}

/// CRC-8 calculation for 1-Wire
fn compute_crc8(data: &[u8]) -> u8 {
    let mut crc = 0u8;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x01 != 0 {
                crc = (crc >> 1) ^ 0x8C;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

/// DS9490R USB 1-Wire Bus Master
pub struct DS9490R {
    handle: DeviceHandle<Context>,
}

impl DS9490R {
    /// Open the first DS9490R device found
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let context = Context::new()?;
        
        for device in context.devices()?.iter() {
            let device_desc = device.device_descriptor()?;
            
            if device_desc.vendor_id() == DS2490_VENDOR_ID 
                && device_desc.product_id() == DS2490_PRODUCT_ID {
                
                let mut handle = device.open()?;
                
                // Detach kernel driver if active (Linux)
                #[cfg(target_os = "linux")]
                {
                    if handle.kernel_driver_active(0)? {
                        handle.detach_kernel_driver(0)?;
                    }
                }
                
                // Set configuration and claim interface
                handle.set_active_configuration(1)?;
                handle.claim_interface(0)?;
                
                let mut bus = Self { handle };
                
                // Reset the adapter to known state
                bus.reset_adapter()?;
                
                return Ok(bus);
            }
        }
        
        Err("DS9490R device not found".into())
    }

    /// Send a control command to the DS2490
    fn control_cmd(&self, request: u8, value: u16, index: u16) -> Result<(), Box<dyn std::error::Error>> {
        let request_type = rusb::request_type(
            rusb::Direction::Out,
            rusb::RequestType::Vendor,
            rusb::Recipient::Device
        );
        
        self.handle.write_control(
            request_type,
            request,
            value,
            index,
            &[],
            USB_TIMEOUT
        )?;
        
        Ok(())
    }

    /// Read status registers
    fn read_status(&self) -> Result<StatusRegisters, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 32];
        
        let len = self.handle.read_bulk(
            EP_STATUS,
            &mut buf,
            USB_TIMEOUT
        )?;
        
        if len >= 16 {
            Ok(StatusRegisters {
                enable_flags: buf[0],
                onewire_speed: buf[1],
                strong_pullup_duration: buf[2],
                programming_pulse_duration: buf[3],
                pulldown_slew_rate: buf[4],
                write1_low_time: buf[5],
                data_sample_offset: buf[6],
                write0_recovery_time: buf[7],
            })
        } else {
            Err("Status read too short".into())
        }
    }

    /// Reset the DS2490 adapter to default state
    fn reset_adapter(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Send reset command
        self.control_cmd(control_cmd::TEST_CMD, 0x0001, 0)?;
        std::thread::sleep(Duration::from_millis(100));
        
        // Clear any pending data
        let mut buf = [0u8; 256];
        let _ = self.handle.read_bulk(EP_DATA_IN, &mut buf, Duration::from_millis(10));
        
        Ok(())
    }

    /// Reset the 1-Wire bus and check for presence
    pub fn reset(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        // Send 1-Wire reset command
        self.control_cmd(control_cmd::COMM_CMD, comm_cmd::RESET, 0)?;
        
        // Read status to get presence detect result
        std::thread::sleep(Duration::from_millis(5));
        let status = self.read_status()?;
        
        // Check result register (presence is indicated in status)
        // Bit 5 of enable_flags indicates presence pulse detected
        Ok((status.enable_flags & 0x20) != 0)
    }

    /// Write a byte to the 1-Wire bus
    pub fn write_byte(&mut self, byte: u8) -> Result<(), Box<dyn std::error::Error>> {
        // Send byte write command
        self.control_cmd(control_cmd::COMM_CMD, comm_cmd::BYTE_IO, 0)?;
        
        // Send the byte via bulk out
        self.handle.write_bulk(EP_DATA_OUT, &[byte], USB_TIMEOUT)?;
        
        // Wait for operation to complete
        std::thread::sleep(Duration::from_micros(100));
        
        Ok(())
    }

    /// Read a byte from the 1-Wire bus
    pub fn read_byte(&mut self) -> Result<u8, Box<dyn std::error::Error>> {
        // Write 0xFF to generate read time slots
        self.write_byte(0xFF)?;
        
        // Read the result
        let mut buf = [0u8; 1];
        self.handle.read_bulk(EP_DATA_IN, &mut buf, USB_TIMEOUT)?;
        
        Ok(buf[0])
    }

    /// Write multiple bytes
    pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if data.is_empty() {
            return Ok(());
        }
        
        // Send block I/O command with length
        let length = data.len() as u16;
        self.control_cmd(control_cmd::COMM_CMD, comm_cmd::BLOCK_IO, length)?;
        
        // Send the data
        self.handle.write_bulk(EP_DATA_OUT, data, USB_TIMEOUT)?;
        
        Ok(())
    }

    /// Read multiple bytes
    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut result = Vec::with_capacity(count);
        
        for _ in 0..count {
            result.push(self.read_byte()?);
        }
        
        Ok(result)
    }

    /// Perform 1-Wire search to find all devices
    pub fn search(&mut self) -> Result<Vec<RomId>, Box<dyn std::error::Error>> {
        let mut devices = Vec::new();
        let mut search_state = SearchState::new();

        loop {
            match self.search_next(&mut search_state)? {
                Some(address) => {
                    if address.is_valid_crc() {
                        devices.push(address);
                    }
                    if search_state.last_device {
                        break;
                    }
                }
                None => break,
            }
        }

        Ok(devices)
    }

    /// Search for next device
    fn search_next(&mut self, state: &mut SearchState) -> Result<Option<RomId>, Box<dyn std::error::Error>> {
        // Reset and check presence
        if !self.reset()? {
            state.last_device = true;
            return Ok(None);
        }

        // Send Search ROM command (0xF0)
        self.write_byte(0xF0)?;

        let mut id_bit_number = 1;
        let mut last_zero = 0;
        let mut rom_byte_number = 0;
        let mut rom_byte_mask = 1u8;

        // Perform search
        while rom_byte_number < 8 {
            // Read two bits: id_bit and complement
            let bits = self.read_byte()?;
            let id_bit = (bits & 0x01) != 0;
            let cmp_id_bit = (bits & 0x02) != 0;

            // Determine search direction
            let search_direction = if id_bit && cmp_id_bit {
                // No devices responded
                return Ok(None);
            } else if id_bit != cmp_id_bit {
                // All devices have same bit
                id_bit
            } else {
                // Discrepancy
                if id_bit_number < state.last_discrepancy {
                    (state.rom[rom_byte_number] & rom_byte_mask) != 0
                } else if id_bit_number == state.last_discrepancy {
                    true
                } else {
                    false
                }
            };

            // Update ROM
            if search_direction {
                state.rom[rom_byte_number] |= rom_byte_mask;
            } else {
                state.rom[rom_byte_number] &= !rom_byte_mask;
            }

            // Write direction bit
            self.write_byte(if search_direction { 0xFF } else { 0x00 })?;

            // Track discrepancies
            if !id_bit && !cmp_id_bit && !search_direction {
                last_zero = id_bit_number;
            }

            id_bit_number += 1;
            rom_byte_mask <<= 1;

            if rom_byte_mask == 0 {
                rom_byte_number += 1;
                rom_byte_mask = 1;
            }
        }

        state.last_discrepancy = last_zero;
        
        if state.last_discrepancy == 0 {
            state.last_device = true;
        }

        Ok(Some(RomId::new(state.rom)))
    }

    /// Select a specific device using Match ROM
    pub fn select(&mut self, address: &RomId) -> Result<(), Box<dyn std::error::Error>> {
        self.reset()?;
        self.write_byte(0x55)?; // MATCH ROM
        self.write_bytes(address.as_bytes())?;
        Ok(())
    }

    /// Skip ROM (address all devices)
    pub fn skip_rom(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.reset()?;
        self.write_byte(0xCC)?; // SKIP ROM
        Ok(())
    }

    /// Enable strong pullup for EEPROM writes
    pub fn enable_strong_pullup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.control_cmd(
            control_cmd::MODE_CMD,
            mode_cmd::STRONG_PULLUP,
            0x0001 // Enable
        )?;
        Ok(())
    }

    /// Disable strong pullup
    pub fn disable_strong_pullup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.control_cmd(
            control_cmd::MODE_CMD,
            mode_cmd::STRONG_PULLUP,
            0x0000 // Disable
        )?;
        Ok(())
    }

    /// Set communication speed
    pub fn set_speed(&mut self, overdrive: bool) -> Result<(), Box<dyn std::error::Error>> {
        let speed_value = if overdrive { 0x0002 } else { 0x0000 };
        
        self.control_cmd(
            control_cmd::MODE_CMD,
            mode_cmd::SPEED_CHANGE_EN,
            speed_value
        )?;
        
        Ok(())
    }
}

impl Drop for DS9490R {
    fn drop(&mut self) {
        let _ = self.handle.release_interface(0);
    }
}

/// Search state
struct SearchState {
    rom: [u8; 8],
    last_discrepancy: u8,
    last_device: bool,
}

impl SearchState {
    fn new() -> Self {
        Self {
            rom: [0; 8],
            last_discrepancy: 0,
            last_device: false,
        }
    }
}
