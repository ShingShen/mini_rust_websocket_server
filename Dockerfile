FROM rust:1.72.1 as builder
WORKDIR /rust_websocket_server
ADD . /rust_websocket_server
RUN cargo build --release

FROM rust:1.72.1-slim
COPY --from=builder /rust_websocket_server/target/release/rust_websocket_server /
EXPOSE 7878
CMD [ "./rust_websocket_server" ]