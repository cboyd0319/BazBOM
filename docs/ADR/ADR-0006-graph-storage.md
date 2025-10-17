# ADR-0006: Dependency Graph Data Structure

**Status:** Accepted
**Date:** 2025-10-17
**Deciders:** Engineering Team

## Context

Dependency graphs for large repos contain 1000+ nodes and 5000+ edges. We need efficient storage and query performance.

## Decision

Use **adjacency list** representation stored as JSON with optional GraphML export.

### Data Structure

```json
{
  "nodes": {
    "pkg:maven/guava@31.1": {
      "name": "guava",
      "version": "31.1-jre",
      "metadata": {...}
    }
  },
  "adjacency": {
    "pkg:maven/guava@31.1": [
      "pkg:maven/failureaccess@1.0.1",
      "pkg:maven/checker-qual@3.12.0"
    ]
  }
}
```

### Rationale

- **Space efficient:** O(V + E) vs O(V²) for adjacency matrix
- **Query efficient:** Reverse deps require index, but buildable in O(E)
- **JSON friendly:** Easy to serialize/deserialize
- **Tool compatible:** Convertible to GraphML for visualization

## Consequences

**Positive:**
- Handles graphs with 10K+ nodes without OOM
- Fast forward dependency queries (O(1) lookup)
- Human-readable JSON

**Negative:**
- Reverse dependency queries require index build (O(E))
- Not optimal for dense graphs (rare in dependencies)

**Mitigations:**
- Build reverse index on demand: `build_graph_index`
- Cache index for repeated queries

## Alternatives

**Alternative 1:** Adjacency Matrix
- **Rejected:** O(V²) space, wasteful for sparse graphs

**Alternative 2:** SQL Database
- **Rejected:** Adds deployment complexity

**Alternative 3:** Graph Database (Neo4j)
- **Rejected:** Overkill for read-only analysis

## References
- [Graph Representations](https://en.wikipedia.org/wiki/Graph_(abstract_data_type))
