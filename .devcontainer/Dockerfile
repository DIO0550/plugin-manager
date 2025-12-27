# Build stage for github-mcp-server
FROM golang:1.24-bookworm AS github-builder
RUN git clone https://github.com/github/github-mcp-server.git /tmp/github-mcp-server \
    && cd /tmp/github-mcp-server \
    && go build -o github-mcp-server ./cmd/github-mcp-server


FROM rust:1-trixie

USER root
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
     pkg-config \
     libssl-dev \
     zip \
     unzip \
     nodejs \
     npm \
  && useradd -m -s /bin/bash vscode \
  && mkdir -p /workspaces \
  && mkdir -p /usr/local/cargo/registry /usr/local/cargo/git \
  && chown -R vscode:vscode /workspaces /usr/local/cargo \
  && rm -rf /var/lib/apt/lists/*

# Set shell for pnpm setup
ENV SHELL=/bin/bash
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

# Create pnpm directory with vscode ownership
RUN mkdir -p /pnpm && chown vscode:vscode /pnpm

# Configure npm security settings
RUN npm config set ignore-scripts true

# Install pnpm
RUN npm install -g pnpm

USER vscode

# Install safe-chain via binary
RUN curl -fsSL https://github.com/AikidoSec/safe-chain/releases/latest/download/install-safe-chain.sh | sh

# Install Claude Code CLI (binary)
RUN curl -fsSL https://claude.ai/install.sh | bash

# Verify safe-chain is working by attempting to install a test malware package
RUN /bin/bash -i -c 'source ~/.bashrc && pnpm install safe-chain-test 2>&1 | tee /tmp/safe-chain-output.txt; \
  grep -qi "malicious\|blocked\|exiting without installing" /tmp/safe-chain-output.txt && echo "✓ safe-chain is working" \
  || (echo "ERROR: safe-chain did not block the test package" && cat /tmp/safe-chain-output.txt && exit 1)'

# Configure pnpm
RUN /bin/bash -i -c 'source ~/.bashrc && pnpm config set ignore-scripts true'

RUN /bin/bash -i -c 'source ~/.bashrc && pnpm config set minimumReleaseAge 7200'

# Verify pnpm settings
RUN /bin/bash -i -c 'source ~/.bashrc && pnpm config get ignore-scripts | grep -q "true" && echo "✓ pnpm ignore-scripts: true"'

RUN /bin/bash -i -c 'source ~/.bashrc && pnpm config get minimumReleaseAge | grep -q "7200" && echo "✓ pnpm minimumReleaseAge: 7200"'

# Install codex (after verification passes)
RUN /bin/bash -i -c 'source ~/.bashrc && pnpm install -g @openai/codex'

COPY --from=github-builder /tmp/github-mcp-server /usr/local/bin/github-mcp-server