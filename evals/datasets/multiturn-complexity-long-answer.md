# Long-answer ambiguity benchmark

`multiturn-complexity-long-answer.jsonl` extends the ambiguous benchmark with
an older exchange and a deliberately long assistant answer. The answer is
designed to exceed 200 model tokens, and the complete transcript exceeds the
current 256-token input window for many records.

Generate it with:

```bash
python3 hack/generate-multiturn-long-answer.py
```

Evaluate it with:

```bash
cargo run --release --bin eval-multiturn -- \
  --model artifacts/models/complexity \
  --classifier classifiers/complexity.json \
  --dataset evals/datasets/multiturn-complexity-long-answer.jsonl \
  --mode all \
  --json /tmp/multiturn-long-answer-report.json
```

Initial results:

```text
current-only       25.0%
full-history       50.0%
bounded-history    52.5%
split-weighted     62.5%
```

The report shows that 19 of 40 full-history and bounded-history inputs reached
the 256-token cap. Bounded history kept the current turn ahead of the long
history and selected two prior messages on average, while full history placed
the current turn after the long answer and allowed right truncation to remove
it. These results are exploratory; the long synthetic answers should later be
replaced with independently authored answer variants.
