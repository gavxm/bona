#!/usr/bin/env bash
set -euo pipefail

# Read model IDs from BONA_MODELS (newline-separated), skip empty lines.
mapfile -t MODELS < <(echo "$BONA_MODELS" | grep -v '^\s*$')

if [ ${#MODELS[@]} -eq 0 ]; then
  echo "::error::No model IDs provided"
  exit 1
fi

HAS_HIGH=false
SUMMARY_TABLE="| Model | Findings | Highest Severity |\n|-------|----------|------------------|\n"
DETAIL_SECTIONS=""

for model in "${MODELS[@]}"; do
  model=$(echo "$model" | xargs) # Trim whitespace.
  echo "::group::Investigating $model"

  # Run bona and capture JSON output.
  JSON=$(bona investigate "$model" --json 2>&1) || {
    echo "::warning::Failed to investigate $model"
    SUMMARY_TABLE+="| $model | error | — |\n"
    echo "::endgroup::"
    continue
  }

  echo "$JSON" | jq -r '.model_id' 2>/dev/null || {
    echo "::warning::Invalid JSON output for $model"
    SUMMARY_TABLE+="| $model | error | — |\n"
    echo "::endgroup::"
    continue
  }

  # Extract findings info.
  FINDING_COUNT=$(echo "$JSON" | jq '.findings | length')
  HIGHEST=$(echo "$JSON" | jq -r '
    if (.findings | length) == 0 then "none"
    else .findings[0].severity
    end
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
    DETAIL_SECTIONS+=$(echo "$JSON" | jq -r '.findings[] | "- **\(.severity | ascii_upcase)** \(.title) — \(.detail)"')
    DETAIL_SECTIONS+="\n"
  fi

  echo "::endgroup::"
done

# Write job summary.
{
  echo "## bona provenance check"
  echo ""
  echo -e "$SUMMARY_TABLE"
  if [ -n "$DETAIL_SECTIONS" ]; then
    echo -e "$DETAIL_SECTIONS"
  fi
} >> "$GITHUB_STEP_SUMMARY"

# Fail if requested and HIGH findings exist.
if [ "$BONA_FAIL_ON_HIGH" = "true" ] && [ "$HAS_HIGH" = "true" ]; then
  echo "::error::HIGH severity findings detected"
  exit 1
fi
