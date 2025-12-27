use noise_protocol::{Cipher, CipherState, DH, HandshakeState, Hash, U8Array, patterns::noise_xx};

use crate::{Device, Error, Host, Role};

use core::marker::PhantomData;

const PROLOGUE: &'static [u8] = b"";

// FIXME: zeroizes

pub trait Backend {
    type Cipher: Cipher;
    type DH: DH;
    type Hash: Hash;

    fn random_bytes(dest: &mut [u8]);

    fn hash_of_two(data1: &[u8], data2: &[u8]) -> <Self::Hash as Hash>::Output {
        let mut h = Self::Hash::default();
        h.input(data1);
        h.input(data2);
        h.result()
    }
}

type DHPrivKey<B> = <<B as Backend>::DH as DH>::Key;
type DHPubKey<B> = <<B as Backend>::DH as DH>::Pubkey;

pub enum NoiseState<R: Role, B: Backend> {
    Initial,
    Handshake {
        state: HandshakeState<B::DH, B::Cipher, B::Hash>,
    },
    Encrypted {
        encrypt: CipherState<B::Cipher>,
        decrypt: CipherState<B::Cipher>,
        handshake_hash: [u8; 32], // TODO
        remote_static_key: DHPubKey<B>,
        // possibly also local_static_privkey to save later
        _phantom: PhantomData<R>,
    },
}

impl<R: Role, B: Backend> NoiseState<R, B> {
    pub fn encrypt(&mut self, in_out: &mut [u8], plaintext_len: usize) -> Result<(), Error> {
        let Self::Encrypted {
            encrypt,
            decrypt,
            handshake_hash,
            remote_static_key,
            _phantom,
        } = self
        else {
            return Err(Error::UnexpectedInput);
        };

        //TODO check min length and avoid panic
        encrypt.encrypt_ad_in_place(&[], in_out, plaintext_len);
        Ok(())
    }

    pub fn decrypt(&mut self, in_out: &mut [u8]) -> Result<usize, Error> {
        let Self::Encrypted {
            encrypt,
            decrypt,
            handshake_hash,
            remote_static_key,
            _phantom,
        } = self
        else {
            return Err(Error::UnexpectedInput);
        };

        //TODO check min length and avoid panic
        //let (ct, tag) = in_out.split_last_chunk_mut::<16>(in_out).ok_or(Error::MalformedData)?;
        decrypt
            .decrypt_ad_in_place(&[], in_out, in_out.len())
            .map_err(|_| Error::InvalidDigest) // orly
    }
}

impl<B: Backend> NoiseState<Host, B> {
    pub fn start_pairing<'a>(
        &mut self,
        device_properties: &[u8],
        try_to_unlock: bool,
        dest: &'a mut [u8],
    ) -> Result<&'a [u8], Error> {
        if !matches!(*self, NoiseState::Initial) {
            return Err(Error::UnexpectedInput);
        }

        // NOTE must set s later
        let payload = &[u8::from(try_to_unlock)];
        let mut hss = HandshakeState::new(
            noise_xx(),
            true,
            /*prologue=*/ device_properties,
            None,
            None,
            None,
            None,
        );
        let len = hss.get_next_message_overhead() + payload.len();
        let dest = dest.get_mut(..len).ok_or(Error::InsufficientBuffer)?;
        hss.write_message(payload, dest)?;
        *self = NoiseState::Handshake { state: hss };
        Ok(dest)
    }

    pub fn complete_pairing<'a, F>(
        &mut self,
        incoming: &[u8],
        cred_lookup: F,
        pairing_credential: &[u8], // coming from cred_lookup ...
        dest: &'a mut [u8],
    ) -> Result<&'a [u8], Error>
    where
        F: FnOnce(&[u8], &[u8]) -> Option<DHPrivKey<B>>,
    {
        let Self::Handshake { state } = self else {
            return Err(Error::UnexpectedInput);
        };
        if incoming.len() != state.get_next_message_overhead() {
            return Err(Error::MalformedData);
        }
        state.read_message(incoming, &mut [])?; // XXX

        // look up static key based on remote keys, or generate a new one
        let remote_static = state.get_rs().ok_or(Error::HandshakeFailed)?;
        let remote_ephemeral = state.get_re().ok_or(Error::HandshakeFailed)?;
        let local_static = cred_lookup(remote_ephemeral.as_slice(), remote_static.as_slice());
        let local_static = local_static.unwrap_or_else(|| <B::DH as DH>::genkey());
        state.set_s(local_static);

        let len = state.get_next_message_overhead();
        let dest = dest.get_mut(..len).ok_or(Error::InsufficientBuffer)?;
        state.write_message(pairing_credential, dest)?;
        if !state.completed() {
            return Err(Error::HandshakeFailed);
        }
        let (encrypt, decrypt) = state.get_ciphers();
        let mut handshake_hash = [0u8; 32];
        handshake_hash.copy_from_slice(state.get_hash());
        *self = NoiseState::Encrypted {
            encrypt,
            decrypt,
            handshake_hash,
            remote_static_key: state.get_rs().unwrap(), // TODO
            _phantom: PhantomData,
        };
        Ok(dest) // still waiting for device to transmit "state"
    }
}

/*
impl<B: Backend> NoiseState<Device, B> {
    pub fn start_pairing<'a>(
        &mut self,
        incoming: &[u8],
        static_key: DHPrivKey<B>,
        mask: DHPrivKey<B>,
        dest: &'a mut [u8],
    ) -> Result<&'a [u8], Error> {
        Ok(dest)
    }
}
*/
