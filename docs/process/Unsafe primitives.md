---
tags: [process]
---

# Unsafe primitives

`unsafe` is allowed only in `draw-sys` and `draw-gfx/src/gpu/gl.rs`
(ADR-T127-2, checked by `tests/unsafe_levels.rs`). Every unsafe primitive,
like `Library` and `Symbol` in `crates/draw-sys/src/loader.rs`
(ADR-T142-1), meets six obligations.

## The six obligations

1. **Safe wrapper.** Callers use a safe API. If the caller must promise
   something the type system cannot check (`Library::symbol` takes the
   function type on trust), the function is `unsafe fn` and says so.
2. **`# Safety`.** Each `unsafe fn` documents what the caller must
   guarantee. Each `unsafe` block has a `// SAFETY:` comment saying why it
   holds (`clippy::undocumented_unsafe_blocks` is `deny`).
3. **Unit tests.** One per error path and one for the success path, in the
   same file. Run them under Miri where the primitive allows it (Miri
   cannot call foreign functions such as `dlopen`; those tests carry
   `#[cfg_attr(miri, ignore = "...")]`).
4. **Integration tests.** Through the public API only, in
   `crates/<crate>/tests/`. The sanitizers run these.
5. **Regression suite.** Every bug found in a primitive gets a test that
   fails without the fix, and it is never deleted.
6. **Sanitizers and fuzz.** The nightly job runs the address and thread
   sanitizers on the integration tests, and Miri on the unit tests.
   Primitives that parse bytes also get a fuzz target.

## The `TEST:` marker

Every `unsafe` block, `unsafe fn`, `unsafe impl` and `unsafe extern` in
`crates/*/src` names the test that covers it with a comment, on the same
line or in the 8 lines above:

```rust
// TEST: symbol_present_can_be_called
// SAFETY: the handle is live and the name is a valid C string.
let raw = unsafe { dlsym(handle, name.as_ptr()) };
```

The named function must exist in the repository. Code after
`#[cfg(test)]` is not scanned. The check is the test
`every_unsafe_block_has_a_test_reference` in
`crates/draw-sys/tests/unsafe_refs.rs`.

## Commands

```sh
scripts/unsafe-tests.sh               # marker check, then Miri if installed
scripts/unsafe-sanitizers.sh          # address and thread sanitizers
scripts/unsafe-sanitizers.sh address  # one of them
```

Both need `rustup toolchain install nightly --component rust-src miri`.
The sanitizers build the standard library, so the first run takes a few
minutes. The `sanitizers` job of `.github/workflows/nightly.yml` runs both
scripts every night.

## Related

- [[ADR-T142-1 dlopen through libc symbols and TEST markers]]
- [[Quality Gates]]
