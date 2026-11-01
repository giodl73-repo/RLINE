# Browser graph workbench

Status: active. Consumer-only delivery of existing graph APIs.

Readers supply their own small undirected weighted multigraph, source, selected
nodes and blocked edges. Existing rgraph-core computes distances, reachable
nodes, components, bridges and selection boundary metrics. Browser UI renders
these results and exports exact input plus native-derived reports.

The consumer is a separate unpublished workspace. Shared kernel source, workspace
manifest/lock, family manifest, serialized schemas, hashes and public API behavior
stay unchanged. No downstream adoption or RCOUNT readiness is claimed. No algebra
implementation or commitment is introduced; the existing algebra candidate wave
remains planning-only and its pending pulses remain pending.

Entry: public master 8426af135619224f073369e93aa05e79c8d45bf6 plus tracked PITFALL
c9d15a6c401b28346ea2a2fe52c15874f7eeb757. The tracked branch changed 18 RHIST fixture
metadata files to hashes which do not match their canonical LF source blobs.
Restore those metadata files from public master, preserving all PITFALL doctrine
and tests. Record both fixture custody and consumer evidence in the pulse.

See [pulse 01](pulses/pulse-01.md) for acceptance and validation.
