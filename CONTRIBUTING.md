# Contributing to WinForge

Thank you for your interest in contributing to **WinForge**! We welcome contributions from everyone — whether it's code, documentation, bug reports, feature requests, or examples.

## Code of Conduct

This project and everyone participating in it is governed by the [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the maintainers.

## How Can I Contribute?

### Reporting Bugs

- Use the [Issues](https://github.com/ar-fullsend/winforge-framework/issues) tab.
- Include as much detail as possible: Windows version, .NET/Rust versions, steps to reproduce, expected vs actual behavior, and relevant logs (especially ETW traces if possible).
- Search existing issues first to avoid duplicates.

### Suggesting Enhancements

We love ideas! Open an issue with the `enhancement` label and describe:
- The problem you're trying to solve
- Your proposed solution
- Any alternatives considered
- Mockups or diagrams if applicable

### Pull Requests

1. Fork the repository and create your branch from `main`.
2. Make your changes.
3. Ensure tests pass and new functionality is covered.
4. Update documentation (README, docs/, code comments).
5. Submit a Pull Request with a clear title and description.

### Development Setup

See the [README.md](README.md#getting-started) for prerequisites and initial setup.

Key commands:
```bash
# Build everything
cargo build --workspace
dotnet build

# Run tests
cargo test --workspace
dotnet test

# Run the CLI in dev mode
cargo run -p winforge-cli -- dev
```

## Style Guidelines

- **Rust**: Follow `rustfmt` and `clippy`. Use `tracing` for logging.
- **C#**: Follow .NET coding conventions. Use nullable reference types and source generators where beneficial.
- **Documentation**: Keep examples up to date. Use Mermaid diagrams where they add clarity.
- **Commits**: Use conventional commits (e.g., `feat:`, `fix:`, `docs:`, `refactor:`).

## Areas Where We Need Help

- Expanding Windows interop bindings (especially newer WinRT APIs and NPU/DirectML)
- Building more plugin examples (especially privacy, data center ops, and agentic workflows)
- Improving the workflow engine durability and visualization
- Creating UI automation and property-based tests
- Writing tutorials and sample applications
- Performance benchmarking and optimization

## Questions?

Feel free to open a discussion in [GitHub Discussions](https://github.com/ar-fullsend/winforge-framework/discussions) or reach out via issues.

We appreciate every contribution — big or small. Let's build the future of Windows software together!
