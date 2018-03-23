Q & A
=====


Why version number and file existence is optional?
--------------------------------------------------

Sometimes we want to put and edit version number in generated files:
lock-files, code generated things and other.

Since entries in ``bulk.yaml`` are almost never modified it's much easier to
check once after editing a file than to learn rules of what is strict and what
isn't.

Here is just one example, when it is useful. Here is how we configure bulk in
rust projects:

.. code-block:: toml

    - file: Cargo.toml
      block-start: ^\[package\]
      block-end: ^\[.*\]
      regex: ^version\s*=\s*"(\S+)"

    - file: Cargo.lock
      block-start: ^name\s*=\s*"project-name"
      regex: ^version\s*=\s*"(\S+)"
      block-end: ^\[.*\]

The important part is that we *must* update ``Cargo.lock`` so that
``bulk set-version/incr-version/bump -g`` works fine *(we modify
``Cargo.lock`` together with ``Cargo.toml`` and commit in the same commit,
if we don't do that lockfile is update on next build and needs to be commited
after)*.

But we also want to be able to run bulk with absent lockfile (in case we don't
commit it into a repository) or if we want cargo to rebuild it from scratch.
