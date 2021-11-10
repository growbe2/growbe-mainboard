
FROM ubuntu

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -yq gcc-arm-linux-gnueabihf gcc
RUN apt-get install -yq curl rsync openssh-client

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add armv7-unknown-linux-gnueabihf