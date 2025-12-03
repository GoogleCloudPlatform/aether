# Aether

**A modern systems programming language with LLM-first design principles**

Aether combines memory safety through an ownership system with a clean, structured syntax optimized for LLM generation and human readability. Designed for high-performance applications while maintaining safety, expressiveness, and AI-friendly code generation.

**NOTE**
This project is a demonstration of vibe coding intended to provide a trustworthy and verifiable example that developers and researchers can use. It is not intended
for use in a production environment.

This is not an officially supported Google product. This project is not
eligible for the [Google Open Source Software Vulnerability Rewards
Program](https://bughunters.google.com/open-source-security).

## 🚀 Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- LLVM 17+
- Git

### Build and Install

```bash
git clone https://github.com/GoogleCloudPlatform/Aether
cd Aether
cargo build --release
```

### Hello World

```aether
module HelloWorld {
    /// Simple greeting program demonstrating Aether syntax

    @extern(library: "libc")
    func puts(s: String) -> Int;

    func main() -> Int {
        puts("Hello, Aether!");
        return 0;
    }
}
```

Compile and run:

```bash
./target/release/aether compile examples/v2/01-basics/hello_world/main.aether
./hello_world
```

## ✨ Key Features

- **🛡️ Memory Safety**: Ownership system with move, borrow, and shared semantics
- **🤖 LLM-First Design**: Explicit intent annotations and structured syntax for AI comprehension
- **⚡ Performance**: Zero-cost abstractions with LLVM backend
- **🌐 Web Ready**: Built-in HTTP server capabilities and FFI networking
- **🔒 Verification**: Contract-based programming with preconditions and postconditions
- **📝 Clean Syntax**: Modern, C-like syntax with strict rules for reliability

## 📚 Documentation

- **[User Guide](user-guide.md)** - Complete language tutorial and reference
- **[Language Reference](LANGUAGE_REFERENCE.md)** - Comprehensive syntax and semantics
- **[Final Design](FINAL_DESIGN.md)** - Core philosophy and architectural principles
- **[Examples](examples/)** - Working code examples and demonstrations
- **[Technical Docs](docs/)** - Implementation details and architectural documentation

## 🌐 Working Examples

### HTTP Blog Server

Aether includes a complete, working HTTP server implementation:

```bash
# Compile the blog server
./target/release/aether compile examples/blog_listen.aether

# Run the server
./blog_listen &

# Test it
curl http://localhost:8080
```

See **[examples/README.md](examples/README.md)** for all available examples including:

- ✅ **Working HTTP blog servers** with styled HTML
- 🚀 **LLM-optimized web applications**
- 🔧 **FFI networking integration**
- 📊 **Resource management demonstrations**

## 🛠️ CLI Commands

```bash
# Compile to executable
aether compile program.aether

# Type checking only
aether check program.aether

# Run directly
aether run program.aether

# View AST
aether ast program.aether

# View tokens
aether tokens program.aether
```

## 🏗️ Aether Project Structure

Aether uses a Go-inspired package-per-directory layout:

```
project_name/
├── aether.toml              # Project manifest
├── cmd/
│   └── project_name/
│       └── main.aether      # Binary entry point
├── package_a/               # Directory = package name
│   ├── package_a.aether     # Implementation
│   └── package_a_test.aether    # Tests alongside code
└── package_b/
    ├── package_b.aether
    └── package_b_test.aether
```

**Conventions:**
- Directory name = package name
- Tests alongside code: `*_test.aether`
- Entry points: `cmd/{binary}/main.aether`
- Import: `import project/package`

See [docs/PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md) for full details.

### Compiler Repository Structure

```
├── src/           # Compiler source code
├── runtime/       # Runtime library (Rust)
├── stdlib/        # Standard library modules
├── examples/      # Example programs
│   └── starling/  # LLM inference server (flagship example)
├── tests/         # Compiler test suite
├── scripts/       # Build and development scripts
├── docs/          # Technical documentation
└── tutorials/     # Learning materials
```

## 🎯 Status

**Production Ready** - AetherScript is fully functional with:

- ✅ **360 unit tests** passing
- ✅ **Complete compiler pipeline** (lexing → parsing → semantic analysis → LLVM codegen)
- ✅ **Ownership system** with move/borrow tracking
- ✅ **HTTP server examples** demonstrating real-world applications
- ✅ **Comprehensive CLI** with multiple commands
- ✅ **Runtime library** with memory management and networking

## 🤝 Contributing

1. Read the [Final Design](FINAL_DESIGN.md) document
2. Check [Technical Documentation](docs/) for implementation details
3. Browse [Examples](examples/) to understand the language
4. See [Build Scripts](scripts/) for development workflow

## 📄 License

This project is licensed under the Apache 2 License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Repository**: https://github.com/GoogleCloudPlatform/Aether
- **Documentation**: Complete docs in this repository
- **Examples**: Live HTTP server demos in `/examples`

---

_Aether: Bridging human intention and machine execution through explicit, verifiable code._
