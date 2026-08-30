// RLINE-PF-01 / RLINE-PF-02 / RLINE-PF-03: retain kernel-boundary coverage.

fn assert_contains(haystack: &str, needle: &str, label: &str) {
    assert!(haystack.contains(needle), "missing `{needle}` in {label}");
}

#[test]
fn open_pitfalls_are_use_case_first_and_test_backed() {
    let pitfalls = include_str!("../../../.pitfall/rline-pitfalls.md");
    for id in ["RLINE-PF-01", "RLINE-PF-02", "RLINE-PF-03"] {
        assert_contains(pitfalls, id, ".pitfall/rline-pitfalls.md");
    }
    for field in [
        "**Actor:**",
        "**Task:**",
        "**Surface:**",
        "**Likely mistake:**",
        "**Consequence:**",
        "**Owner:**",
        "**Test:** `cargo test -p rline-core --test pitfall_policy`.",
    ] {
        assert_contains(pitfalls, field, ".pitfall/rline-pitfalls.md");
    }
}

#[test]
fn kernel_boundary_and_candidate_scope_stay_explicit() {
    let readme = include_str!("../../../README.md");
    assert_contains(readme, "RLINE stays product-neutral.", "README.md");
    assert_contains(
        readme,
        "RLINE must not depend on those application",
        "README.md",
    );
    assert_contains(
        readme,
        "Candidate kernels are not public API commitments.",
        "README.md",
    );
    assert_contains(
        readme,
        "planning-only until concrete consumer pressure",
        "README.md",
    );

    let foundation = include_str!("../../../docs/specs/rline-foundation.md");
    assert_contains(
        foundation,
        "It must not depend on application crates",
        "docs/specs/rline-foundation.md",
    );
    assert_contains(
        foundation,
        "RPLAN and RCOUNT live in their own sibling repos; they are consumers, not",
        "docs/specs/rline-foundation.md",
    );

    let algebra_wave =
        include_str!("../../../context/waves/2026-06-06-rline-algebra-kernel-candidate/WAVE.md");
    assert_contains(
        algebra_wave,
        "real consumer pressure appears",
        "context/waves/2026-06-06-rline-algebra-kernel-candidate/WAVE.md",
    );
    assert_contains(
        algebra_wave,
        "No public API commitment until at least one consumer proves a concrete need.",
        "context/waves/2026-06-06-rline-algebra-kernel-candidate/WAVE.md",
    );
}

#[test]
fn rcount_rehearsal_remains_a_release_gate() {
    let compatibility = include_str!("../../../docs/compatibility.md");
    assert_contains(
        compatibility,
        "RCOUNT is the required first consumer rehearsal",
        "docs/compatibility.md",
    );
    assert_contains(
        compatibility,
        "RLINE foundation changes are not ready until the manifest tests, affected",
        "docs/compatibility.md",
    );
    assert_contains(
        compatibility,
        "cargo test -p rcount-district aggregation_consumes_minimal_rctx_fixture_crosswalk",
        "docs/compatibility.md",
    );
    assert_contains(
        compatibility,
        "cargo test -p rcount-rhist",
        "docs/compatibility.md",
    );
}
