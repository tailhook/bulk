Package Metadata
================

Package information is stored in ``metadata`` section in ``bulk.yaml``.
Here is an example:

.. code-block:: yaml

    metadata:
      name: your-app
      short-description: A great app in python
      long-description:
        A very great app in python. You can use it to do
        amazing things
      depends: [python3]


Options:

``name``
  Package name used to name a ``.deb`` file

``short-description``
  Short description of the package as shown in package search results and in
  other places. It should be a one-liner

``long-description``
  Long description of the package. Usually shown in GUI tools as a part of
  package detail.

``depends``
  List of package dependencies. It can consists of any expression allowed in
  debian packages. But note if you need different dependencies for different
  packages built (i.e. for different ubuntu distributions) you need to use
  different ``bulk.yaml`` configs and specify ones explicity to ``bulk pack``.
