#!/usr/bin/env python3
"""Generate a context-dependent benchmark with deliberately ambiguous follow-ups."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evals/datasets/multiturn-complexity-ambiguous.jsonl"

CASES = {
    "SIMPLE": [
        ("the capital of Portugal", "What is the capital of Portugal?", "The capital is Lisbon."),
        ("the boiling point of water", "What is the boiling point of water?", "Water boils at 100 degrees Celsius at sea level."),
        ("the currency of Japan", "What currency is used in Japan?", "Japan uses the yen."),
        ("the chambers of the heart", "What are the four chambers of the heart?", "They are the right and left atria and ventricles."),
        ("the cause of a solar eclipse", "What causes a solar eclipse?", "The Moon passes between Earth and the Sun."),
        ("the chemical symbol for gold", "What is the chemical symbol for gold?", "The symbol is Au."),
        ("the number of days in a leap year", "How many days are in a leap year?", "A leap year has 366 days."),
        ("the author of Hamlet", "Who wrote Hamlet?", "William Shakespeare wrote Hamlet."),
        ("the largest planet", "Which is the largest planet in our solar system?", "Jupiter is the largest planet."),
        ("the speed of light", "What is the approximate speed of light?", "It is about 300,000 kilometres per second."),
    ],
    "MEDIUM": [
        ("binary search", "Explain how binary search works with an example.", "It repeatedly halves a sorted search interval; the resulting complexity is logarithmic."),
        ("fixed-rate mortgages", "Compare fixed-rate and adjustable-rate mortgages with an example.", "Fixed rates provide payment certainty while adjustable rates can change."),
        ("CSV parsing", "Write a small Python function that sums a CSV column.", "Use the csv module, convert the selected values, and accumulate the total."),
        ("TLS certificates", "Explain how TLS certificates establish trust.", "A certificate binds an identity to a public key and is validated through a trust chain."),
        ("rainfall measurement", "Explain how a rain gauge measures precipitation.", "The gauge collects water and reports its depth over a period."),
        ("inventory turnover", "Explain inventory turnover with a numerical example.", "It relates cost of goods sold to average inventory over a period."),
        ("database indexes", "Explain how a database index speeds up a lookup.", "An index avoids scanning every row by maintaining an ordered access structure."),
        ("HTTP caching", "Explain HTTP caching with a practical example.", "Cache validators and freshness directives let clients reuse a prior response."),
        ("electric circuits", "Explain the difference between series and parallel circuits.", "Series components share a path; parallel branches have separate paths."),
        ("public-key cryptography", "Explain public-key encryption with a small example.", "A public key can encrypt a message that only the matching private key decrypts."),
    ],
    "COMPLEX": [
        ("a multi-region payment service", "Design a multi-region payment service with idempotency and reconciliation.", "A robust design needs durable events, a ledger, regional failover, idempotency, and reconciliation."),
        ("a zero-downtime database migration", "Plan a zero-downtime migration from a monolith database.", "Use compatibility layers, dual writes or change capture, staged backfills, and rollback gates."),
        ("a Kubernetes control plane", "Design a highly available Kubernetes control plane for several clusters.", "The design must cover quorum, upgrades, failure domains, access control, and observability."),
        ("a reliable event pipeline", "Architect an event pipeline with ordering, retries, and exactly-once effects.", "The solution needs durable offsets, idempotent consumers, replay policy, and dead-letter handling."),
        ("a hospital scheduling platform", "Design a hospital scheduling platform with privacy and conflict resolution.", "The plan must combine authorization, auditability, scheduling constraints, and operational recovery."),
        ("a global file synchronization service", "Design a global file synchronization service with offline edits.", "The system needs conflict resolution, durable metadata, chunking, retries, and multi-device recovery."),
        ("a fraud detection platform", "Design a fraud detection platform for high-volume card transactions.", "The architecture must balance streaming features, model decisions, latency, review queues, and failover."),
        ("a public transit redesign", "Plan a city-wide public transit redesign under accessibility and budget constraints.", "The plan must balance coverage, frequency, capacity, accessibility, construction phases, and cost."),
        ("a disaster recovery program", "Design a disaster recovery program for a regulated financial service.", "It requires recovery objectives, regional isolation, backup validation, exercises, and audit evidence."),
        ("a large-scale search platform", "Architect a search platform with indexing, ranking, freshness, and tenant isolation.", "The design must cover ingestion, sharding, ranking, reindexing, query isolation, and operations."),
    ],
    "REASONING": [
        ("binary search correctness", "Prove the correctness and logarithmic complexity of binary search.", "A proof uses an invariant for the remaining interval and shows it shrinks by half each step."),
        ("the sum of odd numbers", "Prove that the sum of the first n odd numbers is n squared.", "Induction establishes the base case and shows adding the next odd number preserves the claim."),
        ("Bayes' theorem", "Derive Bayes' theorem from the definition of conditional probability.", "Equating the joint probability in two orders and dividing by the evidence gives the result."),
        ("a maximum likelihood estimate", "Derive the maximum likelihood estimate for a Gaussian mean.", "Take the log likelihood, differentiate, set the derivative to zero, and verify the optimum."),
        ("Dijkstra's algorithm", "Prove the correctness of Dijkstra's algorithm using an invariant.", "The invariant states that finalized vertices have their true shortest-path distances."),
        ("the central limit theorem", "Explain why the central limit theorem applies under the stated assumptions.", "The argument requires independence, finite variance, normalization, and convergence in distribution."),
        ("a probability bound", "Derive a concentration bound for the stated random variable.", "The derivation requires identifying the moment-generating bound and optimizing its parameter."),
        ("a calculus optimization", "Determine the global minimum of the given constrained function and justify it.", "Check stationary points, boundary behavior, feasibility, and comparison of candidate values."),
        ("a statistical confounder", "Show whether the observed association survives adjustment for the confounder.", "The reasoning must define the estimand, compare conditional effects, and state identification assumptions."),
        ("a distributed consistency claim", "Determine whether the consistency claim follows from the stated distributed-systems assumptions.", "The answer requires tracing the failure schedule and checking whether the guarantee covers it."),
    ],
}


def main() -> None:
    rows = []
    for target, cases in CASES.items():
        for index, (topic, question, answer) in enumerate(cases, start=1):
            rows.append(
                {
                    "conversation_id": f"ambiguous-{target.lower()}-{index:02d}",
                    "turn_id": 2,
                    "history": [
                        {"role": "user", "text": question},
                        {"role": "assistant", "text": answer},
                    ],
                    "current_user_turn": "Can you do that?",
                    "gold_tier": target,
                    "context_required": True,
                    "history_required": True,
                    "ambiguity_type": "anaphora",
                    "rationale": f"The phrase refers to the prior task about {topic}.",
                    "case": f"ambiguous->{target}",
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
