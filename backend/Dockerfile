FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y \
    make \
    default-jdk \
    g++ \
    python3 \
    python3-pip && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /code
