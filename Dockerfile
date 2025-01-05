ARG PACKAGE=rust_multistage_starwars_api_wrapper

FROM cgr.dev/chainguard/rust:latest-dev as build
USER root
RUN apk update && apk add openssl-dev
WORKDIR /app
COPY . .
RUN cargo build --release

FROM cgr.dev/chainguard/glibc-dynamic:latest-dev
USER root
RUN apk update && apk add openssl-dev
COPY --from=build --chown=nonroot:nonroot /app/target/release/rust_multistage_starwars_api_wrapper /usr/local/bin/rust_multistage_starwars_api_wrapper
USER nonroot
CMD ["/usr/local/bin/rust_multistage_starwars_api_wrapper"]
