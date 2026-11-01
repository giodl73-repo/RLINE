# Build the RLINE graph workbench

Use Rust 1.95.0 plus wasm32-unknown-unknown, wasm-bindgen-cli 0.2.127, Python 3.12
and Node 22. The unpublished consumer has its own workspace and lockfile; it uses
the repository's existing rgraph-core without changing the main dependency graph.

```
cargo +1.95.0 test --locked --workspace
cargo +1.95.0 run --locked -p rline-cli -- manifest
cargo +1.95.0 run --locked -p rhist-cli -- --help
cargo +1.95.0 test --locked --manifest-path web/adapter/Cargo.toml --all-features
npm ci
npx playwright install chromium
python -B tools/build-pages.py
npm run test:pages
python -B tools/serve-pages.py
```

Open http://127.0.0.1:4177/RLINE/. Set RLINE_BASE_URL to test the deployment.
The builder emits a full Git source archive, separate Rust source views, compiled
WASM, dependency and verbatim Rust notices, provenance and a file inventory. Dirty
previews are labeled; public deployment requires a clean tested master artifact.

`rline-replay REQUEST.json` produces the same canonical report as WASM;
`--examples` emits authored neutral fixtures. Inputs remain local, up to 128 KiB,
64 nodes and 512 edges. Typed u32 indices, unique edge IDs, valid selection/filter
references, finite non-negative weights and bounded strings are checked before
native execution. Unknown and duplicate known JSON keys are rejected. Browser
file reads, worker startup and computation have independent deadlines and fences.
Original imported UTF-8/CRLF is retained until an explicit edit; graph-node source
activation is an edit which rewrites JSON formatting and reruns the worker.

Native distances retain f64 and absolute 1e-9 comparison semantics. Null means
unreachable. Native path counts are diagnostic, not certified enumeration under
zero-cost cycles or tolerance ties. Topology and boundary metrics ignore weights;
parallel node pairs collapse and loops disappear only in the boundary projection.
Zero-volume conductance is native zero, not evidence of quality. Native bridge
records retain oriented incidences; the companion deduplicates external IDs only
for the physical-edge summary. The graph layout is schematic, independent of
algorithmic results, and may overlay parallel edges; the edge records stay visible.

No public API, family manifest, RCTX/RHIST behavior, algebra implementation or
downstream migration is introduced. The older tracked branch's 18 RHIST metadata
files are restored from public master because its CRLF-derived hashes did not
match the canonical LF source blobs; fixture source bytes are unchanged. Existing
RCOUNT rehearsal rules remain in force for actual foundation contract changes.
