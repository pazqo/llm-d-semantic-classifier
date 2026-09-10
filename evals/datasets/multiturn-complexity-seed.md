# Multi-turn complexity seed set

`multiturn-complexity-seed.jsonl` is a tentative, human-reviewable evaluation
set for context-dependent per-turn classification. It is not yet a benchmark
fixture and is not consumed by the existing single-prompt evaluation scripts.

Each record represents one target user turn. `history` contains the canonical
prior transcript, `current_user_turn` is the text to classify, and
`gold_tier` is the tentative complexity label for the target turn. The
`canonical_assistant_answer` is included because the assistant response can
provide referents and facts needed by a later turn; it should not be treated as
the only valid answer.

The set includes:

- independent first turns;
- anaphora and contextual fact lookup;
- contextual architecture, planning, incident, and code follow-ups;
- a reasoning follow-up whose difficulty depends on the preceding explanation;
- ambiguous `ABSTAIN` versus context-supported `OK` cases;
- a context-dependent summarization case.

## Running it

With the complexity ModelCar available locally:

```bash
cargo run --release --bin eval-multiturn -- \
  --model artifacts/models/complexity \
  --classifier classifiers/complexity.json \
  --dataset evals/datasets/multiturn-complexity-seed.jsonl \
  --mode all \
  --json /tmp/multiturn-report.json
```

`current-only` ignores history, `full-history` serializes the complete
transcript and lets the model's normal truncation apply, and `bounded-history`
puts the current turn first and adds the newest complete prior turns that fit
the tokenizer window. The JSON report contains one result per turn and mode,
including the number of history turns used and the actual token count.

Before using this for reported accuracy, reviewers should verify the labels,
add alternate valid assistant answers, and add incomplete-history variants.
Evaluation should compare current-turn-only, full-history, and bounded-summary
representations. The expected status fields are prospective: the current
runtime does not yet implement conversation-aware abstention.
