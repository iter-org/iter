# Build the executable
FROM --platform=$BUILDPLATFORM rust:bullseye as builder
WORKDIR /app
# RUN rustup toolchain install nightly-2022-03-22
RUN rustup default nightly

# Install dependencies
COPY Cargo.toml ./
COPY secrets ./secrets

WORKDIR /app/server

COPY ./server/Cargo.toml ./

RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

# RUN --mount=type=cache,target=/usr/local/cargo/registry \
RUN cargo build --release

# Build the executable using the actual source code
COPY server /app/server
RUN touch src/main.rs
# RUN --mount=type=cache,target=/usr/local/cargo/registry \
RUN cargo build --release

# == == ==
# Copy the executable and extra files ("static") to an empty Docker image
FROM --platform=$BUILDPLATFORM debian:bullseye

COPY --from=builder /app/target/release/ ~/api

CMD [ "~/api/server" ]