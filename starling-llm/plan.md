# Starling LLM — Delivery Plan

## Guiding Principles
- Test-first: add failing unit/integration tests before implementation.
- Bounded resources: every component must declare limits and enforce backpressure.
- Observability-first: metrics/logs/health for every subsystem.
- Minimal viable path: CPU-only, single model, RAM-only KV for v1; iterate toward extensions.

## Phase 0: Scaffolding & Interfaces
- [ ] Define core crates/modules layout (gateway, tokenizer, sampler, kv_cache, scheduler, model_runtime, registry, telemetry, config).
- [ ] Codify public traits/interfaces (Tokenizer, Sampler, KVCache, Scheduler, ModelRuntime).
- [ ] Add shared types (RequestId, SessionId, ModelId, TokenId, Timestamp) and error enums.
- [ ] Wire config loader (YAML/JSON) with validation for limits.

## Phase 1: Tokenizer Service
- [ ] Implement BPE/WordPiece tokenizer loader (from vocab/merges).
- [ ] Encode/decode with canonical round-trip; offsets.
- [ ] HTTP endpoints: `/v1/tokenize` and `/v1/detokenize`.
- [ ] Tests: golden fixtures, unicode edge cases, round-trip property, bad-token errors.

## Phase 2: Sampler Pipeline
- [ ] Implement sampler steps: masking (stop/eos), repetition penalty, temperature, top-k, top-p, freq/presence penalties, multinomial draw.
- [ ] Deterministic RNG (seeded per request).
- [ ] Tests: fixed logits + seeds → expected tokens; probability mass invariants; stop conditions.

## Phase 3: KV Cache Manager
- [ ] RAM arena allocator with shape/dtype metadata per block.
- [ ] Session lifecycle (allocate, resize, free); LRU eviction (session-level) with protected in-flight sessions.
- [ ] Integrity checks on every borrow; metrics for alloc/free/evict; optional spill API stub (future).
- [ ] Tests: allocate/resize/evict under load; shape mismatch detection; TTL eviction.

## Phase 4: Model Manager & Registry
- [ ] Registry: load GGUF from local path/URL; checksum/etag validation; cache directory management.
- [ ] Model loader: mmap weights, validate tensor shapes, expose ModelRuntime stub (CPU mock).
- [ ] Admin controls: list/load/unload models; enforce max_sessions/max_batch per model.
- [ ] Tests: registry cache hit/miss, checksum failure, load/unload lifecycle.

## Phase 5: Scheduler & Batching
- [ ] Per-model scheduler worker; request queues with high/low watermarks.
- [ ] Micro-batching by seq length/model; configurable batch size and max queue delay.
- [ ] Fairness: weighted round-robin across sessions; per-tenant limits.
- [ ] Backpressure: 429 on admission when above limits; metrics for queue depth, batch sizes.
- [ ] Tests: batching correctness, fairness under mixed seq, backpressure triggers, cancellation.

## Phase 6: HTTP Gateway
- [ ] Endpoints: `/v1/generate` (streaming SSE/chunked JSON), `/v1/session/close`, health (`/healthz`, `/readyz`), `/metrics`.
- [ ] Auth (API key), payload validation, per-tenant quotas, max prompt/max_tokens enforcement.
- [ ] Stream integration with scheduler and KV cache; session creation/lookup.
- [ ] Tests: request validation, auth failure, quota exceed, streaming happy path.

## Phase 7: Telemetry & Observability
- [ ] Prometheus/OpenMetrics export; histograms for latency, tokens/sec; gauges for KV memory.
- [ ] Structured logs with request_id/session_id; error logs with classification.
- [ ] Traces (spans for gateway, tokenize, schedule, forward, sample).
- [ ] Health/readiness gating on model load and resource pressure.
- [ ] Tests: metrics surface expected series; readiness flips under load/no models.

## Phase 8: End-to-End MVP
- [ ] Integrate CPU-only ModelRuntime that consumes a small GGUF (e.g., tiny model).
- [ ] Full generate path: text → tokenize → schedule → forward (mock/logits if needed) → sample → stream.
- [ ] Resource limits validated (KV cap, queue cap); graceful errors/timeouts.
- [ ] E2E tests against tiny model; soak test with concurrent sessions.

## Phase 9: Hardening & Extensions (post-MVP)
- [ ] KV spill to mmap; page table; checksum on spill pages.
- [ ] Hot reload models without downtime (drain + remap).
- [ ] Multi-model routing; per-model threadpools.
- [ ] Safety/policy filters; logprob return; logit bias; stop-sequences; EOS handling polish.
- [ ] Admin APIs: stats, drains, config reload.

## Deliverables per Phase
- Interfaces + docs.
- Tests (unit/integration) passing.
- Metrics/logging hooks where applicable.
- README updates as features land.

## Risks & Mitigations
- Memory blowup: enforce caps at config parse; reject when KV alloc > cap; eviction policy verified by tests.
- Latency regressions: keep batching small; add max batch delay; perf tests early.
- Model format drift: pin GGUF loader version; checksum validation; clear errors on mismatch.
- Stream correctness: property tests for sampler; SSE chunking tests for partial writes.
