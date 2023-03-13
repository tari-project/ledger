To build:
For loading a BOLOS application to a Ledger device, Ledger has actually written a command, called [Cargo Ledger](https://github.com/LedgerHQ/cargo-ledger). This we need to install with:
```
cargo install --git https://github.com/LedgerHQ/cargo-ledger
```

Next up we need install the supporting Python libraries from Ledger to control Ledger devices, [LedgerCTL](https://github.com/LedgerHQ/ledgerctl). This we do with:
```
pip3 install --upgrade protobuf setuptools ecdsa
pip3 install ledgerwallet
```

Lastly install the ARM GCC toolchain: `arm-none-eabi-gcc` for your OS. We are using MacOS, so we can use brew with:
```
brew install armmbed/formulae/arm-none-eabi-gcc
```
Copy over target json file (nanosplus, nanos, etc) from: https://github.com/LedgerHQ/ledger-nanos-sdk

to build:
cargo +nightly build -Zbuild-std=panic_abort,std --release --target={TARGET}.json
with TARGET = nanosplus, nanos, etc.
to load: cargo +nightly ledger --load nanosplus -- -Zbuild-std=panic_abort,std

