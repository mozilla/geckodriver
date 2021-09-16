===========
geckodriver
===========

Proxy for using W3C WebDriver-compatible clients to interact with
Gecko-based browsers.

This program provides the HTTP API described by the `WebDriver protocol`_.
to communicate with Gecko browsers, such as Firefox.  It translates calls
into the :ref:`Firefox remote protocol <Protocol>` by acting as a proxy between the local-
and remote ends.

You can consult the `change log`_ for a record of all notable changes
to the program.  Releases_ are made available on GitHub.

.. _WebDriver protocol: https://w3c.github.io/webdriver/#protocol
.. _change log: https://github.com/mozilla/geckodriver/releases
.. _Releases: https://github.com/mozilla/geckodriver/releases


.. toctree::
   :maxdepth: 1

   Support.md
   WebDriver capabilities <https://developer.mozilla.org/en-US/docs/Web/WebDriver/Capabilities>
   Capabilities.md
   Usage.md
   Flags.md
   Profiles.md
   Bugs.md
   TraceLogs.md
   CrashReports.md
   Notarization.md


For developers
==============
.. toctree::
   :maxdepth: 1

   Building.md
   Testing.md
   Patches.md
   Releasing.md
   ARM.md


Communication
=============

The mailing list for geckodriver discussion is
https://groups.google.com/a/mozilla.org/g/dev-webdriver.

If you prefer real-time chat, ask your questions
on `#webdriver:mozilla.org <https://chat.mozilla.org/#/room/#webdriver:mozilla.org>`__.
