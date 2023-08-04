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

extern crate alloc;
use core::marker::PhantomData;

use borsh::{
    maybestd::io::{Result as BorshResult, Write},
    BorshSerialize,
};
use digest::Update;
use nanos_sdk::{buttons::ButtonEvent, io};
use nanos_ui::ui;
use tari_crypto::{
    commitment::HomomorphicCommitmentFactory,
    hash_domain,
    hashing::DomainSeparation,
    keys::PublicKey,
    ristretto::{
        pedersen::extended_commitment_factory::ExtendedPedersenCommitmentFactory,
        RistrettoPublicKey,
        RistrettoSchnorr,
        RistrettoSecretKey,
    },
    tari_utilities::ByteArray,
};
use blake2::Blake2b;
use crate::alloc::string::{String, ToString};
nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

/// App Version parameters
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Instruction {
    GetVersion,
    Sign,
    Commitment,
    GetPublicKey,
    Exit,
    BadInstruction(u8),
}

impl From<io::ApduHeader> for Instruction {
    fn from(header: io::ApduHeader) -> Instruction {
        match header.ins {
            0x01 => Self::GetVersion,
            0x02 => Self::Sign,
            0x03 => Self::Commitment,
            0x04 => Self::GetPublicKey,
            0x05 => Self::Exit,
            other => Self::BadInstruction(other),
        }
    }
}

hash_domain!(TransactionHashDomain, "com.tari.base_layer.core.transactions", 0);

use nanos_sdk::io::{ApduHeader, Reply, StatusWords};

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = io::Comm::new();
    init();
    ui::SingleMessage::new("Tari test app").show();
    loop {
        let event = comm.next_event::<ApduHeader>();
        match event {
            io::Event::Button(ButtonEvent::BothButtonsRelease) => nanos_sdk::exit_app(0),
            io::Event::Button(ButtonEvent::RightButtonRelease) => {
                display_infos();
            },
            io::Event::Button(ButtonEvent::LeftButtonPress) => {},
            io::Event::Button(_) => {},
            io::Event::Command(apdu_header) => match handle_apdu(&mut comm, apdu_header.into()) {
                Ok(()) => comm.reply_ok(),
                Err(e) => comm.reply(e),
            },
            io::Event::Ticker => {},
        }
    }
}

// Convert a u64 to a string without using the standard library
fn u64_to_string(number: u64) -> String {
    let mut buffer = [0u8; 20]; // Maximum length for a 64-bit integer (including null terminator)
    let mut pos = 0;

    if number == 0 {
        buffer[pos] = b'0';
        pos += 1;
    } else {
        let mut num = number;

        let mut digits = [0u8; 20];
        let mut num_digits = 0;

        while num > 0 {
            digits[num_digits] = b'0' + (num % 10) as u8;
            num /= 10;
            num_digits += 1;
        }

        while num_digits > 0 {
            num_digits -= 1;
            buffer[pos] = digits[num_digits];
            pos += 1;
        }
    }

    String::from_utf8_lossy(&buffer[..pos]).to_string()
}

// Convert a single byte to a hex string
fn byte_to_hex(byte: u8) -> String {
    const HEX_CHARS: [u8; 16] = *b"0123456789abcdef";
    let hex = [HEX_CHARS[(byte >> 4) as usize], HEX_CHARS[(byte & 0x0F) as usize]];
    String::from_utf8_lossy(&hex).to_string()
}

// Get a raw key from the BIP32 path
fn get_raw_key(path: [u32; 5]) -> [u8; 32] {
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
    raw_key
}

fn handle_apdu(comm: &mut io::Comm, instruction: Instruction) -> Result<(), Reply> {
    if comm.rx == 0 {
        return Err(io::StatusWords::NothingReceived.into());
    }

    match instruction {
        Instruction::GetVersion => {
            ui::SingleMessage::new("GetVersion...").show();
            let name_bytes = NAME.as_bytes();
            let version_bytes = VERSION.as_bytes();
            comm.append(&[1]); // Format
            comm.append(&[name_bytes.len() as u8]);
            comm.append(name_bytes);
            comm.append(&[version_bytes.len() as u8]);
            comm.append(version_bytes);
            comm.append(&[0]); // No flags
            ui::SingleMessage::new("GetVersion... Done").show();
            comm.reply_ok();
        },
        Instruction::Sign => {
            ui::SingleMessage::new("Sign...").show();
            // first 5 bytes are instruction details
            let offset = 5;
            let challenge = ArrayString::<32>::from_bytes(comm.get(offset, offset + 32));
            let path: [u32; 5] = nanos_sdk::ecc::make_bip32_path(b"m/44'/535348'/0'/0/0");

            let raw_key = get_raw_key(path);
            let k = RistrettoSecretKey::from_bytes(&raw_key).unwrap();
            let n = Blake2b::<U32>::new().chain(k.as_bytes()).finalize().to_vec();
            let n = RistrettoSecretKey::from_bytes(&n).unwrap();
            let public_key = RistrettoPublicKey::from_secret_key(&k);
            let public_nonce = RistrettoPublicKey::from_secret_key(&n);
            let hash = DomainSeparatedConsensusHasher::<TransactionHashDomain>::new("script_challenge")
                .chain(&public_key)
                .chain(&public_nonce)
                .chain(challenge.bytes())
                .finalize();
            let signature = RistrettoSchnorr::sign_raw(&k, n, &hash).unwrap();
            let sig = signature.get_signature().as_bytes();
            let nonce = signature.get_public_nonce().as_bytes();

            comm.append(&[1]); // version
            comm.append(public_key.as_bytes());
            comm.append(sig);
            comm.append(nonce);
            ui::SingleMessage::new("Sign... Done").show();
            comm.reply_ok();
        },
        Instruction::Commitment => {
            ui::SingleMessage::new("Commitment...").show();
            // first 5 bytes are instruction details
            let offset = 5;
            let mut value_bytes = [0u8; 8];
            value_bytes.clone_from_slice(comm.get(offset, offset + 8));
            let value = u64::from_le_bytes(value_bytes);
            let path: [u32; 5] = nanos_sdk::ecc::make_bip32_path(b"m/44'/535348'/0'/0/0");

            let raw_key = get_raw_key(path);
            let k = RistrettoSecretKey::from_bytes(&raw_key).unwrap();
            let com_factories = ExtendedPedersenCommitmentFactory::default();
            let commitment = com_factories.commit_value(&k, value);
            comm.append(&[1]); // version
            comm.append(commitment.as_bytes());
            ui::SingleMessage::new("Commitment... Done").show();
            comm.reply_ok();
        },
        Instruction::GetPublicKey => {
            ui::SingleMessage::new("GetPublicKey...").show();
            // first 5 bytes are instruction details
            let offset = 5;
            let mut account_bytes = [0u8; 8];
            account_bytes.clone_from_slice(comm.get(offset, offset + 8));
            let account = u64_to_string(u64::from_le_bytes(account_bytes));
            let mut address_index_bytes = [0u8; 8];
            address_index_bytes.clone_from_slice(comm.get(offset + 8, offset + 8 + 8));
            let address_index = u64_to_string(u64::from_le_bytes(address_index_bytes));

            let mut bip32_path = "m/44'/535348'/".to_string();
            bip32_path.push_str(&account);
            bip32_path.push_str(&"'/0/");
            bip32_path.push_str(&address_index);
            let path: [u32; 5] = nanos_sdk::ecc::make_bip32_path(bip32_path.as_bytes());

            let raw_key = get_raw_key(path);
            let k = RistrettoSecretKey::from_bytes(&raw_key).unwrap();
            let pk = RistrettoPublicKey::from_secret_key(&k);
            comm.append(&[1]); // version
            comm.append(pk.as_bytes());
            ui::SingleMessage::new("GetPublicKey... Done").show();
            comm.reply_ok();
        },
        Instruction::BadInstruction(val) => {
            let mut error = "BadInstruction...! (".to_string();
            error.push_str(&byte_to_hex(val));
            error.push_str(&")");
            ui::SingleMessage::new(&error).show();
            return Err(StatusWords::BadIns.into());
        },
        Instruction::Exit => {
            ui::SingleMessage::new("Exit").show();
            comm.reply_ok();
            nanos_sdk::exit_app(0)
        },
    }
    Ok(())
}

pub struct DomainSeparatedConsensusHasher<M>(PhantomData<M>);

impl<M: DomainSeparation> DomainSeparatedConsensusHasher<M> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(label: &'static str) -> ConsensusHasher<Blake2b<U32>> {
        let mut digest = Blake2b::<U32>::new();
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
