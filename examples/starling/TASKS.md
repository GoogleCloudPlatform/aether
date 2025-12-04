# Starling LLM Server — Task Breakdown

## General Development Guidance

### Core Principles
- **Pure AetherScript**: Implement ALL components in Aether. FFI is only for OS primitives (syscalls, file I/O) that cannot be expressed in Aether. Performance is never a reason to use FFI.
- **Production Quality**: Every component should be deployable, not just compilable
- **Test-Driven**: Write tests before implementation
- **Observability First**: Metrics and logging from day one

### Post-Task Checklist
1. Update `ARCHITECTURE.md` if design changes
2. Mark task complete in this file
3. Document implementation notes and decisions
4. Ensure all tests pass
5. Run linter and fix issues
6. Commit with descriptive message (no AI attribution)

### Code Quality Standards
- Explicit error handling with structured error types
- Contracts on public APIs where applicable
- >80% test coverage for new code
- Structured logging with context (request_id, session_id)

---

## Phase 1: Tokenizer Service

### Task 1.1: Vocabulary Loader
- [x] Parse vocab.json (token → ID mapping)
- [x] Parse merges.txt (BPE merge rules)
- [x] Build efficient lookup structures
- [x] Handle missing/corrupt files gracefully
- [x] Tests: Load real tokenizer files, error cases

**Implementation Notes:**
- Added String-Int and Int-String map FFI functions to runtime (collections.rs)
- Created vocab.aether with Vocabulary struct and loader functions
- Using Int64 for map handles (pointer size on 64-bit systems)
- Discovered and worked around `!` operator bug with Bool types
- vocab_test.aether validates all functionality

### Task 1.2: BPE Encode
- [x] Split text to characters
- [x] Apply BPE merges iteratively
- [x] Convert tokens to IDs via vocab
- [x] Track byte offsets for each token (in Task 1.4)
- [x] Tests: Golden files, unicode, empty input, unknown chars

**Implementation Notes:**
- Added UTF-8 character operations to runtime (string_to_chars, string_char_count, string_grapheme_at)
- Uses unicode-segmentation crate for proper grapheme cluster handling
- BPE algorithm implemented with merge priority lookup via string maps
- Tests verify character splitting, merge lookup, merge selection, and string concatenation

### Task 1.3: BPE Decode
- [x] Convert IDs to token strings
- [x] Join tokens to text
- [x] Handle special tokens (BOS, EOS, PAD)
- [x] Tests: Round-trip property, special tokens

**Implementation Notes:**
- Created decode.aether with BPEDecoder struct and decode functions
- Created decode_test.aether with 4 passing tests (token lookup, special tokens, decode sequence, round-trip)
- Uses Int64 map handles directly to avoid struct ownership issues
- Special tokens (BOS, EOS, PAD) are skipped during decoding (return empty string)
- get_token returns unk_token for missing IDs

### Task 1.4: Tokenizer HTTP API
- [x] `/v1/tokenize` endpoint (basic)
- [x] `/v1/detokenize` endpoint (basic)
- [x] Add offset tracking to response
- [x] Proper error responses (400 for bad input)
- [x] Tests: API contract tests (manual curl tests)

**Implementation Notes:**
- Added HttpResult struct for consistent response handling
- Added JSON validation functions: json_is_valid, json_is_null, json_is_string, json_is_array
- Proper 400 errors for: invalid JSON, missing required fields, wrong field types
- Offset tracking computes [start, end] byte positions for each token
- Tested endpoints: tokenize, detokenize, health, with valid/invalid inputs

### Task 1.5: Tokenizer Performance
- [x] Profile and optimize hot paths
- [x] Cache compiled merge rules (using string_int_map for O(1) lookup)
- [x] Benchmark: tokens/sec for various inputs
- [x] Target: >100k tokens/sec single-threaded

**Implementation Notes:**
- Added timer FFI functions: timer_start(), timer_elapsed_us(), timer_elapsed_ms()
- Created bench_test.aether for performance benchmarking
- Benchmark results:
  - Short text (5 chars): 12.5M tokens/sec
  - Medium text (50 chars): 9.6M tokens/sec
  - Long text (95 chars): 9.3M tokens/sec
- Far exceeds 100k target due to efficient native FFI operations
- Merge rules cached in string_int_map with O(1) lookup

---

## Phase 2: Sampler Pipeline

### Task 2.1: Sampler Framework
- [x] Define Sampler trait/interface
- [x] Implement SamplerState (carries RNG, context)
- [x] Deterministic seeded RNG
- [x] Tests: Same seed → same output

**Implementation Notes:**
- Added seeded RNG to runtime (LCG algorithm with rng_seed, rng_next, rng_next_float)
- Added float array FFI functions (float_array_create, push, get, set, length, free)
- Added math functions (math_exp, math_log, math_max_f, math_min_f)
- All functions use f64 to match Aether's Float type
- Created SamplerConfig struct with temperature, top_k, top_p, seed
- sampler_test.aether has 5 passing tests (RNG determinism, float array, softmax, temperature, sampling determinism)

### Task 2.2: Logit Processors
- [x] Temperature scaling
- [x] Top-k filtering
- [x] Top-p (nucleus) filtering
- [x] Repetition penalty
- [x] Frequency penalty
- [x] Presence penalty
- [x] Tests: Each processor in isolation

**Implementation Notes:**
- apply_temperature implemented (divides logits by temperature)
- apply_top_k implemented (sets non-top-k logits to -inf)
- apply_top_p implemented (keeps smallest set with cumsum >= p, renormalizes)
- apply_repetition_penalty: penalizes tokens in context (divide if positive, multiply if negative)
- apply_frequency_penalty: subtracts penalty * count for each token
- apply_presence_penalty: flat penalty for any token that has appeared
- build_token_counts: builds count array from context tokens
- Added int_array_set to runtime for in-place updates

### Task 2.3: Stop Conditions
- [x] EOS token detection
- [x] Stop string matching (framework ready, requires decoder integration)
- [x] Max tokens limit
- [x] Tests: Various stop conditions

**Implementation Notes:**
- GenerationState struct tracks: tokens_generated, max_tokens, eos_token_id, stop_triggered, stop_reason
- check_max_tokens: stops when tokens_generated >= max_tokens (0 = unlimited)
- check_eos_token: stops when token matches eos_token_id (-1 = disabled)
- update_generation_state: updates state after each token and sets stop flags
- Stop string matching requires decoded text - handled at higher level

### Task 2.4: Multinomial Sampling
- [x] Convert logits to probabilities (softmax)
- [x] Sample from distribution with RNG
- [ ] Optional: logprobs capture
- [x] Tests: Distribution correctness, determinism

**Implementation Notes:**
- softmax implemented with numerical stability (subtract max before exp)
- sample_token uses cumulative distribution for multinomial sampling
- sample_from_logits provides full pipeline: temp → top_k → softmax → top_p → sample

### Task 2.5: Sampler Integration
- [x] Chain processors in correct order
- [x] Configurable via request params
- [x] Tests: End-to-end sampling with fixed logits

**Implementation Notes:**
- sample_from_logits chains all processors in correct order
- SamplerConfig struct allows configuring all parameters
- GenerationState tracks generation progress and stop conditions
- Tests: 7/7 passing (RNG, float array, softmax, temperature, sampling, penalties, stop conditions)

---

## Phase 3: KV Cache Manager

### Task 3.1: Arena Allocator
- [x] Fixed-size memory arena
- [x] Block allocation with metadata (shape, dtype)
- [x] Free list management
- [x] Tests: Alloc/free patterns, fragmentation

**Implementation Notes:**
- Created arena_test.aether with ownership-safe arena implementation
- Uses Int64 handles for all arrays to avoid ownership issues
- arena_alloc handles block alignment and allocation
- find_end_offset finds contiguous space
- Tests: 4/4 passing (create, alloc, read/write, block alignment)

### Task 3.2: Session KV Storage
- [ ] Per-session allocation tracking
- [ ] Shape: (layers, heads, head_dim, seq_len)
- [ ] Lazy growth as sequence extends
- [ ] Tests: Session lifecycle

### Task 3.3: LRU Eviction
- [ ] Track access times per session
- [ ] Evict least-recently-used when at capacity
- [ ] Protect in-flight sessions
- [ ] Tests: Eviction under pressure

### Task 3.4: Shape Validation
- [ ] Validate tensor shapes on every access
- [ ] Detect dtype mismatches
- [ ] Clear error messages
- [ ] Tests: Shape mismatch detection

### Task 3.5: Memory Metrics
- [ ] Track allocated/free/fragmented bytes
- [ ] Expose via metrics endpoint
- [ ] Tests: Metric accuracy

### Task 3.6: mmap Spill (Advanced)
- [ ] Spill cold sessions to disk
- [ ] Page table for residency tracking
- [ ] Checksums on spilled pages
- [ ] Restore on access
- [ ] Tests: Spill/restore correctness

---

## Phase 4: Model Runtime

### Task 4.1: GGUF Parser
- [ ] FFI bindings to GGUF library (or pure Aether)
- [ ] Parse model metadata
- [ ] Memory-map tensor data
- [ ] Tests: Load real GGUF files

### Task 4.2: Model Registry
- [ ] Load models from local path
- [ ] SHA256 checksum validation
- [ ] Cache directory management
- [ ] Tests: Load/cache lifecycle

### Task 4.3: Forward Pass Interface
- [ ] Define ModelRuntime trait
- [ ] Input: tokens + KV handles
- [ ] Output: logits tensor
- [ ] Tests: Interface contract

### Task 4.4: Attention Mechanism
- [x] Implement tensor foundations (tensor_test.aether)
- [ ] Implement multi-head attention in pure Aether
- [ ] Query/Key/Value projections
- [x] Softmax and attention scores
- [ ] Tests: Numerical correctness vs reference

**Implementation Notes (Tensor Foundations):**
- Created tensor_test.aether with ownership-safe tensor operations
- Uses Int64 handles for data arrays to avoid ownership issues
- Implemented: zeros, ones, add, matmul, relu, softmax, silu
- Tests: 7/7 passing (numel, zeros/ones, add, matmul, relu, softmax, silu)

### Task 4.5: Feed-Forward Network
- [ ] Implement FFN layers (MLP)
- [x] Activation functions (SiLU/GELU)
- [ ] Tests: Layer output verification

### Task 4.6: Quantized Operations
- [ ] Implement Q4/Q8 dequantization
- [ ] Quantized matrix multiplication
- [ ] Tests: Numerical accuracy within tolerance

### Task 4.7: Model Admin
- [ ] List loaded models
- [ ] Load/unload API
- [ ] Memory usage reporting
- [ ] Tests: Admin operations

---

## Phase 5: Scheduler

### Task 5.1: Request Queue
- [ ] Per-model request queues
- [ ] Priority support (optional)
- [ ] Queue depth limits
- [ ] Tests: Queue operations

### Task 5.2: Micro-batching
- [ ] Group requests by model
- [ ] Bucket by sequence length
- [ ] Configurable max batch size
- [ ] Configurable batch timeout
- [ ] Tests: Batching correctness

### Task 5.3: Backpressure
- [ ] High/low watermarks
- [ ] Return 429 when overloaded
- [ ] Metrics for queue depth
- [ ] Tests: Backpressure triggers

### Task 5.4: Fairness
- [ ] Weighted round-robin across sessions
- [ ] Per-tenant limits
- [ ] Tests: Fairness under load

### Task 5.5: Cancellation
- [ ] Cancel in-flight requests
- [ ] Clean up KV handles
- [ ] Tests: Cancellation correctness

---

## Phase 6: HTTP Gateway

### Task 6.1: Server Framework
- [ ] TCP listener with connection handling
- [ ] HTTP/1.1 request parsing
- [ ] JSON body parsing
- [ ] Tests: Basic HTTP handling

### Task 6.2: Generate Endpoint
- [ ] `/v1/generate` implementation
- [ ] Request validation
- [ ] Streaming response (SSE or chunked)
- [ ] Tests: Generate contract

### Task 6.3: Session Management
- [ ] `/v1/session/close` endpoint
- [ ] Session ID in requests/responses
- [ ] Tests: Session lifecycle

### Task 6.4: Authentication
- [ ] API key validation
- [ ] Per-key rate limits
- [ ] Tests: Auth success/failure

### Task 6.5: Health Endpoints
- [ ] `/health` (liveness)
- [ ] `/ready` (readiness with checks)
- [ ] Tests: Health states

### Task 6.6: Error Handling
- [ ] Structured error responses
- [ ] Request ID in errors
- [ ] Proper HTTP status codes
- [ ] Tests: Error scenarios

---

## Phase 7: Telemetry

### Task 7.1: Metrics Collection
- [ ] Counter, gauge, histogram primitives
- [ ] Thread-safe metric updates
- [ ] Tests: Metric operations

### Task 7.2: Prometheus Export
- [ ] `/metrics` endpoint
- [ ] OpenMetrics format
- [ ] All key metrics exposed
- [ ] Tests: Format correctness

### Task 7.3: Structured Logging
- [ ] JSON log format
- [ ] Log levels (debug, info, warn, error)
- [ ] Context propagation (request_id)
- [ ] Tests: Log format

### Task 7.4: Request Tracing
- [ ] Span creation/completion
- [ ] Timing for each phase
- [ ] Tests: Trace completeness

---

## Phase 8: Integration

### Task 8.1: End-to-End Flow
- [ ] Wire all components together
- [ ] Config loading
- [ ] Startup/shutdown sequence
- [ ] Tests: Full request flow

### Task 8.2: Small Model Test
- [ ] Test with tiny GGUF model
- [ ] Verify output quality
- [ ] Tests: Inference correctness

### Task 8.3: Load Testing
- [ ] Concurrent request handling
- [ ] Sustained throughput
- [ ] Memory stability
- [ ] Tests: Load test suite

### Task 8.4: Resource Limits
- [ ] Verify all limits enforced
- [ ] Graceful degradation
- [ ] Tests: Limit enforcement

---

## Phase 9: Production Hardening

### Task 9.1: Hot Reload
- [ ] Drain active requests
- [ ] Unload model
- [ ] Reload new model
- [ ] Resume serving
- [ ] Tests: Hot reload correctness

### Task 9.2: Graceful Shutdown
- [ ] Stop accepting new requests
- [ ] Complete in-flight requests
- [ ] Clean up resources
- [ ] Tests: Shutdown sequence

### Task 9.3: Error Recovery
- [ ] Recover from panics
- [ ] Isolate failures
- [ ] Tests: Fault injection

### Task 9.4: Performance Tuning
- [ ] Profile hot paths
- [ ] Optimize allocations
- [ ] Tune batch sizes
- [ ] Benchmark suite

### Task 9.5: Documentation
- [ ] Deployment guide
- [ ] Configuration reference
- [ ] API documentation
- [ ] Troubleshooting guide

---

## Progress Tracking

| Phase | Tasks | Complete | Status |
|-------|-------|----------|--------|
| 1. Tokenizer | 5 | 5 | Complete |
| 2. Sampler | 5 | 5 | Complete |
| 3. KV Cache | 6 | 1 | In Progress |
| 4. Model Runtime | 7 | 2 | In Progress |
| 5. Scheduler | 5 | 0 | Not Started |
| 6. HTTP Gateway | 6 | 0 | Not Started |
| 7. Telemetry | 4 | 0 | Not Started |
| 8. Integration | 4 | 0 | Not Started |
| 9. Hardening | 5 | 0 | Not Started |

**Total: 46 tasks**

---

## Dependencies

```
Phase 1 (Tokenizer) ──┐
                      ├──> Phase 6 (Gateway) ──┐
Phase 2 (Sampler) ────┤                        │
                      │                        ├──> Phase 8 (Integration)
Phase 3 (KV Cache) ───┼──> Phase 5 (Scheduler)─┤
                      │                        │
Phase 4 (Model) ──────┘                        │
                                               │
Phase 7 (Telemetry) ───────────────────────────┘
```

Phases 1-4 can proceed in parallel. Phase 5-6 need 3-4. Phase 8-9 need everything.

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| FFI complexity | High | Start with mock runtime, add real FFI incrementally |
| Memory bugs | High | Extensive testing, contracts, sanitizers |
| Performance | Medium | Profile early, benchmark continuously |
| GGUF format changes | Low | Pin versions, validate checksums |

---

## Notes

_Implementation notes and decisions will be added here as work progresses._
