//  Copyright 2023 The Tari Project
//
//  Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//  following conditions are met:
//
//  1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//  disclaimer.
//
//  2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
//  following disclaimer in the documentation and/or other materials provided with the distribution.
//
//  3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
//  products derived from this software without specific prior written permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//  INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//  DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//  SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//  WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
//  USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

// #[macro_use]
// mod macros;
// mod blake2;
// mod errors;
// mod ristretto_keys;
// mod schnorr;

extern crate alloc;
use core::{convert::TryFrom, marker::PhantomData};

use borsh::{
    maybestd::io::{Result as BorshResult, Write},
    BorshSerialize,
};
use curve25519_dalek::Scalar;
use nanos_sdk::{buttons::ButtonEvent, ecc, io, random::LedgerRng};
use nanos_ui::ui;
use tari_crypto::{
    commitment::HomomorphicCommitmentFactory,
    keys::{PublicKey, SecretKey},
    ristretto::{
        pedersen::extended_commitment_factory::ExtendedPedersenCommitmentFactory,
        RistrettoPublicKey,
        RistrettoSchnorr,
        RistrettoSecretKey,
    },
    tari_utilities::ByteArray,
};
nanos_sdk::set_panic!(nanos_sdk::exiting_panic);
use tari_crypto::{hash::blake2::Blake256, hash_domain, hashing::DomainSeparation};

/// App Version parameters
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Instruction {
    GetVersion,
    Sign,
    Commitment,
    BPData,
}

impl TryFrom<u8> for Instruction {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x01 => Ok(Self::GetVersion),
            0x02 => Ok(Self::Sign),
            0x03 => Ok(Self::Commitment),
            0x04 => Ok(Self::BPData),
            _ => Err(()),
        }
    }
}

hash_domain!(TransactionHashDomain, "com.tari.base_layer.core.transactions", 0);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = io::Comm::new();
    init();
    ui::SingleMessage::new("Tari test app").show();
    loop {
        match comm.next_event() {
            io::Event::Button(ButtonEvent::BothButtonsRelease) => nanos_sdk::exit_app(0),
            io::Event::Button(ButtonEvent::RightButtonRelease) => {
                display_infos();
            },
            io::Event::Button(ButtonEvent::LeftButtonPress) => {},
            io::Event::Button(_) => {},
            io::Event::Command(Instruction::GetVersion) => {
                let name_bytes = NAME.as_bytes();
                let version_bytes = VERSION.as_bytes();
                comm.append(&[1]); // Format
                comm.append(&[name_bytes.len() as u8]);
                comm.append(name_bytes);
                comm.append(&[version_bytes.len() as u8]);
                comm.append(version_bytes);
                comm.append(&[0]); // No flags
                comm.reply_ok();
            },
            io::Event::Command(Instruction::Sign) => {
                // first bytes are instruction details

                let offset = 5;
                let challenge = ArrayString::<32>::from_bytes(comm.get(offset, offset + 32));
                let k = RistrettoSecretKey::random(&mut LedgerRng);
                let n = RistrettoSecretKey::random(&mut LedgerRng);
                let signature = RistrettoSchnorr::sign_raw(&k, n, challenge.bytes()).unwrap();
                let public_key = RistrettoPublicKey::from_secret_key(&k);
                let sig = signature.get_signature().as_bytes();
                let nonce = signature.get_public_nonce().as_bytes();

                let hash = DomainSeparatedConsensusHasher::<TransactionHashDomain>::new("script_challenge")
                    .chain(&public_key)
                    .finalize();

                comm.append(&[1]); // version
                comm.append(public_key.as_bytes());
                comm.append(sig);
                comm.append(nonce);
                comm.reply_ok();
            },
            io::Event::Command(Instruction::Commitment) => {
                // first bytes are instruction details
                let offset = 5;
                let mut value_bytes = [0u8; 8];
                value_bytes.clone_from_slice(comm.get(offset, offset + 8));
                let value = u64::from_le_bytes(value_bytes);
                let path: [u32; 5] = nanos_sdk::ecc::make_bip32_path(b"m/44'/535348'/0'/0/0");
                let mut raw_key = [0u8; 32];
                unsafe {
                    os_perso_derive_node_bip32(
                        CurvesId::Ed25519 as u8,
                        (&path).as_ptr(),
                        (&path).len() as u32,
                        (&mut raw_key).as_mut_ptr(),
                        core::ptr::null_mut(),
                    )
                };
                let k = RistrettoSecretKey::from_bytes(&raw_key).unwrap();
                let com_factories = ExtendedPedersenCommitmentFactory::default();
                let commitment = com_factories.commit_value(&k, value);
                comm.append(&[1]); // version
                comm.append(commitment.as_bytes());
                // comm.append(pkey.as_ref());
                comm.reply_ok();
            },
            io::Event::Command(Instruction::BPData) => {
                // first bytes are instruction details
                let offset = 5;
                let mut scalar_bytes = [0u8; 32];
                scalar_bytes.clone_from_slice(comm.get(offset, offset + 32));
                let scalar = Scalar::from_bits(scalar_bytes);
                let path: [u32; 5] = ecc::make_bip32_path(b"m/44'/535348'/0'/0/0");
                let mut raw_key = [0u8; 32];
                unsafe {
                    os_perso_derive_node_bip32(
                        CurvesId::Ed25519 as u8,
                        (&path).as_ptr(),
                        (&path).len() as u32,
                        (&mut raw_key).as_mut_ptr(),
                        core::ptr::null_mut(),
                    )
                };
                let k = RistrettoSecretKey::from_bytes(&raw_key).unwrap();
                let mut k_scalar_bytes = [0u8; 32];
                k_scalar_bytes.clone_from_slice(k.as_bytes());
                let k_scalar = Scalar::from_bits(k_scalar_bytes);
                let blinded = k_scalar * &scalar;
                comm.append(&[1]); // version
                comm.append(blinded.as_bytes());
                comm.reply_ok();
            },
            io::Event::Ticker => {},
        }
    }
}
pub struct DomainSeparatedConsensusHasher<M>(PhantomData<M>);

impl<M: DomainSeparation> DomainSeparatedConsensusHasher<M> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(label: &'static str) -> ConsensusHasher<Blake256> {
        let mut digest = Blake256::new();
        M::add_domain_separation_tag(&mut digest, label);
        ConsensusHasher::from_digest(digest)
    }
}

use digest::{consts::U32, Digest};
#[derive(Clone)]
pub struct ConsensusHasher<D> {
    writer: WriteHashWrapper<D>,
}

impl<D: Digest> ConsensusHasher<D> {
    fn from_digest(digest: D) -> Self {
        Self {
            writer: WriteHashWrapper(digest),
        }
    }
}

impl<D> ConsensusHasher<D>
where D: Digest<OutputSize = U32>
{
    pub fn finalize(self) -> [u8; 32] {
        self.writer.0.finalize().into()
    }

    pub fn update_consensus_encode<T: BorshSerialize>(&mut self, data: &T) {
        BorshSerialize::serialize(data, &mut self.writer)
            .expect("Incorrect implementation of BorshSerialize encountered. Implementations MUST be infallible.");
    }

    pub fn chain<T: BorshSerialize>(mut self, data: &T) -> Self {
        self.update_consensus_encode(data);
        self
    }
}

#[derive(Clone)]
struct WriteHashWrapper<D>(D);

impl<D: Digest> Write for WriteHashWrapper<D> {
    fn write(&mut self, buf: &[u8]) -> BorshResult<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> BorshResult<()> {
        Ok(())
    }
}

/// Display global information about the app:
/// - Current number of passwords stored
/// - App Version
fn display_infos() {
    let stored_n = *b"Our test app";
    let stored_str = unsafe { core::str::from_utf8_unchecked(&stored_n) };
    const APP_VERSION_STR: &str = concat!(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    ui::Menu::new(&[APP_VERSION_STR, stored_str]).show();
}

#[derive(Clone, Copy)]
pub struct ArrayString<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> ArrayString<N> {
    /// Create an empty string
    pub const fn new() -> ArrayString<N> {
        ArrayString { bytes: [0; N] }
    }

    /// Set the string from an array of bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Array of bytes. Max size is N. The string must not have null bytes, but the last bytes of the array
    ///   can be null (zero padding).
    pub fn set_from_bytes(&mut self, bytes: &[u8]) {
        let mut len = bytes.len();
        while (len > 0) && (bytes[len - 1]) == 0 {
            len -= 1;
        }
        assert!(len <= N);
        self.bytes[..len].copy_from_slice(&bytes[..len]);
        for i in len..N {
            self.bytes[i] = 0;
        }
    }

    /// Returns an ArrayString initialized from bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Array of bytes. Max size is N. Must not have null bytes.
    pub fn from_bytes(bytes: &[u8]) -> ArrayString<N> {
        let mut result = ArrayString::new();
        result.set_from_bytes(bytes);
        result
    }

    /// Number of bytes in the string.
    pub fn len(&self) -> usize {
        let mut size = N;
        while (size > 0) && (self.bytes[size - 1] == 0) {
            size -= 1;
        }
        size
    }

    /// Return the bytes, non-mutable!
    pub fn bytes(&self) -> &[u8; N] {
        &self.bytes
    }

    /// Return the bytes as a str
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len()]).unwrap()
    }
}

use core::mem::MaybeUninit;

use critical_section::RawRestoreState;
use nanos_sdk::{bindings::os_perso_derive_node_bip32, ecc::CurvesId};

/// Allocator heap size
const HEAP_SIZE: usize = 1024 * 26;

/// Statically allocated heap memory
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

/// Bind global allocator
#[global_allocator]
static HEAP: embedded_alloc::Heap = embedded_alloc::Heap::empty();

/// Error handler for allocation
#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    ui::SingleMessage::new("oom").show_and_wait();

    nanos_sdk::exit_app(250)
}

/// Initialise allocator
pub fn init() {
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

struct MyCriticalSection;
critical_section::set_impl!(MyCriticalSection);

unsafe impl critical_section::Impl for MyCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        // nothing, it's all good, don't worry bout it
    }

    unsafe fn release(_token: RawRestoreState) {
        // nothing, it's all good, don't worry bout it
    }
}
