====
Bulk
====

Bulk is a super-simple packaging utility. It's similar to fpm_ but implemented
in rust.

It does three things for you:

* Makes directory of files into deb package
* Maintains a number of repos (stable, testing...) from list of packages
* Updates your version numbers

.. _fpm: https://github.com/jordansissel/fpm

:Status: Alpha


Why?
====

Default packaging tools for debian are too complex. Also I wanted:

1. Simple to install zero-dependency tool (comparing to fpm_)
2. Experiment a little bit with reproducible packages (i.e. omit timestamps
   from a package)
3. Simple utility to maintain (multiple) repositories
4. Add tiny wrapper around vagga to actually build the packages for all
   distributions by single command

It turned out that all functionality I needed from fpm_ could be reimplemented
in a night, so we have a new tool, ready for the new experiments.


Limitations
===========

Bulk should be simple. While we may lift few limitation in future versions we
don't aim to support all the features.

Limitations are:

1. No install scripts
2. All files owned by root and no timestamps
3. No devices, sockets, empty dirs and other possible habitants of
   tar/deb archive
4. Limited support of package metadata (focusing on common between different
   linux distributions)


Installation
============

Currently we provide static binary for x86_64:

    wget http://files.zerogw.com/bulk/bulk-0.4.11.tar.gz
    tar -xzf bulk-0.4.11.tar.gz -C /

Or you can install it with cargo:

    cargo install bulk

This will install bulk in ``/usr/bin``. Ubuntu packages will be available
shortly.


How To Use
==========

Build program and install to some directory, say ``pkg``. Put some metadata
into ``bulk.yaml``. Then pack it into a debian package::

    bulk pack --config bulk.yaml --dir pkg --dest-dir dist

And you will get a package in ``dist`` directory. You may find the example
``bulk.yaml`` in this repository.


Building Packages
=================

Just a few examples on how to prepare things to be packaged. With autotools
it looks like this::

    ./configure --prefix=/usr
    make
    rm -rf pkg
    make install DESTDIR=$(pwd)/pkg
    bulk pack --config bulk.yaml --dir pkg --dest-dir dist

Or with new ``cargo install``::

    rm -rf pkg
    cargo install PACKAGE_NAME --root ./pkg/usr
    rm pkg/usr/.crates.toml
    bulk pack --config bulk.yaml --dir pkg --dest-dir dist

This way you may package crate from crates.io.


=======
License
=======

Licensed under either of

* Apache License, Version 2.0, (./LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (./LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

------------
Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
