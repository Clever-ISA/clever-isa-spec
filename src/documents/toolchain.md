# System Toolchain Definitions

## Preamble

This document defines recommendations for system toolchain (compilers, linkers, etc.), to allow aggreement between high-level language toolchains. 
It does not define the two things:
* The psABI or the ELF Format, which is defined by [`D-abi`][D-abi], and
* The syntax for assemblers targetting Clever-ISA, which is defined by [`D-asm`].


## Target Feature Names

Compilers that distinguish between architectures by available features should use the extension name (with the `X-` prefix stripped). 

Specifically the following feature names are defined, and correspond to the instructions in the specified extensions:
* main: [`X-main`][]
* float: [`X-float`]
* vector: [`X-vector`]
* float-ext: [`X-float-ext`]
* rand: [`X-rand`]
* atomic-xchg: [`X-atomic-xchg`]
* int128: [`X-int128`]
* float128: [`X-float128`]
* hash-accel: [`X-hash-accel`]
