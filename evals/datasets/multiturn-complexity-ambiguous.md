# Ambiguous current-turn benchmark

`multiturn-complexity-ambiguous.jsonl` contains 40 records, balanced across
the four complexity labels. Every target turn is exactly:

```text
Can you do that?
```

The preceding user question and assistant answer provide the only information
that identifies the intended task. This makes current-only classification a
deliberately weak baseline and tests whether conversation history is being
used.

Generate it with:

```bash
python3 hack/generate-multiturn-ambiguous.py
```

Evaluate it with:

```bash
cargo run --release --bin eval-multiturn -- \
  --model artifacts/models/complexity \
  --classifier classifiers/complexity.json \
  --dataset evals/datasets/multiturn-complexity-ambiguous.jsonl \
  --mode all \
  --json /tmp/multiturn-ambiguous-report.json
```

The `split-weighted` mode embeds the current turn and prior history separately,
then combines the normalized vectors. Its weights default to 0.4 current / 0.6
history and can be changed with `--current-weight` and `--history-weight`.

Initial results on the current model:

```text
current-only       25.0%
full-history       97.5%
bounded-history    92.5%
split-weighted    100.0%
```

These are encouraging harness results, not yet a production accuracy claim.
The examples are synthetic and should eventually be replaced or supplemented
with independently authored conversations and alternate valid assistant
answers.
