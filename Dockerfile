FROM docker.io/library/rust:1.84.0 as build-env
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build-env /app/target/release/davyjones /
CMD ["./davyjones"]
