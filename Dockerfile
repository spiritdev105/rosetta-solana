FROM rustlang/rust:nightly as builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y libudev-dev
RUN cargo build --release --bin rosetta-solana

FROM rust as runtime
COPY --from=builder /app/target/release/rosetta-solana /app/rosetta-solana
EXPOSE 8080
CMD ["/app/rosetta-solana"]