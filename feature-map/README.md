# Script Kit GPUI Feature Map Atlas

This atlas is the maintained feature map for humans and AI agents. It separates raw Oracle evidence from edited feature chapters so the project can keep complete review output without forcing readers through every session log.

## How To Use This Atlas

- Start with [index.md](./index.md) to find a feature, its owner skill, its raw Oracle reference, and whether a distilled chapter exists.
- Read a feature chapter in [features/](./features/) when you need a human-readable operating model: capabilities, workflows, state, shortcuts, source ownership, and verification paths.
- Read [raw-oracle/](./raw-oracle/) when you need the full uncompressed Oracle answer, the exact prompt, the full session log, or the bundle metadata.
- Read [receipts/](./receipts/) when you need migration proof, verification notes, or the reusable Oracle loop prompt.

## Raw Versus Distilled

Raw Oracle output is evidence. It should not be overwritten during editing. Each run keeps:

- `prompt.md` for the exact Oracle prompt.
- `bundle-map.md` for the bundle/session pointer.
- `answer.md` for the extracted full Oracle answer.
- `output.log` for the complete Oracle session log.
- `session.json` for structured session metadata.

Distilled feature chapters are maintained docs. They should be readable without opening the raw answer, but they must link back to the raw files that seeded them.

## Chapter Standard

Each chapter should answer the questions an implementation agent asks before touching code:

- What can users do?
- Where does the feature start?
- What states can the UI be in?
- What keys, clicks, actions, and protocol calls matter?
- What code owns each behavior?
- What data is read, written, cached, or intentionally withheld?
- Which invariants must not regress?
- What is the smallest proof that can fail if the feature breaks?

The benchmark chapter is [features/006-notes-window.md](./features/006-notes-window.md). Use it as the shape target before distilling the remaining raw Oracle answers.

## Oracle Loop

The reusable loop prompt lives at [receipts/oracle-loop-prompt.md](./receipts/oracle-loop-prompt.md). The short version:

1. Pick a feature id and owner skills.
2. Run `lat expand` on the prompt and `lat search` for relevant contracts.
3. Bundle `AGENTS.md`, `CLAUDE.md`, owner skills, adjacent skills, relevant `lat.md/` pages, `lat.md/verification.md`, source, tests, and agentic scripts.
4. Save the prompt and bundle map before running Oracle.
5. Save the full answer and full `output.log` before distilling.
6. Write or update the feature chapter, update [index.md](./index.md), then run `lat check`.

## Compatibility

`../FEATURE_MAP.md` is a first-pass source and compatibility index. New feature-map work should land here.
