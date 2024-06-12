# ACore
A toy RISC-V micro-kernel (or hybrid-kernel to be precise) that can run on QEMU. The main reference is [rCore-Tutorial-v3](https://github.com/rcore-os/rCore-Tutorial-v3).

## Features
- Simple synchronous remote-procedure-call mechanism to support userspace service, including process manager and other possible extensions.
- Virtual memory with SV39 page table.
- Preemptive round-robin process scheduling with priority.
- Buddy allocator for heap memory allocation.
- UNIX-like syscall primitives including ``fork``, ``exec``, ``waitpid``, ``read``, ``write``.
- A simple interactive shell at userspace.

## Run
To run the kernel, just run

``` bash
$ git clone git@github.com:Arcadia-Y/acore.git
$ cd acore/user
$ make build
$ cd ../os
$ make run
```