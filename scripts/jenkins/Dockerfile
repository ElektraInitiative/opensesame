# docker build -t opensesame_base . --build-arg VARIANT=bullseye RUST_VERSION=1.67.0
ARG VARIANT=bullseye
ARG RUST_VERSION=1.67.0
FROM rust:${RUST_VERSION}-slim-${VARIANT}
ENV DEBIAN_FRONTEND=noninteractive

# Set locale
RUN apt-get update && apt-get upgrade -y  && \
    apt-get install -y --no-install-recommends sudo tzdata locales ssh && \
    rm -rf /var/lib/apt/lists/*
RUN sed -i -e 's/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen && locale-gen
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

# Install libelektra armhf
RUN dpkg --add-architecture armhf && apt-get update
RUN apt-get install -y software-properties-common apt-transport-https
RUN echo "deb [trusted=yes] https://thesis.maxirlinger.at/repo/ armhf/" > /etc/apt/sources.list.d/elektra-armhf.list && apt-get update
RUN apt-get install -y --allow-unauthenticated libelektra-dev:armhf

# elektra symlinks
RUN ln -s /usr/lib/arm-linux-gnueabi /usr/lib/pkgconfig/arm-linux-gnueabi
RUN ln -s /usr/lib/arm-linux-gnueabihf /usr/lib/pkgconfig/arm-linux-gnueabihf
RUN ln -s /usr/lib/arm-none-eabi /usr/lib/pkgconfig/arm-none-eabi

RUN set -eux; \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc \
    curl \
    wget \
    clang \
    pkg-config \
    build-essential \
    ca-certificates \
    libssl-dev:armhf \
    apt-transport-https \
    software-properties-common \
    gettext

# For cross compilation
RUN apt install -y --no-install-recommends \
    openssl \
    libc6-dev \
    libclang-dev \
    libc6-dev-i386 \
    libc6-armhf-cross \
    libc6-dev-armhf-cross \
    gcc-arm-linux-gnueabihf \
    binutils-arm-linux-gnueabihf

# Install deps
RUN cargo install cargo-deb
RUN rustup component add clippy rustfmt
RUN rustup target add armv7-unknown-linux-gnueabihf
RUN rustup toolchain install stable-armv7-unknown-linux-gnueabihf 
RUN chown 47110:47110 -R /usr/local/cargo

# export PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig"
# export PKG_CONFIG_ALLOW_CROSS="true"

# libdbus-1-dev:armhf
# libdbus-1-dev 

# g++-multilib \
#  \
#g++-arm-linux-gnueabihf \ 
