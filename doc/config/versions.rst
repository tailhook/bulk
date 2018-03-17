Versions
========

Versions section help you with bookkeeping a version in your application,
here is the sample of versioning for python application

.. code-block:: yaml

    versions:
    - file: setup.py
      regex: ^\s*version\s*=\s*["']([^"']+)["']
    - file: your_module/__init__.py
      regex: ^__version__\s*=\s*["']([^"']+)["']

You might also add an example to your readme and keep version in documentation
updated too:

.. code-block:: yaml

    versions:

    - file: setup.py
      regex: ^\s*version\s*=\s*["']([^"']+)["']

    - file: your_module/__init__.py
      regex: ^__version__\s*=\s*["']([^"']+)["']

    - file: doc/conf.py
      regex: ^version\s*=\s*u?["']([^"']+)["']
      partial-version: ^\d+\.\d+  # no patch version

    - file: doc/conf.py
      regex: ^release\s*=\s*u?["']([^"']+)["']

    - file: README.rst
      regex: pip\s+install\s+your-module==(\S+)


Options:

``file``
    Filename to search version in, relative to project directory (usually
    a directory that contains ``bulk.yaml``)

``files``
    A list of files to search. This is useful if you can use same regex in
    multiple files.

    .. note:: Neither existence of ``file`` or any one in ``files``
       is enforced.  if you make a typo file will be silently skipped.
       Always use ``bulk check-version`` after modifying rules.

       On the upside is that you can use same ``bulk.yaml`` for many similar
       projects and versions that aren't present will be skipped.


``regex``
    A regular expression that matches version. It must contain a single
    capturing group (i.e. ``a (parenthised expression)``) for capturing
    actual version. Regex can match only on a single line.

    The expression shouldn't be too strict and should not try to validate
    the version number itself. I.e. if version is quoted anything inside the
    quotes should be considered version, if it isn't anything to next white
    space or newline is okay.

    Too strict version pattern risk to be either to replace
    ``1.2.3`` to ``1.2.4`` in ``1.2.3-beta.1`` keeping beta suffix, or to
    skip ``1.2.3-beta.1`` line in a file without updating it because it
    doesn't match.

    If ``regex`` matches multiple times all matching lines are treated as
    version number. Also multiple entries with the same file and different
    rules can be configured.

``partial-version``
    A regular expression that allows to select only portion of version
    number. Few examples:

    1. ``^\d+\.\d+`` -- selects major.minor version but not patch
    2. ``-.*$`` -- selects ``-alpha``, ``-beta.1``, ``-31-g12bd530`` or any
       other pre-release suffix in version number.

``block-start``, ``block-end``
    Marks block where to find version number in.

    For example, in ``Cargo.toml`` version number is in the ``[package]``
    section and named ``version``, whereas ``version=`` in other case may
    denote version of other things like pedependencies. So we use this:

    .. code-block:: yaml

        - file: Cargo.toml
          block-start: ^\[package\]
          block-end: ^\[.*\]
          regex: ^version\s*=\s*"(\S+)"

    You can specify single file multiple times in versions section. Which
    effectively means you can fix version in multiple different sections.

``multiple-blocks``
    (default ``false``) By default bulk stops scanning this file for this rule
    on the first ``block-end`` after ``block-start``.  If this setting is set
    to ``true`` searches for the next ``block-start`` istead. This option
    does nothing if no block defined.
