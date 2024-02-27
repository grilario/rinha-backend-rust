FROM rust:1.76 as builder

WORKDIR /

RUN cargo new app 

WORKDIR /app

COPY Cargo.toml Cargo.lock /app
RUN cargo build --release

RUN touch /app/src/lib.rs
COPY ./src /app/src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/rinha-backend /
EXPOSE 3000 3000

CMD ["./rinha-backend"]


