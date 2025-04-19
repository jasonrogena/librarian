# Use the Rust version in Cargo.toml
FROM rust:1.74 AS builder
WORKDIR /librarian/
COPY ./ ./
RUN cargo build --release

# Use a distroless image with the C runtime since the binary is dynamically linked
FROM gcr.io/distroless/cc:latest AS runner
COPY --from=builder /librarian/target/release/fs-librarian /usr/bin/fs-librarian
ENTRYPOINT ["/usr/bin/fs-librarian"]