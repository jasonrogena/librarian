# Use the Rust version in Cargo.toml
FROM rust:1.86 AS builder
WORKDIR /librarian/
COPY ./ ./
RUN cargo build --release

# Unfortunately some of the CLI dependencies do not allow building a
# statically linked binary. Using Ubuntu as the runner instead of a
# lighter image like Alpine because of known incompatiblities between
# the minimal libc (musl) that ships with Alpine and the more typical
# libc the binary is built using.
FROM ubuntu:24.04 AS runner
COPY --from=builder /librarian/target/release/fs-librarian /usr/bin/fs-librarian
# Need python3 for some scripts called by the librarian
RUN DEBIAN_FRONTEND=noninteractive apt update && apt install -y python3
ENTRYPOINT ["/usr/bin/fs-librarian"]
