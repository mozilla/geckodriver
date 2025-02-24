# Building geckodriver

geckodriver is written in [Rust], a systems programming language
from Mozilla.  Crucially, it relies on the [webdriver crate] to
provide the HTTPD and do most of the heavy lifting of marshalling
the WebDriver protocol. geckodriver translates WebDriver [commands],
[responses], and [errors] to the [Marionette protocol], and acts
as a proxy between [WebDriver] and [Marionette].

To build geckodriver as part of a source Firefox build, add the
following to `mozconfig`:

```shell
ac_add_options --enable-geckodriver
```

With this addition geckodriver will be built when Firefox is built. It
can also be built alone by passing in the source path to the `mach
build` command:

```shell
% ./mach build testing/geckodriver
```

Artifact builds don't download geckodriver by default, but it can be
built using cargo:

```shell
% cd testing/geckodriver
% cargo build
…
Compiling geckodriver v0.21.0 (file:///code/gecko/testing/geckodriver)
…
Finished dev [optimized + debuginfo] target(s) in 7.83s
```

Because all Rust code in central shares the same cargo workspace,
the binary will be put in the `$(topsrcdir)/target` directory.

You can run your freshly built geckodriver this way:

```shell
% ./mach geckodriver -- --other --flags
```

See [Testing](Testing.md) for how to run tests.

[Rust]: https://www.rust-lang.org/
[webdriver crate]: https://crates.io/crates/webdriver
[commands]: https://docs.rs/webdriver/newest/webdriver/command/
[responses]: https://docs.rs/webdriver/newest/webdriver/response/
[errors]: https://docs.rs/webdriver/newest/webdriver/error/enum.ErrorStatus.html
[Marionette protocol]: /testing/marionette/Protocol.md
[WebDriver]: https://w3c.github.io/webdriver/
[Marionette]: /testing/marionette/index.rst
