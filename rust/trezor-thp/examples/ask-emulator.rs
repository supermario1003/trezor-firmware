use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;

use trezor_thp::{
    ChannelState, Host,
    alternating_bit::SyncBits,
    fragment::{Fragmenter, Reassembler},
    header::Header,
    noise::Backend,
};

use noise_rust_crypto::{Aes256Gcm, Sha256, X25519};

struct RustCrypto {}

impl Backend for RustCrypto {
    type DH = X25519;
    type Cipher = Aes256Gcm;
    type Hash = Sha256;

    fn random_bytes(dest: &mut [u8]) {
        dest.fill(42) // XXX not a good random value
    }
}

const REPEAT: u8 = 1;
const PACKET_LEN: usize = 64;

pub fn main() -> std::io::Result<()> {
    let port_str = env::args().nth(1).unwrap_or("21324".to_string());
    let port = u16::from_str(&port_str).expect("UDP port number");
    let emu_addr = SocketAddr::from(([127, 0, 0, 1], port));
    let socket = UdpSocket::bind("127.0.0.1:0")?;

    let mut c = ChannelState::<Host, RustCrypto>::new(1337);
    c.alloc(false).unwrap();

    let mut sockbuf = [0u8; PACKET_LEN];
    let mut reply_data = [0u8; PACKET_LEN];

    while !c.handshake_done() {
        loop {
            let res = c.data_out(&mut sockbuf).unwrap();
            if !res {
                break;
            }
            println!("> {}", hex::encode(&sockbuf));
            socket.send_to(&sockbuf, &emu_addr)?;
        }
        println!("done sending");
        loop {
            let (reply_len, _src_addr) = socket.recv_from(&mut sockbuf).unwrap();
            println!("< {}", hex::encode(&sockbuf[..reply_len]));
            let res = c.data_in(&mut sockbuf[..reply_len]).unwrap();
            if !res {
                break;
            }
        }
        println!("done receiving");
    }
    println!("handshake done");

    Ok(())
}
