//! Wiring test: every detector file must be fully registered in `lib.rs`.
//!
//! A detector is "fully wired" if and only if:
//!   1. `pub mod <stem>;` is declared in `lib.rs`.
//!   2. `impl Check for <Struct>` exists in the file, and
//!      `Box::new(<Struct>)` appears inside `default_checks()` in `lib.rs`.
//!
//! Files excluded from the check (shared helpers, not detectors):
//!   lib.rs, cfg.rs, provenance.rs, util.rs
//!
//! Failing this test means a contributor wrote a detector that is silently
//! never executed — its unit tests never run and scan_directory never calls it.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Helper files that live in `src/` but are not detectors.
const EXCLUDED: &[&str] = &["lib", "cfg", "provenance", "util"];

/// Extract all `impl Check for <Struct>` struct names from source text.
/// Returns an empty vec if the file contains no such `impl` block.
fn impl_check_structs(source: &str) -> Vec<String> {
    // Pattern: `impl Check for SomeName` (ignoring generics / lifetime params)
    let mut results = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("impl Check for ") {
            // Take everything up to the first whitespace, `{`, or `<`
            let struct_name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !struct_name.is_empty() {
                results.push(struct_name);
            }
        }
    }
    results
}

/// Locate the region of `lib_source` that is the body of `default_checks()`.
/// Returns the text of that function body (or the whole file if not found, so
/// the test degrades gracefully).
fn default_checks_body(lib_source: &str) -> &str {
    if let Some(start) = lib_source.find("pub fn default_checks()") {
        // Find the opening `{` after the function signature
        if let Some(brace_offset) = lib_source[start..].find('{') {
            let body_start = start + brace_offset;
            // Walk forward counting braces to find the matching `}`
            let mut depth = 0usize;
            let chars = lib_source[body_start..].char_indices();
            for (i, ch) in chars {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            return &lib_source[body_start..body_start + i + 1];
                        }
                    }
                    _ => {}
                }
            }
            // Brace mismatch – return from opening brace to end of file
            return &lib_source[body_start..];
        }
    }
    lib_source
}

#[test]
fn every_detector_is_fully_wired() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");
    let lib_path = src_dir.join("lib.rs");

    let lib_source = fs::read_to_string(&lib_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {}", lib_path.display(), e));

    let default_checks_fn = default_checks_body(&lib_source);

    let excluded: HashSet<&str> = EXCLUDED.iter().copied().collect();

    let mut failures: Vec<String> = Vec::new();

    // Iterate every *.rs in src/ (non-recursive; all check files are flat)
    let mut entries: Vec<_> = fs::read_dir(&src_dir)
        .unwrap_or_else(|e| panic!("Cannot read {}: {}", src_dir.display(), e))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "rs"))
        .collect();

    // Sort for deterministic output
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();

        if excluded.contains(stem.as_str()) {
            continue;
        }

        // ── Check 1: `pub mod <stem>;` present in lib.rs ──────────────────
        let mod_decl = format!("pub mod {};", stem);
        if !lib_source.contains(&mod_decl) {
            failures.push(format!(
                "[{}] missing `{}` in lib.rs  (mod declaration absent)",
                stem, mod_decl
            ));
        }

        // ── Check 2: every `impl Check for <Struct>` is in default_checks() ─
        let file_source = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Cannot read {}: {}", path.display(), e));

        for struct_name in impl_check_structs(&file_source) {
            let box_expr = format!("Box::new({})", struct_name);
            if !default_checks_fn.contains(&box_expr) {
                failures.push(format!(
                    "[{stem}.rs] `impl Check for {struct_name}` exists but \
                     `{box_expr}` is missing from default_checks() in lib.rs"
                ));
            }
        }
    }

    if !failures.is_empty() {
        let msg = failures.join("\n  ");
        panic!(
            "\n\nWiring test failed — {} unregistered detector(s):\n\n  {}\n\n\
             Every detector in crates/checks/src/ must be wired in lib.rs:\n\
             1. `pub mod <name>;`\n\
             2. `Box::new(<StructName>)` inside default_checks()\n",
            failures.len(),
            msg
        );
    }
}
