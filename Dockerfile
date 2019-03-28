# Note that this Dockerfile does a lot of ugly stuff to make
# build times faster. I'm hopefu the cargo team will release
# a feature to make this easier in the future

FROM rust:1.32

RUN apt-get update
RUN apt-get -y install arping dsniff net-tools tcpdump

# Do all this to cache dependencies because docker is too dumb
# to know what to do
WORKDIR /gthttp
RUN USER=root cargo new --bin shijackr
RUN USER=root cargo new --lib arpspoofr
RUN USER=root cargo new --bin gthttp
COPY ./Cargo.toml ./
COPY ./arpspoofr/Cargo.toml ./arpspoofr/
COPY ./shijackr/Cargo.toml ./shijackr/
COPY ./gthttp/Cargo.toml ./gthttp/

# Cache deps
RUN cargo build
RUN rm **/src/*.rs
# Force next build to recompile local library dependencies
RUN rm ./target/debug/deps/libarpspoofr*

# Copy sources
COPY ./arpspoofr/src/ ./arpspoofr/src/
COPY ./shijackr/src/ ./shijackr/src/
COPY ./gthttp/src/ ./gthttp/src/

# Full build
RUN cargo build

