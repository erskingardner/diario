# Compitutto - Homework Calendar

# List commands
default:
    @just --list

# Start web server
s:
    cargo run --release

# Start server on custom port
serve port="8080":
    cargo run --release -- serve --port {{port}}

# Build release binary
build:
    cargo build --release

# Generate static HTML (no server)
html:
    cargo run --release -- build

# Show data status
status:
    @echo "Export files in data/:"
    @ls -lh data/export_*.xls* 2>/dev/null || echo "  (none)"
    @echo ""
    @if [ -f homework.json ]; then \
        echo "homework.json: $$(grep -c '"date"' homework.json 2>/dev/null || echo 0) entries"; \
    fi

# Clean build artifacts
clean:
    cargo clean

# Run all CI checks
ci: fmt-check check lint test
    @echo "CI passed!"

# Dev commands
check:
    cargo check

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings

test:
    cargo test
