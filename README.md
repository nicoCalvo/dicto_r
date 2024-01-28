# Rust implementation of python's dictor lib

This lib is a re-implementation of python's [dictor](https://github.com/perfecto25/dictor)
ported to Rust using [Pyo3](https://github.com/perfecto25/dictor) and [maturin](https://github.com/PyO3/maturin)



## Pypy release

Each version is build and released into [Pypy](https://pypy.org) so it can be treated as any other regular python lib

## Develop 

`src/lib` contains it's own set of rust test. However, to have a full testing cycle including python, __maturin__ helps by building a wheel and installing it on our current python environment:

```
$maturin develop
...
Finished dev [unoptimized + debuginfo] target(s) in 0.02s
📦 Built wheel for CPython 3.10 to /tmp/.tmpB45abW/dicto_r-0.1.0-cp310-cp310-linux_x86_64.whl
🛠 Installed dicto_r-0.1.0
```
As a common practice for using maturin and pyo3, a directory with the same name as the lib is created containing the python helpers to work with the lib from python's side

In this case, there's only a test.py file exported from original __dictor__ lib within the resources files to run exactly the same test battery in rust implementation as in python.


