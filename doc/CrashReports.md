# Analyzing crash data of Firefox

If Firefox crashes whilst under automation, it's helpful to retrieve the
generated crash data aka minidump files, and report these to us.

## Retrieve the crash data

Since geckodriver creates a temporary user profile for Firefox, it automatically
removes all associated folders once the tests complete. As a result, any minidump
files generated during a crash are also deleted.

To preserve these files, set the `MINIDUMP_SAVE_PATH` environment variable to an
existing folder and pass it into geckodriver:

```shell
MINIDUMP_SAVE_PATH="~/.geckodriver/minidumps" geckodriver
```

For each detected Firefox crash, two files will be stored in the specified folder:

- **`.dmp` file** – Contains the actual crash data.
- **`.extra` file** – Includes details about the running Firefox instance.

Both files are essential for further analysis and should be attached to a
[GitHub issue] for investigation.

[GitHub issue]: https://github.com/mozilla/geckodriver/issues/new

## Getting details of the crash

More advanced users can upload the generated minidump files themselves and
receive details information about the crash. Therefore find the [crash reporter]
folder and copy all the generated minidump files into the `pending` sub directory.
Make sure that both the `.dmp` and `.extra` files are present.

Once done you can also [view the crash reports].

If you submitted a crash please do not forget to also add the link of the
crash report to the geckodriver issue.

[crash reporter]: https://support.mozilla.org/kb/mozillacrashreporter#w_viewing-reports-outside-of-firefox
[view the crash reports]: https://support.mozilla.org/kb/mozillacrashreporter#w_viewing-crash-reports
