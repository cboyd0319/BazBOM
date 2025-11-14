# Dependency Graph Analysis Guide

**Audience:** Developers, security analysts, architects
**Purpose:** Query and visualize dependency relationships for impact analysis and security assessment
**Last Reviewed:** 2025-10-17

## TL;DR

BazBOM generates queryable dependency graphs showing all transitive relationships. Use graphs to answer "what depends on X?" and "what vulnerabilities affect Y?"

```bash
# Generate dependency graph
bazel build //:dep_graph_all

# Query reverse dependencies
bazel run //tools/supplychain:graph_query -- \
  --package="pkg:maven/com.google.guava/guava@31.1-jre" \
  --query=reverse-deps

# Visualize graph
bazel run //tools/supplychain:graph_visualizer -- \
  --input=bazel-bin/dep_graph.graphml \
  --output=dep_graph.png
```

## Graph Formats

BazBOM exports dependency graphs in two formats:

### 1. JSON (Queryable)

```json
{
  "version": "1.0",
  "metadata": {
    "generated_at": "2025-10-17T12:00:00Z",
    "bazel_version": "7.0.0",
    "total_nodes": 523,
    "total_edges": 1247
  },
  "graph": {
    "nodes": [
      {
        "id": "pkg:maven/com.google.guava/guava@31.1-jre",
        "name": "guava",
        "version": "31.1-jre",
        "group": "com.google.guava",
        "depth": 1,
        "scope": "compile",
        "direct": true,
        "licenses": ["Apache-2.0"],
        "size_bytes": 2714802,
        "targets": ["//app:main", "//lib:utils"]
      }
    ],
    "edges": [
      {
        "from": "pkg:maven/com.google.guava/guava@31.1-jre",
        "to": "pkg:maven/com.google.guava/failureaccess@1.0.1",
        "type": "depends_on",
        "scope": "compile"
      }
    ]
  }
}
```

### 2. GraphML (Visualization)

GraphML format for use with:
- **Gephi** - Network analysis and visualization
- **yEd** - Diagramming and layout
- **Cytoscape** - Biological networks (works for dependencies too)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <graph id="dependencies" edgedefault="directed">
    <node id="pkg:maven/com.google.guava/guava@31.1-jre">
      <data key="name">guava</data>
      <data key="version">31.1-jre</data>
      <data key="depth">1</data>
    </node>
    <edge source="pkg:maven/com.google.guava/guava@31.1-jre"
          target="pkg:maven/com.google.guava/failureaccess@1.0.1"/>
  </graph>
</graphml>
```

## Generating Graphs

### Full Workspace Graph

```bash
# Generate graph for all targets
bazel build //:dep_graph_all

# Output files:
# - bazel-bin/dep_graph.json
# - bazel-bin/dep_graph.graphml
```

### Per-Target Graph

```bash
# Generate graph for single target
bazel build //app:app_dep_graph

# Output: bazel-bin/app/app_dep_graph.json
```

### Filtered Graphs

```bash
# Only production dependencies (exclude test scope)
bazel build //:dep_graph_all --define=include_test_deps=false

# Only direct dependencies (depth=1)
bazel build //:dep_graph_all --define=max_depth=1

# Specific subtree
bazel build //services/...:dep_graph_all
```

## Querying Dependency Graphs

### Using graph_query Tool

```bash
# What depends on package X? (reverse dependencies)
bazel run //tools/supplychain:graph_query -- \
  --package="pkg:maven/com.google.guava/guava@31.1-jre" \
  --query=reverse-deps

# Output:
# pkg:maven/com.example/app@1.0.0
# pkg:maven/com.example/lib-utils@2.1.0
# pkg:maven/com.example/lib-common@1.5.0
```

```bash
# What does package X depend on? (forward dependencies)
bazel run //tools/supplychain:graph_query -- \
  --package="pkg:maven/com.example/app@1.0.0" \
  --query=forward-deps \
  --max-depth=2

# Output:
# pkg:maven/com.google.guava/guava@31.1-jre (depth=1)
# pkg:maven/com.google.guava/failureaccess@1.0.1 (depth=2)
# pkg:maven/org.slf4j/slf4j-api@1.7.36 (depth=1)
```

```bash
# Find dependency path from A to B
bazel run //tools/supplychain:graph_query -- \
  --package="pkg:maven/com.example/app@1.0.0" \
  --query=path \
  --target="pkg:maven/commons-codec/commons-codec@1.15"

# Output:
# pkg:maven/com.example/app@1.0.0
#   → pkg:maven/org.apache.httpcomponents/httpclient@4.5.13
#     → pkg:maven/commons-codec/commons-codec@1.15
```

```bash
# Find all packages with version conflicts
bazel run //tools/supplychain:graph_query -- \
  --query=conflicts

# Output:
# commons-codec:
#   1.15 (used by: httpclient:4.5.13, aws-sdk:1.12.261)
#   1.11 (used by: legacy-lib:2.0.0)
# Resolved to: 1.15
```

### Using jq for JSON Queries

```bash
# Find all direct dependencies
jq '.graph.nodes[] | select(.direct == true) | {name, version}' \
  bazel-bin/dep_graph.json

# Find dependencies deeper than 5 levels
jq '.graph.nodes[] | select(.depth > 5) | {id, depth}' \
  bazel-bin/dep_graph.json

# Calculate dependency footprint (total size)
jq '[.graph.nodes[].size_bytes] | add' bazel-bin/dep_graph.json
# Output: 45328192 (45MB)

# Find all GPL-licensed dependencies
jq '.graph.nodes[] | select(.licenses[] | contains("GPL")) | {id, licenses}' \
  bazel-bin/dep_graph.json

# Group dependencies by depth
jq '.graph.nodes | group_by(.depth) |
    map({depth: .[0].depth, count: length})' \
  bazel-bin/dep_graph.json
```

## Impact Analysis

### Vulnerability Impact

```bash
# Given a CVE, find all affected targets
CVE="CVE-2023-12345"
PACKAGE="pkg:maven/org.example/vulnerable-lib@1.2.3"

# Find all reverse dependencies
bazel run //tools/supplychain:graph_query -- \
  --package="$PACKAGE" \
  --query=reverse-deps \
  --output=affected_targets.txt

# Cross-reference with Bazel targets
cat affected_targets.txt | while read PURL; do
  jq -r --arg purl "$PURL" \
    '.graph.nodes[] | select(.id == $purl) | .targets[]' \
    bazel-bin/dep_graph.json
done | sort -u
```

Output:

```
//app:main
//services/api:server
//services/worker:processor
```

**Blast radius:** CVE-2023-12345 affects 3 targets.

### Upgrade Impact

```bash
# Find what breaks if we upgrade package X
PACKAGE="pkg:maven/com.google.guava/guava"
OLD_VERSION="31.1-jre"
NEW_VERSION="32.0.0-jre"

# Find all reverse dependencies (would need retest)
bazel run //tools/supplychain:graph_query -- \
  --package="${PACKAGE}@${OLD_VERSION}" \
  --query=reverse-deps

# Check for version conflicts with new version
bazel run //tools/supplychain:graph_query -- \
  --query=simulate-upgrade \
  --package="${PACKAGE}" \
  --from="${OLD_VERSION}" \
  --to="${NEW_VERSION}"
```

### Transitive Bloat Analysis

```bash
# Find packages with excessive transitive dependencies
bazel run //tools/supplychain:graph_query -- \
  --query=bloat-analysis \
  --threshold=50

# Output:
# pkg:maven/com.amazonaws/aws-java-sdk@1.12.261
#   Direct dependencies: 15
#   Transitive dependencies: 234
#   Total footprint: 78MB
#   Recommendation: Use aws-java-sdk-s3 (specific module) instead
```

## Visualization

### Using Gephi

1. **Export GraphML:**

```bash
bazel build //:dep_graph_all
cp bazel-bin/dep_graph.graphml ~/Desktop/
```

2. **Open in Gephi:**
   - Launch Gephi
   - File → Open → Select `dep_graph.graphml`
   - Layout → ForceAtlas 2 (recommended)
   - Run layout for 30-60 seconds

3. **Color by depth:**
   - Appearance → Nodes → Color → Partition → `depth`
   - Choose color palette (e.g., heat map)

4. **Size by reverse dependencies:**
   - Appearance → Nodes → Size → Ranking → `in-degree`
   - Min size: 10, Max size: 50

5. **Export:**
   - File → Export → PDF/PNG

### Using yEd

1. **Open GraphML:**
   - yEd → File → Open → Select `dep_graph.graphml`

2. **Apply Layout:**
   - Layout → Hierarchical (for tree-like dependencies)
   - Layout → Organic (for complex graphs)

3. **Style:**
   - Tools → Palette → Select node/edge styles
   - Map properties to visual attributes

4. **Export:**
   - File → Export → SVG/PNG

### Using graph_visualizer (Built-in)

```bash
# Generate PNG with GraphViz
bazel run //tools/supplychain:graph_visualizer -- \
  --input=bazel-bin/dep_graph.graphml \
  --output=dep_graph.png \
  --layout=dot \
  --color-by=depth

# Layouts:
#   dot    - Hierarchical (top-down)
#   neato  - Spring model (force-directed)
#   fdp    - Force-directed placement
#   circo  - Circular layout
#   twopi  - Radial layout

# Generate interactive HTML
bazel run //tools/supplychain:graph_visualizer -- \
  --input=bazel-bin/dep_graph.json \
  --output=dep_graph.html \
  --format=d3 \
  --interactive
```

Open `dep_graph.html` in browser for interactive exploration:
- Zoom/pan
- Click nodes to highlight dependencies
- Search for packages
- Filter by depth/scope

## Advanced Graph Queries

### Find Dependency Clusters

```bash
# Packages that depend on each other (circular deps shouldn't exist, but...)
bazel run //tools/supplychain:graph_query -- \
  --query=strongly-connected-components

# Output:
# Component 1: [pkg:maven/a/b@1.0, pkg:maven/c/d@2.0] (size: 2)
# Component 2: [pkg:maven/e/f@3.0] (size: 1)
```

### Critical Path Analysis

```bash
# Find longest dependency chain (critical path)
bazel run //tools/supplychain:graph_query -- \
  --query=critical-path

# Output:
# Length: 12 levels
# Path:
#   pkg:maven/com.example/app@1.0.0
#   → pkg:maven/com.amazonaws/aws-java-sdk@1.12.261
#   → pkg:maven/com.fasterxml.jackson.core/jackson-databind@2.13.3
#   → ...
#   → pkg:maven/commons-logging/commons-logging@1.2 (depth=12)
```

### License Compatibility Analysis

```bash
# Find potential license conflicts
bazel run //tools/supplychain:graph_query -- \
  --query=license-compatibility \
  --product-license=Apache-2.0

# Output:
#   Potential conflicts:
# pkg:maven/org.gnu/some-lib@1.0.0 (GPL-3.0)
#   Used by: pkg:maven/com.example/app@1.0.0
#   Conflict: GPL-3.0 incompatible with Apache-2.0
```

### Unmaintained Dependency Detection

```bash
# Find dependencies with no recent commits
bazel run //tools/supplychain:graph_query -- \
  --query=unmaintained \
  --threshold-years=2

# Output:
# pkg:maven/commons-logging/commons-logging@1.2
#   Last release: 2014-07-11 (11 years ago)
#   Recommendation: Migrate to slf4j
```

## Exporting Graph Data

### CSV Export for Analysis

```bash
# Export nodes as CSV
jq -r '.graph.nodes[] | [.id, .name, .version, .depth, .direct, .size_bytes] | @csv' \
  bazel-bin/dep_graph.json > nodes.csv

# Export edges as CSV
jq -r '.graph.edges[] | [.from, .to, .type, .scope] | @csv' \
  bazel-bin/dep_graph.json > edges.csv

# Import into Excel/Sheets for pivot tables
```

### SQL Database Import

```bash
# Import into SQLite for complex queries
sqlite3 deps.db <<EOF
CREATE TABLE nodes (
  id TEXT PRIMARY KEY,
  name TEXT,
  version TEXT,
  depth INTEGER,
  direct BOOLEAN,
  size_bytes INTEGER
);

CREATE TABLE edges (
  from_id TEXT,
  to_id TEXT,
  type TEXT,
  scope TEXT,
  FOREIGN KEY (from_id) REFERENCES nodes(id),
  FOREIGN KEY (to_id) REFERENCES nodes(id)
);
EOF

# Import CSV
sqlite3 deps.db '.mode csv' '.import nodes.csv nodes' '.import edges.csv edges'

# Query with SQL
sqlite3 deps.db "
  SELECT n.id, COUNT(e.to_id) as dependency_count
  FROM nodes n
  LEFT JOIN edges e ON n.id = e.from_id
  GROUP BY n.id
  ORDER BY dependency_count DESC
  LIMIT 10;
"
```

## Integration with Bazel Query

### Combine with Bazel's Dependency Query

```bash
# BazBOM graph: JVM dependencies (from maven_install.json)
bazel build //:dep_graph_all

# Bazel query: Build graph (Bazel targets)
bazel query 'deps(//app:main)' --output=graph > bazel_graph.dot

# Combined view:
# - Bazel graph shows target-level dependencies
# - BazBOM graph shows package-level dependencies
```

### Find Missing Dependencies

```bash
# Packages used but not declared (phantom dependencies)
bazel run //tools/supplychain:graph_query -- \
  --query=phantom-dependencies \
  --bytecode-analysis=true

# Output:
# //app:main
#   Uses: org.apache.commons.lang3.StringUtils
#   But does not declare: commons-lang3:3.12.0
#   Recommendation: Add to BUILD file deps
```

## Performance Considerations

### Large Graphs (1000+ nodes)

```bash
# Generate graph in streaming mode (lower memory)
bazel build //:dep_graph_all --define=streaming=true

# Reduce graph size by filtering
bazel build //:dep_graph_all \
  --define=exclude_test_deps=true \
  --define=max_depth=5
```

### Caching Graph Analysis

```bash
# Cache graph query results
bazel run //tools/supplychain:graph_query -- \
  --package="pkg:maven/com.google.guava/guava@31.1-jre" \
  --query=reverse-deps \
  --cache-dir=.cache/graph-queries

# Subsequent queries are instant (cache hit)
```

## Troubleshooting

### Graph Missing Nodes

**Cause:** Aspect did not traverse all targets.

**Fix:**

```bash
# Ensure all targets are built
bazel build //... --keep_going

# Rebuild graph
bazel build //:dep_graph_all --nocache_test_results
```

### GraphML Won't Open in Gephi

**Cause:** Large file (> 100MB), invalid XML, or missing attributes.

**Fix:**

```bash
# Validate GraphML
xmllint --noout bazel-bin/dep_graph.graphml

# If too large, filter first
bazel run //tools/supplychain:graph_filter -- \
  --input=bazel-bin/dep_graph.graphml \
  --max-depth=3 \
  --output=filtered_graph.graphml
```

### Query Performance is Slow

**Cause:** O(n²) graph traversal on large graphs.

**Fix:**

```bash
# Build graph index
bazel run //tools/supplychain:build_graph_index -- \
  --input=bazel-bin/dep_graph.json \
  --output=bazel-bin/dep_graph.index

# Use indexed queries (10-100x faster)
bazel run //tools/supplychain:graph_query -- \
  --index=bazel-bin/dep_graph.index \
  --package="pkg:maven/com.google.guava/guava@31.1-jre" \
  --query=reverse-deps
```

## References

- [GraphML Specification](https://graphml.graphdrawing.org/)
- [Gephi Documentation](https://gephi.org/users/)
- [yEd User Manual](https://yed.yworks.com/support/manual/)
- [Bazel Query Documentation](https://bazel.build/query/guide)
- [jq Manual](https://stedolan.github.io/jq/manual/)
