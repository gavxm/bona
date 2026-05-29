#!/usr/bin/env bash
set -euo pipefail

# Read model IDs from BONA_MODELS (newline-separated), skip empty lines.
mapfile -t MODELS < <(printf '%s\n' "$BONA_MODELS" | grep -v '^\s*$')

if [ ${#MODELS[@]} -eq 0 ]; then
  echo "::error::No model IDs provided"
  exit 1
fi

HAS_HIGH=false
SUMMARY_TABLE="| Model | Findings | Highest Severity |\n|-------|----------|------------------|\n"
DETAIL_SECTIONS=""

for model in "${MODELS[@]}"; do
  model=$(printf '%s' "$model" | xargs) # Trim whitespace.
  echo "::group::Investigating $model"

  # Run bona and capture JSON output. Stderr goes to a temp file.
  BONA_ERR=$(mktemp)
  JSON=$(bona investigate "$model" --json 2>"$BONA_ERR") || {
    echo "::warning::Failed to investigate $model: $(cat "$BONA_ERR")"
    rm -f "$BONA_ERR"
    SUMMARY_TABLE+="| $model | error | — |\n"
    echo "::endgroup::"
    continue
  }
  rm -f "$BONA_ERR"

  printf '%s' "$JSON" | jq -r '.model_id' 2>/dev/null || {
    echo "::warning::Invalid JSON output for $model"
    SUMMARY_TABLE+="| $model | error | — |\n"
    echo "::endgroup::"
    continue
  }

  # Extract findings info. Compute max severity explicitly.
  FINDING_COUNT=$(printf '%s' "$JSON" | jq '.findings | length')
  HIGHEST=$(printf '%s' "$JSON" | jq -r '
    [.findings[].severity] |
    if any(. == "high") then "high"
    elif any(. == "medium") then "medium"
    elif any(. == "low") then "low"
    elif any(. == "info") then "info"
    else "none" end
  ')

  # Build summary table row.
  if [ "$HIGHEST" = "high" ]; then
    SUMMARY_TABLE+="| $model | $FINDING_COUNT | :red_circle: HIGH |\n"
    HAS_HIGH=true
  elif [ "$HIGHEST" = "medium" ]; then
    SUMMARY_TABLE+="| $model | $FINDING_COUNT | :orange_circle: MEDIUM |\n"
  elif [ "$HIGHEST" = "low" ]; then
    SUMMARY_TABLE+="| $model | $FINDING_COUNT | :blue_circle: LOW |\n"
  elif [ "$HIGHEST" = "info" ]; then
    SUMMARY_TABLE+="| $model | $FINDING_COUNT | :white_circle: INFO |\n"
  else
    SUMMARY_TABLE+="| $model | 0 | :green_circle: clean |\n"
  fi

  # Build detail section for models with findings.
  if [ "$FINDING_COUNT" -gt 0 ]; then
    DETAIL_SECTIONS+="\n### $model\n\n"
    DETAIL_SECTIONS+=$(printf '%s' "$JSON" | jq -r '.findings[] | "- **\(.severity | ascii_upcase)** \(.title) — \(.detail)"')
    DETAIL_SECTIONS+="\n"
  fi

  echo "::endgroup::"
done

# Write job summary.
{
  printf '## bona provenance check\n\n'
  printf '%b' "$SUMMARY_TABLE"
  if [ -n "$DETAIL_SECTIONS" ]; then
    printf '%b' "$DETAIL_SECTIONS"
  fi
} >> "$GITHUB_STEP_SUMMARY"

# Fail if requested and HIGH findings exist.
if [ "$BONA_FAIL_ON_HIGH" = "true" ] && [ "$HAS_HIGH" = "true" ]; then
  echo "::error::HIGH severity findings detected"
  exit 1
fi
