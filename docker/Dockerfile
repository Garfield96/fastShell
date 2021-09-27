FROM ubuntu:latest

RUN apt-get -y update && apt-get -y upgrade
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    curl build-essential libssl-dev ca-certificates postgresql wget
RUN wget https://github.com/sharkdp/hyperfine/releases/download/v1.11.0/hyperfine_1.11.0_amd64.deb && dpkg -i hyperfine_1.11.0_amd64.deb

USER postgres
RUN service postgresql start && \
    psql -c "ALTER USER postgres WITH PASSWORD 'postgres'" && \
    psql -c "CREATE DATABASE shell";
USER root
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENTRYPOINT /fastshell/docker/start.sh && /bin/bash
