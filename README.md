# Instructions

## Setup

Ledger does not build with the standard library, so we need to install `rust-src`. This can be done with:
```
rustup component add rust-src --toolchain nightly
```

For loading a BOLOS application to a Ledger device, Ledger has actually written a command, called 
[Cargo Ledger](https://github.com/LedgerHQ/cargo-ledger). This we need to install with:
```
cargo install --git https://github.com/LedgerHQ/cargo-ledger
```

As per the [Cargo Ledger setup instructions](https://github.com/LedgerHQ/cargo-ledger#setup) run the following to add 
new build targets for the current rust toolchain:

```
cargo ledger setup
```

Next up we need install the supporting Python libraries from Ledger to control Ledger devices, 
[LedgerCTL](https://github.com/LedgerHQ/ledgerctl). This we do with:
```
pip3 install --upgrade protobuf setuptools ecdsa
pip3 install git+https://github.com/LedgerHQ/ledgerctl
```

Lastly install the ARM GCC toolchain: `arm-none-eabi-gcc` for your OS (https://developer.arm.com/downloads/-/gnu-rm). 
For MacOS, we can use brew with:
```
brew install armmbed/formulae/arm-none-eabi-gcc
```

## Device configuration

See https://github.com/LedgerHQ/ledgerctl#device-configuration

Install a custom certificate on the device to help with development. Start the device in recovery mode (varies per device)
- Nano S Plus: Hold the left button while turning on, and follow on screen instructions
- Nano S: Hold the right button while turning on

Once in recovery mode run the following where <NAME> is simply the name of the CA. It can be anything:

```
ledgerctl install-ca <NAME>
```

## Runtime

### Build and load `ledger`

_**Note:** Windows users should start a "x64 Native Tools Command Prompt for VS 2019" to have the build tools available
and then start a python shell within that terminal to have the Python libraries available._

Open a terminal the subfolder `/ledger`

To build, run
```
cargo ledger build {TARGET} -- "-Zbuild-std=std,alloc"
```
where TARGET = nanosplus, nanos, etc.

To load, run from a Python shell (`pip3 --version` should work) and select both buttons on the Ledger device when 
prompted:
```
cargo ledger build {TARGET} --load -- "-Zbuild-std=std,alloc"
```
where TARGET = nanosplus, nanos, etc.

**Errors**

If the auto-load does not work ("ledgerwallet.client.CommException: Exception : Invalid status 6512 (Unknown reason)"), 
try to do a manual install:
- In some cases the `cargo ledger build` action will invalidate `app_nanosplus.json` by setting the first line to 
  `"apiLevel": "0",` - ensure it is set to `"apiLevel": "1",`
- Manually delete if installed with `ledgerctl delete "Tari Ledger Demo"`
- Manually install with `ledgerctl install app_nanosplus.json`

### Running the test code `desktop`

Start the `Tari Ledger Demo` application on the Ledger by navigating to the app and pressing both buttons. You should see 
`Tari test app` displayed on the screen.

**Note:** Do not press any more buttons!

Open a terminal the subfolder `/desktop`

Run the example with `cargo run`

You should see a similar output, just with different hex values:
```
name: tari_ledger_demo
package version: 0.0.1

challenge:  624bfccec2e62eda5eb3f54093c7996e9a66024a7c16ddf7a20f86ff33a7840d
signature:  35c0402891213f07832ef5973a29a9d06623f4ac8fcaac59c4677abce7d7380e
public key: dad90c3bd61ac63b51181b7f56c3b17afbe33ad2143eba3b5ba3755a5284710c
sign:       true
 
commitment: 3a587a548f9076818dd4d2a328f2b6d9905c08f7aa786135b90826eac4a1134e

path:       m/44'/535348'/32374961496997035060'/0/0
public_key: 4afc33a678d56b7b4a94530382766000048de8e6f45a15e8374a2cc7ecbb8a68
path:       m/44'/535348'/32374961496997035060'/0/1
public_key: cc97e83d92ae9c1e05642f6f919dd4002c0280cdc4c8c965857474a24f37cc1b
path:       m/44'/535348'/32374961496997035060'/0/2
public_key: c0bbfe9c7fe3dcae9992c2d7c986906b9a039e8e859ab3356095ed460423e238
path:       m/44'/535348'/32374961496997035060'/0/3
public_key: 1c4d96dd268f22c8d8a43fb14ae6449e92134cbcc924c08c88aa002f2c87b157
path:       m/44'/535348'/32374961496997035060'/0/4
public_key: c4bde2c65f432c7b979fffee6612f9cb2c5d746fa2928e4679e35b5bd4c5ed48

public_nonce: 14a7cc726515554c5a1960fcab90b134e444d64a8a5c425092bfa624fb361423
public_nonce: 26fdfdbcf1ef81b560561a4b51a7440217edc861bd4832b382aa0bded81a9127
public_nonce: ae5f98f5053c88a8862ffab47fe6dd4b7316ec278717502748729567efe2b220
public_nonce: 5c2831f0a0d9bd8eb2f50d38ae93ccd5a0bfce41963e00c8165fd434bd53821d
public_nonce: 50ef2fdf0b0267ba5f68b22b17ca39725bbeee8623f45112bee48059ea9b1166

BadInstruction response (APDUAnswer { data: [110, 1], retcode: 28161 })

Ledger device disconnected (APDUAnswer { data: [144, 0], retcode: 36864 })

```

Press both buttons on the Ledger again to exit the application.

**Errors**

- If the `Tari Ledger Demo` application on the Ledger is not started, you should see the following output:

  `Error: 'GetVersion' (Uknown error: 28161)`
 
- If you pressed a button on the Ledger after starting the application, but before running the demo, you should see the 
  following output:

  `Error: Transport | Ledger device: Io error`
