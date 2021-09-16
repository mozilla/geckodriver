Releasing geckodriver
=====================

Releasing geckodriver is not as easy as it once used to be when the
project’s canonical home was on GitHub.  Today geckodriver is hosted
in [mozilla-central], and whilst we do want to make future releases
from [Mozilla’s CI infrastructure], we are currently in between two
worlds: development happens in m-c, but releases continue to be made
from GitHub.

In any case, the steps to release geckodriver are as follows:

[mozilla-central]: https://hg.mozilla.org/mozilla-central/
[Mozilla’s CI infrastructure]: https://treeherder.mozilla.org/


Update in-tree dependency crates
--------------------------------

geckodriver depends on a number of Rust crates that also live in
central by using relative paths:

	[dependencies]
	…
    mozdevice = { path = "../mozbase/rust/mozdevice" }
	mozprofile = { path = "../mozbase/rust/mozprofile" }
	mozrunner = { path = "../mozbase/rust/mozrunner" }
	mozversion = { path = "../mozbase/rust/mozversion" }
	…
	webdriver = { path = "../webdriver" }

Because we need to export the geckodriver source code to the old
GitHub repository when we release, we first need to publish these
crates if they have had any changes in the interim since the last
release.  If they have received no changes, you can skip them:

  - `testing/mozbase/rust/mozdevice`
  - `testing/mozbase/rust/mozprofile`
  - `testing/mozbase/rust/mozrunner`
  - `testing/mozbase/rust/mozversion`
  - `testing/webdriver`

For each crate:

  1. Bump the version number in Cargo.toml
  2. Update the crate: `cargo update -p <crate name>`
  3. Commit the changes for the modified `Cargo.toml`, and `Cargo.lock`
     (can be found in the repositories root folder). Use a commit message
     like `Bug XYZ - [rust-<crate name>] Release version <version>.`


Update the change log
---------------------

Notable changes to geckodriver are mentioned in [CHANGES.md]. Many
users rely on this, so it’s important that you make it **relevant
to end-users**.  For example, we only mention changes that are visible
to users.  The change log is not a complete anthology of commits,
as these often will not convey the essence of a change to end-users.
If a feature was added but removed before release, there is no reason
to list it as a change.

It is good practice to also include relevant information from the
[webdriver], [rust-mozrunner], and [rust-mozdevice] crates, since these
are the most important dependencies of geckodriver and a lot of its
functionality is implemented there.

We follow the writing style of the existing change log, with
one section per version (with a release date), with subsections
‘Added’, ‘Changed’, and ‘Removed’.  If the targeted
Firefox or Selenium versions have changed, it is good to make a
mention of this.  Lines are optimally formatted at roughly 72 columns
to make the file readable in a text editor as well as rendered HTML.
fmt(1) does a splendid job at text formatting.

[CHANGES.md]: https://searchfox.org/mozilla-central/source/testing/geckodriver/CHANGES.md
[webdriver]: https://searchfox.org/mozilla-central/source/testing/webdriver
[rust-mozrunner]: https://searchfox.org/mozilla-central/source/testing/mozbase/rust/mozrunner
[rust-mozdevice]: https://searchfox.org/mozilla-central/source/testing/mozbase/rust/mozdevice

Update libraries
----------------

Make relevant changes to [Cargo.toml] to upgrade dependencies, then run

	% ./mach vendor rust
	% ./mach build testing/geckodriver

to pull down and vendor the upgraded libraries.

The updates to dependencies should always be made as a separate
commit to not confuse reviewers, because vendoring involves checking
in a lot of extra code already reviewed downstream.

[Cargo.toml]: https://searchfox.org/mozilla-central/source/testing/geckodriver/Cargo.toml
[Cargo.lock]: https://searchfox.org/mozilla-central/source/Cargo.lock


Bump the version number and update the support page
---------------------------------------------------

Bump the version number in [Cargo.toml] to the next version.
geckodriver follows [semantic versioning] so it’s a good idea to
familiarise yourself with that before deciding on the version number.

After you’ve changed the version number, run

	% ./mach build testing/geckodriver

again to update [Cargo.lock].

Now update the [support page] by adding a new row to the versions table,
including the required versions of Selenium, and Firefox.

Finally commit all those changes.

[semantic versioning]: http://semver.org/
[support page]: https://searchfox.org/mozilla-central/source/testing/geckodriver/doc/Support.md


Add the changeset id
--------------------

To easily allow a release build of geckodriver after cloning the
repository, the changeset id for the release has to be added to the
change log. Therefore add a final place-holder commit to the patch
series, to already get review for.

Once all previous revisions of the patch series have been landed, and got merged
to `mozilla-central`, the changeset id from the merge commit has to picked for
finalizing the change log. This specific id is needed because Taskcluster creates
the final signed builds based on that merge.

Release new in-tree dependency crates
-------------------------------------

Make sure to wait until the complete patch series from above has been
merged to mozilla-central. Then continue with the following steps.

Before releasing geckodriver all dependency crates as
[updated earlier](#update-in-tree-dependency-crates) have to be
released first.

Therefore change into each of the directories for crates with an update
and run the following command to publish the crate:

    % cargo publish

Note that if a crate has an in-tree dependency make sure to first
change the dependency information.


Export to GitHub
----------------

The canonical GitHub repository is

	https://github.com/mozilla/geckodriver.git

so make sure you have a local clone of that.  It has three branches:
_master_ which only contains the [README.md]; _old_ which was the
state of the project when it was exported to mozilla-central; and
_release_, from where releases are made.

Before we copy the code over to the GitHub repository we need to
check out the [release commit that bumped the version number](#add-the-changeset-id)
on mozilla-central:

    % hg update $RELEASE_REVISION

Or:

    % git checkout $RELEASE_REVISION

We will now export the contents of [testing/geckodriver] to the
_release_ branch:

	% cd $SRC/geckodriver
	% git checkout release
    % git pull
	% git rm -rf .
	% git clean -fxd
	% cp -rt $SRC/gecko/testing/geckodriver .

[README.md]: https://searchfox.org/mozilla-central/source/testing/geckodriver/README.md
[testing/geckodriver]: https://searchfox.org/mozilla-central/source/testing/geckodriver


Manually change in-tree path dependencies
------------------------------------------

After the source code has been imported we need to change the dependency
information for the `mozrunner`, `mozprofile`, `mozversion`, and
`webdriver` crates.  As explained previously geckodriver depends
on a relative path in the mozilla-central repository to build
with the latest unreleased source code.

This relative paths do not exist in the GitHub repository and the
build will fail unless we change it to the latest crate versions
from crates.io.  That version will either be the crate you published
earlier, or the latest version available if no changes have been
made to it since the last geckodriver release.


Commit local changes
--------------------

Now commit all the changes you have made locally to the _release_ branch.
It is recommended to setup a [GPG key] for signing the commit, so
that the release commit is marked as `verified`.

	% git add .
    % git commit -S -am "import of vX.Y.Z" (signed)

or if you cannot use signing use:

    % git add .
    % git commit -am "import of vX.Y.Z" (unsigned)

Then push the changes:

    % git push

As indicated above, the changes you make to this branch will not
be upstreamed back into mozilla-central.  It is merely used as a
place for external consumers to build their own version of geckodriver.

[GPG key]: https://help.github.com/articles/signing-commits/


Make the release
----------------

geckodriver needs to be manually released on github.com. Therefore start to
[draft a new release], and make the following changes:

1. Specify the "Tag version", and select "Release" as target.

2. Leave the release title empty

3. Paste the raw Markdown source from [CHANGES.md] into the description field.
   This will highlight for end-users what changes were made in that particular
   package when they visit the GitHub downloads section. Make sure to check that
   all references can be resolved, and if not make sure to add those too.

4. Find the signed geckodriver archives in the [taskcluster index] by
   replacing %changeset% with the full release changeset id. Rename the
   individual files so the basename looks like 'geckodriver-v%version%-%platform%'.
   Upload them all, including the checksum files for both the Linux platforms.

[draft a new release]: https://github.com/mozilla/geckodriver/releases/new
[taskcluster index]: https://firefox-ci-tc.services.mozilla.com/tasks/index/gecko.v2.mozilla-central.revision.%changeset%.geckodriver


Congratulations!  You’ve released geckodriver!

[releases page]: https://github.com/mozilla/geckodriver/releases
