# Issue DB API Client 

This is an experimental work in progress auxiliary package for an ongoing research project. 

More details will be added after the work is done, and all code will be published.

# Installation

The API client requires [The Rust Programming Language](https://www.rust-lang.org/)
to be installed. 
The library has been tested with Rust versions >= 1.60, <= 1.68, although 
we exepct the library to also compile with newer versions of Rust.

The library can then be installed by running `python -m pip install issue-db-api`.

The library can also be installed from source by cloning the repository
and running the following commands:

```shell 
python -m pip install setuptools_rust 
python setup.py build_ext --inplace 
```