# Starling LLM Server — Aether Implementation

## Purpose

Starling is a production-quality LLM inference server written entirely in AetherScript. It serves as the flagship example application for Aether, proving the language's capabilities for:

- **Real-world performance**: Production traffic, latency-sensitive workloads
- **Complex memory management**: KV cache allocation, eviction, mmap spill
- **Production FFI**: Integration with GGUF parsers, BLAS libraries, potentially llama.cpp
- **Concurrent systems**: Request scheduling, batching, backpressure
- **Operational excellence**: Metrics, health checks, graceful degradation

This is not a toy example. Starling should be deployable to production and handle real inference workloads.

---

## System Architecture

```
                                    +------------------+
                                    |   HTTP Gateway   |
                                    | /v1/generate     |
                                    | /v1/tokenize     |
                                    | /health, /metrics|
                                    +--------+---------+
                                             |
                    +------------------------+------------------------+
                    |                        |                        |
           +--------v--------+      +--------v--------+      +--------v--------+
           | Session Manager |      |    Tokenizer    |      |    Telemetry    |
           | - Session IDs   |      | - BPE/WordPiece |      | - Prometheus    |
           | - TTL/eviction  |      | - Round-trip    |      | - Structured    |
           | - Lifecycle     |      | - Offsets       |      |   logs          |
           +--------+--------+      +-----------------+      +-----------------+
                    |
           +--------v--------+
           |    Scheduler    |
           | - Micro-batching|
           | - Backpressure  |
           | - Fairness      |
           +--------+--------+
                    |
        +-----------+-----------+
        |                       |
+-------v-------+       +-------v-------+
|  KV Cache Mgr |       | Model Runtime |
| - Arena alloc |       | - GGUF loader |
| - LRU eviction|       | - Forward pass|
| - mmap spill  |       | - Quantized   |
+---------------+       +---------------+
```

---

## Components

### 1. HTTP Gateway
Entry point for all client requests. Responsibilities:
- Parse and validate JSON requests
- Authentication (API keys)
- Per-tenant rate limiting and quotas
- Route to appropriate handlers
- Stream responses (SSE for generation)
- Health and readiness endpoints

**Endpoints:**
| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/tokenize` | Tokenize text → token IDs + offsets |
| POST | `/v1/detokenize` | Token IDs → text |
| POST | `/v1/generate` | Text generation (streaming) |
| POST | `/v1/session/close` | Release session KV cache |
| GET | `/health` | Liveness check |
| GET | `/ready` | Readiness (models loaded, not overloaded) |
| GET | `/metrics` | Prometheus metrics |

### 2. Tokenizer Service
BPE/WordPiece tokenizer with production guarantees:
- Load vocabulary and merges from files
- Encode text → tokens with byte offsets
- Decode tokens → text (canonical round-trip)
- Handle unicode edge cases correctly
- Cache compiled merge rules for performance

**Contracts:**
- `decode(encode(text)) == text` for valid UTF-8
- Offsets correctly map tokens to source positions

### 3. Session Manager
Manages stateful generation sessions:
- Issue unique session IDs
- Track active sessions with TTL
- Map sessions to KV cache allocations
- Evict idle sessions under memory pressure
- Protect in-flight sessions from eviction

### 4. Sampler Pipeline
Converts model logits to next token selection:

```
logits
  → apply stop/EOS masking
  → apply repetition penalty
  → apply temperature scaling
  → apply top-k filtering
  → apply top-p (nucleus) filtering
  → apply frequency/presence penalties
  → multinomial sampling with seeded RNG
  → selected token
```

**Key properties:**
- Deterministic given seed + request (reproducible)
- Configurable per-request
- Supports logprobs capture

### 5. KV Cache Manager
Most complex component. Manages key-value cache tensors for transformer attention:

**Memory model:**
- Contiguous arena per model with configurable upper bound
- Per-session allocations with (layers, heads, head_dim, seq_len) shape
- Lazy growth as sequence extends
- LRU eviction at session granularity

**Advanced features:**
- mmap-backed spill to disk when RAM exhausted
- Page table mapping session+layer → (ptr, len, residency)
- Checksums on spilled pages
- Shape/dtype validation on every access

**Contracts:**
- Never corrupt active session data
- Always free on session close/eviction
- Bounded total memory usage

### 6. Scheduler
Coordinates inference requests across sessions:

**Responsibilities:**
- Maintain per-model request queues
- Form micro-batches by sequence length
- Enforce max batch size and queue depth
- Weighted round-robin for fairness
- Backpressure: reject with 429 when overloaded

**Concurrency model:**
- Per-model worker thread owns batch formation
- KV mutations only through scheduler
- Model forward runs on threadpool

### 7. Model Manager & Registry
Loads and manages GGUF model files:

**Registry:**
- Load from local path or URL
- SHA256 checksum validation
- Cache models in `~/.starling/models/`

**Loader:**
- Memory-map weights (lazy loading)
- Validate tensor shapes match architecture
- Build inference graph

**Admin operations:**
- List loaded models
- Load/unload dynamically
- Hot reload without downtime (drain → unload → reload)

### 8. Model Runtime
Executes transformer forward pass in **pure AetherScript**:

**Components:**
- Multi-head attention (Q/K/V projections, softmax, attention scores)
- Feed-forward network (MLP with SiLU/GELU activations)
- Layer normalization (RMSNorm)
- Quantized operations (Q4/Q8 dequantization, quantized matmul)

This is implemented entirely in Aether - no FFI for inference. This proves Aether can handle compute-intensive workloads at production quality.

### 9. Telemetry
Production observability:

**Metrics (Prometheus):**
- `starling_requests_total{endpoint, status}`
- `starling_request_duration_seconds{endpoint}` (histogram)
- `starling_tokens_generated_total`
- `starling_tokens_per_second`
- `starling_kv_cache_bytes{state}` (allocated/resident/spilled)
- `starling_kv_cache_evictions_total`
- `starling_queue_depth{model}`
- `starling_batch_size` (histogram)

**Structured logs:**
- JSON format with request_id, session_id, model, timings
- Error classification and stack traces

---

## Data Flow: Generate Request

```
1. Client POST /v1/generate
   {model: "llama-3-8b", prompt: "Hello", max_tokens: 100, temperature: 0.7}

2. Gateway validates, authenticates, checks quota

3. If prompt is text: Tokenizer.encode(prompt) → token_ids

4. Session Manager: create/lookup session, pin KV cache

5. Scheduler: enqueue request
   - If queue full: return 429
   - Otherwise: add to model's queue

6. Scheduler: form micro-batch when ready
   - Group by model, similar seq_len
   - Respect max_batch_size

7. For each generation step:
   a. KV Cache: provide handles for batch
   b. Model Runtime: forward(tokens, kv_handles) → logits
   c. Sampler: sample(logits, params) → next_tokens
   d. KV Cache: append new KV entries
   e. Stream token to client
   f. Check stop conditions (EOS, stop strings, max_tokens)

8. On completion:
   - Session Manager: unpin session
   - KV Cache: eligible for eviction if idle
   - Return final response
```

---

## Resource Bounds & Contracts

Every component has explicit limits:

| Component | Limit | Enforcement |
|-----------|-------|-------------|
| KV Cache | `max_memory_mb` per model | Reject allocations, trigger eviction |
| Queue | `max_queue_depth` per model | Return 429 |
| Batch | `max_batch_size` | Split batches |
| Session | `max_sessions` per model | Reject new sessions |
| Request | `max_prompt_tokens`, `max_output_tokens` | Validate on entry |
| Timeout | `request_timeout_ms` | Abort and cleanup |

**Invariants:**
- Total KV memory never exceeds configured cap
- In-flight requests always have valid KV handles
- Session close always frees all associated resources
- Scheduler never drops references (no leaks)

---

## Configuration

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  request_timeout_ms: 30000

models:
  - name: "llama-3-8b"
    path: "/models/llama-3-8b-q4.gguf"
    max_batch_size: 8
    max_sessions: 100
    kv_cache_mb: 4096
    spill_path: "/tmp/starling/kv"

scheduler:
  max_queue_depth: 1000
  batch_timeout_ms: 5

rate_limits:
  requests_per_minute: 60
  tokens_per_minute: 10000

auth:
  api_keys: ["sk-..."]
```

---

## Testing Strategy

### Unit Tests
- Tokenizer: Round-trip, unicode, edge cases, golden files
- Sampler: Deterministic outputs with fixed seeds
- KV Cache: Allocation, eviction, resize, shape validation

### Integration Tests
- Full request flow with mock model
- Session lifecycle
- Backpressure behavior
- Concurrent requests

### Load Tests
- Sustained throughput
- Memory pressure and eviction
- Latency percentiles under load
- Graceful degradation

### Production Validation
- Deploy with real traffic
- Monitor metrics and alerts
- Chaos testing (kill sessions, memory pressure)

---

## Aether Language Features Exercised

Building Starling will exercise and validate:

| Feature | Where Used |
|---------|------------|
| **Ownership/Borrowing** | KV cache handles, session lifecycle |
| **Contracts** | Tokenizer round-trip, resource bounds |
| **FFI** | GGUF loading, BLAS ops, llama.cpp |
| **Concurrency** | Scheduler workers, async HTTP |
| **Generics** | Cache<K,V>, Result<T,E> patterns |
| **Memory management** | Arena allocation, mmap |
| **Error handling** | Structured errors, timeouts |

---

## Implementation Phases

See `TASKS.md` for detailed breakdown. High-level:

1. **Tokenizer** - BPE with full round-trip ✅ (in progress)
2. **Sampler** - Full pipeline with deterministic RNG
3. **KV Cache** - Arena allocator with eviction
4. **Model Runtime** - GGUF loading, forward pass (FFI)
5. **Scheduler** - Batching and backpressure
6. **HTTP Gateway** - Full API with streaming
7. **Telemetry** - Metrics, logs, health
8. **Integration** - End-to-end with real model
9. **Hardening** - mmap spill, hot reload, production polish

---

## Non-Goals (for now)

- GPU/CUDA support (CPU-first, GPU later)
- Distributed inference (single-node first)
- Fine-tuning or training
- Multiple model architectures (focus on LLaMA-style)

---

## Success Criteria

Starling is "done" when it can:

1. Serve real LLM inference traffic at production quality
2. Handle memory pressure gracefully (eviction, backpressure)
3. Provide full observability (metrics, logs, traces)
4. Run reliably for extended periods under load
5. Demonstrate Aether's capabilities for systems programming
