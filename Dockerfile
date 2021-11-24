FROM rust:latest as build-env
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/sim_orchestrator /
CMD ["./sim_orchestrator"]

LABEL org.opencontainers.image.source=https://github.com/ramblinrocketclub/sim_orchestrator
EXPOSE 8080