#!/usr/bin/env python3
"""Comprehensive tests for vulnerability enrichment modules."""

import json
import os
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch, MagicMock
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import requests for exception testing
try:
    import requests
except ImportError:
    requests = None

from kev_enrichment import KEVEnricher
from epss_enrichment import EPSSEnricher
from ghsa_enrichment import GHSAEnricher
from vulncheck_enrichment import VulnCheckEnricher
from vulnerability_enrichment import VulnerabilityEnricher


class TestKEVEnricher(unittest.TestCase):
    """Test cases for KEV enrichment."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = KEVEnricher(cache_dir=self.temp_dir)

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)

    @patch('kev_enrichment.requests.get')
    def test_fetch_kev_catalog_success(self, mock_get):
        """Test successful KEV catalog fetch."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "catalogVersion": "2025.01.17",
            "vulnerabilities": [
                {
                    "cveID": "CVE-2021-44228",
                    "vendorProject": "Apache",
                    "product": "Log4j",
                    "vulnerabilityName": "Log4Shell",
                    "dateAdded": "2021-12-10",
                    "shortDescription": "Remote code execution",
                    "requiredAction": "Apply updates",
                    "dueDate": "2021-12-24"
                }
            ]
        }
        mock_get.return_value = mock_response

        catalog = self.enricher.fetch_kev_catalog()
        
        self.assertIn("vulnerabilities", catalog)
        self.assertEqual(len(catalog["vulnerabilities"]), 1)
        self.assertEqual(catalog["vulnerabilities"][0]["cveID"], "CVE-2021-44228")

    @patch('kev_enrichment.requests.get')
    def test_is_known_exploited_in_kev(self, mock_get):
        """Test detection of CVE in KEV catalog."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "vulnerabilities": [
                {
                    "cveID": "CVE-2021-44228",
                    "vulnerabilityName": "Log4Shell",
                    "vendorProject": "Apache",
                    "product": "Log4j",
                    "dateAdded": "2021-12-10",
                    "dueDate": "2021-12-24",
                    "requiredAction": "Apply updates",
                    "shortDescription": "Remote code execution"
                }
            ]
        }
        mock_get.return_value = mock_response

        result = self.enricher.is_known_exploited("CVE-2021-44228")
        
        self.assertTrue(result["in_kev"])
        self.assertEqual(result["vulnerability_name"], "Log4Shell")
        self.assertEqual(result["date_added"], "2021-12-10")

    @patch('kev_enrichment.requests.get')
    def test_is_known_exploited_not_in_kev(self, mock_get):
        """Test CVE not in KEV catalog."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"vulnerabilities": []}
        mock_get.return_value = mock_response

        result = self.enricher.is_known_exploited("CVE-2024-99999")
        
        self.assertFalse(result["in_kev"])

    def test_is_known_exploited_empty_cve(self):
        """Test error handling for empty CVE ID."""
        with self.assertRaises(ValueError):
            self.enricher.is_known_exploited("")

    def test_is_known_exploited_invalid_type(self):
        """Test error handling for invalid CVE ID type."""
        with self.assertRaises(TypeError):
            self.enricher.is_known_exploited(123)

    @patch('kev_enrichment.requests.get')
    def test_enrich_finding_with_kev(self, mock_get):
        """Test enriching a finding with KEV data."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "vulnerabilities": [
                {
                    "cveID": "CVE-2021-44228",
                    "vulnerabilityName": "Log4Shell",
                    "dateAdded": "2021-12-10",
                    "dueDate": "2021-12-24",
                    "requiredAction": "Apply updates",
                    "vendorProject": "Apache",
                    "product": "Log4j",
                    "shortDescription": "Remote code execution"
                }
            ]
        }
        mock_get.return_value = mock_response

        finding = {
            "cve": "CVE-2021-44228",
            "severity": "HIGH"
        }

        enriched = self.enricher.enrich_finding(finding)
        
        self.assertTrue(enriched["kev"]["in_kev"])
        self.assertEqual(enriched["effective_severity"], "CRITICAL")
        self.assertEqual(enriched["priority"], "IMMEDIATE")
        self.assertIn("ACTIVELY EXPLOITED", enriched["kev_context"])

    @patch('kev_enrichment.requests.get')
    def test_enrich_finding_without_kev(self, mock_get):
        """Test enriching a finding not in KEV."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"vulnerabilities": []}
        mock_get.return_value = mock_response

        finding = {
            "cve": "CVE-2024-99999",
            "severity": "MEDIUM"
        }

        enriched = self.enricher.enrich_finding(finding)
        
        self.assertFalse(enriched["kev"]["in_kev"])
        self.assertNotIn("effective_severity", enriched)

    def test_enrich_finding_no_cve(self):
        """Test enriching finding without CVE ID."""
        finding = {"severity": "LOW"}
        
        enriched = self.enricher.enrich_finding(finding)
        
        self.assertIn("kev", enriched)
        self.assertFalse(enriched["kev"]["in_kev"])

    def test_enrich_finding_invalid_type(self):
        """Test error handling for invalid finding type."""
        with self.assertRaises(TypeError):
            self.enricher.enrich_finding("not a dict")


class TestEPSSEnricher(unittest.TestCase):
    """Test cases for EPSS enrichment."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = EPSSEnricher(cache_dir=self.temp_dir)

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)

    @patch('epss_enrichment.requests.get')
    def test_fetch_epss_scores_success(self, mock_get):
        """Test successful EPSS score fetch."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [
                {
                    "cve": "CVE-2021-44228",
                    "epss": "0.97538",
                    "percentile": "0.99999",
                    "date": "2025-01-17"
                }
            ]
        }
        mock_get.return_value = mock_response

        scores = self.enricher.fetch_epss_scores(["CVE-2021-44228"])
        
        self.assertIn("CVE-2021-44228", scores)
        self.assertAlmostEqual(scores["CVE-2021-44228"]["epss_score"], 0.97538, places=5)
        self.assertAlmostEqual(scores["CVE-2021-44228"]["epss_percentile"], 0.99999, places=5)

    def test_fetch_epss_scores_empty_list(self):
        """Test fetching EPSS scores with empty list."""
        scores = self.enricher.fetch_epss_scores([])
        self.assertEqual(scores, {})

    def test_fetch_epss_scores_invalid_type(self):
        """Test error handling for invalid CVE list type."""
        with self.assertRaises(TypeError):
            self.enricher.fetch_epss_scores("not a list")

    def test_fetch_epss_scores_invalid_cve_format(self):
        """Test error handling for invalid CVE format."""
        with self.assertRaises(ValueError):
            self.enricher.fetch_epss_scores(["INVALID-123"])

    def test_get_priority_level(self):
        """Test EPSS score to priority mapping."""
        self.assertEqual(self.enricher.get_priority_level(0.9), "CRITICAL")
        self.assertEqual(self.enricher.get_priority_level(0.6), "HIGH")
        self.assertEqual(self.enricher.get_priority_level(0.3), "MEDIUM")
        self.assertEqual(self.enricher.get_priority_level(0.1), "LOW")

    def test_get_priority_level_boundaries(self):
        """Test boundary conditions for priority levels."""
        self.assertEqual(self.enricher.get_priority_level(0.75), "CRITICAL")
        self.assertEqual(self.enricher.get_priority_level(0.50), "HIGH")
        self.assertEqual(self.enricher.get_priority_level(0.25), "MEDIUM")
        self.assertEqual(self.enricher.get_priority_level(0.0), "LOW")

    def test_get_priority_level_invalid_type(self):
        """Test error handling for invalid score type."""
        with self.assertRaises(TypeError):
            self.enricher.get_priority_level("0.5")

    def test_get_priority_level_out_of_range(self):
        """Test error handling for out of range score."""
        with self.assertRaises(ValueError):
            self.enricher.get_priority_level(1.5)
        with self.assertRaises(ValueError):
            self.enricher.get_priority_level(-0.1)

    @patch('epss_enrichment.requests.get')
    def test_enrich_findings(self, mock_get):
        """Test enriching multiple findings with EPSS."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [
                {
                    "cve": "CVE-2021-44228",
                    "epss": "0.97538",
                    "percentile": "0.99999",
                    "date": "2025-01-17"
                }
            ]
        }
        mock_get.return_value = mock_response

        findings = [
            {"cve": "CVE-2021-44228", "severity": "HIGH"}
        ]

        enriched = self.enricher.enrich_findings(findings)
        
        self.assertEqual(len(enriched), 1)
        self.assertIn("epss", enriched[0])
        self.assertEqual(enriched[0]["exploitation_probability"], "97.5%")
        self.assertEqual(enriched[0]["epss_priority"], "CRITICAL")

    def test_enrich_findings_empty_list(self):
        """Test enriching empty findings list."""
        enriched = self.enricher.enrich_findings([])
        self.assertEqual(enriched, [])

    def test_enrich_findings_invalid_type(self):
        """Test error handling for invalid findings type."""
        with self.assertRaises(TypeError):
            self.enricher.enrich_findings("not a list")


class TestGHSAEnricher(unittest.TestCase):
    """Test cases for GHSA enrichment."""

    def setUp(self):
        """Set up test fixtures."""
        self.enricher = GHSAEnricher()

    @patch('ghsa_enrichment.requests.post')
    def test_query_advisory_success(self, mock_post):
        """Test successful GHSA query."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": {
                "securityAdvisories": {
                    "nodes": [
                        {
                            "ghsaId": "GHSA-jfh8-c2jp-5v3q",
                            "summary": "Remote code execution via JNDI",
                            "description": "Detailed description",
                            "severity": "CRITICAL",
                            "publishedAt": "2021-12-10",
                            "updatedAt": "2021-12-15",
                            "withdrawnAt": None,
                            "permalink": "https://github.com/advisories/GHSA-jfh8-c2jp-5v3q",
                            "vulnerabilities": {
                                "nodes": [
                                    {
                                        "package": {
                                            "name": "log4j-core",
                                            "ecosystem": "MAVEN"
                                        },
                                        "vulnerableVersionRange": ">=2.0.0, <2.15.0",
                                        "firstPatchedVersion": {
                                            "identifier": "2.15.0"
                                        }
                                    }
                                ]
                            },
                            "references": [
                                {"url": "https://nvd.nist.gov/vuln/detail/CVE-2021-44228"}
                            ]
                        }
                    ]
                }
            }
        }
        mock_post.return_value = mock_response

        result = self.enricher.query_advisory("CVE-2021-44228")
        
        self.assertEqual(result["ghsa_id"], "GHSA-jfh8-c2jp-5v3q")
        self.assertEqual(result["severity"], "CRITICAL")
        self.assertEqual(len(result["vulnerabilities"]), 1)

    @patch('ghsa_enrichment.requests.post')
    def test_query_advisory_not_found(self, mock_post):
        """Test GHSA query with no results."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": {
                "securityAdvisories": {
                    "nodes": []
                }
            }
        }
        mock_post.return_value = mock_response

        result = self.enricher.query_advisory("CVE-2024-99999")
        
        self.assertEqual(result["ghsa_id"], "")
        self.assertEqual(result["summary"], "")

    def test_query_advisory_empty_cve(self):
        """Test error handling for empty CVE."""
        with self.assertRaises(ValueError):
            self.enricher.query_advisory("")

    def test_query_advisory_invalid_type(self):
        """Test error handling for invalid CVE type."""
        with self.assertRaises(TypeError):
            self.enricher.query_advisory(123)

    def test_query_advisory_invalid_format(self):
        """Test error handling for invalid CVE format."""
        with self.assertRaises(ValueError):
            self.enricher.query_advisory("INVALID-123")


class TestVulnCheckEnricher(unittest.TestCase):
    """Test cases for VulnCheck enrichment."""

    def setUp(self):
        """Set up test fixtures."""
        self.enricher = VulnCheckEnricher()

    def test_get_exploit_status_no_api_key(self):
        """Test VulnCheck without API key."""
        result = self.enricher.get_exploit_status("CVE-2021-44228")
        
        self.assertFalse(result["exploit_available"])
        self.assertIn("note", result)

    @patch('vulncheck_enrichment.requests.get')
    def test_get_exploit_status_with_api_key(self, mock_get):
        """Test VulnCheck with API key."""
        self.enricher.api_key = "test-key"
        
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [
                {
                    "exploit_available": True,
                    "exploit_maturity": "functional",
                    "weaponized": True,
                    "attack_vector": "network",
                    "exploit_type": "remote",
                    "date_added": "2021-12-10",
                    "due_date": "2021-12-24"
                }
            ]
        }
        mock_get.return_value = mock_response

        result = self.enricher.get_exploit_status("CVE-2021-44228")
        
        self.assertTrue(result["exploit_available"])
        self.assertTrue(result["weaponized"])
        self.assertEqual(result["exploit_maturity"], "functional")

    def test_get_exploit_status_empty_cve(self):
        """Test error handling for empty CVE."""
        with self.assertRaises(ValueError):
            self.enricher.get_exploit_status("")


class TestVulnerabilityEnricher(unittest.TestCase):
    """Test cases for master enrichment pipeline."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = VulnerabilityEnricher(
            kev_cache_dir=self.temp_dir,
            epss_cache_dir=self.temp_dir,
            enable_vulncheck=False,  # Disable to avoid API calls
            enable_ghsa=False
        )

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)

    def test_calculate_risk_score_high(self):
        """Test risk score calculation for high-risk vulnerability."""
        finding = {
            "cvss_score": 10.0,
            "epss": {"epss_score": 0.95},
            "kev": {"in_kev": True},
            "exploit": {"weaponized": True}
        }

        score = self.enricher._calculate_risk_score(finding)
        
        # Should be near maximum: 40 + 28.5 + 20 + 10 = 98.5
        self.assertGreater(score, 95)
        self.assertLessEqual(score, 100)

    def test_calculate_risk_score_low(self):
        """Test risk score calculation for low-risk vulnerability."""
        finding = {
            "cvss_score": 3.0,
            "epss": {"epss_score": 0.01},
            "kev": {"in_kev": False},
            "exploit": {"exploit_available": False}
        }

        score = self.enricher._calculate_risk_score(finding)
        
        # Should be low: 12 + 0.3 + 0 + 0 = 12.3
        self.assertLess(score, 20)

    def test_calculate_priority_p0(self):
        """Test P0 priority for KEV vulnerabilities."""
        finding = {
            "risk_score": 50.0,
            "kev": {"in_kev": True}
        }

        priority = self.enricher._calculate_priority(finding)
        
        self.assertEqual(priority, "P0-IMMEDIATE")

    def test_calculate_priority_p1(self):
        """Test P1 priority for high risk score."""
        finding = {
            "risk_score": 85.0,
            "kev": {"in_kev": False}
        }

        priority = self.enricher._calculate_priority(finding)
        
        self.assertEqual(priority, "P1-CRITICAL")

    def test_calculate_priority_p2(self):
        """Test P2 priority for medium-high risk score."""
        finding = {
            "risk_score": 65.0,
            "kev": {"in_kev": False}
        }

        priority = self.enricher._calculate_priority(finding)
        
        self.assertEqual(priority, "P2-HIGH")

    def test_calculate_priority_p3(self):
        """Test P3 priority for medium risk score."""
        finding = {
            "risk_score": 45.0,
            "kev": {"in_kev": False}
        }

        priority = self.enricher._calculate_priority(finding)
        
        self.assertEqual(priority, "P3-MEDIUM")

    def test_calculate_priority_p4(self):
        """Test P4 priority for low risk score."""
        finding = {
            "risk_score": 15.0,
            "kev": {"in_kev": False}
        }

        priority = self.enricher._calculate_priority(finding)
        
        self.assertEqual(priority, "P4-LOW")

    def test_get_priority_summary(self):
        """Test priority summary calculation."""
        findings = [
            {"priority": "P0-IMMEDIATE"},
            {"priority": "P0-IMMEDIATE"},
            {"priority": "P1-CRITICAL"},
            {"priority": "P2-HIGH"},
            {"priority": "P3-MEDIUM"},
            {"priority": "P4-LOW"}
        ]

        summary = self.enricher.get_priority_summary(findings)
        
        self.assertEqual(summary["P0-IMMEDIATE"], 2)
        self.assertEqual(summary["P1-CRITICAL"], 1)
        self.assertEqual(summary["P2-HIGH"], 1)
        self.assertEqual(summary["P3-MEDIUM"], 1)
        self.assertEqual(summary["P4-LOW"], 1)

    def test_enrich_all_empty_list(self):
        """Test enriching empty findings list."""
        enriched = self.enricher.enrich_all([])
        self.assertEqual(enriched, [])

    def test_enrich_all_invalid_type(self):
        """Test error handling for invalid findings type."""
        with self.assertRaises(TypeError):
            self.enricher.enrich_all("not a list")


class TestKEVEnricherAdvanced(unittest.TestCase):
    """Advanced test cases for KEV enrichment."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = KEVEnricher(cache_dir=self.temp_dir)

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)

    @patch('kev_enrichment.requests.get')
    def test_fetch_kev_catalog_with_cache(self, mock_get):
        """Test KEV catalog fetch uses cache when fresh."""
        # First fetch (should call API)
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "vulnerabilities": [{"cveID": "CVE-2021-44228"}]
        }
        mock_get.return_value = mock_response
        
        catalog1 = self.enricher.fetch_kev_catalog()
        self.assertEqual(mock_get.call_count, 1)
        
        # Second fetch (should use cache, no new API call)
        catalog2 = self.enricher.fetch_kev_catalog()
        self.assertEqual(mock_get.call_count, 1)  # Still 1, no new call
        self.assertEqual(catalog1, catalog2)

    @patch('kev_enrichment.requests.get')
    def test_fetch_kev_catalog_network_error_with_stale_cache(self, mock_get):
        """Test KEV fetch falls back to stale cache on network error."""
        # Create stale cache
        cache_file = Path(self.temp_dir) / "kev_catalog.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        with open(cache_file, 'w') as f:
            json.dump({"vulnerabilities": [{"cveID": "CVE-OLD"}]}, f)
        
        # Make the cache file old (set mtime to 48 hours ago)
        import time
        old_time = time.time() - (48 * 3600)
        os.utime(cache_file, (old_time, old_time))
        
        # Network error
        mock_get.side_effect = requests.RequestException("Network error")
        
        # Should return stale cache
        catalog = self.enricher.fetch_kev_catalog()
        self.assertEqual(catalog["vulnerabilities"][0]["cveID"], "CVE-OLD")

    @patch('kev_enrichment.requests.get')
    def test_fetch_kev_catalog_invalid_json(self, mock_get):
        """Test KEV fetch handles invalid JSON response."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.side_effect = json.JSONDecodeError("Invalid JSON", "", 0)
        mock_get.return_value = mock_response
        
        with self.assertRaises(json.JSONDecodeError):
            self.enricher.fetch_kev_catalog()

    @patch('kev_enrichment.requests.get')
    def test_fetch_kev_catalog_missing_vulnerabilities_field(self, mock_get):
        """Test KEV fetch validates response structure."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"invalid": "structure"}
        mock_get.return_value = mock_response
        
        with self.assertRaises(ValueError) as ctx:
            self.enricher.fetch_kev_catalog()
        self.assertIn("vulnerabilities", str(ctx.exception))


class TestEPSSEnricherAdvanced(unittest.TestCase):
    """Advanced test cases for EPSS enrichment."""

    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = EPSSEnricher(cache_dir=self.temp_dir)

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)

    @patch('epss_enrichment.requests.get')
    def test_fetch_epss_with_cache(self, mock_get):
        """Test EPSS fetch uses cache for previously fetched CVEs."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [{
                "cve": "CVE-2021-44228",
                "epss": "0.97538",
                "percentile": "0.99999",
                "date": "2025-01-17"
            }]
        }
        mock_get.return_value = mock_response
        
        # First fetch
        scores1 = self.enricher.fetch_epss_scores(["CVE-2021-44228"])
        self.assertEqual(mock_get.call_count, 1)
        
        # Second fetch (should use cache)
        scores2 = self.enricher.fetch_epss_scores(["CVE-2021-44228"])
        self.assertEqual(mock_get.call_count, 1)  # No new API call
        self.assertEqual(scores1, scores2)

    @patch('epss_enrichment.requests.get')
    def test_fetch_epss_network_error(self, mock_get):
        """Test EPSS fetch handles network errors gracefully."""
        mock_get.side_effect = requests.RequestException("Network error")
        
        # Should return empty scores with error info
        scores = self.enricher.fetch_epss_scores(["CVE-2021-44228"])
        self.assertIn("CVE-2021-44228", scores)
        self.assertEqual(scores["CVE-2021-44228"]["epss_score"], 0.0)
        self.assertIn("error", scores["CVE-2021-44228"])

    @patch('epss_enrichment.requests.get')
    def test_fetch_epss_invalid_score(self, mock_get):
        """Test EPSS fetch handles invalid score values."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [{
                "cve": "CVE-2021-44228",
                "epss": "invalid",
                "percentile": "not_a_number",
                "date": "2025-01-17"
            }]
        }
        mock_get.return_value = mock_response
        
        scores = self.enricher.fetch_epss_scores(["CVE-2021-44228"])
        # Should handle gracefully and not include invalid CVE
        self.assertTrue(len(scores) == 0 or scores["CVE-2021-44228"]["epss_score"] == 0.0)

    @patch('epss_enrichment.requests.get')
    def test_enrich_findings_with_api_error(self, mock_get):
        """Test enrich_findings continues with error."""
        mock_get.side_effect = Exception("API error")
        
        findings = [{"cve": "CVE-2021-44228", "cvss_score": 10.0}]
        enriched = self.enricher.enrich_findings(findings)
        
        # Should return findings without EPSS data
        self.assertEqual(len(enriched), 1)
        self.assertNotIn("epss", enriched[0])

    def test_enrich_finding_single(self):
        """Test enrich_finding method for single finding."""
        self.enricher._cache = {
            "CVE-2021-44228": {
                "epss_score": 0.97,
                "epss_percentile": 0.99,
                "date": "2025-01-17"
            }
        }
        self.enricher._cache_loaded = True
        
        finding = {"cve": "CVE-2021-44228", "cvss_score": 10.0}
        enriched = self.enricher.enrich_finding(finding)
        
        self.assertIn("epss", enriched)
        self.assertEqual(enriched["epss"]["epss_score"], 0.97)


class TestGHSAEnricherAdvanced(unittest.TestCase):
    """Advanced test cases for GHSA enrichment."""

    def setUp(self):
        """Set up test fixtures."""
        self.enricher = GHSAEnricher()

    @patch('ghsa_enrichment.requests.post')
    def test_query_advisory_network_error(self, mock_post):
        """Test GHSA query handles network errors."""
        mock_post.side_effect = requests.RequestException("Network error")
        
        result = self.enricher.query_advisory("CVE-2021-44228")
        # Should return empty result with error field
        self.assertEqual(result["ghsa_id"], "")
        self.assertEqual(result["summary"], "")
        self.assertIn("error", result)

    @patch('ghsa_enrichment.requests.post')
    def test_query_advisory_invalid_response(self, mock_post):
        """Test GHSA query handles malformed responses."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"invalid": "structure"}
        mock_post.return_value = mock_response
        
        result = self.enricher.query_advisory("CVE-2021-44228")
        # Should return empty result when no advisory found
        self.assertEqual(result["ghsa_id"], "")
        self.assertEqual(result["summary"], "")
        self.assertEqual(result["vulnerabilities"], [])


class TestVulnCheckEnricherAdvanced(unittest.TestCase):
    """Advanced test cases for VulnCheck enrichment."""

    def setUp(self):
        """Set up test fixtures."""
        self.enricher = VulnCheckEnricher(api_key="test_key")

    @patch('vulncheck_enrichment.requests.get')
    def test_get_exploit_status_success(self, mock_get):
        """Test VulnCheck exploit status fetch success."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [{
                "exploit_available": True,
                "exploit_maturity": "functional",
                "weaponized": True,
                "attack_vector": "network"
            }]
        }
        mock_get.return_value = mock_response
        
        result = self.enricher.get_exploit_status("CVE-2021-44228")
        
        self.assertTrue(result["exploit_available"])
        self.assertTrue(result["weaponized"])

    @patch('vulncheck_enrichment.requests.get')
    def test_get_exploit_status_network_error(self, mock_get):
        """Test VulnCheck handles network errors."""
        mock_get.side_effect = requests.RequestException("Network error")
        
        result = self.enricher.get_exploit_status("CVE-2021-44228")
        self.assertEqual(result["exploit_available"], False)
        self.assertIn("error", result)


class TestVulnerabilityEnricherAdvanced(unittest.TestCase):
    """Advanced test cases for vulnerability enrichment pipeline."""

    def setUp(self):
        """Set up test fixtures."""
        self.enricher = VulnerabilityEnricher(
            enable_vulncheck=False,
            enable_ghsa=False
        )

    @patch('vulnerability_enrichment.EPSSEnricher.enrich_findings')
    @patch('vulnerability_enrichment.KEVEnricher.enrich_finding')
    def test_enrich_all_with_findings(self, mock_kev, mock_epss):
        """Test full enrichment pipeline with actual findings."""
        mock_epss.return_value = [
            {
                "cve": "CVE-2021-44228",
                "cvss_score": 10.0,
                "epss": {"epss_score": 0.97}
            }
        ]
        
        mock_kev.return_value = {
            "cve": "CVE-2021-44228",
            "cvss_score": 10.0,
            "epss": {"epss_score": 0.97},
            "kev": {"in_kev": True}
        }
        
        findings = [{"cve": "CVE-2021-44228", "cvss_score": 10.0}]
        enriched = self.enricher.enrich_all(findings)
        
        self.assertEqual(len(enriched), 1)
        self.assertIn("risk_score", enriched[0])
        self.assertIn("priority", enriched[0])

    def test_calculate_risk_score_all_components(self):
        """Test risk score calculation with all components."""
        finding = {
            "cvss_score": 10.0,
            "epss": {"epss_score": 1.0},
            "kev": {"in_kev": True},
            "exploit": {"weaponized": True}
        }
        
        score = self.enricher._calculate_risk_score(finding)
        
        # Max score: (10/10 * 40) + (1.0 * 30) + 20 + 10 = 100
        self.assertEqual(score, 100.0)

    def test_calculate_risk_score_no_exploit(self):
        """Test risk score without exploit data."""
        finding = {
            "cvss_score": 5.0,
            "epss": {"epss_score": 0.5},
            "kev": {"in_kev": False},
            "exploit": {"weaponized": False}
        }
        
        score = self.enricher._calculate_risk_score(finding)
        
        # (5/10 * 40) + (0.5 * 30) + 0 + 0 = 20 + 15 = 35
        self.assertEqual(score, 35.0)

    def test_get_priority_summary_with_non_dict(self):
        """Test priority summary handles non-dict items."""
        findings = [
            {"priority": "P0-IMMEDIATE"},
            "invalid",
            {"priority": "P1-CRITICAL"},
            None
        ]
        
        summary = self.enricher.get_priority_summary(findings)
        
        self.assertEqual(summary["P0-IMMEDIATE"], 1)
        self.assertEqual(summary["P1-CRITICAL"], 1)


if __name__ == "__main__":
    unittest.main()
