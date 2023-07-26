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
- Manually delete if installed with `ledgerctl delete Tari`
- Manually install with `ledgerctl install app_nanosplus.json`

### Running the test code `desktop`

Start the `Tari` application on the Ledger by navigating to the app and pressing both buttons. You should see 
`Tari test app` displayed on the screen.

**Note:** Do not press any more buttons!

Open a terminal the subfolder `/desktop`

Run the example with `cargo run`

You should see a similar output, just with different hex values:
```
name: tari
package version: 0.0.1

challenge:  907427e1444d8fb652f3d8a580b5ad9b630938b3f05041781efd643f5d5c6a0c
signature:  584d7c2c8ee2f08568635553cccd13d10f3a1345ef07142f58dd1216c0eb8203
public key: dad90c3bd61ac63b51181b7f56c3b17afbe33ad2143eba3b5ba3755a5284710c
sign:       true

commitment: 3a587a548f9076818dd4d2a328f2b6d9905c08f7aa786135b90826eac4a1134e

path:       m/44'/535348'/0'/0/0
public_key: dad90c3bd61ac63b51181b7f56c3b17afbe33ad2143eba3b5ba3755a5284710c
path:       m/44'/535348'/0'/0/1
public_key: 1cec04609602115212fd60fd94a91c6f870d0d7dd9196f1e5d7dcc178a45ac31
path:       m/44'/535348'/0'/0/2
public_key: f8a7fc373821fe111056c5ef9b56b8b09e16b4ab870051dd7a1fd9f93e510b77
path:       m/44'/535348'/0'/0/3
public_key: 061c33e40b423ed96eae0d4cea89f3716ddd40c48aa8050ccf89f701667b4579
path:       m/44'/535348'/0'/0/4
public_key: d0e2fd4e8a2ac58113c91d9ec17c2b780718eb20dfaa8bd3e88006aee59f4a5f
path:       m/44'/535348'/0'/0/5
public_key: be38fa70d498d6092c84272d9aafcd291a80f61b6b0a075ed1f8f98fa9288c3b
path:       m/44'/535348'/0'/0/6
public_key: c8cf60ea96f4585fb399c41dab3b44c528a74bca0d20b95e0752e08c7d61f012
path:       m/44'/535348'/0'/0/7
public_key: 26d4be20928e2380be6ceb331012343159eb853bfe478cf82631779ebedd360d
path:       m/44'/535348'/0'/0/8
public_key: 68b53190bbda74d6f91d38f56e4c1fa3dd4dda600274153007554f62c787ab6b
path:       m/44'/535348'/0'/0/9
public_key: 70a143730591525176312c9555dec97b7ab543ae8a95f7d8a69eeaad43f54540

Ledger device disconnected (APDUAnswer { data: [144, 0], retcode: 36864 })
```

Press both buttons on the Ledger again to exit the application.

**Errors**

- If the `Tari` application on the Ledger is not started, you should see the following output:

  `Error: Uknown error: 28161`
 
- If you pressed a button on the Ledger after starting the application, but before running the demo, you should see the 
  following output:

  `Error: Transport | Ledger device: Io error`
