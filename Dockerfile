# Builder layer
FROM alpine:3.20 AS builder

# Create upp user and setup Rust
ENV RUST_VERSION=1.82.0 \
    PATH=$PATH:/root/.cargo/bin \
    USER=rates \
    UID=10001 \
    CARGO_NET_GIT_FETCH_WITH_CLI=true \
    CARGO_BUILD_JOBS=4

RUN apk --no-cache add musl-dev openssl-dev openssl-libs-static openssl rustup clang lld curl
RUN rustup-init --profile minimal --default-toolchain $RUST_VERSION -y
RUN rustup update

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /rates

# Copy source code and build
COPY ./ .
RUN cargo build --release && \
    strip target/release/exchange-rate-service

####################################################################################################
## Final image
####################################################################################################
FROM debian:bookworm-20240722-slim

RUN apt-get update -y && \
    apt-get dist-upgrade -y && \
    apt-get install -y --no-install-recommends \
    libssl-dev openssl clang ca-certificates && \
    update-ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /rates

# Copy our build
COPY --from=builder /rates/target/release/exchange-rate-service ./
COPY --from=builder /rates/static ./static/

# enable logging with env_logger and display capturing stacktrace via backtrace
ENV RUST_LOG=info \
    RUST_BACKTRACE=1

# Use an unprivileged user.
USER rates:rates

EXPOSE 9012

CMD ["/rates/exchange-rate-service"]
