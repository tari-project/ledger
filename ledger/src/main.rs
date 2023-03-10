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
// #![feature(alloc_error_handler)]

#[macro_use]
mod macros;
//mod blake2;
//mod errors;
//mod ristretto_keys;
//mod schnorr;

use core::convert::TryFrom;

use nanos_sdk::{buttons::ButtonEvent, io};
use nanos_ui::ui;

use tari_crypto::{
   ristretto::{
        //pedersen::{extended_commitment_factory::ExtendedPedersenCommitmentFactory},
        RistrettoPublicKey,
        //RistrettoSchnorr,
       // RistrettoSecretKey,
   },
};

nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

/// App Version parameters
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Instruction {
    GetVersion,
    Sign,
}

impl TryFrom<u8> for Instruction {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x01 => Ok(Self::GetVersion),
            0x02 => Ok(Self::Sign),
            _ => Err(()),
        }
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = io::Comm::new();
    ui::SingleMessage::new("Tari test app").show();
    loop {
        match comm.next_event() {
            io::Event::Button(ButtonEvent::BothButtonsRelease) => nanos_sdk::exit_app(0),
            io::Event::Button(ButtonEvent::RightButtonRelease) => {
                display_infos();
            },
            io::Event::Button(ButtonEvent::LeftButtonPress) => {
            },
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
                panic!("pie");
                let challenge = ArrayString::<32>::from_bytes(comm.get(offset, offset + 32));

               /* let k = RistrettoSecretKey::random();
                let signature = RistrettoSchnorr::sign_message(&k, challenge.bytes()).unwrap();
                let public_key = RistrettoPublicKey::from_secret_key(&k);
                let sig = signature.get_signature().as_bytes();
                let nonce = signature.get_public_nonce().as_bytes();
                let spending_key = RistrettoSecretKey::random();
                let com_factories = ExtendedPedersenCommitmentFactory::default();
                let commitment = com_factories.commit_value(&spending_key, 50);

                comm.append(&[1]); // version
                comm.append(public_key.as_bytes());
                comm.append(sig);
                comm.append(nonce);*/
                comm.reply_ok();
            },
            io::Event::Ticker => {},
        }
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

/// Allocator heap size
const HEAP_SIZE: usize = 1024;

/// Statically allocated heap memory
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

/// Bind global allocator
#[global_allocator]
static HEAP: embedded_alloc::Heap = embedded_alloc::Heap::empty();

/// Error handler for allocation
// #[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    nanos_sdk::exit_app(250)
}

/// Initialise allocator
pub fn init() {
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}