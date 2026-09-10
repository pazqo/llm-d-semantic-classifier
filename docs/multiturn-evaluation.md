# Multiturn classification

This change adds a `ClassifyChain` RPC for classifying a list of conversation
messages. The request includes a strategy so callers can choose how the
conversation is turned into classifier input:

- `CONVERSATION_STRATEGY_LAST_MESSAGE`: classify only the final user message.
- `CONVERSATION_STRATEGY_FULL_HISTORY`: concatenate all messages with their
  roles (`system:`, `user:`, and `assistant:`). This is also the default for an
  unspecified strategy.

The RPC keeps the existing classification path, telemetry, cache, admission
control, and signal handling. It requires at least one message, non-empty
content, and a final message from the user.

## Build and test

From the repository root:

```bash
cargo fmt --check
cargo check --all-targets
cargo test --lib
cargo test --test grpc i001_real_tonic_round_trip
```

The integration test exercises both the chain RPC and the last-message
strategy against the local server.

## Generating evaluation data

The generators create synthetic JSONL files. They are intentionally kept out
of this contribution until the desired datasets have been selected for
upload:

```bash
python3 hack/generate-multiturn-ambiguous.py
python3 hack/generate-multiturn-long-answer.py
python3 hack/generate-multiturn-transitions.py
```

The corresponding Markdown files in `evals/datasets/` describe each dataset,
its intended use, and the commands used to evaluate it. The generated files
are expected at:

```text
evals/datasets/multiturn-complexity-ambiguous.jsonl
evals/datasets/multiturn-complexity-long-answer.jsonl
evals/datasets/multiturn-complexity-transitions.jsonl
```

## Local evaluation harness

The direct harness supports current-only, full-history, bounded-history, and
split-weighted inputs. Split-weighted embeds the current turn and history
separately, then combines their normalized vectors. Its default is 40% current
turn and 60% history; use `--current-weight` and `--history-weight` to change
that balance.

```bash
cargo run --release --bin eval-multiturn -- \
  --model artifacts/models/complexity \
  --classifier classifiers/complexity.json \
  --dataset evals/datasets/multiturn-complexity-ambiguous.jsonl \
  --mode all \
  --json /tmp/multiturn-ambiguous-report.json
```

The `--max-length` option lets you reproduce smaller or larger tokenizer
windows, for example `--max-length 50` or `--max-length 512`.

## RPC strategy comparison

`eval-chain-strategies` sends every record through the actual gRPC chain
endpoint and compares the two strategies:

```bash
cargo run --release --bin eval-chain-strategies -- \
  --model artifacts/models/complexity \
  --classifier complexity \
  --dataset evals/datasets/multiturn-complexity-ambiguous.jsonl
```

On the 40-record synthetic ambiguous benchmark, the preliminary results were:

| Strategy | Accuracy |
| --- | ---: |
| Last message | 25.0% (10/40) |
| Full history | 97.5% (39/40) |

For comparison, the direct harness measured 25.0% current-only, 97.5%
full-history, 92.5% bounded-history, and 100.0% split-weighted on the same
dataset. On the 40-record long-answer benchmark, the corresponding results
were 25.0%, 50.0%, 52.5%, and 62.5%. These are preliminary synthetic
benchmarks, useful for regression testing rather than production accuracy
claims.

