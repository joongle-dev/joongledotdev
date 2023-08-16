FROM messense/rust-musl-cross:x86_64-musl AS build
WORKDIR /joongledotdev
#Copy the source code
COPY . .
#Build the application
RUN cargo build --release --target x86_64-unknown-linux-musl

#Create a new stage with a minimal image
FROM scratch
COPY ./assets /assets
COPY --from=build /joongledotdev/target/x86_64-unknown-linux-musl/release/server /joongledotdev
ENTRYPOINT [ "/joongledotdev" ]
EXPOSE 8000