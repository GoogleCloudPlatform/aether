# Starling LLM Server

A production-quality LLM inference server written entirely in AetherScript.

## Purpose

Starling is the flagship example application for Aether, demonstrating the language's capabilities for building real-world, production systems:

- Pure AetherScript implementation (no FFI for performance)
- Complex memory management (KV cache with eviction)
- Concurrent request handling with backpressure
- Full observability (metrics, logging, tracing)

## Project Structure

```
starling/
├── aether.toml              # Project manifest
├── cmd/starling/            # Binary entry point
│   └── main.aether
├── tokenizer/               # BPE tokenizer package
│   ├── tokenizer.aether
│   └── tokenizer_test.aether
├── sampler/                 # Sampling pipeline
├── kvcache/                 # KV cache manager
├── scheduler/               # Request scheduler
├── model/                   # Model runtime (pure Aether)
├── http/                    # HTTP server
└── telemetry/               # Metrics and logging
```

## Quick Start

```bash
# Build the server
aether build cmd/starling

# Or with current compiler:
AETHER_RUNTIME_PATH=./runtime/target/debug \
  ./target/debug/aether-compiler compile examples/starling/cmd/starling/main.aether

# Run
./main

# Test endpoints
curl http://localhost:8080/health
curl -X POST http://localhost:8080/v1/tokenize -d '{"text": "hello world"}'
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/tokenize` | Tokenize text → token IDs |
| POST | `/v1/detokenize` | Token IDs → text |
| POST | `/v1/generate` | Text generation (streaming) |
| GET | `/health` | Liveness check |
| GET | `/metrics` | Prometheus metrics |

## Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design
- [TASKS.md](./TASKS.md) - Development tasks and progress

## Development Status

| Component | Status |
|-----------|--------|
| Tokenizer | In Progress |
| Sampler | Not Started |
| KV Cache | Not Started |
| Model Runtime | Not Started |
| Scheduler | Not Started |
| HTTP Gateway | Partial |
| Telemetry | Not Started |

See [TASKS.md](./TASKS.md) for detailed progress.
