# Geckodriver

Geckodriver is a proxy for interacting with Gecko-based browsers (like Firefox) using W3C [WebDriver]-compatible clients. It facilitates communication by translating WebDriver calls into the [Marionette remote protocol], acting as an intermediary between the local and remote ends.

- [WebDriver protocol]: https://w3c.github.io/webdriver/#protocol
- [Marionette remote protocol]: https://firefox-source-docs.mozilla.org/testing/marionette/
- [WebDriver]: https://developer.mozilla.org/en-US/docs/Web/WebDriver

## Installation

You can install Geckodriver through various methods:

- Download pre-built binaries for common platforms from the [Releases] page on GitHub.
- Alternatively, you can compile it yourself using:
  * `cargo install geckodriver`
  * Or, check out the `release` branch or a specific tag.

For a full list of changes in each release, check the [change log].

- [Change log]: https://github.com/mozilla/geckodriver/blob/release/CHANGES.md
- [Releases]: https://github.com/mozilla/geckodriver/releases/latest

## Documentation

- [WebDriver](https://developer.mozilla.org/en-US/docs/Web/WebDriver) (Work in Progress)
  - [Commands](https://developer.mozilla.org/en-US/docs/Web/WebDriver/Commands)
  - [Errors](https://developer.mozilla.org/en-US/docs/Web/WebDriver/Errors)
  - [Types](https://developer.mozilla.org/en-US/docs/Web/WebDriver/Types)
  
- [Cross-browser testing](https://developer.mozilla.org/en-US/docs/Learn/Tools_and_testing/Cross_browser_testing)

- [Selenium](https://www.selenium.dev/documentation/)
  - [C# API](https://seleniumhq.github.io/selenium/docs/api/dotnet/)
  - [JavaScript API](https://seleniumhq.github.io/selenium/docs/api/javascript/)
  - [Java API](https://seleniumhq.github.io/selenium/docs/api/java/)
  - [Perl API](https://metacpan.org/pod/Selenium::Remote::Driver)
  - [Python API](https://seleniumhq.github.io/selenium/docs/api/py/)
  - [Ruby API](https://seleniumhq.github.io/selenium/docs/api/rb/)

- [Geckodriver Usage](https://firefox-source-docs.mozilla.org/testing/geckodriver/Usage.html)
  - [Supported Platforms](https://firefox-source-docs.mozilla.org/testing/geckodriver/Support.html)
  - [Firefox Capabilities](https://firefox-source-docs.mozilla.org/testing/geckodriver/Capabilities.html)
  - [Capabilities Example](https://firefox-source-docs.mozilla.org/testing/geckodriver/Capabilities.html#capabilities-example)
  - [Enabling Trace Logs](https://firefox-source-docs.mozilla.org/testing/geckodriver/TraceLogs.html)
  - [Analyzing Crash Data from Firefox](https://firefox-source-docs.mozilla.org/testing/geckodriver/CrashReports.html)

- [Contributing](https://firefox-source-docs.mozilla.org/testing/geckodriver/#for-developers)
  - [Building](https://firefox-source-docs.mozilla.org/testing/geckodriver/Building.html)
  - [Testing](https://firefox-source-docs.mozilla.org/testing/geckodriver/Testing.html)
  - [Releasing](https://firefox-source-docs.mozilla.org/testing/geckodriver/Releasing.html)
  - [Self-serving an ARM Build](https://firefox-source-docs.mozilla.org/testing/geckodriver/ARM.html)

## Source Code

Geckodriver is available under the [Mozilla Public License] (MPL 2.0). You can find the source code in [mozilla-central], specifically in the `testing/geckodriver` directory. This GitHub repository is used primarily for issue tracking and releases.

- [Mozilla Public License](https://www.mozilla.org/en-US/MPL/2.0/)
- [mozilla-central](https://hg.mozilla.org/mozilla-central/file/tip/testing/geckodriver)

## Custom Release Builds

If a pre-built binary isn’t available for your platform, you can create a custom build using the [Rust] toolchain. Checkout the desired release tag and run `cargo build`, or build and install the latest version via `cargo install geckodriver`.

- [Rust](https://rustup.rs/)

## Contact

For discussions on Geckodriver, join the mailing list:  
<https://groups.google.com/a/mozilla.org/g/dev-webdriver>

You can also participate in the [Matrix](https://wiki.mozilla.org/Matrix) chat on Mozilla's server in the [#webdriver](https://chat.mozilla.org/#/room/#webdriver:mozilla.org) channel.
