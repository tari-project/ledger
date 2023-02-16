// Copyright 2019. The Tari Project
// SPDX-License-Identifier: BSD-3-Clause

//! The Tari-compatible implementation of Ristretto based on the curve25519-dalek implementation

use core::{
    borrow::Borrow,
    fmt,
    ops::{Add, Mul, Sub},
};

use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_TABLE,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};
use nanos_sdk::random::LedgerRng;
use once_cell::unsync::OnceCell;
use rand_core::RngCore;

use crate::errors::Error;

#[derive(Clone, Default)]
pub struct RistrettoSecretKey(pub(crate) Scalar);

const SCALAR_LENGTH: usize = 32;

//----------------------------------   RistrettoSecretKey Debug --------------------------------------------//
impl fmt::Debug for RistrettoSecretKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RistrettoSecretKey(***)")
    }
}

impl RistrettoSecretKey {
    pub fn key_length() -> usize {
        SCALAR_LENGTH
    }

    // Return a random secret key on the `ristretto255` curve using the supplied CSPRNG.
    pub fn random() -> Self {
        let mut scalar_bytes = [0u8; 64];
        LedgerRng.fill_bytes(&mut scalar_bytes);
        let scalar = Scalar::from_bytes_mod_order_wide(&scalar_bytes);
        RistrettoSecretKey(scalar)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<RistrettoSecretKey, Error>
    where Self: Sized {
        if bytes.len() != 32 {
            return Err(Error::IncorrectByteLength);
        }
        let mut a = [0u8; 32];
        a.copy_from_slice(bytes);
        let k = Scalar::from_bytes_mod_order(a);
        Ok(RistrettoSecretKey(k))
    }

    /// Return the byte array for the secret key in little-endian order
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

//----------------------------------   RistrettoSecretKey Mul / Add / Sub --------------------------------------------//

impl<'a, 'b> Mul<&'b RistrettoPublicKey> for &'a RistrettoSecretKey {
    type Output = RistrettoPublicKey;

    fn mul(self, rhs: &'b RistrettoPublicKey) -> RistrettoPublicKey {
        let p = self.0 * rhs.point;
        RistrettoPublicKey::new_from_pk(p)
    }
}

impl<'a, 'b> Add<&'b RistrettoSecretKey> for &'a RistrettoSecretKey {
    type Output = RistrettoSecretKey;

    fn add(self, rhs: &'b RistrettoSecretKey) -> RistrettoSecretKey {
        let k = self.0 + rhs.0;
        RistrettoSecretKey(k)
    }
}

impl<'a, 'b> Sub<&'b RistrettoSecretKey> for &'a RistrettoSecretKey {
    type Output = RistrettoSecretKey;

    fn sub(self, rhs: &'b RistrettoSecretKey) -> RistrettoSecretKey {
        RistrettoSecretKey(self.0 - rhs.0)
    }
}

define_add_variants!(
    LHS = RistrettoSecretKey,
    RHS = RistrettoSecretKey,
    Output = RistrettoSecretKey
);
define_sub_variants!(
    LHS = RistrettoSecretKey,
    RHS = RistrettoSecretKey,
    Output = RistrettoSecretKey
);
define_mul_variants!(
    LHS = RistrettoSecretKey,
    RHS = RistrettoPublicKey,
    Output = RistrettoPublicKey
);

//---------------------------------------------      Conversions     -------------------------------------------------//

impl From<u64> for RistrettoSecretKey {
    fn from(v: u64) -> Self {
        let s = Scalar::from(v);
        RistrettoSecretKey(s)
    }
}

impl From<Scalar> for RistrettoSecretKey {
    fn from(s: Scalar) -> Self {
        RistrettoSecretKey(s)
    }
}

//---------------------------------------------      Borrow impl     -------------------------------------------------//

impl<'a> Borrow<Scalar> for &'a RistrettoSecretKey {
    fn borrow(&self) -> &Scalar {
        &self.0
    }
}

//--------------------------------------------- Ristretto Public Key -------------------------------------------------//

#[derive(Clone)]
pub struct RistrettoPublicKey {
    point: RistrettoPoint,
    compressed: OnceCell<CompressedRistretto>,
}

impl RistrettoPublicKey {
    // Private constructor
    pub(super) fn new_from_pk(pk: RistrettoPoint) -> Self {
        Self {
            point: pk,
            compressed: OnceCell::new(),
        }
    }

    pub fn new_from_compressed(compressed: CompressedRistretto) -> Option<Self> {
        compressed.decompress().map(|point| Self {
            compressed: compressed.into(),
            point,
        })
    }

    /// Return the embedded RistrettoPoint representation
    pub fn point(&self) -> RistrettoPoint {
        self.point
    }

    pub(super) fn compressed(&self) -> &CompressedRistretto {
        self.compressed.get_or_init(|| self.point.compress())
    }

    /// Generates a new Public key from the given secret key
    pub fn from_secret_key(k: &RistrettoSecretKey) -> RistrettoPublicKey {
        let pk = &k.0 * &RISTRETTO_BASEPOINT_TABLE;
        RistrettoPublicKey::new_from_pk(pk)
    }

    // pub fn batch_mul(scalars: &[RistrettoSecretKey], points: &[Self]) -> Self {
    //     let p = points.iter().map(|p| &p.point);
    //     let s = scalars.iter().map(|k| &k.0);
    //     let p = RistrettoPoint::multiscalar_mul(s, p);
    //     RistrettoPublicKey::new_from_pk(p)
    // }

    // pub fn to_hex(self) -> String {
    //     let bytes = self.as_bytes();
    //     let mut s = String::with_capacity(bytes.len() * 2);
    //     for byte in bytes {
    //         write!(&mut s, "{:02x}", byte).expect("Unable to write");
    //     }
    //     s
    // }

    /// Create a new `RistrettoPublicKey` instance form the given byte array. The constructor returns errors under
    /// the following circumstances:
    /// * The byte array is not exactly 32 bytes
    /// * The byte array does not represent a valid (compressed) point on the ristretto255 curve
    pub fn from_bytes(bytes: &[u8]) -> Result<RistrettoPublicKey, Error>
    where Self: Sized {
        // Check the length here, because The Ristretto constructor panics rather than returning an error
        if bytes.len() != 32 {
            return Err(Error::IncorrectByteLength);
        }
        let compressed = CompressedRistretto::from_slice(bytes);
        match RistrettoPublicKey::new_from_compressed(compressed) {
            Some(p) => Ok(p),
            None => Err(Error::ConversionError),
        }
    }

    /// Return the little-endian byte array representation of the compressed public key
    pub fn as_bytes(&self) -> &[u8] {
        self.compressed().as_bytes()
    }
}

//----------------------------------    Ristretto Public Key Default   -----------------------------------------------//

impl Default for RistrettoPublicKey {
    fn default() -> Self {
        RistrettoPublicKey::new_from_pk(RistrettoPoint::default())
    }
}

//----------------------------------         PublicKey Add / Sub / Mul   ---------------------------------------------//

impl<'a, 'b> Add<&'b RistrettoPublicKey> for &'a RistrettoPublicKey {
    type Output = RistrettoPublicKey;

    fn add(self, rhs: &'b RistrettoPublicKey) -> RistrettoPublicKey {
        let p_sum = self.point + rhs.point;
        RistrettoPublicKey::new_from_pk(p_sum)
    }
}

impl<'a, 'b> Sub<&'b RistrettoPublicKey> for &'a RistrettoPublicKey {
    type Output = RistrettoPublicKey;

    fn sub(self, rhs: &RistrettoPublicKey) -> RistrettoPublicKey {
        let p_sum = self.point - rhs.point;
        RistrettoPublicKey::new_from_pk(p_sum)
    }
}

impl<'a, 'b> Mul<&'b RistrettoSecretKey> for &'a RistrettoPublicKey {
    type Output = RistrettoPublicKey;

    fn mul(self, rhs: &'b RistrettoSecretKey) -> RistrettoPublicKey {
        let p = rhs.0 * self.point;
        RistrettoPublicKey::new_from_pk(p)
    }
}

impl<'a, 'b> Mul<&'b RistrettoSecretKey> for &'a RistrettoSecretKey {
    type Output = RistrettoSecretKey;

    fn mul(self, rhs: &'b RistrettoSecretKey) -> RistrettoSecretKey {
        let p = &rhs.0 * &self.0;
        RistrettoSecretKey(p)
    }
}

define_add_variants!(
    LHS = RistrettoPublicKey,
    RHS = RistrettoPublicKey,
    Output = RistrettoPublicKey
);
define_sub_variants!(
    LHS = RistrettoPublicKey,
    RHS = RistrettoPublicKey,
    Output = RistrettoPublicKey
);
define_mul_variants!(
    LHS = RistrettoPublicKey,
    RHS = RistrettoSecretKey,
    Output = RistrettoPublicKey
);
define_mul_variants!(
    LHS = RistrettoSecretKey,
    RHS = RistrettoSecretKey,
    Output = RistrettoSecretKey
);

//----------------------------------         PublicKey From implementations      -------------------------------------//

impl From<RistrettoSecretKey> for Scalar {
    fn from(k: RistrettoSecretKey) -> Self {
        k.0
    }
}

impl From<RistrettoPublicKey> for RistrettoPoint {
    fn from(pk: RistrettoPublicKey) -> Self {
        pk.point
    }
}

impl From<&RistrettoPublicKey> for RistrettoPoint {
    fn from(pk: &RistrettoPublicKey) -> Self {
        pk.point
    }
}

impl From<RistrettoPublicKey> for CompressedRistretto {
    fn from(pk: RistrettoPublicKey) -> Self {
        *pk.compressed()
    }
}
