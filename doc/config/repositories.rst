Repositories
============

When using bulk it's common to track multiple repositories using single
config. Here is an example:

.. code-block:: yaml

    repositories:

    - kind: debian
      suite: bionic
      component: your-app-testing
      keep-releases: 1000

    - kind: debian
      suite: bionic
      component: your-app
      keep-releases: 1
      match-version: ^\d+\.\d+\.\d+$

This keeps 1000 releases in testing repository. And just one release in
stable repository. Where stable release has strict semantic version and
all releases are included in testing repository (including stable).
Non-stable releases themeselves are probably versioned with ``git describe``
yielding versions like this: ``1.2.3-34-gde103b3``.

Options:

``kind``
    Kind of the repository. Only ``debian`` is currently supported.

``suite``
    Suite of the repository. For ubuntu it's usually a release codename such
    as ``xenial`` or ``bionic``.

``component``
    Component of the repository. Common convention is that it's a application
    name (so technically you can put multiple applications in the same
    repository). Also it may include modifier like ``-testing`` or ``-stable``.

``keep-releases``
    Number of releases of the package to keep in this repository. By default
    all releases are kept (i.e. it's never cleaned up). Usual debian tools
    keep exactly one package.

    It's also a good idea to keep two repositories: ``your-app`` with
    ``keep-releases: 1`` and ``your-app-stable`` with ``keep-releases: 100``
    which keep older packages. The index of the first repository is smaller
    and faster to download and the latter can be used to downgrade. Note:
    repositories share a pool of packages so ``.deb`` file itself isn't
    duplicated for two repositories.

``match-version``
    Only add version matching this regex to the repository.

    There are two good usecases for the feature:

    1. Sort out testing and stable versions (as in example above)

    2. Use a single ``bulk repo-add`` command to add packages for every
       distro. This works by append something like ``+bionic1`` suffix
       to a package version and add a respective ``match-version``
       for that distribution.

``skip-version``
    This is the same as ``match-version`` but is a negative filter. If
    both are matched ``skip-version`` takes precedence.

``add-empty-i386-repo``
    (default ``false``) When building ``amd64``-only repo also add an empty
    index for ``i386`` counterpart. This is needed to prevent errors on
    ``apt update`` on systems which are configured to fetch both 64bit and
    32bit versions of packages.

    For now it's known that ubuntu precise (12.04) default install only has
    this problem. So since precise reached its end of life this option is
    deprecated.
