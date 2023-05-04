# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
