use ledger_transport::APDUCommand;
use ledger_transport_hid::{hidapi::HidApi, TransportNativeHID};
use ledger_zondax_generic::{App, AppExt};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use tari_crypto::{
    keys::SecretKey,
    ristretto::{RistrettoPublicKey, RistrettoSchnorr, RistrettoSecretKey},
    tari_utilities::ByteArray,
};

fn hidapi() -> &'static HidApi {
    static HIDAPI: Lazy<HidApi> = Lazy::new(|| HidApi::new().expect("unable to get HIDAPI"));

    &HIDAPI
}
struct Tari;
impl App for Tari {
    const CLA: u8 = 0x0;
}

fn main() {
    let command = APDUCommand {
        cla: 0x80,
        ins: 0x01,
        p1: 0x00,
        p2: 0x00,
        data: vec![0],
    };
    let message = vec![0];
    let ledger = TransportNativeHID::new(hidapi()).expect("Could not get a device");

    // use device info command that works in the dashboard
    let result = futures::executor::block_on(Tari::send_chunks(&ledger, command, &message)).unwrap();
    let data_len = result.data()[1] as usize;
    let name = &result.data()[2..data_len + 2];
    let name = std::str::from_utf8(name).unwrap();
    println!("name: {}", name);
    let package_len = result.data()[data_len + 2] as usize;
    let package = &result.data()[data_len + 3..data_len + package_len + 3];
    let package = std::str::from_utf8(package).unwrap();
    println!("package version: {}", package);

    let challenge = RistrettoSecretKey::random(&mut OsRng);
    let command2 = APDUCommand {
        cla: 0x80,
        ins: 0x02,
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

    let signature = RistrettoSchnorr::new(nonce, sig);
    let result = signature.verify(&public_key, &challenge);
    println!("sign: {}", result);
}
