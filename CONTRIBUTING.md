# Contributing to NNOE

Thank you for your interest in contributing to NNOE! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to a code of conduct. Please be respectful and professional in all interactions.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in the issue tracker
2. If not, create a new issue with:
   - Clear description of the problem
   - Steps to reproduce
   - Expected vs. actual behavior
   - Environment details (OS, Rust version, etc.)

### Suggesting Enhancements

1. Check if the enhancement has already been suggested
2. Open an issue describing:
   - The enhancement and its use case
   - Potential implementation approach (if applicable)

### Submitting Code

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/my-feature`
3. **Make your changes**:
   - Follow Rust coding standards
   - Add tests for new functionality
   - Update documentation as needed
4. **Ensure tests pass**: `cargo test`
5. **Run linters**: `cargo clippy` and `cargo fmt`
6. **Commit your changes**: Use clear, descriptive commit messages
7. **Push to your fork**: `git push origin feature/my-feature`
8. **Open a Pull Request**

## Development Guidelines

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Fix all clippy warnings (`cargo clippy -- -D warnings`)
- Document public APIs with doc comments
- Use meaningful variable and function names

### Testing

- Write unit tests for new functionality
- Ensure integration tests pass
- Aim for >80% code coverage for new code

### Documentation

- Update relevant documentation when adding features
- Add examples for new APIs
- Keep README and getting started guides current

### Commit Messages

Follow the conventional commits format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`

## Project Structure

- `agent/`: Core Rust agent code
- `integrations/`: External service integrations
- `management/`: Management components
- `deployments/`: Deployment configurations
- `testing/`: Test infrastructure
- `docs/`: Documentation

## Getting Help

- Check existing documentation in `docs/`
- Open an issue for questions
- Join discussions in project channels (if available)

Thank you for contributing to NNOE!

