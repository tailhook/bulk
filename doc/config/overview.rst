Overview
========

Bulk's configuration file is usually ``bulk.yaml`` in the root if your project
but can be overrident by ``-c`` or ``--config`` for most subcommands.

This is yaml config parsed by quire_ so you can use most of its features
there.

.. _quire: http://quire.readthedocs.io/

Configuration file consists of a declaration of minimum supported version
of bulk and three sections, here is an example:

.. code-block:: yaml

    minimum-bulk: v0.4.5

    versions:
    - file: setup.py
      regex: ^\s*version\s*=\s*["']([^"']+)["']
    - file: your_module/__init__.py
      regex: ^__version__\s*=\s*["']([^"']+)["']

    metadata:
      name: your-app
      short-description: A great app in python
      long-description:
        A very great app in python

    repositories:
    - kind: debian
      suite: bionic
      component: your-app

All sections are optional if you don't want to use some of the functionality
here. In particular:

1. ``versions`` needed if you want to keep project version in source code in
   multiple places and want to update it using bulk
2. ``metadata`` is a package metadata, if you don't build ``.deb`` package
   you don't need it
3. ``repositories`` is a repository metadata, if you don't have debian/ubuntu
   repository you don't need it.

