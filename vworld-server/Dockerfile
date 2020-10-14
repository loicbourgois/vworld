# TODO: proper caching
# syntax=docker/dockerfile:experimental
#FROM rust:1.46
#WORKDIR /usr/src/vworld-server
#COPY . .
#RUN --mount=type=cache,target=/usr/local/cargo/registry \
#    --mount=type=cache,target=/home/root/app/target \
#    cargo build --release
#RUN cargo install --path .
#CMD ["vworld-server"]

FROM rust
WORKDIR /usr/src/vworld-server
COPY Cargo.toml .
RUN echo "fn main() {}" > dummy.rs
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN rm dummy.rs
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
COPY . .
RUN cargo build --release
CMD ["target/release/vworld-server"]
