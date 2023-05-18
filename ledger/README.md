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
pip3 install ledgerwallet
```

Lastly install the ARM GCC toolchain: `arm-none-eabi-gcc` for your OS. We are using MacOS, so we can use brew with:
```
brew install armmbed/formulae/arm-none-eabi-gcc
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

