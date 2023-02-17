// Copyright 2020. The Tari Project
// SPDX-License-Identifier: BSD-3-Clause

//! A convenience wrapper produce 256 bit hashes from Blake2b

use blake2::{digest::VariableOutput, VarBlake2b};
use digest::{
    consts::{U32, U64},
    generic_array::{typenum::Unsigned, GenericArray},
    FixedOutput,
    Reset,
    Update,
};

/// A convenience wrapper produce 256 bit hashes from Blake2b
#[derive(Clone, Debug)]
pub struct Blake256(VarBlake2b);

impl Blake256 {
    // /// Constructs a `Blake256` hashing context with parameters that allow hash keying, salting and personalization.
    // pub fn with_params(key: &[u8], salt: &[u8], persona: &[u8]) -> Result<Self, HashError> {
    //     Self::with_params_var_size(key, salt, persona, <Self as FixedOutput>::OutputSize::USIZE)
    // }
    //
    // /// Constructs a `Blake256` hashing context with an explicitly specified output size.
    // pub fn with_params_var_size(
    //     key: &[u8],
    //     salt: &[u8],
    //     persona: &[u8],
    //     output_size: usize,
    // ) -> Result<Self, HashError> {
    //     if key.len() > 64 || salt.len() > 16 || persona.len() > 16 || output_size < 1 || output_size >
    // U64::to_usize() {         Err(HashError::WrongLength)
    //     } else {
    //         Ok(Self(VarBlake2b::with_params(key, salt, persona, output_size)))
    //     }
    // }
}

impl Default for Blake256 {
    fn default() -> Self {
        let h = VariableOutput::new(<Self as FixedOutput>::OutputSize::USIZE).unwrap();
        Blake256(h)
    }
}

impl FixedOutput for Blake256 {
    type OutputSize = U32;

    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        self.0.finalize_variable(|res| out.copy_from_slice(res));
    }

    fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        self.0.finalize_variable_reset(|res| out.copy_from_slice(res));
    }
}

impl Reset for Blake256 {
    fn reset(&mut self) {
        (self.0).reset()
    }
}

impl Update for Blake256 {
    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.0.update(data);
    }
}
// pub enum HashError {
//     WrongLength,
// }
