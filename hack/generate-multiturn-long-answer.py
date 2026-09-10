#!/usr/bin/env python3
"""Generate an ambiguity benchmark with long assistant answers."""

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evals/datasets/multiturn-complexity-long-answer.jsonl"
SOURCE = ROOT / "hack/generate-multiturn-ambiguous.py"

spec = importlib.util.spec_from_file_location("ambiguous_source", SOURCE)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

LONG_DETAILS = {
    "SIMPLE": (
        "The answer is a direct factual lookup, so the important qualification is to state the "
        "reference point clearly. It is useful to distinguish the ordinary definition from nearby "
        "exceptions, regional conventions, historical changes, and measurement assumptions. "
        "Those qualifications do not require a design, implementation, comparison of many options, "
        "or formal proof. A concise answer should identify the fact, give the usual unit or name, "
        "and mention the one condition that could change the interpretation. If a reader needs more "
        "detail, the answer can provide a short example, a source to consult, and a clarification "
        "question. The response should avoid inventing precision that the question does not request. "
        "In particular, it should not turn a straightforward lookup into a long plan or an extended "
        "argument. The appropriate next step is simply to apply the stated fact to the immediate "
        "question and check whether the user meant a different location, date, unit, or convention."
    ),
    "MEDIUM": (
        "A useful explanation should introduce the main concept, walk through one concrete example, "
        "and identify the most important limitation. Start by defining the terms in plain language, "
        "then show the sequence of operations or decisions a reader would actually follow. Include "
        "a small numerical or code-like illustration where that makes the distinction clearer. "
        "After the example, explain what changes when an assumption no longer holds and name a "
        "reasonable alternative. The answer should be detailed enough for a reader to apply the idea "
        "but should not become a full production architecture, migration program, or formal proof. "
        "A short checklist can make the result practical: identify the inputs, perform the central "
        "operation, inspect the output, and validate the common failure case. End by asking which "
        "environment or constraint matters most if the user wants a tailored version."
    ),
    "COMPLEX": (
        "A production solution needs an explicit decomposition into components, ownership boundaries, "
        "data flows, and operational controls. Begin with the success criteria and constraints, then "
        "choose interfaces that allow components to evolve independently. Describe how state is stored, "
        "replicated, validated, and recovered after partial failure. Include idempotency, retries, "
        "timeouts, backpressure, observability, access control, and a rollback or migration path. "
        "The design should explain what happens during dependency failure, duplicate delivery, stale "
        "data, a regional outage, and an operator mistake. It should also identify the decisions that "
        "need load testing and the metrics that would determine whether the system is healthy. A "
        "credible implementation plan should start with a bounded capability, introduce compatibility "
        "interfaces, validate behavior in parallel, and expand only after recovery procedures have "
        "been exercised. The final recommendation must balance reliability, latency, cost, and team "
        "complexity rather than optimizing one dimension in isolation."
    ),
    "REASONING": (
        "A rigorous argument should first state the claim, define every variable, and list the "
        "assumptions under which the claim is supposed to hold. Establish a base case or initial "
        "condition, then identify an invariant, recurrence, likelihood, or comparison that connects "
        "one step to the next. Work through the algebra or logical implication explicitly instead of "
        "relying on an intuitive diagram. Check edge cases, boundary conditions, and the possibility "
        "that a denominator, independence assumption, or termination condition fails. After deriving "
        "the candidate result, verify that it satisfies the original constraints and explain why an "
        "alternative conclusion does not follow from the premises. If the claim is conditional, state "
        "exactly which assumptions are necessary and what counterexample would invalidate it. A final "
        "summary should distinguish what has been proved from what is only suggested by an example or "
        "empirical observation. This level of detail is needed because the requested outcome depends "
        "on a chain of intermediate steps rather than on recalling a single fact."
    ),
}


def main() -> None:
    rows = []
    for target, cases in module.CASES.items():
        for index, (topic, question, answer) in enumerate(cases, start=1):
            long_answer = f"{answer} {LONG_DETAILS[target]}"
            rows.append(
                {
                    "conversation_id": f"long-answer-{target.lower()}-{index:02d}",
                    "turn_id": 3,
                    "history": [
                        {"role": "user", "text": "What should I remember from our earlier discussion?"},
                        {"role": "assistant", "text": "Remember the main definition and the immediate constraint."},
                        {"role": "user", "text": question},
                        {"role": "assistant", "text": long_answer},
                    ],
                    "current_user_turn": "Can you do that?",
                    "gold_tier": target,
                    "context_required": True,
                    "history_required": True,
                    "answer_token_target": 200,
                    "ambiguity_type": "anaphora_with_long_answer",
                    "rationale": f"The phrase refers to the prior task about {topic}; its answer is intentionally long.",
                    "case": f"long-answer->{target}",
                    "expected_status": "OK",
                }
            )

    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w") as stream:
        for row in rows:
            stream.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(f"wrote {len(rows)} records to {OUT}")


if __name__ == "__main__":
    main()
