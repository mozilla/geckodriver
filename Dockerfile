FROM debian:latest
LABEL authors=Mozilla

#========= tags
# local/geckodriver
#=========

ARG GECKODRIVER_VERSION

USER root


#===========
# Install dependencies and clone geckodriver source
#===========
WORKDIR /opt
RUN echo "deb http://ftp.hk.debian.org/debian/ sid main" >> /etc/apt/sources.list \
  && apt-get update -qqy \
  && apt install gcc build-essential git cargo ca-certificates curl --no-install-recommends -y \
  && curl https://sh.rustup.rs -sSf | bash -s -- -y \
  && git clone https://github.com/mozilla/geckodriver.git && cd geckodriver \
  && git checkout v$GECKODRIVER_VERSION \
  && apt-get autoremove -y && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/* /var/cache/apt/* 

RUN apt-get update -qqy \
  && apt install -y gcc-arm-linux-gnueabihf libc6-armhf-cross libc6-dev-armhf-cross \
  && apt install -y gcc-aarch64-linux-gnu libc6-arm64-cross libc6-dev-arm64-cross \
  && /root/.cargo/bin/rustup target install armv7-unknown-linux-gnueabihf \
  && /root/.cargo/bin/rustup target install aarch64-unknown-linux-gnu \
  && cd geckodriver \
  && echo "[target.armv7-unknown-linux-gnueabihf]" >> .cargo/config \
  && echo "linker = \"arm-linux-gnueabihf-gcc\"" >> .cargo/config \
  && echo "[target.aarch64-unknown-linux-gnu]" >> .cargo/config \
  && echo "linker = \"aarch64-linux-gnu-gcc\""  >> .cargo/config \
  && apt-get autoremove -y && apt-get clean -y \
  && rm -rf /var/lib/apt/list/* /var/cache/apt/*

#===========
# Copy build script to container
#===========
COPY build-arm.sh /opt/geckodriver/

#===========
# Build geckodriver arm binary and copy to $PWD/artifacts
#===========
#RUN cd geckodriver && sh build-arm.sh
CMD sh build-arm.sh
