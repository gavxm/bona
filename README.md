# Bona

> Know where your models came from.

Bona investigates the provenance of an AI model. Point it at a HuggingFace
model id and it traces the model's lineage, license inheritance, and trust
signals, then flags the places where what the model *claims* doesn't match
what the evidence actually shows.

**Status:** early development, v1 in progress.

## Why this exists

What's missing in existing tools: a way to actually
*investigate* a single model and answer the question you really have, which is
"where did this thing come from, and should I trust it?"

That's what Bona is. The gaps and contradictions between a model's claims and
its evidence aren't buried in a log somewhere, they're the findings.

## What it catches

- **Lineage inconsistencies**: a declared base model that doesn't match the
  actual architecture
- **License inheritance violations**: for example, an MIT license slapped on
  a Llama derivative that's actually governed by Meta's Community License
- **Documentation gaps, trust signals, and metadata anomalies**: missing
  training-data info, brand-new uploader accounts, parameter counts that don't
  add up

## Install

```sh
cargo install bona
```

## Usage

```sh
# Human-readable report
bona investigate meta-llama/Llama-3.1-8B-Instruct

# Full investigation document as JSON
bona investigate meta-llama/Llama-3.1-8B-Instruct --json
```

## License

AGPL-3.0. See [LICENSE](./LICENSE).
