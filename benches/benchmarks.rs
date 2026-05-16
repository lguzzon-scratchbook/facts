use assert_cmd::Command;
/// Performance benchmarks for the `facts` CLI.
///
/// Run with: `cargo bench`
/// For flame graphs: `cargo flamegraph --bench benchmarks`
/// For heap profiling: `cargo run --features dhat --bin facts-dhat`
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::fs;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Benchmark data generators
// ---------------------------------------------------------------------------

fn small_sheet() -> String {
    "- fact one\n- fact two @tag\n- label: fact three\n  command: echo test\n".to_string()
}

fn medium_sheet() -> String {
    let mut s = String::new();
    for i in 0..50 {
        s.push_str(&format!("- fact number {i} @tag{i}\n"));
    }
    s.push_str("\n# section A\n\n");
    for i in 0..50 {
        s.push_str(&format!("- section A fact {i}\n"));
    }
    s.push_str("\n## subsection A1\n\n");
    for i in 0..25 {
        s.push_str(&format!("- subsection A1 fact {i}\n"));
    }
    s.push_str("\n# section B\n\n");
    for i in 0..50 {
        s.push_str(&format!("- section B fact {i}\n"));
    }
    s
}

fn large_sheet() -> String {
    let mut s = String::new();
    for i in 0..200 {
        s.push_str(&format!("- preamble fact {i} @tag{}\n", i % 10));
    }
    for section in 0..10 {
        s.push_str(&format!("\n# Section {section}\n\n"));
        for i in 0..100 {
            s.push_str(&format!("- section {section} fact {i} @tag{}\n", i % 5));
        }
        for sub in 0..5 {
            s.push_str(&format!("\n## Subsection {section}.{sub}\n\n"));
            for i in 0..20 {
                s.push_str(&format!("- sub {section}.{sub} fact {i}\n"));
            }
        }
    }
    s
}

fn setup_project(content: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".facts"), content).unwrap();
    dir
}

// ---------------------------------------------------------------------------
// CLI parsing benchmarks
// ---------------------------------------------------------------------------

fn bench_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("list");

    for (name, content) in [
        ("small", small_sheet()),
        ("medium", medium_sheet()),
        ("large", large_sheet()),
    ] {
        let dir = setup_project(&content);
        group.bench_with_input(BenchmarkId::new("parse", name), &dir, |b, dir| {
            b.iter(|| {
                let mut cmd = Command::cargo_bin("facts").unwrap();
                cmd.current_dir(dir.path())
                    .env("NO_COLOR", "1")
                    .output()
                    .unwrap();
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Lint benchmarks
// ---------------------------------------------------------------------------

fn bench_lint(c: &mut Criterion) {
    let mut group = c.benchmark_group("lint");

    for (name, content) in [
        ("small", small_sheet()),
        ("medium", medium_sheet()),
        ("large", large_sheet()),
    ] {
        let dir = setup_project(&content);
        group.bench_with_input(BenchmarkId::new("validate", name), &dir, |b, dir| {
            b.iter(|| {
                let mut cmd = Command::cargo_bin("facts").unwrap();
                cmd.current_dir(dir.path())
                    .env("NO_COLOR", "1")
                    .arg("lint")
                    .output()
                    .unwrap();
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Add benchmarks (write performance)
// ---------------------------------------------------------------------------

fn bench_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("add");

    for i in 0..20 {
        let _i = i; // iteration counter for benchmark ID
        group.bench_with_input(BenchmarkId::new("single", i), &i, |b, _| {
            let dir = setup_project("- initial fact\n");
            b.iter(|| {
                let mut cmd = Command::cargo_bin("facts").unwrap();
                cmd.current_dir(dir.path())
                    .env("NO_COLOR", "1")
                    .args(["add", "benchmark fact"])
                    .output()
                    .unwrap();
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Tag filter benchmarks
// ---------------------------------------------------------------------------

fn bench_tag_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("tag_filter");
    let dir = setup_project(&large_sheet());

    for expr in [
        "mvp",
        "tag0 and tag1",
        "tag0 or tag1",
        "not tag5",
        "tag0 and not tag1",
    ] {
        group.bench_with_input(BenchmarkId::new("filter", expr), expr, |b, expr| {
            b.iter(|| {
                let mut cmd = Command::cargo_bin("facts").unwrap();
                cmd.current_dir(dir.path())
                    .env("NO_COLOR", "1")
                    .args(["list", "--tags", expr])
                    .output()
                    .unwrap();
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Section filter benchmarks
// ---------------------------------------------------------------------------

fn bench_section_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("section_filter");
    let dir = setup_project(&large_sheet());

    for section in ["Section 0", "Section 5", "Subsection 0.0", "Subsection 9.4"] {
        group.bench_with_input(
            BenchmarkId::new("section", section),
            section,
            |b, section| {
                b.iter(|| {
                    let mut cmd = Command::cargo_bin("facts").unwrap();
                    cmd.current_dir(dir.path())
                        .env("NO_COLOR", "1")
                        .args(["list", "--section", section])
                        .output()
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion entry point
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_list,
    bench_lint,
    bench_add,
    bench_tag_filter,
    bench_section_filter,
);
criterion_main!(benches);
