# Run the full CI check suite locally.
check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo deny check
    cargo test

# Format code.
fmt:
    cargo fmt

# Run tests.
test:
    cargo test

# Run tests, re-running on file changes.
watch:
    cargo watch -x test

# Audit dependencies for license and vulnerability issues.
deny:
    cargo deny check

# Investigate a model (ex. `just inv meta-llama/Llama-3.1-8B-Instruct`).
inv model_id:
    cargo build --quiet && ./target/debug/yurai investigate {{model_id}} || true

# Investigate a model and output JSON.
inv-json model_id:
    cargo build --quiet && ./target/debug/yurai investigate {{model_id}} --json

# Review pending insta snapshots.
snapshots:
    cargo insta review

# Start the web UI dev server.
web-dev:
    cd web && npm run dev

# Build the web UI for production.
web-build:
    cd web && npm run build

# Pre-bake an investigation JSON for the web gallery.
web-prebake model_id:
    cargo build --quiet && ./target/debug/yurai investigate {{model_id}} --json > web/public/investigations/$(echo {{model_id}} | tr '/' '--').json
    just web-gallery

# Regenerate the gallery manifest from pre-baked JSONs.
web-gallery:
    cd web && node -e "const fs=require('fs'),dir='public/investigations',files=fs.readdirSync(dir).filter(f=>f.endsWith('.json')&&f!=='gallery.json'),models=files.map(f=>{const id=f.replace('.json','').replace('--','/'),d=JSON.parse(fs.readFileSync(dir+'/'+f,'utf-8'));return{id,file:f,tag:d.findings.length>0?'has findings':'clean',findingCount:d.findings.length}});fs.writeFileSync(dir+'/gallery.json',JSON.stringify(models,null,2));console.log(models.length+' models in gallery')"
