use core::marker::PhantomData;
use std::{thread::sleep, time::Duration};

use borsh::{
    maybestd::io::{Result as BorshResult, Write},
    BorshSerialize,
};
use digest::Digest;
use ledger_transport::APDUCommand;
use ledger_transport_hid::{hidapi::HidApi, TransportNativeHID};
use ledger_zondax_generic::{App, AppExt};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use tari_crypto::{
    hash::blake2::Blake256,
    hash_domain,
    hashing::DomainSeparation,
    keys::{PublicKey, SecretKey},
    ristretto::{pedersen::PedersenCommitment, RistrettoPublicKey, RistrettoSchnorr, RistrettoSecretKey},
    tari_utilities::{hex::Hex, ByteArray},
};

fn hidapi() -> &'static HidApi {
    static HIDAPI: Lazy<HidApi> = Lazy::new(|| HidApi::new().expect("unable to get HIDAPI"));

    &HIDAPI
}
struct Tari;
impl App for Tari {
    const CLA: u8 = 0x0;
}
hash_domain!(TransactionHashDomain, "com.tari.base_layer.core.transactions", 0);

fn main() {
    // GetVersion
    let command = APDUCommand {
        cla: 0x80,
        ins: 0x01, // GetVersion
        p1: 0x00,
        p2: 0x00,
        data: vec![0],
    };
    let message = vec![0];
    let ledger = TransportNativeHID::new(hidapi()).expect("Could not get a device");

    // use device info command that works in the dashboard
    let result = match futures::executor::block_on(Tari::send_chunks(&ledger, command, &message)) {
        Ok(result) => result,
        Err(e) => {
            println!("Error: {}", e);
            return;
        },
    };
    let data_len = result.data()[1] as usize;
    let name = &result.data()[2..data_len + 2];
    let name = std::str::from_utf8(name).unwrap();
    println!();
    println!("name: {}", name);
    let package_len = result.data()[data_len + 2] as usize;
    let package = &result.data()[data_len + 3..data_len + package_len + 3];
    let package = std::str::from_utf8(package).unwrap();
    println!("package version: {}", package);
    println!();

    // Sign
    sleep(Duration::from_millis(2000));
    let challenge = RistrettoSecretKey::random(&mut OsRng);
    let command2 = APDUCommand {
        cla: 0x80,
        ins: 0x02, // Sign
        p1: 0x00,
        p2: 0x00,
        data: challenge.as_bytes().clone(),
    };
    let result = ledger.exchange(&command2).unwrap();

    let public_key = &result.data()[1..33];
    let public_key = RistrettoPublicKey::from_bytes(public_key).unwrap();

    let sig = &result.data()[33..65];
    let sig = RistrettoSecretKey::from_bytes(sig).unwrap();

    let nonce = &result.data()[65..97];
    let nonce = RistrettoPublicKey::from_bytes(nonce).unwrap();

    let signature = RistrettoSchnorr::new(nonce.clone(), sig);
    let mut challenge_bytes = [0u8; 32];
    challenge_bytes.clone_from_slice(challenge.as_bytes());
    let hash = DomainSeparatedConsensusHasher::<TransactionHashDomain>::new("script_challenge")
        .chain(&public_key)
        .chain(&nonce)
        .chain(&challenge_bytes)
        .finalize();
    let e = RistrettoSecretKey::from_bytes(&hash).unwrap();
    println!("challenge:  {}", e.to_hex());
    println!("signature:  {}", signature.get_signature().to_hex());
    println!("public key: {}", public_key.to_hex());

    let result = signature.verify(&public_key, &e);
    println!("sign:       {}", result);
    println!(" ");

    // Commitment
    sleep(Duration::from_millis(2000));
    let value: u64 = 60;
    let value_bytes = value.to_le_bytes();
    let command3 = APDUCommand {
        cla: 0x80,
        ins: 0x03, // Commitment
        p1: 0x00,
        p2: 0x00,
        data: value_bytes.as_bytes().clone(),
    };
    let result = ledger.exchange(&command3).unwrap();

    let commitment = &result.data()[1..33];
    let commitment = PedersenCommitment::from_bytes(commitment).unwrap();
    println!("commitment: {}", commitment.to_hex());
    println!();

    // GetPublicKey
    sleep(Duration::from_millis(2000));
    let account_k = RistrettoSecretKey::random(&mut OsRng);
    let account_pk = RistrettoPublicKey::from_secret_key(&account_k);
    let account_pk = &account_pk.as_bytes()[0..8].to_vec().to_hex(); // We only use the 1st 8 bytes
    let account_pk = u64::from_str_radix(account_pk, 16).unwrap();
    for i in 0u64..10 {
        let address_index = i.to_le_bytes();
        let mut data = account_pk.to_le_bytes().to_vec();
        data.extend_from_slice(&address_index);
        let command4 = APDUCommand {
            cla: 0x80,
            ins: 0x04, // GetPublicKey
            p1: 0x00,
            p2: 0x00,
            data: data.clone(),
        };
        let result = ledger.exchange(&command4).unwrap();

        let bip32_path = "path:       m/44'/535348'/".to_owned() +
            &account_pk.to_string() +
            "0'/0/" +
            &u64::from_le_bytes(address_index).to_string();
        println!("{}", bip32_path);
        if result.data().len() < 33 {
            println!("Error: no data!");
        } else {
            let public_key = RistrettoPublicKey::from_bytes(&result.data()[1..33]).unwrap();
            println!("public_key: {}", public_key.to_hex());
        }
    }
    println!();

    // BadInstruction
    sleep(Duration::from_millis(2000));
    let command5 = APDUCommand {
        cla: 0x80,
        ins: 0x33, // Exit
        p1: 0x00,
        p2: 0x00,
        data: vec![0],
    };
    match ledger.exchange(&command5) {
        Ok(result) => println!("BadInstruction response ({:?})", result),
        Err(e) => println!("BadInstruction response ({})", e),
    };
    println!();

    // Exit
    sleep(Duration::from_millis(2000));
    let command6 = APDUCommand {
        cla: 0x80,
        ins: 0x05, // Exit
        p1: 0x00,
        p2: 0x00,
        data: vec![0],
    };
    match ledger.exchange(&command6) {
        Ok(result) => println!("Ledger device disconnected ({:?})", result),
        Err(e) => println!("Ledger device disconnected ({})", e),
    };
    println!();
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

use digest::consts::U32;
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
