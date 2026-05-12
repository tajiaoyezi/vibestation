#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${ROOT_DIR}/web/tests/fixtures/syntax-highlight"
SEED="${SEED:-mvp-15-phase-d-v1}"

mkdir -p "${OUT_DIR}"

SEED="${SEED}" OUT_DIR="${OUT_DIR}" node --input-type=module <<'NODE'
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { TextEncoder } from "node:util";

const outDir = process.env.OUT_DIR;
const seed = process.env.SEED;
const encoder = new TextEncoder();

if (!outDir || !seed) {
  throw new Error("OUT_DIR and SEED are required");
}

const MB = 1024 * 1024;
const source = "scripts/fixtures/generate-syntax-highlight-fixtures.sh";

function byteLength(text) {
  return encoder.encode(text).length;
}

function repeatToSize(header, blockFactory, targetBytes) {
  const chunks = [header];
  let totalBytes = byteLength(header);
  let index = 0;

  while (totalBytes < targetBytes) {
    const block = blockFactory(index);
    chunks.push(block);
    totalBytes += byteLength(block);
    index += 1;
  }

  return chunks.join("");
}

function writeFixture(fileName, headerPrefix, targetBytes, blockFactory) {
  const header = `${headerPrefix} AUTO-GENERATED · ${source} · seed=${seed}\n`;
  const content = repeatToSize(header, blockFactory, targetBytes);
  writeFileSync(join(outDir, fileName), content, "utf8");
  console.log(`${fileName}\t${byteLength(content)} bytes`);
}

const tsBlock = (index) => {
  const id = `${seed.replace(/[^a-zA-Z0-9]/g, "_")}_${index}`;
  return `
export interface FixturePayload${index} {
  readonly id: string;
  readonly sequence: number;
  readonly tags: readonly string[];
}

export class FixtureService${index} {
  private readonly cache = new Map<string, FixturePayload${index}>();

  constructor(private readonly namespace = "${id}") {}

  hydrate(input: FixturePayload${index}): FixturePayload${index} {
    const next = {
      ...input,
      id: \`\${this.namespace}:\${input.id}\`,
      tags: [...input.tags, "syntax-highlight", "typescript"],
    };
    this.cache.set(next.id, next);
    return next;
  }

  serialize(): string {
    return JSON.stringify(Array.from(this.cache.values()), null, 2);
  }
}
`;
};

const rustBlock = (index) => `
pub struct FixturePayload${index} {
    pub id: String,
    pub sequence: usize,
    pub tags: Vec<String>,
}

impl FixturePayload${index} {
    pub fn hydrate(mut self) -> Self {
        self.tags.push("syntax-highlight".to_string());
        self.tags.push("rust".to_string());
        self
    }

    pub fn summary(&self) -> String {
        format!("{}:{}:{}", self.id, self.sequence, self.tags.join(","))
    }
}
`;

const pythonBlock = (index) => `
from dataclasses import dataclass, field


@dataclass(frozen=True)
class FixturePayload${index}:
    id: str
    sequence: int
    tags: tuple[str, ...] = field(default_factory=tuple)

    def hydrate(self) -> "FixturePayload${index}":
        return FixturePayload${index}(
            id=f"${seed}:{self.id}",
            sequence=self.sequence,
            tags=(*self.tags, "syntax-highlight", "python"),
        )
`;

const goBlock = (index) => `
package fixtures

type FixturePayload${index} struct {
	ID       string
	Sequence int
	Tags     []string
}

func HydrateFixture${index}(input FixturePayload${index}) FixturePayload${index} {
	next := FixturePayload${index}{
		ID:       "${seed}-" + input.ID,
		Sequence: input.Sequence,
		Tags:     append(input.Tags, "syntax-highlight", "go"),
	}
	return next
}
`;

const javaBlock = (index) => `
package fixtures;

import java.util.List;

public final class FixturePayload${index} {
    private final String id;
    private final int sequence;
    private final List<String> tags;

    public FixturePayload${index}(String id, int sequence, List<String> tags) {
        this.id = id;
        this.sequence = sequence;
        this.tags = List.copyOf(tags);
    }

    public String summary() {
        return id + ":" + sequence + ":" + String.join(",", tags);
    }
}
`;

writeFixture("1mb-typescript.ts", "//", 1 * MB, tsBlock);
writeFixture("10mb-typescript.ts", "//", 10 * MB, tsBlock);
writeFixture("50mb-typescript.ts", "//", 50 * MB, tsBlock);
writeFixture("1mb-rust.rs", "//", 1 * MB, rustBlock);
writeFixture("1mb-python.py", "#", 1 * MB, pythonBlock);
writeFixture("1mb-go.go", "//", 1 * MB, goBlock);
writeFixture("1mb-java.java", "//", 1 * MB, javaBlock);
NODE
