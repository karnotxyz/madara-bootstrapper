# ==============================================
# Karnot Bootstrapper
# ==============================================
FROM rustlang/rust:nightly-slim as builder
LABEL authors="karnot.xyz"

# Arguments :
ARG APP_CHAIN_ID
ARG ETH_CHAIN_ID
ARG ETH_PRIV_KEY
ARG ETH_RPC
ARG FEE_TOKEN_ADDRESS
ARG L1_DEPLOYER_ADDRESS
ARG L1_WAIT_TIME
ARG ROLLUP_PRIV_KEY
ARG ROLLUP_SEQ_URL
ARG SN_OS_CONFIG_HASH_VERSION
ARG SN_OS_PROGRAM_HASH
ARG CROSS_CHAIN_WAIT_TIME
ARG LOG_LEVEL
ARG L1_MULTISIG_ADDRESS
ARG L2_MULTISIG_ADDRESS
ARG VERIFIER_ADDRESS

# Assigning the env vars
ENV APP_CHAIN_ID=${APP_CHAIN_ID} \
    ETH_CHAIN_ID=${ETH_CHAIN_ID} \
    ETH_PRIV_KEY=${ETH_PRIV_KEY} \
    ETH_RPC=${ETH_RPC} \
    FEE_TOKEN_ADDRESS=${FEE_TOKEN_ADDRESS} \
    L1_DEPLOYER_ADDRESS=${L1_DEPLOYER_ADDRESS} \
    L1_WAIT_TIME=${L1_WAIT_TIME} \
    ROLLUP_PRIV_KEY=${ROLLUP_PRIV_KEY} \
    SN_OS_CONFIG_HASH_VERSION=${SN_OS_CONFIG_HASH_VERSION} \
    SN_OS_PROGRAM_HASH=${SN_OS_PROGRAM_HASH} \
    CROSS_CHAIN_WAIT_TIME=${CROSS_CHAIN_WAIT_TIME} \
    LOG_LEVEL=${LOG_LEVEL} \
    L1_MULTISIG_ADDRESS=${L1_MULTISIG_ADDRESS} \
    L2_MULTISIG_ADDRESS=${L2_MULTISIG_ADDRESS} \
    VERIFIER_ADDRESS=${VERIFIER_ADDRESS}

# adding musl (build failing fix)
ENV RUST_TARGET=x86_64-unknown-linux-musl
ENV PKG_CONFIG_ALLOW_CROSS=1
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new --bin karnot-bridge-deploy && chmod -R 777 /karnot-bridge-deploy

WORKDIR /karnot-bridge-deploy

# Copy your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Cache dependencies
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN rm src/*.rs

# Copy source tree
COPY ./src ./src

# Build for release
RUN rm -rf ./target/x86_64-unknown-linux-musl/release/deps/karnot_bridge_deploy*
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN mkdir data && touch data/addresses.json

ENV RUST_LOG=${LOG_LEVEL}

# Running the program
ENTRYPOINT ["cargo", "run"]