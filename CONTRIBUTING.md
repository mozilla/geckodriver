# Contributing to GeckoDriver

The GeckoDriver project welcomes contributions from everyone. There are a
number of ways you can help:

## Bug Reports

When opening new issues or commenting on existing issues please make
sure discussions are related to concrete technical issues with the
GeckoDriver or Marionette software.

It's imperative that issue reports outline the steps to reproduce
the defect. If the issue can't be reproduced it will be closed.
Please provide [concise reproducible test cases](http://sscce.org/)
and describe what results you are seeing and what results you expect.

## Code Contributions

The GeckoDriver project welcomes new contributors.

If you're looking for easy bugs, have a look at
[issues labelled E-easy](https://github.com/mozilla/geckodriver/issues?utf8=%E2%9C%93&q=is%3Aopen+is%3Aissue+label%3Aeasy+)
on Github.

This document will guide you through the contribution process.

### Step 1: Fork

Fork the project [on Github](https://github.com/mozilla/geckodriver)
and check out your copy locally.

```text
% git clone git@github.com:username/geckodriver.git
% cd geckodriver
% git remote add upstream git://github.com/mozilla/geckodriver.git
```

### Step 2: Branch

Create a feature branch and start hacking:

```text
% git checkout -b my-feature-branch
```

We practice HEAD-based development, which means all changes are applied
directly on top of master.

### Step 3: Commit

First make sure git knows your name and email address:

```text
% git config --global user.name 'Santa Claus'
% git config --global user.email 'santa@example.com'
```

**Writing good commit messages is important.** A commit message
should describe what changed, why, and reference issues fixed (if
any).  Follow these guidelines when writing one:

1. The first line should be around 50 characters or less and contain a
   short description of the change.
2. Keep the second line blank.
3. Wrap all other lines at 72 columns.
4. Include `Fixes #N`, where _N_ is the issue number the commit
   fixes, if any.

A good commit message can look like this:

```text
explain commit normatively in one line

Body of commit message is a few lines of text, explaining things
in more detail, possibly giving some background about the issue
being fixed, etc.

The body of the commit message can be several paragraphs, and
please do proper word-wrap and keep columns shorter than about
72 characters or so. That way `git log` will show things
nicely even when it is indented.

Fixes #141
```

The first line must be meaningful as it's what people see when they
run `git shortlog` or `git log --oneline`.

### Step 4: Rebase

Use `git rebase` (not `git merge`) to sync your work from time to time.

```text
% git fetch upstream
% git rebase upstream/master
```

### Step 5: Push

```text
% git push origin my-feature-branch
```

Go to https://github.com/yourusername/geckodriver.git and press the _Pull
Request_ and fill out the form.

Pull requests are usually reviewed within a few days. If there are
comments to address, apply your changes in new commits (preferably
[fixups](http://git-scm.com/docs/git-commit)) and push to the same
branch.

### Step 6: Integration

When code review is complete, a committer will take your PR and
integrate it on Selenium's master branch. Because we like to keep a
linear history on the master branch, we will normally squash and rebase
your branch history.

## Communication

GeckoDriver contributors frequent the `#ateam` channel on
[`irc.freenode.org`](https://webchat.freenode.net/).
