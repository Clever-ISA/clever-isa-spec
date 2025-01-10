# Basic Instruction Set

## Registers

### Register Properties

| Property | Description | Notes |
|----------|-------------|-------|
| GPR      | Register is a General Purpose Register | Includes and only includes all registers with number < 16 |
| JUMP     | Cannot be written to except by a control transfer instruction | `mode` and `ip` only |
| CTRL     | Register is a control register | May restrict possible values |
| READONLY | Register cannot be written to | |
| SUPER    | Supervisor Register   | |
| CPUID    | CPU Identity Register | |

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
            !>]
        }, "Most Operands are read"
    ],
    row ["WRITE", "Operand is written by the Instruction", table T:1:2 {
        row [<!**UND**!>, <!!
        * If a register has the `READONLY` or `JUMP` property
        * If the instruction would store an invalid value to a register with the `CTRL` property
        * If the operand is an immediate
        !>],
        row [<!**PROT (0)**!>, <!!
        * If a register has property `SUPER` and `mode.XM=1`
        * If a register has property `CPUID` and the corresponding bit of `ciread` is not set while `mode.XM=1`
        * If paging is enabled, and a memory reference violates page permissions
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
        * If a register operand is not a general purpose register
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

### Explicitly Undefined Operations

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

#### Behaviour

```clever-psuedo
instruction {0o0000, 0o7777}(x: i4):
    raise UND
```

### Arithmetic/Logic Operations

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0001`   |
| **Operands** | 2          |
| **Op1 Props**| `READ`, `WRITE`, `INT`, `LOCK` |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `ARITH`     |
| **Mnemonic** | `add`     |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0002`   |
| **Operands** | 2          |
| **Op1 Props**| `READ`, `WRITE`, `INT`, `LOCK` |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `ARITH`     |
| **Mnemonic** | `sub`     |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0003`   |
| **Operands** | 2          |
| **Op1 Props**| `READ`, `WRITE`, `INT`, `LOCK` |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `LOGIC`     |
| **Mnemonic** | `and`     |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0004`   |
| **Operands** | 2          |
| **Op1 Props**| `READ`, `WRITE`, `INT`, `LOCK` |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `LOGIC`     |
| **Mnemonic** | `or`     |

| Property     | Definition |
|:------------:|------------|
| **Opcode**   | `0o0005`   |
| **Operands** | 2          |
| **Op1 Props**| `READ`, `WRITE`, `INT`, `LOCK` |
| **Control**  | `l00f`     |
| **Properties** | `LOCKABLE`, `LOGIC`     |
| **Mnemonic** | `xor`     |

#### Behaviour

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

!{#copyright}