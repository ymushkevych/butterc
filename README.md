# butterc

butterc (Butter Compiler) is a compiler for Butter, a hobby programming language I've been developing.

It currently can do basic functions, variables, print statements, and basic math.

# dependencies

An x86_64 Linux machine is needed to run the generated executable.

nasm and ld are needed to run the compiler. ld should come pre-installed with all linux machines, but nasm can be installed with

```bash
sudo apt install nasm
```

or 

```bash
apt install nasm
```
To compile the compiler, the rust compiler (rustc) is needed. For information on how to install that, visit the [rust website](https://rust-lang.org/tools/install/)

# building

for those who either do not wish to install the rust compiler or would rather not build the compiler themselves, there is a pre-built executable in the build directory

Otherwise, run

```bash
cd ./build # move into the build directory
bash ./buildsudo.sh # this will build the executable and move it into your binaries
```

If you do not have sudo (admin) permissions, run

```
cd ./build # move into the build directory
bash ./build.sh
```
