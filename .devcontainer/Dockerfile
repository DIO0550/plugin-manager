# Build stage for github-mcp-server
FROM golang:1.24-bookworm AS github-builder
RUN git clone https://github.com/github/github-mcp-server.git /tmp/github-mcp-server \
    && cd /tmp/github-mcp-server \
    && go build -o github-mcp-server ./cmd/github-mcp-server


FROM rust:1-bookworm

USER root
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
     pkg-config \
     libssl-dev \
     zip \
     unzip \
  && useradd -m -s /bin/bash vscode \
  && mkdir -p /workspaces \
  && mkdir -p /usr/local/cargo/registry /usr/local/cargo/git \
  && chown -R vscode:vscode /workspaces /usr/local/cargo \
  && rm -rf /var/lib/apt/lists/*

COPY --from=github-builder /tmp/github-mcp-server /usr/local/bin/github-mcp-server
USER vscode
