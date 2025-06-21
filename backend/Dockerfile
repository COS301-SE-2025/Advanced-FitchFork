FROM alpine:3.20

# Install Python, pip, full Java (with javac), g++, and make
RUN apk add --no-cache \
    python3 \
    py3-pip \
    openjdk17 \
    g++ \
    libstdc++ \
    make

WORKDIR /code