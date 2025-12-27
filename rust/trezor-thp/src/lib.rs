//! Trezor Host Protocol implementation in Rust.

// #![no_std]
#![forbid(unsafe_code)]

pub mod alternating_bit;
pub mod channel;
mod control_byte;
mod crc32;
pub mod error;
pub mod fragment;
pub mod header;
pub mod noise;

pub use channel::ChannelState;
pub use error::Error;

pub trait Role: Clone + PartialEq + Default {
    fn is_host() -> bool;
}

#[cfg_attr(any(test, debug_assertions), derive(Debug))]
#[derive(Clone, PartialEq)]
#[repr(u8)]
pub enum Device {
    Unallocated = 0,        // `UNALLOCATED`
    Handshake1 = 1,         // `TH1`
    Handshake2 = 2,         // `TH2`
    Pairing0 = 3,           // `TP0`
    Pairing1 = 4,           // `TP1`
    Pairing2 = 5,           // `TP2`
    Pairing3 = 6,           // `TP3`
    Pairing4 = 7,           // `TP4`
    Credential1 = 8,        // `TC1`
    EncryptedTransport = 9, // `ENCRYPTED_TRANSPORT``
    Invalidated = 10,       // FIXME: needed?
}

impl Role for Device {
    fn is_host() -> bool {
        false
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::Unallocated
    }
}

#[cfg_attr(any(test, debug_assertions), derive(Debug))]
#[derive(Clone, PartialEq)]
#[repr(u8)]
pub enum Host {
    Unallocated = 0,         // `UNALLOCATED`
    Handshake0 = 1,          // `HH0`
    Handshake1 = 2,          // `HH1`
    Handshake2 = 3,          // `HH2`
    Pairing0 = 4,            // `HP0`
    Pairing1 = 5,            // `HP1`
    Pairing2 = 6,            // `HP2`,
    Pairing3a = 7,           // `HP3a`
    Pairing3b = 8,           // `HP3b`
    Pairing4 = 9,            // `HP4`
    Pairing5 = 10,           // `HP5`
    Pairing6 = 11,           // `HP6`
    Pairing7 = 12,           // `HP7`
    Credential0 = 13,        // `HC0`
    Credential1 = 14,        // `HC1`
    EncryptedTransport = 15, // `ENCRYPTED_TRANSPORT``
}

impl Role for Host {
    fn is_host() -> bool {
        true
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::Unallocated
    }
}

/*
#[repr(u8)]
enum SessionState {
    Unallocated = 0,
    Allocated = 1,
    Seedless = 2, // AllocatedSeedless?
}*/

/*
#[cfg_attr(any(test, debug_assertions), derive(Debug))]
#[derive(Clone, PartialEq)]
pub struct HostOld;

impl Role for HostOld {
    fn is_host() -> bool {
        true
    }
}

#[cfg_attr(any(test, debug_assertions), derive(Debug))]
#[derive(Clone, PartialEq)]
pub struct DeviceOld;

impl Role for DeviceOld {
    fn is_host() -> bool {
        false
    }
}
    */
