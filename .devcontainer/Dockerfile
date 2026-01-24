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
  && mkdir -p /workspace \
  && mkdir -p /usr/local/cargo/registry /usr/local/cargo/git \
  && chown -R vscode:vscode /workspace /usr/local/cargo \
  && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://gist.githubusercontent.com/DIO0550/1830ced050035c34668aa426f7258346/raw/install-gh.sh | bash

USER vscode

WORKDIR /workspace

SHELL ["/bin/bash", "-l", "-c"]
RUN curl -fsSL https://gist.githubusercontent.com/DIO0550/dcc09dc9d2dcefceb9b0cda183e7ef8e/raw/pnpm-safe-chain-setup.sh | sh
RUN curl -fsSL https://gist.githubusercontent.com/DIO0550/148b1392cf435ba3562da0d398bee442/raw/install-ai-tools.sh | bash