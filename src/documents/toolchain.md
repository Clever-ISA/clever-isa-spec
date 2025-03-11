# System Toolchain Definitions

## Preamble

This document defines recommendations for system toolchains (compilers, linkers, etc.) that target Clever-ISA, to allow agreement between high-level language toolchains. 
It does not define the two things:
* The psABI or the ELF Format, which is defined by [`D-abi`][D-abi], and
* The syntax for assemblers targetting Clever-ISA, which is defined by [`D-asm`].


## Target Feature Names

Compilers that distinguish between architectures by available features should use the extension name (with the `X-` prefix stripped). 

Specifically the following feature names are defined, and correspond to the instructions in the specified extensions:
* main[^main-note]: [`X-main`]
* float: [`X-float`]
* vector: [`X-vector`]
* float-ext: [`X-float-ext`]
* rand: [`X-rand`]
* atomic-xchg: [`X-atomic-xchg`]
* int128: [`X-int128`]
* float128: [`X-float128`]
* hash-accel: [`X-hash-accel`]
* crypto: [`X-crypto`]

Extensions that don't belong to a defined version should not be exposed to through stable language or toolchain features.

If a toolchain has a specified feature enabled by a control flag or a language feature, it may assume 

[^main-note]: [`X-main`] is the base instruction set and therefore every toolchain may assume the availability of the features. It is included in the list for completeness only and toolchains should not provide a mechanism to disable the availability of instructions from [`X-main`]

## Inline Assembly

Toolchains that provide access to inline assembly should use the following conventions for defining it:
* The syntax, or the syntax corresponding to `"default"` (if configurable) should be the assembly syntax specified by [`D-asm`],
    * No other syntax names are defined by this document. 
* Register `r8` should not be directly exposed as a valid explicit register name,
* It may be assumed that the following is preserved on exit, in addition to usable registers not marked as outputs or clobbers:
  * The `mode` register,
  * `r7`
  * If the inline assembly specification allows the toolchain to assume the condition code is preserved, the lower 8 bits of the `flags` register, and bits 8-15 of `fpcw` if the toolchain supports hardware floating-point operations `float`,


The following register groups should be supported with the specified types, corresponding to enabled target features:

```clever-spec,render
table toolchain:reggroups ["Group Name", "Short Constraint Code", "Supported Primitive Types", "Available Registers", "Required Target Feature"] {
    row [<!`reg`!>, <!`"r"`!>, table toolchain:reggroups:regtypes {
        row ["Integer", <!!
        * 8
        * 16
        * 32
        * 64
        !>]
    }, <!!
    * `r[0-15]` except `r7`
    !>, "main" ],
    row [<!`freg`!>, <!`"f"`!>, table toolchain:reggroups:fregtypes {
        row ["Float", <!!
        * 16
        * 32
        * 64
        !>]
    }, <!!
    * `f[0-7]`
    !>, "float"],
    row [<!`vec`!>, <!`"v"`!>, table toolchain:reggroups:vectypes {
        row ["Integer", <!!
        * 8
        * 16
        * 32
        * 64
        * 128
        !>],
        row ["Float", <!!
        * 16
        * 32
        * 64
        * 128
        !>],
        row ["Vector", <!!
        * 8
        * 16
        * 32
        * 64
        * 128
        !>]
    }, <!!
    * `v[0-15]`
    !>, "vector"]
}
```