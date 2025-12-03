# Starling LLM Stack — Technical Architecture

## Vision
Starling is a lean, self-hosted LLM serving stack with a focus on correctness, observability, and predictable resource use. It exposes HTTP APIs for tokenization and text generation, manages per-session KV caches (in-RAM with optional mmap spill), and hosts one or more quantized GGUF models. The design favors explicit contracts, bounded queues, and deterministic cleanup to align with Aether’s ownership/verification goals.

## Goals
- Low-latency local inference for quantized GGUF models (e.g., Mistral/LLaMA families).
- Deterministic resource envelopes: bounded memory for KV caches, bounded concurrency per model, backpressure everywhere.
- Pluggable model registry and storage; hot reload/start/stop of models.
- First-class metrics/health and debuggability (token/s, latency histograms, cache hit/miss, per-layer memory).
- Modular tokenizer and sampler components for reuse and testing in isolation.


## System Overview
Components:
1) HTTP Gateway: JSON/stream endpoints for generate and tokenize; enforces auth, quotas, and request validation.
2) Session Manager: issues session IDs, maps sessions to KV caches, owns lifecycle (TTL, eviction).
3) Model Manager: loads/unloads models, tracks per-model limits (max sessions, max batch), exposes model handles to the scheduler.
4) Scheduler: batches decode tokens across active sessions per model; coordinates with KV Cache Manager; applies backpressure.
5) KV Cache Manager: allocates/resizes/evicts KV blocks; supports RAM arena + optional mmap-backed spill; exports metrics.
6) Tokenizer Service: BPE/WordPiece tokenizer with round-trip invariants; available as HTTP and in-process.
7) Sampler Pipeline: modular steps (temperature, top-k/p, repetition penalty, frequency/presence penalties); deterministic RNG seeding per request for reproducibility.
8) Telemetry: metrics (Prometheus/OpenMetrics), structured logs, traces; health/readiness endpoints.
9) Storage/Registry: local/remote registry for GGUF; verifies checksums; warms model weights into memory-mapped files.

## Data Flow (Generate)
1) Client → HTTP Gateway `/v1/generate` with model, prompt (tokens or text), parameters, and optional session_id.
2) Gateway validates payload, authenticates, and enforces per-tenant quotas; resolves/creates session via Session Manager.
3) Tokenizer encodes prompt (if text); Session Manager pins/creates KV cache for session.
4) Scheduler enqueues request in the model’s decode queue; applies admission control if queue/backpressure limits hit.
5) Scheduler forms micro-batches (by token budget, not time) and calls Model Runtime to produce next token logits using the session’s KV state.
6) Sampler selects next tokens per request; KV Cache Manager appends KV tensors; responses stream tokens back to client.
7) On completion/abort/timeout, Session Manager decrements references; KV Cache Manager may evict based on policy (LRU, size, TTL).

## APIs
### HTTP (JSON)
- `POST /v1/tokenize` → `{tokens:[int], offsets:[int]}`; accepts `text` or `tokens` (for detokenization round-trip).
- `POST /v1/generate` → streaming SSE or chunked JSON; fields: `model`, `prompt` (text|tokens), `max_tokens`, `temperature`, `top_k`, `top_p`, `repetition_penalty`, `presence_penalty`, `frequency_penalty`, `seed`, `session_id?`, `stop` (strings/tokens), `logprobs?`, `return_full_text?`.
- `POST /v1/session/close` → release KV for session.
- `GET /healthz`, `/readyz`, `/metrics`.

### Internal Traits (to map into Aether modules later)
- `Tokenizer::encode(text) -> Vec<Token>`; `decode(tokens) -> String`; guarantees canonical round-trip for valid vocab.
- `Sampler::sample(logits, sampler_state) -> Token`; pure + deterministic given seed and state.
- `KVCache::allocate(session, layers, heads, head_dim, seq_len)`; returns handles for key/value buffers; supports resize/evict.
- `ModelRuntime::forward(batch_inputs, kv_handles) -> logits`; abstraction over GGUF runtime.
- `Scheduler::enqueue(request) -> Result<handle, Rejection>`; batches by model; exposes backpressure signals.

## KV Cache Architecture
- Memory model: contiguous arena per model with fixed upper bound; optional mmap spill file segmented into pages; page table maps session + layer → (ptr, len, residency).
- Eviction: LRU by session with protection for active in-flight batches; two-phase: mark → drain → free.
- Resize: grow seq_len per session lazily; if out of quota, trigger eviction; if still out, reject with 429/backpressure.
- Concurrency: Session KV mutations go through the Scheduler thread; readers are the model runtime during forward only.
- Integrity: store shape/dtype metadata per block; assertions on every borrow; checksum optional on spill pages.

## Scheduler & Batching
- Token-level scheduling; micro-batches grouped by (model, dtype, seq_len bucket).
- Fairness: weighted round-robin across sessions; per-tenant limits to avoid starvation.
- Latency vs throughput: configurable max batch size, max queue delay (default 2–5 ms), and max tokens/sec per model.
- Backpressure: if queue > high-watermark or memory near limit, return 429 with retry-after; emit metrics.

## Model Manager & Registry
- Registry supports local fs path and remote URL; enforces SHA256/etag; caches models under `~/.starling/models`.
- Loader maps GGUF weights with mmap; validates tensor shapes; builds runtime graph (CPU for now).
- Hot reload: stop-taking-new, drain in-flight, unload KV arena, remap weights; exposed via admin API.

## Sampler Pipeline
Order: logit masking (eos/stop), repetition penalty, temperature scaling, top-k/p filtering, frequency/presence penalties, then multinomial draw. Seeded RNG; deterministic given seed + request id. Supports `logprobs` k-best capture.

## Metrics & Observability
- Metrics: request rate, p50/p95/p99 latency, tokens/sec, queue depth, batch size distribution, KV bytes allocated/resident/spilled, eviction counts, sampler time, model forward time, errors by type.
- Logs: structured JSON with request_id, session_id, model, timings, and sampler params; separate channel for errors.
- Traces: spans for gateway, tokenize, schedule, forward, sample; propagate request_id in responses.
- Health: `/healthz` = process up; `/readyz` = models loaded AND below memory pressure AND scheduler not overloaded.

## Configuration
- Models: list with `name`, `path/url`, `max_batch`, `max_sessions`, `kv_mem_limit_mb`, `spill_path?`.
- Runtime: `num_threads`, `intra_op_threads`, `queue_limits`, `batch_timeout_ms`.
- Policies: eviction (LRU/TTL), admission thresholds, rate limits per tenant.
- Security: optional API keys; CORS settings; limits on prompt size and max_tokens.

## Error Handling & Contracts
- Input validation errors → 400; quota/admission → 429; model missing → 404; internal → 500 with request_id.
- Contracts: tokenizer round-trip must hold for valid tokens; KV shapes must match model (layers, heads, head_dim); sampler must return in bounded time; scheduler must never drop in-flight references (all resources freed on completion/abort).
- Timeouts: per-request overall timeout; per-forward timeout; abort cascades to scheduler and frees KV handles.

## Concurrency Model
- Gateway: async per-connection.
- Scheduler: per-model worker thread(s) owning batch formation and KV mutations.
- Model runtime: runs on threadpool; returns logits into scheduler-owned buffers.
- KV cache: mutated only by scheduler threads; read during model forward; no shared mutable access outside scheduler.

## Testing Strategy
- Tokenizer: golden tests, round-trip tests, unicode edge cases.
- Sampler: deterministic fixtures with fixed logits/seed; property tests for probability mass invariants.
- KV cache: allocation/eviction/resize under load; spill/restore; shape/dtype mismatch detection.
- Scheduler: batching fairness and backpressure; soak tests with mixed seq lens and cancellations.
- Integration: end-to-end generate against a small GGUF model; load/unload; readiness gating.
- Perf: throughput/latency benchmarks; memory pressure + eviction behavior.

## Minimal Footprint (MVP)
- Defined core crates/modules layout.
- Codified public traits/interfaces (Tokenizer, Sampler, KVCache, Scheduler, ModelRuntime).
- Added shared types (RequestId, SessionId, ModelId, TokenId, Timestamp) and error enums.
- Wired config loader (YAML/JSON) with validation for limits.

## Extensions (Future)
- GPU/Metal/CUDA backend; quantization-aware runtime; kv-cache compression/quantization; multi-model routing; wasm sampler sandbox; distributed sharding.
