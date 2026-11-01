# Pulse 01: graph inspection in WebAssembly

Status: implemented; clean hosted/live qualification follows local verification.

1. Preserve tracked PITFALL doctrine/tests and public-master LF fixture custody.
2. Add an isolated consumer of existing rgraph-core APIs, with bounded own-input
   JSON, a visible graph, source/selection/blocked-edge scenarios and exports.
3. Retain original UTF-8/CRLF input independently of successful WASM operation.
4. Explain native floating tolerance, unreachable nodes, parallel edges, loops,
   topology-only boundary projection and zero-volume conductance convention.
5. Provide complete source/WASM/build/license/provenance downloads and test the
   exact clean artifact before deployment from master.

Acceptance: GRAPH-01 own graph task; GRAPH-02 unchanged native calculations;
GRAPH-03 exact exports and clear topology/cost boundaries; GRAPH-04 input/resource
validation; GRAPH-05 keyboard/mobile and file/worker recovery; GRAPH-06 source,
fixture and release custody. No geographic, election, route-service, fairness,
algebra, consumer-adoption or performance certification is implied.

Validation: workspace fmt/tests, rline/rhist CLI smokes, isolated consumer native
tests and strict host/wasm Clippy, exact native/WASM parity, Chromium own-input,
visual/export/recovery tasks and desktop/mobile screenshots, role simulation,
fixture-source hashes and unchanged kernel/API/manifest checks, exact-master
hosted/live files and fresh live browser replay before TRACKER snapshot.

## Local verification

252 workspace tests, eight consumer tests, documented CLI smokes and fmt pass.
Strict consumer host/wasm Clippy passes. Initial Chromium run passes all 22 cases
and 59 exact native/WASM responses. Mobile review added a full-size source-node
dropdown alongside SVG keyboard activation; clean qualification retests this UI.
The 18 restored metadata files match public master, preserving canonical LF
source bytes; all kernel code, root manifest/lock, schemas and expected behavior
remain unchanged. See [review and acceptance trace](../REVIEW.md).
