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
                "epss": "invalid",  # Invalid float
                "percentile": "not_a_number",
                "date": "2025-01-17"
            }]
        }
        mock_get.return_value = mock_response
        
        scores = self.enricher.fetch_epss_scores(["CVE-2021-44228"])
        # Should handle gracefully by skipping invalid entry
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
    
    @patch('epss_enrichment.requests.get')
    def test_enrich_findings_with_non_dict_items(self, mock_get):
        """Test enrich_findings handles non-dict items gracefully."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"data": []}
        mock_get.return_value = mock_response
        
        findings = [
            {"cve": "CVE-2021-11111"},
            "not a dict",  # This should trigger line 221-222
            None,
            {"cve": "CVE-2021-22222"}
        ]
        
        enriched = self.enricher.enrich_findings(findings)
        
        # Should still return all items, but only enrich dicts
        self.assertEqual(len(enriched), 4)
    
    @patch('epss_enrichment.requests.get')
    def test_enrich_findings_with_priority_calculation_error(self, mock_get):
        """Test enrich_findings handles priority calculation errors."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [{
                "cve": "CVE-2021-11111",
                "epss": "0.75",
                "percentile": "0.90",
                "date": "2025-01-17"
            }]
        }
        mock_get.return_value = mock_response
        
        # Mock get_priority_level to raise an error
        with patch.object(self.enricher, 'get_priority_level', side_effect=ValueError("Invalid score")):
            findings = [{"cve": "CVE-2021-11111"}]
            enriched = self.enricher.enrich_findings(findings)
            
            # Should have epss data but unknown priority
            self.assertIn("epss", enriched[0])
            self.assertEqual(enriched[0].get("epss_priority"), "UNKNOWN")
    
    @patch('epss_enrichment.requests.get')
    def test_cache_save_error(self, mock_get):
        """Test cache save error handling."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [{
                "cve": "CVE-2021-11111",
                "epss": "0.75",
                "percentile": "0.90"
            }]
        }
        mock_get.return_value = mock_response
        
        # Use invalid path to trigger save error
        enricher = EPSSEnricher(cache_dir="/invalid/path/that/cannot/be/created")
        
        # Should handle save error gracefully
        scores = enricher.fetch_epss_scores(["CVE-2021-11111"])
        self.assertIn("CVE-2021-11111", scores)
    
    def test_cache_load_with_corrupted_file(self):
        """Test cache loading with corrupted JSON file."""
        # Create corrupted cache file
        cache_file = Path(self.temp_dir) / "epss_cache.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        with open(cache_file, 'w') as f:
            f.write("{ invalid json")
        
        # Should handle error gracefully
        cache = self.enricher._load_cache()
        self.assertEqual(cache, {})


class TestEPSSCLI(unittest.TestCase):
    """Test EPSS CLI main function."""
    
    @patch('epss_enrichment.EPSSEnricher.fetch_epss_scores')
    @patch('epss_enrichment.EPSSEnricher.get_priority_level')
    def test_main_with_json_output(self, mock_priority, mock_fetch):
        """Test main function with JSON output."""
        mock_fetch.return_value = {
            "CVE-2021-11111": {
                "epss_score": 0.75,
                "epss_percentile": 0.90,
                "date": "2025-01-17"
            }
        }
        mock_priority.return_value = "CRITICAL"
        
        from epss_enrichment import main
        with patch('sys.argv', ['epss_enrichment.py', 'CVE-2021-11111', '--json']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('epss_enrichment.EPSSEnricher.fetch_epss_scores')
    @patch('epss_enrichment.EPSSEnricher.get_priority_level')
    def test_main_with_text_output(self, mock_priority, mock_fetch):
        """Test main function with text output."""
        mock_fetch.return_value = {
            "CVE-2021-11111": {
                "epss_score": 0.75,
                "epss_percentile": 0.90,
                "date": "2025-01-17"
            }
        }
        mock_priority.return_value = "CRITICAL"
        
        from epss_enrichment import main
        with patch('sys.argv', ['epss_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('epss_enrichment.EPSSEnricher.fetch_epss_scores')
    def test_main_with_cache_dir(self, mock_fetch):
        """Test main function with custom cache directory."""
        mock_fetch.return_value = {}
        
        from epss_enrichment import main
        with patch('sys.argv', ['epss_enrichment.py', 'CVE-2021-11111', '--cache-dir', '/tmp/test']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('epss_enrichment.EPSSEnricher.fetch_epss_scores')
    def test_main_with_error(self, mock_fetch):
        """Test main function handles errors."""
        mock_fetch.side_effect = Exception("Test error")
        
        from epss_enrichment import main
        with patch('sys.argv', ['epss_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 1)


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


class TestGHSAEnrichFinding(unittest.TestCase):
    """Test GHSA enrich_finding method."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.enricher = GHSAEnricher()
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_enrich_finding_with_remediation(self, mock_query):
        """Test enrich_finding adds remediation info."""
        mock_query.return_value = {
            "ghsa_id": "GHSA-test-1234",
            "summary": "Test vulnerability",
            "vulnerabilities": [{
                "first_patched_version": "2.0.0",
                "vulnerable_version_range": "< 2.0.0"
            }]
        }
        
        finding = {"cve": "CVE-2021-12345"}
        result = self.enricher.enrich_finding(finding)
        
        self.assertIn("ghsa", result)
        self.assertIn("remediation", result)
        self.assertEqual(result["remediation"]["fixed_version"], "2.0.0")
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_enrich_finding_exception_handling(self, mock_query):
        """Test enrich_finding handles exceptions gracefully."""
        mock_query.side_effect = Exception("Network error")
        
        finding = {"cve": "CVE-2021-12345"}
        result = self.enricher.enrich_finding(finding)
        
        self.assertIn("ghsa", result)
        self.assertEqual(result["ghsa"]["ghsa_id"], "")
    
    def test_enrich_finding_non_cve_id(self):
        """Test enrich_finding with non-CVE ID."""
        finding = {"cve": "INVALID-123"}
        result = self.enricher.enrich_finding(finding)
        
        self.assertIn("ghsa", result)
        self.assertEqual(result["ghsa"]["ghsa_id"], "")
    
    def test_enrich_finding_with_id_field(self):
        """Test enrich_finding uses 'id' field as fallback."""
        with patch.object(self.enricher, 'query_advisory') as mock_query:
            mock_query.return_value = {"ghsa_id": "GHSA-test"}
            finding = {"id": "CVE-2021-12345"}
            result = self.enricher.enrich_finding(finding)
            self.assertIn("ghsa", result)
    
    def test_enrich_finding_with_vulnerability_id_field(self):
        """Test enrich_finding uses nested 'vulnerability.id' field."""
        with patch.object(self.enricher, 'query_advisory') as mock_query:
            mock_query.return_value = {"ghsa_id": "GHSA-test"}
            finding = {"vulnerability": {"id": "CVE-2021-12345"}}
            result = self.enricher.enrich_finding(finding)
            self.assertIn("ghsa", result)
    
    def test_enrich_findings_list(self):
        """Test enrich_findings processes list of findings."""
        findings = [
            {"cve": "CVE-2021-12345"},
            {"cve": "CVE-2021-67890"}
        ]
        with patch.object(self.enricher, 'query_advisory') as mock_query:
            mock_query.return_value = {"ghsa_id": "GHSA-test"}
            result = self.enricher.enrich_findings(findings)
            self.assertEqual(len(result), 2)
    
    def test_enrich_findings_with_non_dict_items(self):
        """Test enrich_findings handles non-dict items."""
        findings = [
            {"cve": "CVE-2021-12345"},
            "not a dict",
            {"cve": "CVE-2021-67890"}
        ]
        with patch.object(self.enricher, 'query_advisory') as mock_query:
            mock_query.return_value = {"ghsa_id": "GHSA-test"}
            result = self.enricher.enrich_findings(findings)
            # Should not crash, just skip non-dict items
            self.assertEqual(len(result), 3)


class TestVulnCheckEnrichFinding(unittest.TestCase):
    """Test VulnCheck enrich_finding method."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.enricher = VulnCheckEnricher()
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_enrich_finding_with_weaponized_exploit(self, mock_get):
        """Test enrich_finding with weaponized exploit."""
        mock_get.return_value = {
            "exploit_available": True,
            "weaponized": True,
            "exploit_maturity": "functional"
        }
        
        finding = {"cve": "CVE-2021-44228"}
        result = self.enricher.enrich_finding(finding)
        
        self.assertIn("exploit", result)
        self.assertTrue(result["exploit"]["weaponized"])
        self.assertIn("priority", result)
        self.assertEqual(result["priority"], "P1-CRITICAL")
        self.assertIn("exploit_context", result)
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_enrich_finding_preserves_p0_priority(self, mock_get):
        """Test enrich_finding preserves P0-IMMEDIATE priority."""
        mock_get.return_value = {"exploit_available": True, "weaponized": True}
        
        finding = {"cve": "CVE-2021-44228", "priority": "P0-IMMEDIATE"}
        result = self.enricher.enrich_finding(finding)
        
        # Should not downgrade P0 to P1
        self.assertEqual(result["priority"], "P0-IMMEDIATE")
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_enrich_finding_exception_handling(self, mock_get):
        """Test enrich_finding handles exceptions."""
        mock_get.side_effect = Exception("API error")
        
        finding = {"cve": "CVE-2021-12345"}
        result = self.enricher.enrich_finding(finding)
        
        self.assertIn("exploit", result)
        self.assertFalse(result["exploit"]["exploit_available"])
    
    def test_enrich_finding_non_cve_id(self):
        """Test enrich_finding with non-CVE ID."""
        finding = {"cve": "INVALID-123"}
        result = self.enricher.enrich_finding(finding)
        
        self.assertIn("exploit", result)
        self.assertFalse(result["exploit"]["exploit_available"])


class TestEPSSCaching(unittest.TestCase):
    """Test EPSS caching behavior."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = EPSSEnricher(cache_dir=self.temp_dir)
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('epss_enrichment.requests.get')
    def test_cache_functionality(self, mock_get):
        """Test EPSS scores are cached properly."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [{
                "cve": "CVE-2021-11111",
                "epss": "0.75",
                "percentile": "0.90"
            }]
        }
        mock_get.return_value = mock_response
        
        # First call should hit API
        scores1 = self.enricher.fetch_epss_scores(["CVE-2021-11111"])
        self.assertEqual(mock_get.call_count, 1)
        
        # Second call should use cache
        scores2 = self.enricher.fetch_epss_scores(["CVE-2021-11111"])
        # Should still be 1 call (used cache)
        self.assertEqual(mock_get.call_count, 1)
        
        # Scores should match
        self.assertEqual(scores1, scores2)
    
    @patch('epss_enrichment.requests.get')
    def test_fetch_epss_invalid_response_structure(self, mock_get):
        """Test EPSS handles invalid response structure."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = "invalid"  # String instead of dict
        mock_get.return_value = mock_response
        
        with self.assertRaises(ValueError) as ctx:
            self.enricher.fetch_epss_scores(["CVE-2021-11111"])
        
        self.assertIn("Invalid EPSS API response", str(ctx.exception))
    
    @patch('epss_enrichment.requests.get')
    def test_fetch_epss_missing_cve_in_entry(self, mock_get):
        """Test EPSS handles entries without CVE field."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [
                {"epss": "0.75", "percentile": "0.90"},  # Missing 'cve' field
                {"cve": "CVE-2021-11111", "epss": "0.50", "percentile": "0.50"}
            ]
        }
        mock_get.return_value = mock_response
        
        scores = self.enricher.fetch_epss_scores(["CVE-2021-11111"])
        
        # Should skip entry without CVE and process valid one
        self.assertIn("CVE-2021-11111", scores)
        self.assertEqual(len(scores), 1)


class TestKEVCaching(unittest.TestCase):
    """Test KEV caching behavior."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = KEVEnricher(cache_dir=self.temp_dir)
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('kev_enrichment.requests.get')
    def test_cache_freshness_check(self, mock_get):
        """Test KEV catalog cache freshness."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "catalogVersion": "2025.01.17",
            "vulnerabilities": []
        }
        mock_get.return_value = mock_response
        
        # First fetch
        catalog1 = self.enricher.fetch_kev_catalog()
        call_count_first = mock_get.call_count
        
        # Immediate second fetch should use cache
        catalog2 = self.enricher.fetch_kev_catalog()
        self.assertEqual(mock_get.call_count, call_count_first)  # No new call

    @patch('kev_enrichment.requests.get')
    def test_fetch_kev_catalog_failure_no_cache(self, mock_get):
        """Test KEV catalog fetch failure with no cache raises RuntimeError."""
        import requests
        mock_get.side_effect = requests.RequestException("Network error")
        
        with self.assertRaises(RuntimeError) as context:
            self.enricher.fetch_kev_catalog()
        
        self.assertIn("Failed to fetch KEV catalog and no cache available", str(context.exception))


class TestResponseValidation(unittest.TestCase):
    """Test validation of API responses."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('epss_enrichment.requests.get')
    def test_epss_invalid_score_values(self, mock_get):
        """Test EPSS handles invalid score values."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [{
                "cve": "CVE-2021-11111",
                "epss": "invalid",  # Invalid float
                "percentile": "0.90"
            }]
        }
        mock_get.return_value = mock_response
        
        enricher = EPSSEnricher(cache_dir=self.temp_dir)
        
        # Should handle gracefully by skipping invalid entry
        try:
            scores = enricher.fetch_epss_scores(["CVE-2021-11111"])
            # If it doesn't raise, check it handled it somehow
            self.assertIsInstance(scores, dict)
        except (ValueError, TypeError):
            # Also acceptable to raise for invalid data
            pass


class TestEPSSEnrichFinding(unittest.TestCase):
    """Test EPSS enrich_finding method."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = EPSSEnricher(cache_dir=self.temp_dir)
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('epss_enrichment.requests.get')
    def test_enrich_finding_with_cve(self, mock_get):
        """Test enriching a single finding with CVE."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "data": [{
                "cve": "CVE-2021-44228",
                "epss": "0.95",
                "percentile": "0.99"
            }]
        }
        mock_get.return_value = mock_response
        
        finding = {"cve": "CVE-2021-44228"}
        result = self.enricher.enrich_finding(finding)
        
        self.assertIn("epss", result)
        self.assertAlmostEqual(result["epss"]["epss_score"], 0.95)
    
    def test_enrich_finding_no_cve(self):
        """Test enriching finding without CVE."""
        finding = {"id": "GHSA-1234"}
        result = self.enricher.enrich_finding(finding)
        
        # Should not crash
        self.assertIsInstance(result, dict)
    
    def test_enrich_finding_non_cve_format(self):
        """Test enriching finding with non-CVE format ID."""
        finding = {"cve": "INVALID-123"}
        result = self.enricher.enrich_finding(finding)
        
        # Should handle gracefully
        self.assertIsInstance(result, dict)


class TestVulnerabilityEnricherWithAllSources(unittest.TestCase):
    """Test enricher with VulnCheck and GHSA enabled."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = VulnerabilityEnricher(
            kev_cache_dir=self.temp_dir,
            epss_cache_dir=self.temp_dir,
            enable_vulncheck=True,  # Enable VulnCheck
            enable_ghsa=True  # Enable GHSA
        )
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('vulnerability_enrichment.GHSAEnricher.enrich_finding')
    @patch('vulnerability_enrichment.VulnCheckEnricher.enrich_finding')
    @patch('vulnerability_enrichment.KEVEnricher.enrich_finding')
    @patch('vulnerability_enrichment.EPSSEnricher.enrich_findings')
    def test_enrich_all_with_vulncheck_enabled(self, mock_epss, mock_kev, mock_vulncheck, mock_ghsa):
        """Test enrichment with VulnCheck enabled."""
        # Setup mocks to return findings
        mock_epss.return_value = [{"cve": "CVE-2021-11111", "cvss_score": 9.0}]
        mock_kev.return_value = {"cve": "CVE-2021-11111", "cvss_score": 9.0, "kev": {"in_kev": False}}
        mock_vulncheck.return_value = {
            "cve": "CVE-2021-11111",
            "cvss_score": 9.0,
            "kev": {"in_kev": False},
            "exploit": {"exploit_available": True, "weaponized": False}
        }
        mock_ghsa.return_value = {
            "cve": "CVE-2021-11111",
            "cvss_score": 9.0,
            "kev": {"in_kev": False},
            "exploit": {"exploit_available": True, "weaponized": False},
            "ghsa": {"ghsa_id": "GHSA-test"}
        }
        
        findings = [{"cve": "CVE-2021-11111", "cvss_score": 9.0}]
        enriched = self.enricher.enrich_all(findings)
        
        # Verify VulnCheck enrichment was called
        self.assertEqual(mock_vulncheck.call_count, 1)
        # Verify GHSA enrichment was called
        self.assertEqual(mock_ghsa.call_count, 1)
        self.assertEqual(len(enriched), 1)
    
    @patch('vulnerability_enrichment.GHSAEnricher.enrich_finding')
    @patch('vulnerability_enrichment.VulnCheckEnricher.enrich_finding')
    @patch('vulnerability_enrichment.KEVEnricher.enrich_finding')
    @patch('vulnerability_enrichment.EPSSEnricher.enrich_findings')
    def test_enrich_all_with_ghsa_enabled(self, mock_epss, mock_kev, mock_vulncheck, mock_ghsa):
        """Test enrichment with GHSA enabled."""
        mock_epss.return_value = [{"cve": "CVE-2021-11111"}]
        mock_kev.return_value = {"cve": "CVE-2021-11111", "kev": {"in_kev": False}}
        mock_vulncheck.return_value = {"cve": "CVE-2021-11111", "kev": {"in_kev": False}, "exploit": {}}
        mock_ghsa.return_value = {"cve": "CVE-2021-11111", "kev": {"in_kev": False}, "exploit": {}, "ghsa": {}}
        
        findings = [{"cve": "CVE-2021-11111"}]
        enriched = self.enricher.enrich_all(findings)
        
        # Verify GHSA was called
        self.assertEqual(mock_ghsa.call_count, 1)


class TestRiskScoreEdgeCases(unittest.TestCase):
    """Test risk score calculation edge cases."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.enricher = VulnerabilityEnricher(
            enable_vulncheck=False,
            enable_ghsa=False
        )
    
    def test_calculate_risk_score_with_non_weaponized_exploit(self):
        """Test risk score with non-weaponized but available exploit."""
        finding = {
            "cvss_score": 5.0,
            "epss": {"epss_score": 0.5},
            "kev": {"in_kev": False},
            "exploit": {"exploit_available": True, "weaponized": False}
        }
        
        score = self.enricher._calculate_risk_score(finding)
        
        # (5/10 * 40) + (0.5 * 30) + 0 + 5 = 20 + 15 + 0 + 5 = 40
        self.assertAlmostEqual(score, 40.0, places=1)
    
    def test_calculate_risk_score_missing_exploit_data(self):
        """Test risk score when exploit data is missing fields."""
        finding = {
            "cvss_score": 5.0,
            "epss": {"epss_score": 0.5},
            "kev": {"in_kev": False},
            "exploit": {}  # Empty exploit data
        }
        
        score = self.enricher._calculate_risk_score(finding)
        
        # Should handle gracefully: (5/10 * 40) + (0.5 * 30) = 20 + 15 = 35
        self.assertAlmostEqual(score, 35.0, places=1)


class TestCLIMain(unittest.TestCase):
    """Test the CLI main function."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.input_file = os.path.join(self.temp_dir, "input.json")
        self.output_file = os.path.join(self.temp_dir, "output.json")
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    @patch('sys.argv', ['vulnerability_enrichment.py', 'test.json'])
    def test_main_with_dict_input(self, mock_enrich):
        """Test main with dict input format."""
        # Create input file with dict format
        input_data = {
            "scan_date": "2025-01-17",
            "packages_scanned": 5,
            "vulnerabilities": [
                {"cve": "CVE-2021-11111", "cvss_score": 9.0}
            ]
        }
        with open(self.input_file, 'w') as f:
            json.dump(input_data, f)
        
        mock_enrich.return_value = [
            {"cve": "CVE-2021-11111", "cvss_score": 9.0, "priority": "P1-CRITICAL"}
        ]
        
        # Test main function
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file]):
            result = main()
        
        self.assertEqual(result, 0)
        self.assertEqual(mock_enrich.call_count, 1)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    @patch('kev_enrichment.requests.get')
    @patch('epss_enrichment.requests.get')
    def test_main_with_list_input(self, mock_epss_req, mock_kev_req, mock_enrich):
        """Test main with list input format."""
        # Mock network requests
        mock_kev_response = Mock()
        mock_kev_response.status_code = 200
        mock_kev_response.json.return_value = {"vulnerabilities": []}
        mock_kev_req.return_value = mock_kev_response
        
        mock_epss_response = Mock()
        mock_epss_response.status_code = 200
        mock_epss_response.json.return_value = {"data": []}
        mock_epss_req.return_value = mock_epss_response
        
        input_data = [
            {"cve": "CVE-2021-11111", "cvss_score": 9.0}
        ]
        with open(self.input_file, 'w') as f:
            json.dump(input_data, f)
        
        mock_enrich.return_value = [
            {"cve": "CVE-2021-11111", "priority": "P1-CRITICAL"}
        ]
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file]):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    @patch('vulnerability_enrichment.VulnerabilityEnricher.get_priority_summary')
    def test_main_with_summary_flag(self, mock_summary, mock_enrich):
        """Test main with --summary flag."""
        input_data = {"vulnerabilities": [{"cve": "CVE-2021-11111"}]}
        with open(self.input_file, 'w') as f:
            json.dump(input_data, f)
        
        mock_enrich.return_value = [{"cve": "CVE-2021-11111", "priority": "P1-CRITICAL"}]
        mock_summary.return_value = {
            "P0-IMMEDIATE": 0,
            "P1-CRITICAL": 1,
            "P2-HIGH": 0,
            "P3-MEDIUM": 0,
            "P4-LOW": 0
        }
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file, '--summary']):
            result = main()
        
        self.assertEqual(result, 0)
        self.assertEqual(mock_summary.call_count, 1)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    def test_main_with_output_file(self, mock_enrich):
        """Test main with output file."""
        input_data = {"vulnerabilities": [{"cve": "CVE-2021-11111"}]}
        with open(self.input_file, 'w') as f:
            json.dump(input_data, f)
        
        mock_enrich.return_value = [{"cve": "CVE-2021-11111", "priority": "P1-CRITICAL"}]
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file, '--output', self.output_file]):
            result = main()
        
        self.assertEqual(result, 0)
        self.assertTrue(os.path.exists(self.output_file))
        
        # Verify output file content
        with open(self.output_file) as f:
            output_data = json.load(f)
        self.assertIn("vulnerabilities", output_data)
    
    def test_main_file_not_found(self):
        """Test main with non-existent file."""
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', 'nonexistent.json']):
            result = main()
        
        self.assertEqual(result, 1)
    
    def test_main_invalid_json(self):
        """Test main with invalid JSON."""
        with open(self.input_file, 'w') as f:
            f.write("invalid json{")
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file]):
            result = main()
        
        self.assertEqual(result, 1)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    def test_main_invalid_input_format(self, mock_enrich):
        """Test main with invalid input format (not dict or list)."""
        with open(self.input_file, 'w') as f:
            json.dump("string_input", f)
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file]):
            result = main()
        
        self.assertEqual(result, 1)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    def test_main_with_github_token(self, mock_enrich):
        """Test main with GitHub token argument."""
        input_data = {"vulnerabilities": [{"cve": "CVE-2021-11111"}]}
        with open(self.input_file, 'w') as f:
            json.dump(input_data, f)
        
        mock_enrich.return_value = []
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file, '--github-token', 'test-token']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    def test_main_with_vulncheck_api_key(self, mock_enrich):
        """Test main with VulnCheck API key."""
        input_data = {"vulnerabilities": [{"cve": "CVE-2021-11111"}]}
        with open(self.input_file, 'w') as f:
            json.dump(input_data, f)
        
        mock_enrich.return_value = []
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file, '--vulncheck-api-key', 'test-key']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    def test_main_with_disable_flags(self, mock_enrich):
        """Test main with disable flags."""
        input_data = {"vulnerabilities": [{"cve": "CVE-2021-11111"}]}
        with open(self.input_file, 'w') as f:
            json.dump(input_data, f)
        
        mock_enrich.return_value = []
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file, '--disable-vulncheck', '--disable-ghsa']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    def test_main_with_findings_key(self, mock_enrich):
        """Test main with 'findings' key instead of 'vulnerabilities'."""
        input_data = {
            "scan_date": "2025-01-17",
            "findings": [{"cve": "CVE-2021-11111"}]
        }
        with open(self.input_file, 'w') as f:
            json.dump(input_data, f)
        
        mock_enrich.return_value = [{"cve": "CVE-2021-11111", "priority": "P1-CRITICAL"}]
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file]):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulnerability_enrichment.VulnerabilityEnricher.enrich_all')
    def test_main_exception_handling(self, mock_enrich):
        """Test main handles general exceptions."""
        input_data = {"vulnerabilities": [{"cve": "CVE-2021-11111"}]}
        with open(self.input_file, 'w') as f:
            json.dump(input_data, f)
        
        mock_enrich.side_effect = Exception("Unexpected error")
        
        from vulnerability_enrichment import main
        with patch('sys.argv', ['vulnerability_enrichment.py', self.input_file]):
            result = main()
        
        self.assertEqual(result, 1)


class TestKEVEnricherCLI(unittest.TestCase):
    """Test KEV CLI main function."""
    
    @patch('kev_enrichment.KEVEnricher.fetch_kev_catalog')
    @patch('kev_enrichment.KEVEnricher.is_known_exploited')
    def test_kev_main_with_json_output(self, mock_is_known, mock_fetch):
        """Test KEV main function with JSON output."""
        mock_fetch.return_value = {"vulnerabilities": []}
        mock_is_known.return_value = {
            "in_kev": True,
            "vulnerability_name": "Test",
            "date_added": "2021-01-01",
            "due_date": "2021-01-15"
        }
        
        from kev_enrichment import main
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-11111', '--json']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('kev_enrichment.KEVEnricher.fetch_kev_catalog')
    @patch('kev_enrichment.KEVEnricher.is_known_exploited')
    def test_kev_main_with_text_output(self, mock_is_known, mock_fetch):
        """Test KEV main function with text output."""
        mock_fetch.return_value = {"vulnerabilities": []}
        mock_is_known.return_value = {
            "in_kev": False
        }
        
        from kev_enrichment import main
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('kev_enrichment.KEVEnricher.fetch_kev_catalog')
    @patch('kev_enrichment.KEVEnricher.is_known_exploited')
    def test_kev_main_with_text_output_in_kev(self, mock_is_known, mock_fetch):
        """Test KEV main function with text output when CVE is in KEV."""
        mock_fetch.return_value = {"vulnerabilities": []}
        mock_is_known.return_value = {
            "in_kev": True,
            "vulnerability_name": "Test Vuln",
            "vendor_project": "Test Vendor",
            "product": "Test Product",
            "date_added": "2021-01-01",
            "due_date": "2021-01-15",
            "required_action": "Apply patches"
        }
        
        from kev_enrichment import main
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('kev_enrichment.KEVEnricher.fetch_kev_catalog')
    def test_kev_main_with_error(self, mock_fetch):
        """Test KEV main function handles errors."""
        mock_fetch.side_effect = Exception("Test error")
        
        from kev_enrichment import main
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 1)


class TestKEVEnrichFindingError(unittest.TestCase):
    """Test KEV enrich_finding error handling."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = KEVEnricher(cache_dir=self.temp_dir)
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('kev_enrichment.KEVEnricher.is_known_exploited')
    def test_enrich_finding_with_exception(self, mock_is_known):
        """Test enrich_finding handles exceptions gracefully."""
        mock_is_known.side_effect = ValueError("Test error")
        
        finding = {"cve": "CVE-2021-11111"}
        result = self.enricher.enrich_finding(finding)
        
        # Should have kev field with in_kev=False
        self.assertIn("kev", result)
        self.assertFalse(result["kev"]["in_kev"])


class TestKEVEnricherCLIOutputFormats(unittest.TestCase):
    """Test KEV CLI main function output formats."""
    
    @patch('kev_enrichment.KEVEnricher.fetch_kev_catalog')
    @patch('kev_enrichment.KEVEnricher.is_known_exploited')
    def test_kev_main_with_json_output(self, mock_is_known, mock_fetch):
        """Test KEV main function with JSON output."""
        mock_fetch.return_value = {"vulnerabilities": []}
        mock_is_known.return_value = {
            "in_kev": True,
            "vulnerability_name": "Test",
            "date_added": "2021-01-01",
            "due_date": "2021-01-15"
        }
        
        from kev_enrichment import main
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-11111', '--json']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('kev_enrichment.KEVEnricher.fetch_kev_catalog')
    @patch('kev_enrichment.KEVEnricher.is_known_exploited')
    def test_kev_main_with_text_output(self, mock_is_known, mock_fetch):
        """Test KEV main function with text output."""
        mock_fetch.return_value = {"vulnerabilities": []}
        mock_is_known.return_value = {
            "in_kev": False
        }
        
        from kev_enrichment import main
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('kev_enrichment.KEVEnricher.fetch_kev_catalog')
    def test_kev_main_with_error(self, mock_fetch):
        """Test KEV main function handles errors."""
        mock_fetch.side_effect = Exception("Test error")
        
        from kev_enrichment import main
        with patch('sys.argv', ['kev_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 1)


class TestGHSAEnricherCLI(unittest.TestCase):
    """Test GHSA CLI main function."""
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_json_output(self, mock_query):
        """Test GHSA main function with JSON output."""
        mock_query.return_value = {
            "ghsa_id": "GHSA-test-1234",
            "summary": "Test vulnerability",
            "severity": "HIGH",
            "vulnerabilities": []
        }
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111', '--json']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_text_output(self, mock_query):
        """Test GHSA main function with text output."""
        mock_query.return_value = {
            "ghsa_id": "",
            "summary": "",
            "severity": "",
            "vulnerabilities": []
        }
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_text_output_found(self, mock_query):
        """Test GHSA main function with text output when advisory found."""
        mock_query.return_value = {
            "ghsa_id": "GHSA-test-1234",
            "summary": "Test vulnerability",
            "severity": "HIGH",
            "published_at": "2021-01-01",
            "permalink": "https://github.com/advisories/GHSA-test-1234",
            "vulnerabilities": [
                {
                    "ecosystem": "MAVEN",
                    "package_name": "test:package",
                    "vulnerable_version_range": "< 1.0.0",
                    "first_patched_version": "1.0.0"
                }
            ]
        }
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_text_output_no_patched_version(self, mock_query):
        """Test GHSA main function with vulnerability without patched version."""
        mock_query.return_value = {
            "ghsa_id": "GHSA-test-1234",
            "summary": "Test vulnerability",
            "severity": "HIGH",
            "published_at": "2021-01-01",
            "permalink": "https://github.com/advisories/GHSA-test-1234",
            "vulnerabilities": [
                {
                    "ecosystem": "MAVEN",
                    "package_name": "test:package",
                    "vulnerable_version_range": "< 1.0.0",
                    "first_patched_version": None  # No patch available
                }
            ]
        }
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_github_token(self, mock_query):
        """Test GHSA main function with GitHub token."""
        mock_query.return_value = {
            "ghsa_id": "",
            "summary": "",
            "severity": "",
            "vulnerabilities": []
        }
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111', '--token', 'test_token']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_error(self, mock_query):
        """Test GHSA main function handles errors."""
        mock_query.side_effect = Exception("Test error")
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 1)


class TestGHSAEnricherCLIVariations(unittest.TestCase):
    """Test GHSA CLI main function output variations."""
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_json_output(self, mock_query):
        """Test GHSA main function with JSON output."""
        mock_query.return_value = {
            "ghsa_id": "GHSA-test-1234",
            "summary": "Test vulnerability",
            "severity": "HIGH",
            "vulnerabilities": []
        }
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111', '--json']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_text_output(self, mock_query):
        """Test GHSA main function with text output."""
        mock_query.return_value = {
            "ghsa_id": "",
            "summary": "",
            "severity": "",
            "vulnerabilities": []
        }
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_github_token(self, mock_query):
        """Test GHSA main function with GitHub token."""
        mock_query.return_value = {
            "ghsa_id": "",
            "summary": "",
            "severity": "",
            "vulnerabilities": []
        }
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111', '--token', 'test_token']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('ghsa_enrichment.GHSAEnricher.query_advisory')
    def test_ghsa_main_with_error(self, mock_query):
        """Test GHSA main function handles errors."""
        mock_query.side_effect = Exception("Test error")
        
        from ghsa_enrichment import main
        with patch('sys.argv', ['ghsa_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 1)


class TestVulnCheckEnricherCLI(unittest.TestCase):
    """Test VulnCheck CLI main function."""
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_json_output(self, mock_get):
        """Test VulnCheck main function with JSON output."""
        mock_get.return_value = {
            "exploit_available": True,
            "weaponized": True,
            "exploit_maturity": "functional"
        }
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111', '--json']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_text_output(self, mock_get):
        """Test VulnCheck main function with text output."""
        mock_get.return_value = {
            "exploit_available": False
        }
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_text_output_exploit_found(self, mock_get):
        """Test VulnCheck main function with text output when exploit found."""
        mock_get.return_value = {
            "exploit_available": True,
            "exploit_maturity": "functional",
            "attack_vector": "network",
            "weaponized": True,
            "ransomware_use": True
        }
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_text_output_with_note(self, mock_get):
        """Test VulnCheck main function with text output including note."""
        mock_get.return_value = {
            "exploit_available": False,
            "note": "API key required for detailed intelligence"
        }
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_api_key(self, mock_get):
        """Test VulnCheck main function with API key."""
        mock_get.return_value = {"exploit_available": False}
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111', '--api-key', 'test_key']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_error(self, mock_get):
        """Test VulnCheck main function handles errors."""
        mock_get.side_effect = Exception("Test error")
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 1)


class TestVulnCheckEnricherCLIVariations(unittest.TestCase):
    """Test VulnCheck CLI main function."""
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_json_output(self, mock_get):
        """Test VulnCheck main function with JSON output."""
        mock_get.return_value = {
            "exploit_available": True,
            "weaponized": True,
            "exploit_maturity": "functional"
        }
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111', '--json']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_text_output(self, mock_get):
        """Test VulnCheck main function with text output."""
        mock_get.return_value = {
            "exploit_available": False
        }
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_api_key(self, mock_get):
        """Test VulnCheck main function with API key."""
        mock_get.return_value = {"exploit_available": False}
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111', '--api-key', 'test_key']):
            result = main()
        
        self.assertEqual(result, 0)
    
    @patch('vulncheck_enrichment.VulnCheckEnricher.get_exploit_status')
    def test_vulncheck_main_with_error(self, mock_get):
        """Test VulnCheck main function handles errors."""
        mock_get.side_effect = Exception("Test error")
        
        from vulncheck_enrichment import main
        with patch('sys.argv', ['vulncheck_enrichment.py', 'CVE-2021-11111']):
            result = main()
        
        self.assertEqual(result, 1)


class TestKEVCacheErrorHandling(unittest.TestCase):
    """Test KEV cache error handling."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = KEVEnricher(cache_dir=self.temp_dir)
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_cache_load_with_corrupted_file(self):
        """Test cache loading with corrupted JSON file."""
        cache_file = Path(self.temp_dir) / "kev_catalog.json"
        cache_file.parent.mkdir(parents=True, exist_ok=True)
        with open(cache_file, 'w') as f:
            f.write("{ invalid json")
        
        # Should handle error gracefully and fetch fresh data
        # Mock the network request to avoid actual API call
        with patch('kev_enrichment.requests.get') as mock_get:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.json.return_value = {"vulnerabilities": []}
            mock_get.return_value = mock_response
            
            catalog = self.enricher.fetch_kev_catalog()
            self.assertIn("vulnerabilities", catalog)
    
    @patch('kev_enrichment.requests.get')
    def test_cache_save_error(self, mock_get):
        """Test cache save error handling."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"vulnerabilities": []}
        mock_get.return_value = mock_response
        
        # Use invalid path to trigger save error
        enricher = KEVEnricher(cache_dir="/invalid/path/that/cannot/be/created")
        
        # Should handle save error gracefully
        catalog = enricher.fetch_kev_catalog()
        self.assertIn("vulnerabilities", catalog)


class TestGHSAEnricherErrorHandling(unittest.TestCase):
    """Test GHSA enricher error handling."""
    
    @patch('ghsa_enrichment.requests.post')
    def test_query_advisory_with_invalid_response_structure(self, mock_post):
        """Test GHSA query with malformed response."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"unexpected": "structure"}
        mock_post.return_value = mock_response
        
        enricher = GHSAEnricher()
        result = enricher.query_advisory("CVE-2021-11111")
        
        # Should return empty result
        self.assertEqual(result["ghsa_id"], "")
        self.assertEqual(result["vulnerabilities"], [])


class TestVulnCheckEnricherErrorHandling(unittest.TestCase):
    """Test VulnCheck enricher error handling."""
    
    @patch('vulncheck_enrichment.requests.get')
    def test_get_exploit_status_with_invalid_response(self, mock_get):
        """Test VulnCheck with invalid response structure."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"unexpected": "structure"}
        mock_get.return_value = mock_response
        
        enricher = VulnCheckEnricher(api_key="test_key")
        result = enricher.get_exploit_status("CVE-2021-11111")
        
        # Should return safe defaults
        self.assertFalse(result["exploit_available"])


class TestEPSSValidation(unittest.TestCase):
    """Test EPSS input validation."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = EPSSEnricher(cache_dir=self.temp_dir)
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_fetch_epss_scores_with_non_string_cve(self):
        """Test fetching EPSS scores with non-string CVE ID."""
        with self.assertRaises(TypeError) as ctx:
            self.enricher.fetch_epss_scores([123])  # Integer instead of string
        self.assertIn("CVE ID must be string", str(ctx.exception))


class TestKEVValidation(unittest.TestCase):
    """Test KEV input validation and error paths."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.enricher = KEVEnricher(cache_dir=self.temp_dir)
    
    def tearDown(self):
        """Clean up."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('kev_enrichment.requests.get')
    def test_fetch_kev_catalog_invalid_data_type(self, mock_get):
        """Test KEV fetch with invalid data type (not dict)."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = ["list", "instead", "of", "dict"]  # Invalid type
        mock_get.return_value = mock_response
        
        with self.assertRaises(ValueError) as ctx:
            self.enricher.fetch_kev_catalog()
        self.assertIn("expected dict", str(ctx.exception))


class TestGHSAValidation(unittest.TestCase):
    """Test GHSA input validation."""
    
    @patch('ghsa_enrichment.requests.post')
    def test_query_advisory_with_network_timeout(self, mock_post):
        """Test GHSA query with network timeout."""
        if requests is None:
            self.skipTest("requests module not available")
        
        mock_post.side_effect = requests.Timeout("Request timeout")
        
        enricher = GHSAEnricher()
        result = enricher.query_advisory("CVE-2021-11111")
        
        # Should handle gracefully
        self.assertEqual(result["ghsa_id"], "")
        self.assertIn("error", result)


class TestVulnCheckValidation(unittest.TestCase):
    """Test VulnCheck input validation."""
    
    @patch('vulncheck_enrichment.requests.get')
    def test_get_exploit_status_with_rate_limit(self, mock_get):
        """Test VulnCheck with rate limit response."""
        mock_response = Mock()
        mock_response.status_code = 429  # Rate limit
        mock_response.json.return_value = {"error": "Rate limit exceeded"}
        mock_get.return_value = mock_response
        
        enricher = VulnCheckEnricher(api_key="test_key")
        result = enricher.get_exploit_status("CVE-2021-11111")
        
        # Should handle gracefully
        self.assertFalse(result["exploit_available"])


if __name__ == "__main__":
    unittest.main()
