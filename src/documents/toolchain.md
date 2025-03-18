# System Toolchain Definitions

## Preamble

This document defines recommendations for system toolchains (compilers, linkers, etc.) that target Clever-ISA, to allow agreement between high-level language toolchains. 
It does not define the two things:
* The psABI or the ELF Format, which is defined by [`D-abi`][D-abi], and
* The syntax for assemblers targetting Clever-ISA, which is defined by [`D-asm`].

## Target Names

The Canonical architecture name takes the following format: `clever[<version>]`. `<version>` may be any published version of the specification. If a version suffix is specified, the toolchain may assume the availability of any extension included in that version, except those marked `limited` in [`R-status`] (as such extensions are not intended for wide-scale implementation). 

The default vendor for Clever-ISA should be `unknown` but toolchains may accept any vendor string. 

The special targets `clever-unknown-elf` and `clever[<version>]-unknown-elf` shall be targets that produce objects that comply with the psABI defined in [`D-abi`], but do not assume the presence of an operating system or any particular operating system. This is the canonical "Bare Metal" or "Freestanding" target name, and toolchains that support freestanding targets should support these names. 

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
* The assembly string may be assumed to not end by parsing a prefix
* The assembly block may be assumed to not end by executing a prefix opcode
* Register `r8` should not be directly exposed as a valid explicit register name
* It may be assumed that the following is preserved on exit, in addition to usable registers not marked as outputs or clobbers:
  * The `mode` register,
  * `r7`
  * If the inline assembly specification allows the toolchain to assume the condition code is preserved, the lower 8 bits of the `flags` register, and bits 8-15 of `fpcw`,
  * If `c`*`n`* is not marked as a clobber or output, the corresponding bits in `crszw`
* It may be assumed that the assembly block may be entered or exited without executing a Forced Control Transfer instruction. 

### ASM Register Groups

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
    !>, "vector"],
    row [<!`vechalf`!>, <!`"V"`!>, table toolchain:reggroups:vechalftypes {
        row ["Integer", <!!
        * 8
        * 16
        * 32
        * 64
        !>],
        row ["Float", <!!
        * 16
        * 32
        * 64
        !>],
        row ["Vector", <!!
        * 8
        * 16
        * 32
        * 64
        !>]
    }, <!!
    * `v[0-15]l`
    * `v[0-15]h`
    !>, "vector"],
    row [<!`veclo`!>, <!`"V"`!>, table toolchain:reggroups:vechalftypes {
        row ["Integer", <!!
        * 8
        * 16
        * 32
        * 64
        !>],
        row ["Float", <!!
        * 16
        * 32
        * 64
        !>],
        row ["Vector", <!!
        * 8
        * 16
        * 32
        * 64
        !>]
    }, <!!
    * `v[0-15]l`
    !>, "vector"],
    row [<!`vechi`!>, <!`"V"`!>, table toolchain:reggroups:vechalftypes {
        row ["Integer", <!!
        * 8
        * 16
        * 32
        * 64
        !>],
        row ["Float", <!!
        * 16
        * 32
        * 64
        !>],
        row ["Vector", <!!
        * 8
        * 16
        * 32
        * 64
        !>]
    }, <!!
    * `v[0-15]h`
    !>, "vector"]
}
```

### ASM Expansion Modifiers

The following expansion modifiers should be supported

```clever-spec,render
table toolchain:modifiers ["Modifier", "Supported Register Groups", "Effect", "Extension"]  {
    row [<!`l`!>, <!!
    * `vec`
    * `vechalf`
    * `vechi`
    !>, <!!
    Produces the lo vector half register corresponding to the assigned register
    !>, "vector"],
    row [<!`h`!>, <!!
    * `vec`
    * `vechalf`
    * `veclo`
    !>, <!!
    Produces the hi vector half register corresponding to the assigned register
    !>, "vector"],
    row [<!`v`!>, <!!
    * `veclo`
    * `vechalf`
    * `vechi`
    !>, <!!
    Produces the vector pair corresponding to the assigned register
    !>, "vector"]
}
```

Note that register sizes should be selected by a size specifier in the assembly string, rather than using a modifier. 

### Additional Registers

Toolchains should support explicit clobbers of any register in a group that corresponds to a target feature it does not support or report as enabled.
Additionally, it should support explicit clobbers of the following registers. They may be assumed to be preserved if not explicitly clobbered and the corresponding target feature is supported by the toolchain:

| Registers | Extension |
|:---------:|-----------|
| `c0-15`   | `crypto`  |

## Intrinsic Routines

### Special Types

The following types should be defined by toolchains the specified Extensions:

| Type            | Width | Class   | Extension |
|:---------------:|-------|---------|-----------|
| `__int128`      | 128   | integer | `int128`  |
| `_Float16`      | 16    | float   | `float`   |
| `__float128`    | 128   | float   | `float128`|
| `__vec16`       | 16    | vector  | `vec`     |
| `__vec32`       | 32    | vector  | `vec`     |
| `__vec64`       | 64    | vector  | `vec`     |
| `__vec128`      | 128   | vector  | `vec`     |


### Generic Routines

Certain routines are defined to be generic. This is denoted by `<>` surrounding an arguments list. Types are denoted with simple identifiers, usually a capital letter.
Constants are denoted by a type followed by an identifier, usually a capital letter.

Languages that do not support generic routines should support generic intrinsics as follows:
* Each const parameter should be prepended to the argument list. The toolchain should only accept values it recognizes as constants for these arguments.
* Type parameters used only in results should then follow the parameter list when generics.
* Type parameters used in arguments should be inferred from the input expressions.

In languages that do not support generic routines, generics intrinsics may be defined as macros or using constructs that cannot be directly called. Toolchain vendors should document the mechanism for calling generic intrinsics, if supported, in that case.

Some generic routines may have non-generic versions defined. In languages that do not support generic routines, the generic version may not be defined instead.

Generic Routines will define an example call in three languages:
* C,
* C++, and
* Rust.

All three calls will be identical, and should be expected to produce the same result regardless of language and (when toolchains do not optimize intrinsic calls) produce approximately the same machine instructions.

### `X-main` routines

#### Shift Routines

`I` may be any integer type with width at most 64-bit. 

##### Wrapping Left Shift

```c
uint8_t __shift_left8_w(uint8_t val, uint32_t sh);
uint16_t __shift_left16_w(uint16_t val, uint32_t sh);
uint32_t __shift_left32_w(uint32_t val, uint32_t sh);
uint64_t __shift_left64_w(uint64_t val, uint32_t sh);
I __shift_left_w<I>(I val, uint32_t sh);
```

Shifts `val` left by `sh`, wrapping `sh` at the bounds of the type. 

**Expected Code Sequence**:

```as
   lsh.w {val}, {sh}
```

##### Wrapping Unsigned Right Shift

```c
uint8_t __shift_uright8_w(uint8_t val, uint32_t sh);
uint16_t __shift_uright16_w(uint16_t val, uint32_t sh);
uint32_t __shift_uright32_w(uint32_t val, uint32_t sh);
uint64_t __shift_uright64_w(uint64_t val, uint32_t sh);
I __shift_uright_w<I>(I val, uint32_t sh);
```


Shifts `val` logically right by `sh`, wrapping `sh` at the bounds of the type. 

**Expected Code Sequence**:

```as
   ursh.w {val}, {sh}
```

##### Wrapping Signed Right Shift

```c
uint8_t __shift_sright8_w(uint8_t val, uint32_t sh);
uint16_t __shift_sright16_w(uint16_t val, uint32_t sh);
uint32_t __shift_sright32_w(uint32_t val, uint32_t sh);
uint64_t __shift_sright64_w(uint64_t val, uint32_t sh);
I __shift_sright_w<I>(I val, uint32_t sh);
```

Shifts `val` arithmetically right by `sh`, wrapping `sh` at the bounds of the type. 

**Expected Code Sequence**:

```as
   srsh.w {val}, {sh}
```

##### Unbounded Left Shift

```c
uint8_t __shift_left8_u(uint8_t val, uint32_t sh);
uint16_t __shift_left16_u(uint16_t val, uint32_t sh);
uint32_t __shift_left32_u(uint32_t val, uint32_t sh);
uint64_t __shift_left64_u(uint64_t val, uint32_t sh);
I __shift_left_u<I>(I val, uint32_t sh);
```

Shifts `val` left by `sh`, as though with infinite intermediate precision.

**Expected Code Sequence**:

```as
   lsh.u {val}, {sh}
```

##### Unbounded Unsigned Right Shift

```c
uint8_t __shift_uright8_u(uint8_t val, uint32_t sh);
uint16_t __shift_uright16_u(uint16_t val, uint32_t sh);
uint32_t __shift_uright32_u(uint32_t val, uint32_t sh);
uint64_t __shift_uright64_u(uint64_t val, uint32_t sh);
I __shift_uright_u<I>(I val, uint32_t sh);
```


Shifts `val` logically right by `sh`, as though with infinite intermediate precision.


**Expected Code Sequence**:

```as
   ursh.u {val}, {sh}
```

##### Unbounded Signed Right Shift

```c
uint8_t __shift_sright8_u(uint8_t val, uint32_t sh);
uint16_t __shift_sright16_u(uint16_t val, uint32_t sh);
uint32_t __shift_sright32_u(uint32_t val, uint32_t sh);
uint64_t __shift_sright64_u(uint64_t val, uint32_t sh);
I __shift_sright_u<I>(I val, uint32_t sh);
```


Shifts `val` arithmetically right by `sh`, as though with infinite intermediate precision.

**Expected Code Sequence**:

```as
   srsh.u {val}, {sh}
```

##### Rotate Left

```c
uint8_t __rotate_left8(uint8_t val, uint32_t sh);
uint16_t __rotate_left16(uint16_t val, uint32_t sh);
uint32_t __rotate_left32(uint32_t val, uint32_t sh);
uint64_t __rotate_left64(uint64_t val, uint32_t sh);
I __rotate_left<I>(I val, uint32_t sh);
```
Rotates `val` left by `sh` bits.

**Expected Code Sequence**:

```as
   lrot {val}, {sh}
```

##### Rotate Right

```c
uint8_t __rotate_right8(uint8_t val, uint32_t sh);
uint16_t __rotate_right16(uint16_t val, uint32_t sh);
uint32_t __rotate_right32(uint32_t val, uint32_t sh);
uint64_t __rotate_right64(uint64_t val, uint32_t sh);
I __rotate_right<I>(I val, uint32_t sh);
```
Rotates `val` left by `sh` bits.

**Expected Code Sequence**:

```as
   rrot {val}, {sh}
```