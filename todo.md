# To Do

- [x] Rewrite the compiler in Rust
- [ ] Built-in documentation generator
- [ ] Built-in code formatter
- [ ] Emit ASM instead of C?
- [ ] Translate from C to Sea
- [ ] Memory utilities
  - [ ] `std/mem/arena` - arena allocator
  - [ ] `--memtrace` flag to trace alloc/free calls to debug memory leaks
- [x] tags and tagrec
- [x] Namespaces?
- [ ] ~~Function overloading?~~
- [x] Reef implementation in Sea
- [ ] Traits?
- [x] ~~Polish or~~ remove each/of (`of` is staying for now since I may implement `for of`)
- [ ] Multithreading
- [/] Type ~~and generic~~ inference
- [ ] Prevent common C vulnerabilities (buffer overflows, use-after-free, double-free, etc)
- [ ] Cache compiled libraries
- [ ] Contracts
- [ ] `array.len`
- [ ] **Optional** garbage collector
- [ ] Automatically forward declare structs, functions, etc
- [ ] Revisit doc comment syntax
- [ ] Header file generation?

- [ ] Make the compiler mean!

  - [ ] Throw errors when a user doesn't return something in a non-void function
  - [ ] Prevent implicit type conversions
  - [ ] Non-nil pointers (syntax subject to change)

    ```sea
    fun refref(pointer: notnil ^void): notnil ^^void {
      ret ref pointer
    }
    // Invoking refref(nil) throws a compiler error
    ```
