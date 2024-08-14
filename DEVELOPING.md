## Build Container Image

```
$ docker build --build-arg GECKODRIVER_VERSION=0.32.0 -t local/geckodriver .
```

## Build geckodriver ARM64 binary

### Release build

```
$ docker run --rm -it -v $PWD/artifacts:/media/host -w /opt/geckodriver --name geckodriver local/geckodriver
```

### Debug build

```
$ docker run --rm -it -v $PWD/artifacts:/media/host -w /opt/geckodriver --name geckodriver local/geckodriver bash -c "sh build-arm.sh debug"
```

## Building with QEMU emulation

If you're not on an arm64 platform or wish to build for another platform, such as armv7, or arm64 if you're on x86_64, you can use QEMU emulation to build the driver:

First, unregister any platforms already registered:

```
$ docker run --rm -it --privileged aptman/qus -- -r
```

Next, re-register the emulated architectures:

```
$ docker run --rm -it --privileged aptman/qus -s -- -p
```

Then, build the container image with buildx:

```
$ docker buildx build --platform linux/arm/v7 --build-arg GECKODRIVER_VERSION=0.32.0 -t local/geckodriver .
```

Then build the geckodriver binary. Here's an example building geckodriver for armhf with QEMU:

```
docker run --rm -it --platform linux/arm/v7 -v $PWD/artifacts:/media/host -w /opt/geckodriver --name geckodriver local/geckodriver
```

## Building with cross-compilation

It's also possible to build a geckodriver binary on one host architecture that targets another architecture. This information is adapted from Mozilla's developer documentation on [Self Serving an ARM build](https://firefox-source-docs.mozilla.org/testing/geckodriver/ARM.html).

### armv7l/armhf

If you don’t have Rust installed:
```
# curl https://sh.rustup.rs -sSf | sh
```

Install cross-compiler toolchain:
```
# apt install gcc-arm-linux-gnueabihf libc6-armhf-cross libc6-dev-armhf-cross
```

Create a new shell, or to reuse the existing shell:
```
source $HOME/.cargo/env
```

Install rustc target toolchain:
```
% rustup target install armv7-unknown-linux-gnueabihf
```

Put this in testing/geckodriver/.cargo/config:
```
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

Build geckodriver from testing/geckodriver:
```
% cd testing/geckodriver
% cargo build --release --target armv7-unknown-linux-gnueabihf
```

### aarch64/arm64

Install cross-compiler toolchain:
```
$ apt install gcc-aarch64-linux-gnu libc6-arm64-cross libc6-dev-arm64-cross
```

Create a new shell, or to reuse the existing shell:
```
$ source $HOME/.cargo/env
```

Install rustc target toolchain:
```
$ rustup target install aarch64-unknown-linux-gnu
```

Put this in testing/geckodriver/.cargo/config:
```
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

Build geckodriver from testing/geckodriver:
```
$ cargo build --release --target aarch64-unknown-linux-gnu
```

## Additional information

The binary is copied to $PWD/artifacts.  If you're using podman-machine or running Docker in a VM, then you'll need to copy the binary from Podman or the VM via scp or by mounting a shared volume.
