# Cache eviction proposal: measured LRU

Issue #9 asks whether the result cache's deliberate FIFO policy is leaving a
useful hot set behind, and asks for evidence before changing it.

## Current state

`ExactCache` stores results in a `HashMap` plus a `VecDeque`. A hit clones the
value but does not update the deque, so inserting a new key evicts the oldest
insertion rather than the least recently used key. The cache is protected by
the existing `SharedCache` mutex and the single-flight table remains separate.

## Proposed solution

Use Moka's synchronous cache with an explicit LRU eviction policy:

```rust
let cache = moka::sync::Cache::builder()
    .max_capacity(capacity as u64)
    .eviction_policy(moka::policy::EvictionPolicy::lru())
    .build();
```

Keep `SharedCache` as the owner of the outer synchronization and single-flight
coordination. Moka would replace only the `HashMap`/`VecDeque` storage and
eviction bookkeeping. The public `ExactCache` contract would remain unchanged:

- versioned `CacheKey` identity is unchanged;
- successful results only are cached;
- failed forwards are never cached;
- capacity remains bounded;
- hit/miss/forward/eviction counters remain exposed by the wrapper;
- single-flight waiters remain classified as `CachePath::Coalesced`.

Moka's policy bookkeeping is eventually consistent, so the wrapper must not
use an approximate entry count to enforce correctness. Capacity enforcement
belongs to Moka; the existing exact `len()` and eviction counters should either
be documented as observational or backed by explicit wrapper bookkeeping.

## Evidence required before migration

Add a deterministic trace benchmark with a bounded cache and compare FIFO and
LRU under at least these workloads:

1. uniform one-pass keys, where LRU should not be expected to help;
2. a recency-biased hot set mixed with cold keys;
3. repeated single-key hits, measuring hit-path overhead;
4. concurrent single-flight misses, verifying no duplicate forwards.

Report hit rate, forward count, eviction count, and p50/p95/p99 operation
latency. Migrate only if LRU improves the hot-set hit rate without an
unacceptable hit-path or memory cost.

## Open implementation choice

The architecture document currently says Moka is a candidate rather than an
existing dependency. This proposal intentionally does not add the dependency
until the maintainer confirms that Moka is the intended library for #9.
