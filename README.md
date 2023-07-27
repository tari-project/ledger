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

challenge:  17b4211e325fec8958863892549320cf4982b2f1345ef9dbe14b6597a3b3320c
signature:  72ff661947bedacc27822c259f8d783e357ce2035c2446386ac5f383d57b9005
public key: dad90c3bd61ac63b51181b7f56c3b17afbe33ad2143eba3b5ba3755a5284710c
sign:       true
 
commitment: 3a587a548f9076818dd4d2a328f2b6d9905c08f7aa786135b90826eac4a1134e

path:       m/44'/535348'/84212125668703186330'/0/0
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935
path:       m/44'/535348'/84212125668703186330'/0/1
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935
path:       m/44'/535348'/84212125668703186330'/0/2
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935
path:       m/44'/535348'/84212125668703186330'/0/3
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935
path:       m/44'/535348'/84212125668703186330'/0/4
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935
path:       m/44'/535348'/84212125668703186330'/0/5
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935
path:       m/44'/535348'/84212125668703186330'/0/6
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935
path:       m/44'/535348'/84212125668703186330'/0/7
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935
path:       m/44'/535348'/84212125668703186330'/0/8
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935
path:       m/44'/535348'/84212125668703186330'/0/9
public_key: c03ba93072c6d7d8704e2bf7f80beebcd36a3a856416fb174fb7e615dff1e935

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
