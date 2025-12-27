use heapless;

use crate::alternating_bit::{ChannelSync, SyncBits};
use crate::control_byte::ControlByte;
use crate::error::{Error, TransportError};
use crate::fragment::{Fragmenter, Reassembler};
use crate::header::{HandshakeMessage, Header, NONCE_LEN, parse_u16};
use crate::noise::{Backend, NoiseState};
use crate::{Device, Host, Role};

#[repr(u8)]
pub enum DeviceState {
    Unpaired = 0,
    Paired = 1,
    PairedAutoconnect = 2,
}

impl DeviceState {
    pub fn from(bytes: &[u8]) -> Option<Self> {
        Some(match bytes {
            [0] => Self::Unpaired,
            [1] => Self::Paired,
            [2] => Self::PairedAutoconnect,
            _ => return None,
        })
    }
}

const INTERNAL_BUFFER_LEN: usize = 128; // header, two x25519 keys, two aesgcm tags, some bytes
const MAX_DEVICE_PROPERTIES_LEN: usize = 64; // not in spec

/*
       self.fields = (
           2,  # CHANNEL_ID
           1,  # CHANNEL_STATE
           1,  # CHANNEL_IFACE
           1,  # CHANNEL_SYNC

           32,  # CHANNEL_HANDSHAKE_HASH
           32,  # CHANNEL_KEY_RECEIVE
           32,  # CHANNEL_KEY_SEND
           8,  # CHANNEL_NONCE_RECEIVE
           8,  # CHANNEL_NONCE_SEND
           32,  # CHANNEL_HOST_STATIC_PUBKEY

           2,  # CHANNEL_ACK_LATENCY_MS
       )
*/

enum PacketState<R: Role> {
    Idle,
    Sending(Fragmenter<R>),
    Receiving(Reassembler<R>),
    // Ignoring, // TODO
}

// TODO: pairing related data, reassembler|fragmentor
pub struct ChannelState<R: Role, B: Backend> {
    channel_id: u16,
    state: R,
    iface: u8, // opaque, maybe not needed FIXME
    sync: ChannelSync,
    noise_state: NoiseState<R, B>, // None | HandshakeState | (CipherState, CipherState, HandshakeHash, HostStaticPubkey)
    ack_latency_ms: u16,           // not sure

    send_ack: Option<SyncBits>,
    internal_buffer: heapless::Vec<u8, INTERNAL_BUFFER_LEN>,
    alloc_nonce: [u8; NONCE_LEN], // make part of state?
    packet_state: PacketState<R>,
    // maybe: buffer for ChannelAllocationResponse properties (host only)
    try_to_unlock: bool, // XXX host only
}

impl<R: Role, B: Backend> ChannelState<R, B> {
    pub fn new(channel_id: u16) -> Self {
        Self {
            channel_id,
            state: R::default(),
            iface: 0xff, //TODO
            sync: ChannelSync::new(),
            noise_state: NoiseState::Initial,
            ack_latency_ms: 0,
            send_ack: None,
            internal_buffer: heapless::Vec::new(),
            alloc_nonce: [0; NONCE_LEN],
            packet_state: PacketState::Idle,
            try_to_unlock: false,
        }
    }
}

impl<B: Backend> ChannelState<Host, B> {
    pub fn alloc(&mut self, try_to_unlock: bool) -> Result<(), Error> {
        if !matches!(self.state, Host::Unallocated) {
            return Err(Error::UnexpectedInput);
        }
        self.try_to_unlock = try_to_unlock;
        B::random_bytes(&mut self.alloc_nonce);
        self.internal_buffer.clear();
        self.internal_buffer.resize(NONCE_LEN, 0u8).unwrap();
        self.internal_buffer.copy_from_slice(&self.alloc_nonce);
        let header = Header::new_channel_request();
        // XXX XXX copy to internal_buffer and use that
        let frag = Fragmenter::new(header, SyncBits::new(), &self.internal_buffer)?;
        self.packet_state = PacketState::Sending(frag);
        self.state = Host::Handshake0;
        // XXX set state to "awaiting channel"
        Ok(())
    }

    fn zero_internal_buffer(&mut self) {
        self.internal_buffer.clear();
        self.internal_buffer
            .resize(self.internal_buffer.capacity(), 0u8)
            .unwrap();
    }

    fn incoming_internal(&mut self) -> Result<(), Error> {
        let PacketState::Receiving(r) = &mut self.packet_state else {
            return Err(Error::UnexpectedInput);
        };
        if !r.is_done() {
            return Err(Error::UnexpectedInput);
        }
        let len = r.verify(&self.internal_buffer)?;
        // TODO send ACK
        self.internal_buffer.truncate(len);
        println!("incoming: {}", hex::encode(&self.internal_buffer));
        let header = r.header();

        if header.is_error() {
            let b = self.internal_buffer.get(0).ok_or(Error::MalformedData)?;
            let e: TransportError = b.try_into()?;
            //let e = TransportError::from(self.internal_buffer.get(0).ok_or(Error::MalformedData))?;
            println!("transport error: {:?}", e)
        }

        match self.state {
            // TODO match header too
            Host::Handshake0 => {
                let device_properties = self.get_channel()?;
                let copy =
                    heapless::Vec::<u8, MAX_DEVICE_PROPERTIES_LEN>::from_slice(device_properties)
                        .unwrap();
                self.start_handshake(&copy)?;
                self.state = Host::Handshake1;
            }
            Host::Handshake1 => {
                self.continue_handshake()?;
                self.state = Host::Handshake2;
            }
            Host::Handshake2 => {
                let device_state = self.finish_handshake()?;
                self.state = match device_state {
                    DeviceState::Paired | DeviceState::PairedAutoconnect => {
                        Host::EncryptedTransport
                    }
                    _ => Host::Pairing0,
                };
            }
            Host::EncryptedTransport => return Err(Error::UnexpectedInput),
            Host::Pairing0 => {
                panic!("don't know how to pair");
            },
            _ => panic!(),
        }
        Ok(())
    }

    fn has_cid(&self) -> bool {
        !matches!(self.state, Host::Unallocated | Host::Handshake0)
    }

    fn get_channel(&mut self) -> Result<&[u8], Error> {
        let payload = &self.internal_buffer;
        let (nonce, payload) = payload
            .split_at_checked(NONCE_LEN)
            .ok_or(Error::MalformedData)?;
        if nonce != self.alloc_nonce {
            // TODO: transition to invalidated state
            return Err(Error::MalformedData);
        }
        let (cid, device_properties) = parse_u16(payload)?;
        self.channel_id = cid;
        Ok(device_properties)
    }

    fn start_handshake(&mut self, device_properties: &[u8]) -> Result<(), Error> {
        self.zero_internal_buffer();
        let msg = self.noise_state.start_pairing(
            device_properties,
            self.try_to_unlock,
            &mut self.internal_buffer,
        )?;
        let len = msg.len();
        self.internal_buffer.truncate(len);
        let header = Header::new_handshake(
            self.channel_id,
            HandshakeMessage::InitiationRequest,
            &self.internal_buffer,
        );
        let sb = self.sync.send_start().ok_or(Error::UnexpectedInput)?;
        let frag = Fragmenter::new(header, sb, &self.internal_buffer)?;
        self.packet_state = PacketState::Sending(frag);
        Ok(())
    }

    fn continue_handshake(&mut self) -> Result<(), Error> {
        let payload = &self.internal_buffer.clone();
        self.zero_internal_buffer();
        let cred_lookup = |_re: &[u8], _rs: &[u8]| None;
        let encoded_host_pairing_cred = &[]; // FIXME
        let msg = self.noise_state.complete_pairing(
            payload,
            cred_lookup,
            encoded_host_pairing_cred,
            &mut self.internal_buffer,
        )?;
        let len = msg.len();
        self.internal_buffer.truncate(len);
        let header = Header::new_handshake(
            self.channel_id,
            HandshakeMessage::CompletionRequest,
            &self.internal_buffer,
        );
        let sb = self.sync.send_start().ok_or(Error::UnexpectedInput)?;
        let frag = Fragmenter::new(header, sb, &self.internal_buffer)?;
        self.packet_state = PacketState::Sending(frag);
        Ok(())
    }

    fn finish_handshake(&mut self) -> Result<DeviceState, Error> {
        let mut payload = self.internal_buffer.clone();
        // TODO check if noise_state is finished
        let len = self.noise_state.decrypt(payload.as_mut_slice())?;
        payload.truncate(len); // assumes tag at the end
        println!("payload {}", hex::encode(&payload));
        DeviceState::from(&payload).ok_or(Error::MalformedData)
    }

    pub fn data_out(&mut self, buffer: &mut [u8]) -> Result<bool, Error> {
        if let Some(sb) = self.send_ack.take() {
            let header = Header::<Host>::new_ack(self.channel_id);
            Fragmenter::single(header, sb, &[], buffer)?;
            return Ok(true); // FIXME
        }
        let PacketState::Sending(f) = &mut self.packet_state else {
            return Err(Error::UnexpectedInput); // FIXME end of buffer
        };
        if f.is_done() {
            // Not before channel_id is known
            if self.has_cid() {
                self.sync.send_finish();
            }
            self.packet_state = PacketState::Idle;
            return Ok(false);
        }
        f.next(&self.internal_buffer, buffer) // FIXME internal buffer
        // FIXME consider it sent only after we receive an ack?
    }

    pub fn data_in(&mut self, buffer: &[u8]) -> Result<bool, Error> {
        let has_cid = self.has_cid();
        let is_cont = ControlByte::from(*buffer.get(0).unwrap_or(&0u8)).is_continuation();

        match &mut self.packet_state {
            PacketState::Receiving(_) | PacketState::Idle if !is_cont => {
                if has_cid {
                    let sb = SyncBits::from(*buffer.get(0).unwrap_or(&0u8));
                    if !self.sync.receive_start(sb) {
                        // bad sync bit, drop this message and continuations
                        self.packet_state = PacketState::Idle;
                        return Ok(true);
                    }
                };
                self.zero_internal_buffer();
                let ra = Reassembler::new(buffer, &mut self.internal_buffer)?;
                self.packet_state = PacketState::Receiving(ra);
            }
            PacketState::Idle => return Ok(true), // ignore continuations
            PacketState::Receiving(r) => {
                r.update(buffer, &mut self.internal_buffer)?;
            }
            PacketState::Sending(_) => panic!(),
        };

        let PacketState::Receiving(r) = &mut self.packet_state else {
            return Err(Error::UnexpectedInput);
        };

        // packet done, what now?
        if r.is_done() {
            if r.header().is_ack() && has_cid {
                let sb = SyncBits::from(buffer[0]);
                self.sync.send_mark_delivered(sb); // TODO what about ChannelAllocationResponse
                self.packet_state = PacketState::Idle;
                return Ok(true);
            }
            if has_cid {
                // except ChannelAllocationResponse
                self.send_ack = Some(self.sync.receive_acknowledge());
            }
            // TODO handle encrypted transport
            self.incoming_internal()?;
            return Ok(false);
        }
        Ok(true)
    }

    pub fn handshake_done(&self) -> bool {
        ![
            Host::Unallocated,
            Host::Handshake0,
            Host::Handshake1,
            Host::Handshake2,
        ]
        .contains(&self.state)
        // TODO what about send_ack
        //matches!(self.state, Host::Handshake2) // FIXME 2 = waiting for state
    }
}
