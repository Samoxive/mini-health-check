FROM rust:1.70.0-slim-bullseye AS builder

WORKDIR /app
COPY . .
RUN set -eux; \
	 	cargo build --release --locked; \
		objcopy --compress-debug-sections target/release/mini-health-check ./mini-health-check

FROM debian:11.7-slim

RUN rm -rf /var/lib/{apt,dpkg,cache,log}/;

WORKDIR /app

COPY --from=builder /app/mini-health-check ./mini-health-check
ENTRYPOINT ["./mini-health-check"]