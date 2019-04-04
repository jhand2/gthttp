# Note that this Dockerfile does a lot of ugly stuff to make
# build times faster. I'm hopefu the cargo team will release
# a feature to make this easier in the future

FROM rust:1.32

RUN apt-get update
RUN apt-get -y install arping dsniff net-tools tcpdump

# Do this to cache dependencies because docker is too dumb
# to know what to do
WORKDIR /gthttp
RUN USER=root cargo init
COPY ./Cargo.toml ./

# Cache deps
RUN cargo build
RUN rm src/*.rs
RUN rm ./target/debug/gthttp
RUN rm -r ./target/debug/gthttp.d
RUN rm -r ./target/debug/incremental

# Copy sources
COPY ./src/* ./src/

# Full build
RUN cargo build

