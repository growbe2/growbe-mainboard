
FROM ubuntu:20.04

COPY docker/source.list /etc/apt/sources.list

WORKDIR /usr/src/app

RUN dpkg --add-architecture armhf &&  dpkg --add-architecture arm64 && \
	apt update -yq && apt upgrade -yq && apt install -yq gcc-arm-linux-gnueabihf gcc libc6-armhf-cross libc6-dev-armhf-cross 
RUN DEBIAN_FRONTEND="noninteractive"  TZ="America/New_York" apt install -yq curl rsync openssh-client libdbus-1-dev pkg-config cmake
RUN apt install -yq protobuf-compiler
RUN apt install -yq libsqlite3-dev libsqlite3-dev:armhf
RUN apt install -yq libssl-dev libssl-dev:armhf
RUN apt install -yq libdbus-1-dev libdbus-1-dev:armhf


RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add armv7-unknown-linux-gnueabihf
RUN rustup target add arm-unknown-linux-gnueabihf

