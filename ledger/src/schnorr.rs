// Copyright 2022. The Tari Project
// SPDX-License-Identifier: BSD-3-Clause

//! Schnorr Signature module
//! This module defines generic traits for handling the digital signature operations, agnostic
//! of the underlying elliptic curve implementation

use core::ops::Add;

use crate::{
    errors::*,
    ristretto_keys::{RistrettoPublicKey, RistrettoSecretKey},
};

/// # SchnorrSignature
///
/// Provides a Schnorr signature that is agnostic to a specific public/private key implementation.
/// For a concrete implementation see [RistrettoSchnorr](crate::ristretto::RistrettoSchnorr).
///
/// More details on Schnorr signatures can be found at [TLU](https://tlu.tarilabs.com/cryptography/introduction-schnorr-signatures).
#[allow(non_snake_case)]
#[derive(Clone)]
pub struct SchnorrSignature {
    public_nonce: RistrettoPublicKey,
    signature: RistrettoSecretKey,
}

impl SchnorrSignature {
    /// Create a new `SchnorrSignature`.
    pub fn new(public_nonce: RistrettoPublicKey, signature: RistrettoSecretKey) -> Self {
        SchnorrSignature {
            public_nonce,
            signature,
        }
    }

    /// Sign a challenge with the given `secret` and private `nonce`. Returns an SchnorrSignatureError if `<K as
    /// ByteArray>::from_bytes(challenge)` returns an error.
    ///
    /// WARNING: The public key and nonce are NOT bound to the challenge. This method assumes that the challenge has
    /// been constructed such that all commitments are already included in the challenge.
    ///
    /// If you want a simple API that binds the nonce and public key to the message, use [`sign_message`] instead.
    pub fn sign_raw(secret: &RistrettoSecretKey, nonce: RistrettoSecretKey, challenge: &[u8]) -> Result<Self, Error> {
        // s = r + e.k
        let e = match RistrettoSecretKey::from_bytes(challenge) {
            Ok(e) => e,
            Err(_) => return Err(Error::InvalidChallenge),
        };
        let public_nonce = RistrettoPublicKey::from_secret_key(&nonce);
        let ek = e * secret;
        let s = ek + nonce;
        Ok(Self::new(public_nonce, s))
    }

    /// Returns a reference to the `s` signature component.
    pub fn get_signature(&self) -> &RistrettoSecretKey {
        &self.signature
    }

    /// Returns a reference to the public nonce component.
    pub fn get_public_nonce(&self) -> &RistrettoPublicKey {
        &self.public_nonce
    }
}

impl<'a, 'b> Add<&'b SchnorrSignature> for &'a SchnorrSignature {
    type Output = SchnorrSignature;

    fn add(self, rhs: &'b SchnorrSignature) -> SchnorrSignature {
        let r_sum = self.get_public_nonce() + rhs.get_public_nonce();
        let s_sum = self.get_signature() + rhs.get_signature();
        SchnorrSignature::new(r_sum, s_sum)
    }
}

impl<'a> Add<SchnorrSignature> for &'a SchnorrSignature {
    type Output = SchnorrSignature;

    fn add(self, rhs: SchnorrSignature) -> SchnorrSignature {
        let r_sum = self.get_public_nonce() + rhs.get_public_nonce();
        let s_sum = self.get_signature() + rhs.get_signature();
        SchnorrSignature::new(r_sum, s_sum)
    }
}

impl Default for SchnorrSignature {
    fn default() -> Self {
        SchnorrSignature::new(RistrettoPublicKey::default(), RistrettoSecretKey::default())
    }
}
