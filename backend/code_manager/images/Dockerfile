FROM alpine:3.20

RUN apk add --no-cache \
    python3 \
    py3-pip \
    openjdk17 \
    maven \
    gcc \
    g++ \
    libstdc++ \
    make \
    lcov \
    rust \
    cargo \
    go \
    bash \
    coreutils \
    unzip \
    wget \
    valgrind

# Install JaCoCo CLI
ENV JACOCO_VERSION=0.8.11
RUN wget https://repo1.maven.org/maven2/org/jacoco/jacoco/${JACOCO_VERSION}/jacoco-${JACOCO_VERSION}.zip \
    && unzip jacoco-${JACOCO_VERSION}.zip -d /opt/jacoco \
    && rm jacoco-${JACOCO_VERSION}.zip

ENV JACOCO_HOME=/opt/jacoco/jacoco-${JACOCO_VERSION}
ENV PATH="${JACOCO_HOME}/bin:${PATH}"

WORKDIR /code
