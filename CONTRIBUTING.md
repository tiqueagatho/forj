# Contributing to forj

Thank you for considering contributing to forj! We welcome contributions of all
kinds: bug reports, feature requests, documentation, and code.

## Code of Conduct

This project is committed to providing a welcoming, inclusive environment for
everyone. Be respectful, constructive, and professional in all interactions.

## How to Contribute

### Reporting Bugs

1. Check existing [issues](https://github.com/javier/forj/issues) to avoid duplicates.
2. Open a new issue with:
   - A clear title and description.
   - Steps to reproduce.
   - Expected vs. actual behavior.
   - Environment details (OS, Rust version, features enabled).

### Feature Requests

Open an issue with the `enhancement` tag describing:
- The problem you're solving.
- Proposed API or behavior.
- Alternative approaches considered.

### Pull Requests

1. Fork the repository.
2. Create a feature branch: `git checkout -b feat/my-feature`.
3. Make your changes.
4. Ensure all checks pass:

   ```bash
   cargo test --features full
   cargo clippy --all-features -- -D warnings
   cargo fmt --check
   ```

5. Commit with a descriptive message.
6. Open a PR against `main`.

### Development Setup

```bash
git clone https://github.com/javier/forj
cd forj

# Run tests (default features: modbus-tcp + onnx)
cargo test

# Run tests with all features (some require system dependencies)
cargo test --features tract,opcua,mqtt

# Run lint
cargo clippy --all-features -- -D warnings

# Format code
cargo fmt
```

Note: `openvino` and `candle` features require system libraries. Install them:

```bash
# OpenVINO
wget https://storage.openvinotoolkit.org/repositories/openvino/packages/...

# Candle (uses system BLAS)
apt install libopenblas-dev
```

### Feature Gate Policy

All optional backends and protocols must be behind Cargo feature flags.
Do not add required dependencies for functionality that some users may not
need. Use `#[cfg(feature = "...")]` for conditional compilation.

### Testing

- Every module must have unit tests in `#[cfg(test)] mod tests { ... }`.
- New backends must include basic load/infer tests.
- Pipeline tests should cover error propagation, cancellation, and backpressure.

## License

By contributing, you agree that your contributions will be dual-licensed under
MIT and Apache 2.0 as described in [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE).
