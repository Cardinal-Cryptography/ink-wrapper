# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- `ReadCall` gets new argument `value: u128` representing native tokens transferred. 
- `ExecCall` gets new trait bound `T` to reperesent type of the returned value.

## 0.6.1

### Changed

- Replaced usage of `genco` with `quote`.

## 0.6.0

### Changed

- Updated ink to `4.2.1`

### Fixed

- Fix incorrect codegen on event fields with generic types (#60)

## 0.5.0

### Added

- Ability to send native tokens with calls
- Ability to get the `tx_info` for a constructor call
- Ability to set the `tx_status` to wait for when submitting calls

### Fixed

- Invalid version of an indirect dependency causing mangled output when installed without `--locked`

### Changed

- Contracts are now called with `conn.exec(contract.some_method(args))`, etc.

## 0.4.1

### Fixed

- Publish the correct version of `Cargo.lock` to enable `cargo install --locked`

## 0.4.0

### Added

- Helper for converting any `AsRef[u8; 32]` to an `AccountId`
- Expose the contract's code hash as a constant
- Option to print the version number

### Fixed

- Unhygienic use of the name `account_id` in constructors - could cause constructors using an argument named `account_id` to fail to compile

## 0.3.0

### Added

- Support for uploading contract code

### Fixed

- Arrays in events being referenced as `super::[T; N]` instead of just `[T; N]`
- Related to the above - in events types in arrays (and tuples, etc.) being referenced without the `super::`

## 0.2.0

### Added

- Support for fetching contract events

### Fixed

- Handle messages in openbrush-style traits (`PSP22::transfer`, etc.)

### Changed

- `ink-wrapper-types` released to crates.io
- Return a custom type that implements `std::error::Error` instead of `ink_primitives::LangError`

## 0.1.0

### Added

- Initial support for messages and constructors
