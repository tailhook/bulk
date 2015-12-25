===
Tin
===

Tin is a super-simple packaging utility. It's similar to fpm_ but implemented
in rust.

All it can is package a directory of files into deb package. More features
will be done in future.

.. _fpm: https://github.com/jordansissel/fpm

:Status: Proof of Concept


Why?
====

Default packaging tools for debian are too complex. Also I wanted:

1. Simple to install zero-dependency tool (comparing to fpm_)
2. Experiment a little bit with reproducible packages (i.e. omit timestamps
   from package)
3. Add support for maintaining repositories in future
4. Add tiny wrapper around vagga to actually build the packages for all
   distributions by single command

It turned out that all functionality I needed from fpm_ could be reimplemnted
in a night, so we have a new tool, ready for the new experiments.


Limitations
===========

Tin should be simple. While we may lift few limitation in future versions we
don't aim to support all the features.

Limitations are:

1. No install scripts
2. All files owned by root and no timestamps
3. No devices, sockets, empty dirs and other possible habitants of
   tar/deb archive
4. Limited support of package metadata (focusing on common between different
   linux distributions)


How To Use
==========

Build program and install to some directory, say ``pkg``. Put some metadata
into ``package.yaml``. Then pack it into a debian package::

    tin pack --config package.yaml --dir pkg --dest-dir dist

And you will get a package in ``dist`` directory. You may find the example
``package.yaml`` in this repository.


Building Packages
=================

Just a few examples on how to prepare things to be packaged. With autotools
it looks like this::

    ./configure --prefix=/usr
    make
    rm -rf pkg
    make install DESTDIR=$(pwd)/pkg
    tin pack --config package.yaml --dir pkg --dest-dir dist

Or with new ``cargo install``::

    rm -rf pkg
    cargo install PACKAGE_NAME --root ./pkg/usr
    rm pkg/usr/.crates.toml
    tin pack --config package.yaml --dir pkg --dest-dir dist

This way you may package crate from crates.io.


