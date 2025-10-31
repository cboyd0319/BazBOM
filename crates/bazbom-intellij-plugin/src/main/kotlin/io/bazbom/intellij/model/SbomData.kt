package io.bazbom.intellij.model

import com.fasterxml.jackson.annotation.JsonIgnoreProperties
import com.fasterxml.jackson.annotation.JsonProperty

/**
 * Data models for BazBOM SBOM JSON output.
 * These map to the JSON structure produced by `bazbom scan --format json`.
 */

@JsonIgnoreProperties(ignoreUnknown = true)
data class SbomDocument(
    @JsonProperty("bomFormat") val bomFormat: String = "SPDX",
    @JsonProperty("specVersion") val specVersion: String = "SPDX-2.3",
    @JsonProperty("packages") val packages: List<Package> = emptyList(),
    @JsonProperty("vulnerabilities") val vulnerabilities: List<Vulnerability> = emptyList()
)

@JsonIgnoreProperties(ignoreUnknown = true)
data class Package(
    @JsonProperty("SPDXID") val spdxId: String = "",
    @JsonProperty("name") val name: String = "",
    @JsonProperty("versionInfo") val version: String = "",
    @JsonProperty("externalRefs") val externalRefs: List<ExternalRef> = emptyList(),
    @JsonProperty("scope") val scope: String? = null
)

@JsonIgnoreProperties(ignoreUnknown = true)
data class ExternalRef(
    @JsonProperty("referenceType") val referenceType: String = "",
    @JsonProperty("referenceLocator") val referenceLocator: String = ""
)

@JsonIgnoreProperties(ignoreUnknown = true)
data class Vulnerability(
    @JsonProperty("id") val id: String = "",
    @JsonProperty("affects") val affects: List<AffectedPackage> = emptyList(),
    @JsonProperty("severity") val severity: String? = null,
    @JsonProperty("cvssScore") val cvssScore: Double? = null,
    @JsonProperty("summary") val summary: String? = null,
    @JsonProperty("cisaKev") val cisaKev: Boolean = false,
    @JsonProperty("epss") val epss: Double? = null,
    @JsonProperty("reachable") val reachable: Boolean = false,
    @JsonProperty("priority") val priority: String? = null,
    @JsonProperty("fixedVersion") val fixedVersion: String? = null
)

@JsonIgnoreProperties(ignoreUnknown = true)
data class AffectedPackage(
    @JsonProperty("ref") val ref: String = ""
)

/**
 * View model for dependency tree nodes.
 */
data class DependencyNode(
    val name: String,
    val version: String,
    val purl: String,
    val scope: String,
    val vulnerabilities: List<Vulnerability>,
    val children: MutableList<DependencyNode> = mutableListOf()
) {
    val securityStatus: SecurityStatus
        get() = when {
            vulnerabilities.any { it.severity == "CRITICAL" } -> SecurityStatus.CRITICAL
            vulnerabilities.any { it.severity == "HIGH" } -> SecurityStatus.HIGH
            vulnerabilities.any { it.severity == "MEDIUM" } -> SecurityStatus.MEDIUM
            vulnerabilities.any { it.severity == "LOW" } -> SecurityStatus.LOW
            else -> SecurityStatus.SAFE
        }
}

enum class SecurityStatus {
    SAFE,
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL
}
