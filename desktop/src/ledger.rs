use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
    Mutex,
};

use dialoguer::{theme::ColorfulTheme, Select};
use ledger_transport_hid::{
    hidapi::{DeviceInfo, HidApi},
    TransportNativeHID,
};
use once_cell::sync::Lazy;
use tari_crypto::ristretto::RistrettoPublicKey;
use thiserror::Error;

/// Ledger device errors.
#[derive(Error, Debug, Clone)]
pub enum LedgerDeviceError {
    /// HID API error
    #[error("HID API error `{0}`")]
    HidApi(String),
    /// Native HID transport error
    #[error("Native HID transport error `{0}`")]
    NativeTransport(String),
}

/// Hardware wallet.
#[derive(Clone)]
pub struct LedgerWallet {
    account_public_key: RistrettoPublicKey,
}

impl LedgerWallet {
    /// Create a new hardware wallet.
    pub fn new(account_public_key: RistrettoPublicKey) -> Result<Self, LedgerDeviceError> {
        initialize_ledger_device()?;
        Ok(Self { account_public_key })
    }

    /// Get a reference to the account public key.
    pub fn account_public_key(&self) -> &RistrettoPublicKey {
        &self.account_public_key
    }
}

/// Ledger transport.
pub static TRANSPORT: Lazy<Arc<Mutex<Option<TransportNativeHID>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));
static INITIALIZED: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

// Initialize the ledger device with transport.
fn initialize_ledger_device() -> Result<(), LedgerDeviceError> {
    if INITIALIZED.load(Ordering::Relaxed) {
        return Ok(());
    }

    let hid_api = hidapi()?;
    const LEDGER_VID: u16 = 0x2c97;
    const LEDGER_USAGE_PAGE: u16 = 0xFFA0;
    let device_list: Vec<_> = hid_api
        .device_list()
        .filter(|dev| dev.vendor_id() == LEDGER_VID && dev.usage_page() == LEDGER_USAGE_PAGE)
        .collect();
    let index = if device_list.is_empty() {
        return Err(LedgerDeviceError::NativeTransport("No ledger device found".to_string()));
    } else if device_list.len() > 1 {
        select_ledger(&device_list)
    } else {
        0
    };
    let transport = match TransportNativeHID::open_device(hid_api, device_list[index]) {
        Ok(val) => val,
        Err(e) => return Err(LedgerDeviceError::NativeTransport(format!("{}", e))),
    };

    *TRANSPORT.lock().expect("lock exists") = Some(transport);
    INITIALIZED.store(true, Ordering::Relaxed);
    Ok(())
}

fn hidapi() -> Result<&'static HidApi, LedgerDeviceError> {
    static HIDAPI: Lazy<Result<HidApi, String>> =
        Lazy::new(|| HidApi::new().map_err(|e| format!("Unable to get HIDAPI: {}", e)));

    HIDAPI.as_ref().map_err(|e| LedgerDeviceError::HidApi(e.to_string()))
}

// Select the appropriate ledger device
fn select_ledger(device_list: &[&DeviceInfo]) -> usize {
    println!();
    let binding = ColorfulTheme::default();
    let mut binding = Select::with_theme(&binding);
    let prompt = binding
        .with_prompt("More than one Ledger device detected, which one should be used?")
        .default(0);
    for device in device_list {
        prompt.item(&format!(
            "Device: {}, {}, {:?}",
            device.manufacturer_string().unwrap_or("no manufacturer string"),
            device.product_string().unwrap_or("no product string"),
            device.path()
        ));
    }
    let index = prompt.interact().unwrap();
    println!();
    index
}
