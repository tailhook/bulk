Version Bookkeeping
===================

Bulk can be used to sync version of your application to various places in code.


Basics
------

Bulk uses regular expressions to find versions in some file. For example,
here is how we track versions in typical python project:

.. code-block:: yaml

    versions:

    # There is usually a version in setup.py
    - file: setup.py
      # this isn't 100% correct as version can end in different quote or
      # there might be few version parameters in a file, but this is good
      # enough for many projects, other projects might need to tweak matcher
      regex: ^\s*version\s*=\s*["']([^"']+)["']


    # Also it's a good idea to put library version into a __version__ attribute
    # in the version itself
    - file: your_module/__init__.py
      regex: ^__version__\s*=\s*["']([^"']+)["']

Put it in ``bulk.yaml`` and now you can find out version with:

.. code-block:: console

    > bulk get-version
    1.3.5

Yes, the first time you've written ``setup.py`` and ``__init__.py`` you needed
to put version yourself. This is usually handled by project boilerplate.


Releasing a Project
-------------------

If you obey **semantic versioning** in the project version run one of:

.. code-block:: console

    > bulk bump --breaking -g
    > bulk bump --feature -g
    > bulk bump --bugfix -g

The commands above will increment a major, minor or patch version of your
version number, commit the changes with a comment of
``Version bumped to v1.3.6`` and create an annotated tag ``v0.3.6`` by
starting an editor and showing you changes since previous tag. You can opt-out
commit and tag creation by omitting ``-g`` which is equivalent of longer
``--git-commit-and-tag``.

You can also use ``-1``, ``-2`` and ``-3`` which increment the specific
component of version. Technically they are are equivalent to above except
when version is zero-based ``0.x``.

.. note:: in case of ``0.x`` versions the version numbers are shifted.
   I.e. if you have two zeros numbers ``0.0.x`` any bump with increment a
   single version. If you have ``0.x.y`` number second component will increment
   with both ``--breaking`` and ``--feature``. This is how many existing tools
   handle semver. Use ``-1``, ``-2`` if in doubt or to switch from ``0.x``
   versions to ``1.x``.

For **date-based versioning** use:

.. code-block:: console

    > bulk bump -dg

This will force your version to something like ``v180317.0``. If you will
subsequently run this command on the same day you will get ``v180317.1`` and
so forth.

.. note:: The date here is UTC to avoid issues with different people releasing
   in different timezones.

Another way to update is to use ``set-version``:

.. code-block:: console

    > bulk set-version v1.3.5-beta.1
    ./your_module/__init__.py:1: (v1.3.5 -> v1.3.5-beta.1) __version__ = '1.3.5-beta.1'
    ./setup.py:6: (v1.3.5 -> v1.3.5-beta.1)       version='1.3.5-beta.1',

This is useful to set some pre-release version as you see in example because we
don't have a command-line flag for that or in case you have different version
format or just want to skip version number for some reason.


Building a Pre-Release Project
------------------------------

Everyting above assumes that version is stored in source code and commited to
git. Which is true for many tools. But you don't want to commit version for
a prerelease version of application. We have a nice command for this use
case too:

.. code-block:: console

    > bulk with-version v1.3.6-pre4 your-build-command
    1.3.5 -> 1.3.6-pre4
    [ .. output of your-build-command .. ]
    1.3.6-pre4 -> 1.3.5

This runs build with correct version and ensures that when build is complete
you will get no version change in git status.

Since the common case is using ``git describe`` for actual version we have a
shortcut for that:

.. code-block:: console

    > bulk with-git-version your-build-command
    1.3.5 -> 1.3.5-4-gd923e59-dirty
    [ .. output of your-build-command .. ]
    1.3.5-4-gd923e59-dirty -> 1.3.5

(the ``-dirty`` here means you have modified git-tracked files locally)

.. note:: The ``git describe`` command is not strictly semver-compatible.
   I.e. the version ``x.y.z-n`` is treated as lower than ``x.y.z`` and you're
   supposed to use ``x.y.z+n`` for that. But for now we decided to stick to
   what ``git describe`` provides for now. We may provide an option to fix
   that in future, in the meantime you can use ``with-version``.


Other Commands
--------------

To check if version number is fine (consistent) run:

.. code-block:: console

    > vagga bulk check-version
    setup.py:6: (v1.3.5)       version='1.3.5',
    trafaret_config/__init__.py:1: (v1.3.5) __version__ = '1.3.5'

It shows you files and lines where version number is present and will fail
if there is no version at all or version is inconsistent between multiple
files.

.. note:: it will **not** show you files and lines which are present in config
   file but has no version number found. So when adding an entry in
   ``bulk.yaml`` you should run ``check-version`` and make sure the actual
   entry exists in the file.

To fix inconsistent version run:

.. code-block:: console

    > vagga bulk set-version v1.3.5 --force
    setup.py:6: (v1.3.4 -> v1.3.5)       version='1.3.5',
    trafaret_config/__init__.py:1: (v1.2.3 -> v1.3.5) __version__ = '1.3.5'

Same restriction for not found version as for ``check-version`` applies here.
