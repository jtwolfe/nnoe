# Contributing to NNOE

Thank you for your interest in contributing to NNOE! This guide will help you get started.

## Code of Conduct

This project adheres to the Contributor Covenant Code of Conduct. Please be respectful and professional.

## How to Contribute

### Reporting Bugs

1. Check if the issue has already been reported
2. Create a new issue with:
   - Clear description
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version, etc.)
   - Relevant logs

### Suggesting Enhancements

1. Check existing issues for similar suggestions
2. Create an issue describing:
   - The enhancement and use case
   - Potential implementation approach
   - Benefits

### Contributing Code

1. **Fork the repository**
2. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes:**
   - Follow Rust coding standards
   - Add tests for new functionality
   - Update documentation
4. **Ensure tests pass:**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```
5. **Commit changes:**
   ```bash
   git commit -m "feat: Add feature description"
   ```
6. **Push to your fork:**
   ```bash
   git push origin feature/your-feature-name
   ```
7. **Open a Pull Request**

## Development Setup

See [Getting Started Guide](getting-started.md) for detailed setup instructions.

Quick setup:

```bash
# Clone fork
git clone https://github.com/your-username/nnoe.git
cd nnoe

# Install dependencies
rustup toolchain install stable
cargo build
```

## Coding Standards

### Rust Style

- Follow standard Rust formatting: `cargo fmt`
- Fix all clippy warnings: `cargo clippy -- -D warnings`
- Document public APIs with doc comments
- Use meaningful names

### Code Organization

- Keep modules focused and cohesive
- Limit function complexity
- Prefer composition over inheritance
- Use traits for polymorphism

### Testing

- Write unit tests for new functionality
- Aim for >80% code coverage
- Include integration tests for complex flows
- Document test setup requirements

### Documentation

- Document all public APIs
- Include usage examples
- Keep README files updated
- Update changelog for user-facing changes

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions/changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

**Examples:**
```
feat(agent): Add DNS zone validation
fix(etcd): Handle connection timeout gracefully
docs(api): Update agent API documentation
```

## Pull Request Process

1. **Update your branch:**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Ensure all checks pass:**
   - CI tests
   - Code review
   - Documentation review

3. **Address review feedback:**
   - Respond to comments
   - Make requested changes
   - Update PR description

4. **Squash commits** (if requested by maintainers)

## Review Process

- Maintainers review PRs within 2-3 business days
- Address all review comments
- Request re-review after changes
- PRs need at least one approval before merge

## Project Structure

```
nnoe/
├── agent/              # Core agent code
├── integrations/       # External integrations
├── management/         # Management tools
├── deployments/       # Deployment configs
├── testing/           # Test infrastructure
└── docs/              # Documentation
```

## Areas for Contribution

### High Priority

- Service integrations (Knot, Kea, dnsdist improvements)
- Testing coverage
- Documentation improvements
- Performance optimizations
- Security enhancements

### Plugin Development

- New service plugins
- Custom integrations
- Extended functionality

### Documentation

- Tutorials and guides
- API documentation
- Deployment examples
- Troubleshooting guides

## Getting Help

- **GitHub Issues**: For bugs and feature requests
- **Discussions**: For questions and discussions
- **Documentation**: Check existing docs first

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

Thank you for contributing to NNOE!

