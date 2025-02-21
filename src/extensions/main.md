# Basic Instruction Set

| Document Status |     |
|-----------------|-----|
| TYPE            | X   |
| STATUS          | V   |
| DATE            |2024-01-05|


## Registers

### Register Properties

| Property | Description | Notes |
|----------|-------------|-------|
| GPR      | Register is a General Purpose Register | Includes and only includes all registers with number < 16 |
| JUMP     | Cannot be written to except by a control transfer instruction | `mode` and `ip` only |
| FLAGS    | writes masked to defined bit patterns | `flags` register only |
| CTRL     | Register is a control register | May restrict possible values |
| READONLY | Register cannot be written to | |
| SUPER    | Supervisor Register   | |
| CPUID    | CPU Identity Register | |
| COMPLEX  | Register is complex to access | Generally cannot be combined with memory operands |
| INT      | Register is an integer value | GPR registers |

### Register Table

```clever-spec,render
table T:R1 ["Number", "Name", "Alias Names", "Properties"] {
    row [<!`0`!>, "r0","", <!!
    * GPR
    * INT
    !>],
    row [<!`1`!>, "r1","", <!!
    * GPR
    * INT
    !>],
    row [<!`2`!>, "r2","", <!!
    * GPR
    * INT
    !>],
    row [<!`3`!>, "r3","", <!!
    * GPR
    * INT
    !>],
    row [<!`4`!>, "r4","", <!!
    * GPR
    * INT
    !>],
    row [<!`5`!>, "r5","", <!!
    * GPR
    * INT
    !>],
    row [<!`6`!>, "r6","", <!!
    * GPR
    * INT
    !>],
    row [<!`7`!>, "r7",<!`sp`!>, <!!
    * GPR
    * INT
    !>],
    row [<!`8`!>, "r8",<!`fp`!>, <!!
    * GPR
    * INT
    !>],
    row [<!`9`!>, "r9","", <!!
    * GPR
    * INT
    !>],
    row [<!`10`!>, "r10","", <!!
    * GPR
    * INT
    !>],
    row [<!`11`!>, "r11","", <!!
    * GPR
    * INT
    !>],
    row [<!`12`!>, "r12","", <!!
    * GPR
    * INT
    !>],
    row [<!`13`!>, "r13","", <!!
    * GPR
    * INT
    !>],
    row [<!`14`!>, "r14","", <!!
    * GPR
    * INT
    !>],
    row [<!`15`!>, "r15","", <!!
    * GPR
    * INT
    !>],
    row [<!`16`!>, "ip", "", <!!
    * JUMP
    !>],
    row [<!`17`!>, "flags", "", <!!
    * FLAGS
    * COMPLEX
    !>],
    row [<!`18`!>, "mode", "", <!!
    * JUMP
    * COMPLEX
    !>]
}
```

## Instruction Set

### Instruction Operand Characteristics

```clever-spec,render
table T:1 ["Property", "Description", "Exception Conditions", "Notes"]{
    row ["READ", "Operand is read by the Instruction", 
        table T:1:1 {
            row [<!**PROT (0)**!>, <!!
            * If a register has property `SUPER` and `mode.XM=1`
            * If a register has property `CPUID` and the corresponding bit of `ciread` is not set while `mode.XM=1`
            * If paging is enabled, and a memory reference violates page permissions
            !>],
            row [<!**PF**!>, <!!
            * If paging is enabled, and a memory reference accesses a non-present page
            * If paging is enabled, and page resolution causes an error
            * If paging is enabled, and a memory reference has an out of bounds virtual address
            !>],
            row [<!**UND**!>, <!!
            * If more than one `READ` operand is not either an immediate or register without the `COMPLEX` property
            !>]
        }, "Most Operands are read"
    ],
    row ["WRITE", "Operand is written by the Instruction", table T:1:2 {
        row [<!**UND**!>, <!!
        * If a register has the `READONLY` or `JUMP` property
        * If the operand is an immediate
        * If more than one `WRITE` operand is not a register without the `COMPLEX` property
        * If the operand is not a register without the `COMPLEX` property and any `READ` operand is a memory reference or register with the complex property.
        !>],
        row [<!**PROT (0)**!>, <!!
        * If a register has property `SUPER` and `mode.XM=1`
        * If a register has property `CPUID` and the corresponding bit of `ciread` is not set while `mode.XM=1`
        * If paging is enabled, and a memory reference violates page permissions
        * If the instruction would store an invalid value to a register with the `CTRL` property
        !>],
        row [<!**PF**!>, <!!
        * If paging is enabled, and a memory reference accesses a non-present or non-writable page
        * If paging is enabled, and page resolution causes an error
        * If paging is enabled, and a memory reference has an out of bounds virtual address
        !>]
    }, "Immediates are not writable"],
    row ["MEM", "Operand is a memory reference", table T:1:3 {
        row [<!**UND**!>, <!!
        * If the operand is not a memory reference
        !>]
    }, ""],
    row ["LOCK", <!If the instruction has the `LOCKED` property enabled, perform a locked-rmw on the memory reference!>, "", <!Instructions with the `LOCKABLE` property use the `l` bit in the h field!>],
    row ["INTEGER", <!Only permits General Purpose Registers!>, table T:1:4 {
        row [<!**UND**!>, <!!
        * If a register operand does not have the INT property
        !>]
    }, ""]
}
```

### Instruction Properties

| Property | Description | Notes |
|----------|-------------|-------|
| LOCKABLE | `l` bit in h field determines `LOCKED` property | |
| LOCKED   | Operands with the `LOCK` property perform a locked-rmw | |
| ARITH    | If `f` bit in h is not set/not present, sets all `flags` bits according to arithmatic result | |
| LOGIC    | If `f` bit in h is not set/not present, sets `flags.Z`, `flags.P`, and `flags.N` according to logical result | |
| SUPER | **`PROT (0)`** if executed while `mode.XM=1` | |

### Explicitly Undefined Operations {#und}

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0000`   |
| **Operands** | 0          |
| **Control**  | `xxxx`     |
| **Properties** | NONE     |
| **Mnemonic** | `und0`     |
| **Aliases**  | `und`      |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o7777`   |
| **Operands** | 0          |
| **Control**  | `xxxx`     |
| **Properties** | NONE     |
| **Mnemonic** | `und1`     |
| **Aliases**  | `und`      |

#### Behaviour {#und-behaviour}

```clever-psuedo
instruction {0o0000, 0o7777}(x: i4):
    raise UND
```

### Arithmetic/Logic Operations {#alu}

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0001`   |
| **Operands** | 2          |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `ARITH`     |
| **Mnemonic** | `add`     |
| **Op Group** | [ALU 2 Op](#alu-2-op) |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0002`   |
| **Operands** | 2          |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `ARITH`     |
| **Mnemonic** | `sub`     |
| **Op Group** | [ALU 2 Op](#alu-2-op) |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0003`   |
| **Operands** | 2          |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `LOGIC`     |
| **Mnemonic** | `and`     |
| **Op Group** | [ALU 2 Op](#alu-2-op) |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0004`   |
| **Operands** | 2          |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `LOGIC`     |
| **Mnemonic** | `or`     |
| **Op Group** | [ALU 2 Op](#alu-2-op) |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0005`   |
| **Operands** | 2          |
| **Op1 Props**| `READ`, `WRITE`, `INT`, `LOCK` |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `LOGIC`     |
| **Mnemonic** | `xor`     |
| **Op Group** | [ALU 2 Op](#alu-2-op) |

#### Operand Properties {#alu-ops-operands}

##### ALU 2 Op

| Pos | Operand | Properties                     |
|:---:|:-------:|:------------------------------:|
|**1**| `dest`  | `READ`, `WRITE`, `INT`, `LOCK` |
|**2**| `src2`  | `READ`, `INT`                  |

#### Behaviour {#alu-ops-behaviour}

```clever-psuedo
enum AluOp : i3{
    Nop,
    Add,
    Sub,
    And,
    Or,
    Xor
}

instruction {0o0001, 0o0002}(dest: Operand, src2: Operand, l: bool, f: bool):
    if l:
       lock(dest);
    let v1 = read_zx(dest);
    let v2 = read_zx(src2);
    let op: AluOp;
    switch (instruction):
        case 0o0001:
            op = AluOp::Add;
        case 0o0002:
            op = AluOp::Sub;

    let (res, fl) = alu_compute(v1, v2, op);
    if f:
        set_flags(fl, 0x1F);
    write_truncate(dest, v1);
    if l:
        unlock(dest);
    finish

instruction {0o0003, 0o0004, 0o0005}(dest: Operand, src2: Operand, l: bool, f: bool):
    if l:
       lock(dest);
    let v1 = read_zx(dest);
    let v2 = read_zx(src2);
    let op: AluOp;
    switch (instruction):
        case 0o0003:
            op = AluOp::And;
        case 0o0004:
            op = AluOp::Or;
        case 0o0005:
            op = AluOp::Xor;

    let (res, fl) = alu_compute(v1, v2, op);
    if f:
        set_flags(fl, 0x1A);
    write_truncate(dest, v1);
    if l:
        unlock(dest);
    finish
```

### Simple Moves {#mov}

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0010`   |
| **Operands** | 2          |
| **Control**  | `000f`     |
| **Properties** | `LOGIC`  |
| **Mnemonic** | `mov`      |
| **Op Group** | [Mov 2 Op](#mov-2-op) |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0011`   |
| **Operands** | 2          |
| **Control**  | `0000`     |
| **Properties** |    |
| **Mnemonic** | `lea`      |
| **Op Group** | [Lea 2 Op](#lea-2-op) |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0012`   |
| **Operands** | 1          |
| **Control**  | `rrrr`     |
| **Properties** | `LOGIC`  |
| **Mnemonic** | `mov`      |
| **Op Group** | [Mov 1 Op Dest](#mov-1-op-dest) |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0013`   |
| **Operands** | 1          |
| **Control**  | `rrrr`     |
| **Properties** | `LOGIC`  |
| **Mnemonic** | `mov`      |
| **Op Group** | [Mov 1 Op Src](#mov-1-op-src) |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0014`   |
| **Operands** | 1          |
| **Control**  | `rrrr`     |
| **Properties** |          |
| **Mnemonic** | `lea`      |
| **Op Group** | [Lea 1 Op](#lea-1-op) |

#### Operand Properties {#mov-op-properties}

##### Mov 2 Op

| Pos | Operand | Properties      |
|:---:|:-------:|:---------------:|
|**1**| `dest`  | `WRITE`         |
|**2**| `src`   | `READ`          |

##### Lea 2 Op

| Pos | Operand | Properties      |
|:---:|:-------:|:---------------:|
|**1**| `dest`  | `WRITE`         |
|**2**| `src`   | `MEM`, `ADDR`   |

##### Mov 1 Op Dest

| Pos | Operand | Properties      |
|:---:|:-------:|:---------------:|
|**1**| `dest`  | `WRITE`         |

##### Mov 1 Op Src

| Pos | Operand | Properties      |
|:---:|:-------:|:---------------:|
|**1**| `src`   | `READ`          |


##### Lea 1 Op

| Pos | Operand | Properties      |
|:---:|:-------:|:---------------:|
|**1**| `src`  | `MEM`, `ADDR`    |


## Behaviour {#mov-behaviour}


```clever-psuedo
instruction 0o0010(dest: Operand, src: Operand, f: bool){
    let val = read_zero_ext(src);
    write_truncate(dest, val);
}
```

!{#copyright}
