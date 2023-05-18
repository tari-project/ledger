### Setup
For loading a BOLOS application to a Ledger device, Ledger has actually written a command, called [Cargo Ledger](https://github.com/LedgerHQ/cargo-ledger). This we need to install with:
```
cargo install --git https://github.com/LedgerHQ/cargo-ledger
```

As per the [Cargo Ledger setup instructions](https://github.com/LedgerHQ/cargo-ledger#setup) run the following to add new build targets for the current rust toolchain:

```
cargo ledger setup
```

Next up we need install the supporting Python libraries from Ledger to control Ledger devices, [LedgerCTL](https://github.com/LedgerHQ/ledgerctl). This we do with:
```
pip3 install --upgrade protobuf setuptools ecdsa
pip3 install git+https://github.com/LedgerHQ/ledgerctl
```

Lastly install the ARM GCC toolchain: `arm-none-eabi-gcc` for your OS. We are using MacOS, so we can use brew with:
```
brew install armmbed/formulae/arm-none-eabi-gcc
```

Install a custom certificate on the device to help with development. Start the device in recovery mode (varies per device)
- Nano S Plus: Hold the left button while turning on, and follow on screen instructions
- Nano S: Hold the right button while turning on

Once in recovery mode run the following where <NAME> is simply the name of the CA. It can be anything:

```
ledgerctl install-ca <NAME>
```

### Building
Where TARGET = nanosplus, nanos, etc.
```
cargo ledger build {TARGET} -- -Zbuild-std=std,alloc
```

### Loading
```
cargo ledger build {TARGET} --load -- -Zbuild-std=std,alloc
```

