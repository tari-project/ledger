use core::marker::PhantomData;
use std::{thread::sleep, time::Duration};

use blake2::Blake2b;
use borsh::{
    maybestd::io::{Result as BorshResult, Write},
    BorshSerialize,
};
use digest::{consts::U32, Digest};
use ledger::LedgerWallet;
use ledger_transport::APDUCommand;
use ledger_zondax_generic::{App, AppExt};
use rand::rngs::OsRng;
use tari_crypto::{
    hash_domain,
    hashing::DomainSeparation,
    keys::{PublicKey, SecretKey},
    ristretto::{pedersen::PedersenCommitment, RistrettoPublicKey, RistrettoSchnorr, RistrettoSecretKey},
    tari_utilities::{hex::Hex, ByteArray},
};

use crate::ledger::TRANSPORT;

mod ledger;

struct LedgerApp;

impl App for LedgerApp {
    const CLA: u8 = 0x0;
}

const EXPECTED_NAME: &str = "tari_ledger_demo";
const EXPECTED_PACKAGE: &str = "0.0.1";

hash_domain!(TransactionHashDomain, "com.tari.base_layer.core.transactions", 0);

enum Instruction {
    GetVersion,
    Sign,
    Commitment,
    GetPublicKey,
    GetPublicNonce,
    Exit,
}

impl Instruction {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::GetVersion => 0x01,
            Self::Sign => 0x02,
            Self::Commitment => 0x03,
            Self::GetPublicKey => 0x04,
            Self::GetPublicNonce => 0x05,
            Self::Exit => 0x06,
        }
    }
}

fn main() {
    let account_k = RistrettoSecretKey::random(&mut OsRng);
    let account_pk = RistrettoPublicKey::from_secret_key(&account_k);

    let ledger = match LedgerWallet::new(account_pk) {
        Ok(wallet) => wallet,
        Err(e) => {
            println!("\nError: {:?}", e);
            return;
        },
    };
    let binding = TRANSPORT.lock().expect("lock exists");
    let transport = binding.as_ref().expect("transport exists");

    // GetVersion
    let command = APDUCommand {
        cla: 0x80,
        ins: Instruction::GetVersion.as_u8(),
        p1: 0x00,
        p2: 0x00,
        data: vec![0],
    };
    let message = vec![0];

    // This call will exit immediately if the application not started.
    let result = match futures::executor::block_on(LedgerApp::send_chunks(transport, command, &message)) {
        Ok(result) => result,
        Err(e) => {
            println!("\nError: 'GetVersion' ({})\n", e);
            return;
        },
    };
    if result.data().is_empty() {
        println!("\nError: 'GetVersion' insufficient response! ({:?})\n", result);
        return;
    }

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
    if name != EXPECTED_NAME {
        println!(
            "Error: Unexpected '{}' application, looking for '{}'\n",
            name, EXPECTED_NAME
        );
        return;
    }
    if package != EXPECTED_PACKAGE {
        println!(
            "Error: Unexpected '{}' package version, looking for '{}'\n",
            package, EXPECTED_PACKAGE
        );
        return;
    }

    // Sign
    sleep(Duration::from_millis(2000));
    let challenge = RistrettoSecretKey::random(&mut OsRng);
    let command = APDUCommand {
        cla: 0x80,
        ins: Instruction::Sign.as_u8(),
        p1: 0x00,
        p2: 0x00,
        data: challenge.as_bytes().clone(),
    };
    let result = match transport.exchange(&command) {
        Ok(result) => result,
        Err(e) => {
            println!("\nError: Sign {}\n", e);
            return;
        },
    };
    if result.data().len() < 97 {
        println!("\nError: 'Sign' insufficient response! ({:?})\n", result);
        return;
    }

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
    let command = APDUCommand {
        cla: 0x80,
        ins: Instruction::Commitment.as_u8(),
        p1: 0x00,
        p2: 0x00,
        data: value_bytes.as_bytes().clone(),
    };
    let result = match transport.exchange(&command) {
        Ok(result) => result,
        Err(e) => {
            println!("\nError: Commitment {}\n", e);
            return;
        },
    };
    if result.data().len() < 33 {
        println!("\nError: 'Commitment' insufficient response! ({:?})\n", result);
        return;
    }

    let commitment = &result.data()[1..33];
    let commitment = PedersenCommitment::from_bytes(commitment).unwrap();
    println!("commitment: {}", commitment.to_hex());
    println!();

    // GetPublicKey
    sleep(Duration::from_millis(2000));
    let account_bytes = &ledger.account_public_key().as_bytes()[0..8].to_vec().to_hex(); // We only use the 1st 8 bytes
    let account = u64::from_str_radix(account_bytes, 16).unwrap();
    for i in 0u64..5 {
        let address_index = i.to_le_bytes();
        let mut data = account.to_le_bytes().to_vec();
        data.extend_from_slice(&address_index);
        let command = APDUCommand {
            cla: 0x80,
            ins: Instruction::GetPublicKey.as_u8(),
            p1: 0x00,
            p2: 0x00,
            data: data.clone(),
        };
        let result = match transport.exchange(&command) {
            Ok(result) => result,
            Err(e) => {
                println!("\nError: GetPublicKey {}\n", e);
                return;
            },
        };

        let bip32_path = "path:       m/44'/535348'/".to_owned() +
            &account.to_string() +
            "0'/0/" +
            &u64::from_le_bytes(address_index).to_string();
        println!("{}", bip32_path);
        if result.data().len() < 33 {
            println!("\nError: 'GetPublicKey' insufficient response! ({:?})\n", result);
            return;
        }
        let public_key = RistrettoPublicKey::from_bytes(&result.data()[1..33]).unwrap();
        println!("public_key: {}", public_key.to_hex());
    }
    println!();

    // GetPublicNonce
    sleep(Duration::from_millis(2000));
    for _i in 0..5 {
        let command = APDUCommand {
            cla: 0x80,
            ins: Instruction::GetPublicNonce.as_u8(),
            p1: 0x00,
            p2: 0x00,
            data: vec![0],
        };
        let result = match transport.exchange(&command) {
            Ok(result) => result,
            Err(e) => {
                println!("\nError: GetPublicNonce {}\n", e);
                return;
            },
        };

        if result.data().len() < 33 {
            println!("\nError: 'GetPublicNonce' insufficient response! ({:?})\n", result);
            return;
        }
        let public_key = RistrettoPublicKey::from_bytes(&result.data()[1..33]).unwrap();
        println!("public_nonce: {}", public_key.to_hex());
    }
    println!();

    // BadInstruction
    sleep(Duration::from_millis(2000));
    let command = APDUCommand {
        cla: 0x80,
        ins: 0x33, // Not defined
        p1: 0x00,
        p2: 0x00,
        data: vec![0],
    };
    match transport.exchange(&command) {
        Ok(result) => println!("BadInstruction response ({:?})", result),
        Err(e) => println!("BadInstruction response ({})", e),
    };
    println!();

    // Exit
    sleep(Duration::from_millis(2000));
    let command = APDUCommand {
        cla: 0x80,
        ins: Instruction::Exit.as_u8(),
        p1: 0x00,
        p2: 0x00,
        data: vec![0],
    };
    match transport.exchange(&command) {
        Ok(result) => println!("Ledger device disconnected ({:?})", result),
        Err(e) => println!("Ledger device disconnected with error ({})", e),
    };
    println!();
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
