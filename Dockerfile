# builder layer
FROM alpine:3.20 AS builder

ENV RUST_VERSION=1.82.0
ENV PATH=$PATH:/root/.cargo/bin

RUN apk --no-cache add musl-dev openssl-dev openssl-libs-static openssl rustup clang lld
RUN rustup-init --profile default --default-toolchain $RUST_VERSION -y -t "$(uname -m)-unknown-linux-musl"
RUN rustup update

# Create appuser
ENV USER=rates
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /rates

COPY ./ .

# We no longer need to use the x86_64-unknown-linux-musl target
RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
#FROM debian:bookworm-slim - security issue
FROM debian:bookworm-20240722-slim

RUN apt-get update -y && \
    apt-get dist-upgrade -y && \
    apt-get install -y libssl-dev openssl clang ca-certificates && \
    update-ca-certificates

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /rates

# Copy our build
COPY --from=builder /rates/target/release/exchange-rate-service ./
COPY --from=builder /rates/static ./static/

# Use an unprivileged user.
USER rates:rates

# enable logging with env_logger
ENV RUST_LOG=info
# display capturing stacktrace via backtrace
ENV RUST_BACKTRACE=1

EXPOSE 9012

CMD ["/rates/exchange-rate-service"]
