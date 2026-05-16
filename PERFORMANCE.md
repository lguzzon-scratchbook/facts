# Performance Assessment — facts CLI

**Date:** 2026-05-16
**Version:** 0.5.4
**Tools:** criterion 0.5.1, cargo-llvm-cov 0.8.7

## Coverage

**Overall: 93.84%** (6241 lines, 330 missed)

| Module | Line Coverage | Notes |
|--------|--------------|-------|
| add.rs | 97.39% | |
| check.rs | 91.14% | |
| color.rs | 84.85% | |
| edit.rs | 96.52% | |
| fmt.rs | 80.85% | |
| get.rs | 96.55% | |
| id.rs | 98.67% | |
| init.rs | 93.31% | |
| lint.rs | 96.81% | |
| list.rs | 97.49% | |
| locate.rs | 100.00% | |
| lock.rs | 83.13% | |
| main.rs | 100.00% | |
| model.rs | 100.00% | |
| move_fact.rs | 94.29% | |
| parser.rs | 98.98% | |
| project.rs | 82.98% | |
| remove.rs | 96.45% | |
| skills.rs | 95.69% | |
| tags.rs | 97.94% | |
| uninit.rs | 100.00% | |
| update.rs | 74.77% | Network-dependent |
| writer.rs | 98.39% | |

## Performance Fixes Applied

### Fix 1: Compile tag/search expressions once (was: re-parse per fact)
**Before:** `matches_tag_expr(expr, &fact.tags)` called `parse_expr()` N times per fact
**After:** `compile_tag_expr(expr)` called once, `eval_tag_expr(compiled, &fact.tags)` per fact
**Impact:** ~18% faster on complex tag expressions (measured: tag0 and not tag1: 5.3ms → 4.35ms)

### Fix 2: Pre-lowercase haystack once (was: to_ascii_lowercase per fact)
**Before:** `matches_search_expr(expr, &haystack)` lowercased inside eval each time
**After:** Caller lowercases once, `eval_search_expr` works on pre-lowered string
**Impact:** Eliminates N string allocations per filtered search

### Fix 3: Reduce allocations in format_display_path
**Before:** Allocated Vec<&str>, then Vec<String> via map
**After:** Single iterator chain with `.map(color::bold)` (no redundant closure)
**Impact:** Fewer intermediate allocations per fact displayed

## Benchmark Results

| Benchmark | Median Time | Notes |
|-----------|------------|-------|
| list (parse+display) small | ~0.9 ms | |
| list (parse+display) medium | ~2.3 ms | |
| list (parse+display) large | ~9.3 ms | ~1500 facts |
| lint small | ~0.9 ms | |
| lint medium | ~2.3 ms | |
| lint large | ~9.1 ms | |
| add single | ~14 ms | File I/O dominated |
| tag filter: mvp | ~4.5 ms | |
| tag filter: tag0 and not tag1 | ~4.35 ms | Fixed (was ~5.3ms) |
| tag filter: not tag5 | ~7.4 ms | |
| section filter: Section 0 | ~4.6 ms | |
| section filter: Subsection 9.4 | ~4.3 ms | |

## Profiling Tools

### Criterion Benchmarks
```bash
cargo bench --bench benchmarks              # run all
cargo bench --bench benchmarks -- --save-baseline base   # save baseline
# ... after changes ...
cargo bench --bench benchmarks -- --baseline base        # compare
open target/criterion/report/index.html                  # HTML report
```

### Flame Graphs
```bash
cargo install flamegraph
sudo flamegraph --bench benchmarks          # macOS: requires --rootless
# Output: flamegraph.svg in project root
```

### Heap Profiling (dhat)
```bash
cargo run --features dhat --bin facts-dhat -- list   # produces dhat-heap.json
# Upload to https://nnethercote.github.io/dh_view/dh_view.html
```

### Coverage
```bash
cargo llvm-cov              # terminal report
cargo llvm-cov --html       # HTML report in target/llvm-cov/html/
```
