
FROM ubuntu


COPY docker/source.list /etc/apt/sources.list

WORKDIR /usr/src/app

RUN dpkg --add-architecture armhf && \
	apt-get update && apt-get upgrade && apt-get install -yq gcc-arm-linux-gnueabihf gcc
RUN DEBIAN_FRONTEND="noninteractive"  TZ="America/New_York" apt-get install -yq curl rsync openssh-client pkg-config cmake
RUN apt-get install -yq protobuf-compiler
RUN apt-get install -yq libsqlite3-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add armv7-unknown-linux-gnueabihf