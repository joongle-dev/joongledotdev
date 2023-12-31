FROM messense/rust-musl-cross:x86_64-musl AS chef
RUN cargo install cargo-chef
WORKDIR /joongledotdev

FROM chef AS planner
COPY ./crates .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /joongledotdev/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY ./crates .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY ./assets /assets
COPY --from=builder /joongledotdev/target/x86_64-unknown-linux-musl/release/server /joongledotdev
ENTRYPOINT [ "/joongledotdev" ]
EXPOSE 8000 8001