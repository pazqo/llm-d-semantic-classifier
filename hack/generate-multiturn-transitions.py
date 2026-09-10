#!/usr/bin/env python3
"""Generate a tentative balanced multi-turn complexity transition set."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evals/datasets/multiturn-complexity-transitions.jsonl"

LABELS = ["SIMPLE", "MEDIUM", "COMPLEX", "REASONING"]

TOPICS = [
    ("water", "What is the boiling point of water?", "water boils at 100 degrees Celsius at sea level."),
    ("Portugal", "What is the capital of Portugal?", "the capital of Portugal is Lisbon."),
    ("mortgage", "What is a fixed-rate mortgage?", "a fixed-rate mortgage keeps its interest rate constant."),
    ("CSV files", "How do I read a CSV file in Python?", "Python's csv module can read rows from a CSV file."),
    ("binary search", "What is the time complexity of binary search?", "binary search runs in O(log n) time on sorted data."),
    ("heart chambers", "What are the four chambers of the heart?", "the chambers are the two atria and two ventricles."),
    ("TLS certificates", "What is a TLS certificate?", "it binds a site identity to a public key for secure connections."),
    ("rainfall", "How is rainfall measured?", "a rain gauge measures the depth of precipitation."),
    ("inventory", "What is inventory turnover?", "it measures how often inventory is sold and replaced."),
    ("solar eclipse", "What causes a solar eclipse?", "the Moon passes between Earth and the Sun."),
]

SOURCE_PROMPTS = {
    "SIMPLE": "What is {topic} in one sentence?",
    "MEDIUM": "Explain {topic} with a practical example and mention one important limitation.",
    "COMPLEX": "Design a production system involving {topic}, including data flow, failure handling, and operational safeguards.",
    "REASONING": "Prove or derive the key claim about {topic}, making every assumption and intermediate step explicit.",
}

TARGET_PROMPTS = {
    "SIMPLE": "Give me the short answer about {topic}.",
    "MEDIUM": "Explain how {topic} works with one practical example and a brief comparison.",
    "COMPLEX": "Design an end-to-end solution involving {topic} with constraints, failure handling, and an implementation plan.",
    "REASONING": "Work through a rigorous proof or derivation involving {topic}, including assumptions and intermediate steps.",
}


def main() -> None:
    rows = []
    for source in LABELS:
        for target in LABELS:
            for index, (topic, question, answer) in enumerate(TOPICS, start=1):
                source_text = SOURCE_PROMPTS[source].format(topic=topic)
                current_text = TARGET_PROMPTS[target].format(topic=topic)
                rows.append(
                    {
                        "conversation_id": f"transition-{source.lower()}-{target.lower()}-{index:02d}",
                        "turn_id": 2,
                        "history": [
                            {"role": "user", "text": source_text},
                            {"role": "assistant", "text": f"A prior answer established that {answer}"},
                        ],
                        "current_user_turn": current_text,
                        "gold_tier": target,
                        "previous_gold_tier": source,
                        "context_required": True,
                        "case": f"{source}->{target}",
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
