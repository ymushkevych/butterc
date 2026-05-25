# butterc

butterc (Butter Compiler) is a compiler for Butter, a hobby programming language I've been developing.

It currently can do basic functions, variables, print statements, and basic math.

# dependencies

An x86_64 Linux machine is needed to run the generated executable.

[nasm](https://www.nasm.us/), [ld](https://ftp.gnu.org/pub/old-gnu/Manuals/ld-2.9.1/html_node/ld_toc.html), and the [rust compiler](https://rust-lang.org/) are needed to build and use the executable.

Sudo (administrator) permissions are also needed.

The dependencies can be installed by navigating into the `build` directory and running

```shell
bash depend.sh
```

This will only download the dependencies that you are missing

# building

To build the compiler executable, navigate into the `build` directory and run

```shell
bash build.sh
```

# running

To use the compiler, run

```shell
butterc <butter file>
```
