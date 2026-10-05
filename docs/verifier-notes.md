# Verifier Notes — The Walls You Will Hit

> Rationale: [design.md](design.md). Decisions: [decisions.md](decisions.md).

## Bounded Loops

Unbounded loops are rejected. Use bounded ranges in Rust iterators or `#pragma unroll` equivalent.

```rust
for i in 0..64 {
    // Verified loop bounded by 64
}
```

## Pointer Arithmetic

Every dereference needs a bounds proof.

```rust
let data_end = ctx.data_end();
let data = ctx.data();

if data + IP_HDR_LEN > data_end {
    return Ok(0); // Bounds check fail
}
```

## Stack Depth

Keep local structs small. Total stack < ~400 bytes is safe. Allocate
larger buffers via BPF Maps.

## No Unbounded Allocation

All loops must be compile-time bounded. Dynamic allocation (`Box`, `Vec`) is not available in `no_std` kernel Rust.

## Instruction Budget

Updating two maps in one hook can exceed the instruction limit. Fixes:
1. Move work to userspace
2. Split into two BPF programs
3. Use tail calls

## Common Rejection Logs

| Error | Meaning | Fix |
|-------|---------|-----|
| `invalid mem access` | Missing bounds check | Add `data_end` check |
| `back-edge from insn` | Unbounded loop | Use bounded `for` loop |
| `program too large` | Instruction limit | Split program |
| `stack limit exceeded` | Too many locals | Move to map |
| `Permission denied (os error 13)` | LLVM bounds optimization | Force a >1 byte read (e.g., cast to `*const u16`) to prevent `start >= end` optimization |

## LLVM and 1-Byte Bounds Checks
If you try to read a single byte like `flags = *(ptr + 13)` and write a bounds check `if start + 14 > end`, LLVM might simplify this algebraically to `if start + 13 >= end`. The verifier doesn't understand `>=` on pointers and will reject the program. 

**Fix:** Cast the pointer to a larger type (like `u16`) for the bounds check so LLVM cannot use the `>=` optimization.

## Loopback Interface (lo) Quirks
- **No Ethernet Header:** `lo` often uses a 0-byte or 4-byte (WSL Null Encapsulation) header instead of a 14-byte Ethernet header. Do a dynamic signature scan (e.g. looking for `0x45` for IPv4).
- **TSO/GSO:** Localhost traffic uses jumbo frames. Always call `ctx.skb.pull_data(128)` to force the headers into linear memory, otherwise bounds checks will fail because the data is in paged memory!

## Cross-References
- [design.md](design.md)
- [data-model.md](data-model.md)
- [decisions.md](decisions.md)
- [runbook.md](runbook.md)
