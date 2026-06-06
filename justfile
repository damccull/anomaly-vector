# THIS JUSTFILE REQUIRES NUSHELL TO BE INSTALLED
set shell := ["nu", "-c"]

LOG_LEVEL := 'INFO'

default:
    @just --list

# bacon with default tracing (tracing-subscriber)
bacon-no-bunyan level=LOG_LEVEL:
    RUST_LOG="INFO,anomaly-vector={{level}}" bacon --headless run-long -- --no-default-features

# bacon with bunyan tracing
bacon level=LOG_LEVEL:
    RUST_LOG="INFO,anomaly-vector={{level}}" bacon --headless run-long | bunyan

# bacon with bunyan tracing and output tee'd to /tmp/syncup-server.log
bacon-log level=LOG_LEVEL:
    RUST_LOG="INFO,axum::rejection=trace,anomaly-vector={{level}}" bacon --headless run-long out+err>| ^tee /tmp/anomaly-vector-server.log | bunyan

# build the program
build:
    cargo build

# Run the Claude CLI with a dedicated Git name using your personal email
claude *args="":
    #!/usr/bin/env bash
    # Claude becomes the author for its commits
    export GIT_AUTHOR_NAME="Claude AI"
    export GIT_AUTHOR_EMAIL="claude_ai+damccull@users.noreply.github.com"

    # I stay the committer for claude commits
    export GIT_COMMITTER_NAME="damccull"
    export GIT_COMMITTER_EMAIL="401104+damccull@users.noreply.github.com"

    nix run github:numtide/llm-agents.nix#claude-code -- {{args}}

# build the program for release
release:
    cargo build --release

# run the program
run-no-bunyan level=LOG_LEVEL:
    RUST_LOG="INFO,anomaly-vector={{level}}" cargo run --no-default-features

# run the program with bunyan tracing
run level=LOG_LEVEL:
    RUST_LOG="INFO,anomaly-vector={{level}}" cargo run | bunyan

# run cargo nextest
test:
    cargo watch -x "nextest run"

# run the surrealdb server for development
surrealdb:
    surreal start --username anomaly-vector --password anomaly-vector --allow-all --bind 127.0.0.1:8001 surrealkv://anomaly-vector.surrealdb

# Launch zellij with the layout for this app
zellij:
    zellij --layout ./zellij_layout.kdl
