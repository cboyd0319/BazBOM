# OSV API Integration Plan

## Problem

The current implementation attempts to load OSV vulnerability data from local JSON files in `.bazbom/advisories/osv/`. However, the OSV database is **HUGE** (100GB+) and cannot be practically cached locally.

## Solution

Replace local OSV database scanning with **OSV API calls** using the batch query endpoint.

## OSV API Batch Endpoint

**Endpoint:** `POST https://api.osv.dev/v1/querybatch`

**Documentation:** https://google.github.io/osv.dev/post-v1-querybatch/

### Benefits

1. **No local storage required** - eliminates 100GB+ database download
2. **Always up-to-date** - real-time vulnerability data
3. **Efficient batch queries** - query multiple packages in one request
4. **Pagination support** - handles large result sets

## Request Format

```json
{
  "queries": [
    {
      "package": {
        "purl": "pkg:maven/com.example/library@1.2.3"
      }
    },
    {
      "package": {
        "ecosystem": "Maven",
        "name": "org.example:another-lib"
      },
      "version": "2.0.0"
    }
  ]
}
```

### Query Rules

- Use either `version` **or** a versioned `purl`, not both
- Items with both will return `400 Bad Request`
- Each query can specify `page_token` for pagination

## Response Format

```json
{
  "results": [
    {
      "vulns": [
        {
          "id": "GHSA-xxxx-yyyy-zzzz",
          "modified": "2023-03-14T05:47:39.989396Z"
        },
        {
          "id": "CVE-2023-12345",
          "modified": "2023-03-24T22:28:29.389429Z"
        }
      ],
      "next_page_token": "token_if_more_results"
    },
    {
      "vulns": []
    }
  ]
}
```

## Pagination

Pagination occurs when:
- An individual query returns more than **1,000 vulnerabilities**
- The entire queryset returns more than **3,000 vulnerabilities** total

When `next_page_token` is present in a result, make another request with that token in the corresponding query's `page_token` field.

## Implementation Plan

### Phase 1: Create OSV API Client

**Location:** `crates/bazbom-vulnerabilities/src/osv_client.rs` (new file)

**Key Functions:**
```rust
pub struct OsvClient {
    api_base: String,
    http_client: reqwest::Client,
}

impl OsvClient {
    pub fn new() -> Self { ... }

    pub async fn query_batch(
        &self,
        components: &[Component],
    ) -> Result<Vec<VulnerabilityInfo>> { ... }

    fn build_queries(&self, components: &[Component]) -> Vec<OsvQuery> { ... }

    fn handle_pagination(&self, ...) -> Result<...> { ... }
}
```

### Phase 2: Update SCA Analyzer

**File:** `crates/bazbom/src/analyzers/sca.rs`

**Changes:**
1. Remove `scan_osv_database()` function (lines 206-296)
2. Update `match_vulnerabilities()` to call OSV API instead:
   ```rust
   // Replace OSV file scanning with API call
   let osv_client = OsvClient::new();
   let osv_vulnerabilities = osv_client.query_batch(components).await?;
   ```

### Phase 3: Convert PURLs

Components need to be converted to proper Package URLs (PURLs) for OSV queries:

**Maven:** `pkg:maven/group.id/artifact-id@version`
**Gradle:** `pkg:maven/group:name@version`
**NPM:** `pkg:npm/package-name@version`
**PyPI:** `pkg:pypi/package-name@version`

### Phase 4: Response Processing

1. Parse vulnerability IDs from API response
2. Fetch full vulnerability details if needed (use `GET /v1/vulns/{id}`)
3. Match with EPSS scores and KEV entries
4. Calculate priority scores
5. Generate SARIF results

### Phase 5: Error Handling

Handle:
- Network failures (retry with exponential backoff)
- Rate limiting (respect HTTP 429 responses)
- Timeouts (configurable timeout for large batch queries)
- Partial failures (some queries succeed, others fail)

### Phase 6: Caching Strategy

**Optional optimization:**
- Cache API responses for a short duration (e.g., 1 hour)
- Cache key: hash of (package name, version, ecosystem)
- Store in `.bazbom/cache/osv_queries/`
- Respect cache-control headers from API

## Benefits of This Approach

1. ✅ **No massive database download** - only query what you need
2. ✅ **Always fresh data** - real-time vulnerability information
3. ✅ **Efficient** - batch queries reduce API calls
4. ✅ **Scalable** - works for any project size
5. ✅ **Maintainable** - no database sync logic to maintain

## Current Status

**As of 2025-11-17:**
- ❌ OSV API client not implemented
- ❌ SCA analyzer still expects local OSV files
- ⚠️ Local OSV scanning prints warning message directing to API
- ⚠️ Debug logging added to highlight the issue

**Next Steps:**
1. Implement `OsvClient` in `bazbom-vulnerabilities` crate
2. Add `reqwest` dependency with `json` feature
3. Update SCA analyzer to use API client
4. Add tests with mocked API responses
5. Document rate limiting and best practices

## Testing

**Mock API responses** for unit tests:
```rust
#[cfg(test)]
mod tests {
    use mockito::mock;

    #[tokio::test]
    async fn test_osv_batch_query() {
        let _m = mock("POST", "/v1/querybatch")
            .with_status(200)
            .with_body(r#"{"results": [...]}"#)
            .create();

        // Test API client...
    }
}
```

## Dependencies

Add to `crates/bazbom-vulnerabilities/Cargo.toml`:
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## References

- [OSV API Documentation](https://google.github.io/osv.dev/)
- [OSV Batch Query Endpoint](https://google.github.io/osv.dev/post-v1-querybatch/)
- [Package URL (PURL) Specification](https://github.com/package-url/purl-spec)
- [OSV Schema](https://ossf.github.io/osv-schema/)
