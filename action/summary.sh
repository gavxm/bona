#!/usr/bin/env bash
set -euo pipefail

# Write model IDs to a temp file for yurai batch --from.
MODELS_FILE=$(mktemp)
printf '%s\n' "$YURAI_MODELS" | grep -v '^\s*$' > "$MODELS_FILE"

if [ ! -s "$MODELS_FILE" ]; then
  echo "::error::No model IDs provided"
  rm -f "$MODELS_FILE"
  exit 1
fi

# Build the yurai batch command. One invocation produces both JSON and SARIF.
YURAI_CMD=(yurai batch --from "$MODELS_FILE" --json)
if [ "${YURAI_UPLOAD_SARIF:-false}" = "true" ]; then
  YURAI_CMD+=(--sarif "${RUNNER_TEMP}/yurai.sarif")
fi

YURAI_ERR=$(mktemp)
JSON=$("${YURAI_CMD[@]}" 2>"$YURAI_ERR") || {
  echo "::warning::Batch investigation failed: $(cat "$YURAI_ERR")"
  cat "$YURAI_ERR" >&2
  rm -f "$MODELS_FILE" "$YURAI_ERR"
  exit 1
}
rm -f "$MODELS_FILE" "$YURAI_ERR"

# Build summary table from the JSON array.
HAS_HIGH=false
SUMMARY_TABLE="| Model | Findings | Highest Severity |\n|-------|----------|------------------|\n"
DETAIL_SECTIONS=""

NUM_MODELS=$(printf '%s' "$JSON" | jq 'length')

for i in $(seq 0 $((NUM_MODELS - 1))); do
  MODEL_ID=$(printf '%s' "$JSON" | jq -r ".[$i].model_id")
  FINDING_COUNT=$(printf '%s' "$JSON" | jq ".[$i].findings | length")
  HIGHEST=$(printf '%s' "$JSON" | jq -r "
    [.[$i].findings[].severity] |
    if any(. == \"high\") then \"high\"
    elif any(. == \"medium\") then \"medium\"
    elif any(. == \"low\") then \"low\"
    elif any(. == \"info\") then \"info\"
    else \"none\" end
  ")

  case "$HIGHEST" in
    high)
      SUMMARY_TABLE+="| $MODEL_ID | $FINDING_COUNT | :red_circle: HIGH |\n"
      HAS_HIGH=true
      ;;
    medium)
      SUMMARY_TABLE+="| $MODEL_ID | $FINDING_COUNT | :orange_circle: MEDIUM |\n"
      ;;
    low)
      SUMMARY_TABLE+="| $MODEL_ID | $FINDING_COUNT | :blue_circle: LOW |\n"
      ;;
    info)
      SUMMARY_TABLE+="| $MODEL_ID | $FINDING_COUNT | :white_circle: INFO |\n"
      ;;
    *)
      SUMMARY_TABLE+="| $MODEL_ID | 0 | :green_circle: clean |\n"
      ;;
  esac

  if [ "$FINDING_COUNT" -gt 0 ]; then
    DETAIL_SECTIONS+="\n### $MODEL_ID\n\n"
    DETAIL_SECTIONS+=$(printf '%s' "$JSON" | jq -r ".[$i].findings[] | \"- **\\(.severity | ascii_upcase)** \\(.title) — \\(.detail)\"")
    DETAIL_SECTIONS+="\n"
  fi
done

# Write job summary.
{
  printf '## yurai provenance check\n\n'
  printf '%b' "$SUMMARY_TABLE"
  if [ -n "$DETAIL_SECTIONS" ]; then
    printf '%b' "$DETAIL_SECTIONS"
  fi
} >> "$GITHUB_STEP_SUMMARY"

# Fail if requested and HIGH findings exist.
if [ "$YURAI_FAIL_ON_HIGH" = "true" ] && [ "$HAS_HIGH" = "true" ]; then
  echo "::error::HIGH severity findings detected"
  exit 1
fi
