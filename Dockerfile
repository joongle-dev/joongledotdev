FROM messense/rust-musl-cross:x86_64-musl AS builder
WORKDIR /joongle.dev
#Copy the source code
COPY . .
#Build the application
RUN cargo build --release --target x86_64-unknown-linux-musl

#Create a new stage with a minimal image
FROM scratch
COPY --from=builder /joongle.dev/target/x86_64-unknown-linux-musl/release/server /joongle
ENTRYPOINT [ "/joongle" ]
EXPOSE 8000