# docker build -t opensesame .
# docker run -it --rm opensesame
ARG VARIANT=bullseye
FROM rust:1.67.0-slim-${VARIANT}
ENV DEBIAN_FRONTEND=noninteractive
ARG VARIANT

# Set locale
RUN apt-get update && apt-get upgrade -y  && \
    apt-get install -y --no-install-recommends sudo gnupg tzdata locales ssh && \
    rm -rf /var/lib/apt/lists/*
RUN sed -i -e 's/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen && locale-gen
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

# Install libelektra for amd64
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys F26BBE02F3C315A19BF1F791A9A25CC1CC83E839 && \
    echo "deb https://debs.libelektra.org/$VARIANT $VARIANT main" | sudo tee /etc/apt/sources.list.d/elektra.list && \
    apt-get update && apt-get install -y libelektra5-all

RUN set -eux; \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    gcc \
    wget \
    libssl-dev \
    clang \
    pkg-config \
    build-essential \
    ca-certificates \
    software-properties-common \
    apt-transport-https \
    gettext && \
    rm -rf /var/lib/apt/lists/*

# Install deps
RUN cargo install cargo-deb
RUN rustup component add clippy rustfmt
RUN chown 47110:47110 -R /usr/local/cargo
RUN mkdir -p /etc/kdb && mkdir -p /usr/share/elektra/specification
RUN chown 47110:47110 -R /etc/kdb /usr/share/elektra/specification


# ENV OPENSESAME_PATH /home/batman/opensesame
# WORKDIR $OPENSESAME_PATH
# COPY . $OPENSESAME_PATH
# ENV GETTEXT_SYSTEM=1
# RUN ./debian/postinst
