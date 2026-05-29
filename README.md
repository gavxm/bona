# Bona

> Know where your models came from.

Bona investigates the provenance of HuggingFace models. It traces lineage,
cross-references licenses, and flags the gaps between what a model *claims*
and what the evidence actually shows.

Most tools show you metadata. Bona shows you where the metadata contradicts
itself.

![bona web UI](docs/UI.png)

**[Live demo](https://gavxm.github.io/bona)** · **[Install](#install)**

## Findings

- **License inheritance violations**: Apache-2.0 declared on a Llama
  derivative that's actually governed by Meta's Community License
- **Lineage inconsistencies**: declared base model doesn't match the
  architecture in config.json
- **Gated-derivative detection**: public models derived from gated parents,
  bypassing access controls
- **Documentation gaps**: missing license or base model declarations
- **Trust signals**: new uploader accounts, zero community engagement
- **Metadata anomalies**: weight sizes that don't match the declared
  architecture

Each finding includes a severity, a reason explaining *why* it matters, and
the raw declared-vs-actual values that triggered it.

## Install

```sh
cargo install bona
```

## Usage

```sh
# investigate a model
bona investigate meta-llama/Llama-3.1-8B-Instruct

# JSON output
bona investigate ruslanmv/Medical-Llama3-8B --json

# fail CI on high-severity findings
bona investigate some/model --fail-on-high
```

Set `HF_TOKEN` to access gated models:

```sh
export HF_TOKEN=hf_...

bona investigate meta-llama/Llama-3.1-8B-Instruct
```

## Web Explorer

Three-panel investigation UI: lineage graph, tabbed evidence details, and
findings with declared-vs-actual diffs. Click a finding to highlight the
related evidence across all panels.

**[gavxm.github.io/bona](https://gavxm.github.io/bona)**

Run locally:

```sh
cd web && npm install && npm run dev
```

## GitHub Action

Add provenance checks to your CI pipeline:

```yaml
- uses: gavxm/bona@main
  with:
    models: |
      meta-llama/Llama-3.1-8B-Instruct
      ruslanmv/Medical-Llama3-8B
    fail-on-high: true
    hf-token: ${{ secrets.HF_TOKEN }}
```

The Action investigates each model and posts a summary to the job output.
Set `fail-on-high: true` to block merges when HIGH severity findings exist.

## How It Works

Bona fetches evidence from four HuggingFace sources concurrently, then runs
cross-referenced checks across them:

| Source                    | What it provides                          |
| ------------------------- | ----------------------------------------- |
| HF metadata               | license, base model, tags, downloads      |
| Model tree                | parent model's license, sibling models    |
| config.json + safetensors | architecture, parameters, weight size     |
| Community signals         | uploader account age, discussion activity |

The key insight is **gap-as-signal**: contradictions between sources are the
findings, not incidental noise.

## Architecture

```text
src/lib.rs       :engine library
src/main.rs      :CLI
src/sources/     :evidence fetchers
src/findings/    :cross-referenced checks
web/             :React + Vite + Tailwind
```

## License

AGPL-3.0. See [LICENSE](./LICENSE).
