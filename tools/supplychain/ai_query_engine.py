#!/usr/bin/env python3
"""AI Query Engine for Natural Language SBOM Queries.

This module provides a chat interface for querying SBOM data using natural language.
It uses pattern matching and heuristics to understand queries without requiring
external LLM dependencies (can be extended with local LLM if needed).
"""

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Any


@dataclass
class QueryResult:
    """Result from a query execution."""
    
    query: str
    answer: str
    data: List[Dict[str, Any]]
    confidence: float
    suggestions: List[str]


class AIQueryEngine:
    """Natural language query engine for SBOM data."""
    
    # Query patterns with associated handlers
    QUERY_PATTERNS = [
        # Dependency queries
        (r"(?:what|which).*(?:uses?|depends? on|requires?).*([a-zA-Z0-9\-_.]+)", "find_dependencies"),
        (r"show.*(?:using|with).*([a-zA-Z0-9\-_.]+)", "find_dependencies"),
        (r"list.*([a-zA-Z0-9\-_.]+).*dependencies", "find_dependencies"),
        
        # License queries
        (r"(?:show|list|find|what).*(?:GPL|MIT|Apache|BSD|LGPL|AGPL).*(?:license|dependencies)", "find_by_license"),
        (r".*license.*([A-Z\-0-9.]+)", "find_by_license"),
        
        # Vulnerability queries
        (r"(?:what|which).*(?:vulnerable|vulns?|CVE)", "find_vulnerabilities"),
        (r"CVE-\d{4}-\d+", "find_specific_cve"),
        (r"(?:new|introduced).*vulnerabilities.*v?([\d.]+)", "find_new_vulnerabilities"),
        (r"blast radius.*CVE-\d{4}-\d+", "find_cve_impact"),
        
        # Upgrade queries
        (r"(?:upgrade|update).*([a-zA-Z0-9\-_.]+)", "suggest_upgrade"),
        (r"(?:which|what).*(?:breaks?|affected).*(?:upgrade|update).*([a-zA-Z0-9\-_.]+)", "find_upgrade_impact"),
        
        # Statistics
        (r"how many.*(?:dependencies|packages)", "count_dependencies"),
        (r"statistics|stats|summary", "show_statistics"),
    ]
    
    def __init__(self, sbom_path: Optional[str] = None):
        """Initialize query engine.
        
        Args:
            sbom_path: Optional path to SBOM file to load
            
        Raises:
            FileNotFoundError: If sbom_path doesn't exist
            ValueError: If SBOM is invalid JSON
        """
        self.sbom_data: Optional[Dict] = None
        self.packages: List[Dict] = []
        self.vulnerabilities: List[Dict] = []
        
        if sbom_path:
            self.load_sbom(sbom_path)
    
    def load_sbom(self, sbom_path: str) -> None:
        """Load SBOM file for querying.
        
        Args:
            sbom_path: Path to SPDX JSON SBOM file
            
        Raises:
            FileNotFoundError: If file doesn't exist
            ValueError: If file is not valid SPDX JSON
        """
        path = Path(sbom_path)
        if not path.exists():
            raise FileNotFoundError(f"SBOM file not found: {sbom_path}")
        
        try:
            with open(path, 'r', encoding='utf-8') as f:
                self.sbom_data = json.load(f)
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON in SBOM file: {e}")
        
        # Extract packages
        self.packages = self.sbom_data.get("packages", [])
        if not self.packages:
            raise ValueError("SBOM contains no packages")
        
        # Load vulnerabilities if available (from enriched SBOM or separate file)
        self._load_vulnerabilities()
    
    def _load_vulnerabilities(self) -> None:
        """Load vulnerability data from SBOM or external source."""
        # Check if SBOM has embedded vulnerability data
        for package in self.packages:
            if "externalRefs" in package:
                for ref in package.get("externalRefs", []):
                    if ref.get("referenceCategory") == "SECURITY":
                        # Extract CVE from reference
                        cve_match = re.search(r"CVE-\d{4}-\d+", ref.get("referenceLocator", ""))
                        if cve_match:
                            self.vulnerabilities.append({
                                "cve": cve_match.group(0),
                                "package": package.get("name", "unknown"),
                                "package_id": package.get("SPDXID", ""),
                            })
    
    def query(self, query_text: str) -> QueryResult:
        """Execute natural language query against SBOM data.
        
        Args:
            query_text: Natural language query string
            
        Returns:
            QueryResult with answer and supporting data
            
        Raises:
            ValueError: If no SBOM is loaded
        """
        if not self.sbom_data:
            raise ValueError("No SBOM loaded. Call load_sbom() first.")
        
        query_lower = query_text.lower().strip()
        
        # Match query against patterns
        for pattern, handler_name in self.QUERY_PATTERNS:
            match = re.search(pattern, query_lower, re.IGNORECASE)
            if match:
                handler = getattr(self, f"_handle_{handler_name}")
                return handler(query_text, match)
        
        # No pattern matched
        return QueryResult(
            query=query_text,
            answer="I'm not sure how to answer that. Try asking about dependencies, licenses, or vulnerabilities.",
            data=[],
            confidence=0.0,
            suggestions=[
                "What uses log4j?",
                "Show GPL dependencies",
                "Which packages are vulnerable?",
                "How many dependencies are there?",
            ]
        )
    
    def _handle_find_dependencies(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries about finding dependencies."""
        search_term = match.group(1).lower()
        
        results = []
        for pkg in self.packages:
            pkg_name = pkg.get("name", "").lower()
            if search_term in pkg_name:
                results.append({
                    "name": pkg.get("name"),
                    "version": pkg.get("versionInfo", "unknown"),
                    "spdx_id": pkg.get("SPDXID"),
                    "license": pkg.get("licenseConcluded", "NOASSERTION"),
                })
        
        if results:
            answer = f"Found {len(results)} package(s) matching '{search_term}':\n\n"
            for r in results:
                answer += f"- {r['name']} ({r['version']}) - License: {r['license']}\n"
        else:
            answer = f"No packages found matching '{search_term}'"
        
        return QueryResult(
            query=query,
            answer=answer,
            data=results,
            confidence=0.9 if results else 0.5,
            suggestions=["Show license for these packages", "Are any vulnerable?"]
        )
    
    def _handle_find_by_license(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries about licenses."""
        # Extract license from query
        license_match = re.search(r"(GPL|MIT|Apache|BSD|LGPL|AGPL)", query, re.IGNORECASE)
        if not license_match:
            return QueryResult(
                query=query,
                answer="Please specify a license type (GPL, MIT, Apache, BSD, etc.)",
                data=[],
                confidence=0.0,
                suggestions=[]
            )
        
        license_term = license_match.group(1).upper()
        
        results = []
        for pkg in self.packages:
            license_str = pkg.get("licenseConcluded", "")
            if license_term in license_str:
                results.append({
                    "name": pkg.get("name"),
                    "version": pkg.get("versionInfo", "unknown"),
                    "license": license_str,
                })
        
        if results:
            answer = f"Found {len(results)} package(s) with {license_term} license:\n\n"
            for r in results[:10]:  # Limit to first 10
                answer += f"- {r['name']} ({r['version']}) - {r['license']}\n"
            
            if len(results) > 10:
                answer += f"\n... and {len(results) - 10} more"
        else:
            answer = f"No packages found with {license_term} license"
        
        return QueryResult(
            query=query,
            answer=answer,
            data=results,
            confidence=0.9 if results else 0.7,
            suggestions=[]
        )
    
    def _handle_find_vulnerabilities(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries about vulnerabilities."""
        if not self.vulnerabilities:
            answer = "No vulnerability data available in this SBOM.\n\n"
            answer += "Tip: Run 'bazel run //:sca_scan' to scan for vulnerabilities."
            return QueryResult(
                query=query,
                answer=answer,
                data=[],
                confidence=0.8,
                suggestions=["Run vulnerability scan first"]
            )
        
        answer = f"Found {len(self.vulnerabilities)} vulnerability/vulnerabilities:\n\n"
        for vuln in self.vulnerabilities[:20]:  # Limit display
            answer += f"- {vuln['cve']} in {vuln['package']}\n"
        
        if len(self.vulnerabilities) > 20:
            answer += f"\n... and {len(self.vulnerabilities) - 20} more"
        
        return QueryResult(
            query=query,
            answer=answer,
            data=self.vulnerabilities,
            confidence=0.9,
            suggestions=["Show only critical vulnerabilities", "Which are in CISA KEV?"]
        )
    
    def _handle_find_specific_cve(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries about specific CVEs."""
        cve_match = re.search(r"CVE-\d{4}-\d+", query)
        if not cve_match:
            return QueryResult(
                query=query,
                answer="Please provide a valid CVE ID (e.g., CVE-2021-44228)",
                data=[],
                confidence=0.0,
                suggestions=[]
            )
        
        cve_id = cve_match.group(0)
        
        # Find packages with this CVE
        affected = [v for v in self.vulnerabilities if v["cve"] == cve_id]
        
        if affected:
            answer = f"Found {cve_id} in {len(affected)} package(s):\n\n"
            for vuln in affected:
                answer += f"- {vuln['package']}\n"
        else:
            answer = f"{cve_id} not found in this SBOM"
        
        return QueryResult(
            query=query,
            answer=answer,
            data=affected,
            confidence=0.95 if affected else 0.9,
            suggestions=["Show upgrade recommendations"]
        )
    
    def _handle_find_new_vulnerabilities(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries about new vulnerabilities in a version."""
        return QueryResult(
            query=query,
            answer="This feature requires comparing two SBOMs. Use 'bazel run //tools/supplychain:sbom_diff'",
            data=[],
            confidence=0.5,
            suggestions=["Use sbom_diff tool instead"]
        )
    
    def _handle_find_cve_impact(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries about CVE blast radius."""
        cve_match = re.search(r"CVE-\d{4}-\d+", query)
        if not cve_match:
            return QueryResult(
                query=query,
                answer="Please specify a CVE ID",
                data=[],
                confidence=0.0,
                suggestions=[]
            )
        
        cve_id = cve_match.group(0)
        affected = [v for v in self.vulnerabilities if v["cve"] == cve_id]
        
        if affected:
            answer = f"Blast radius for {cve_id}:\n\n"
            answer += f"Directly affected packages: {len(affected)}\n"
            answer += "\nPackages:\n"
            for vuln in affected:
                answer += f"- {vuln['package']}\n"
            
            # TODO: Calculate transitive dependencies (requires dependency graph)
            answer += "\nNote: Transitive impact analysis requires dependency graph"
        else:
            answer = f"{cve_id} not found in this SBOM"
        
        return QueryResult(
            query=query,
            answer=answer,
            data=affected,
            confidence=0.7,
            suggestions=[]
        )
    
    def _handle_suggest_upgrade(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries about upgrades."""
        package_name = match.group(1)
        
        return QueryResult(
            query=query,
            answer=f"Upgrade recommendations for {package_name} are not yet implemented.\n\n"
                   "This feature is coming in Phase 9!",
            data=[],
            confidence=0.5,
            suggestions=["Check Maven Central for latest version"]
        )
    
    def _handle_find_upgrade_impact(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries about upgrade impact."""
        return QueryResult(
            query=query,
            answer="Upgrade impact analysis coming in Phase 9 (AI-Powered Recommendations)",
            data=[],
            confidence=0.5,
            suggestions=[]
        )
    
    def _handle_count_dependencies(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries about dependency count."""
        count = len(self.packages)
        
        # Try to distinguish direct vs. transitive (rough heuristic)
        direct_count = sum(1 for p in self.packages if "supplier" in p)
        
        answer = f"Total dependencies: {count}\n"
        if direct_count > 0:
            answer += f"Direct dependencies: ~{direct_count}\n"
            answer += f"Transitive dependencies: ~{count - direct_count}\n"
        
        return QueryResult(
            query=query,
            answer=answer,
            data=[{"total": count, "direct": direct_count}],
            confidence=0.9,
            suggestions=["Show all dependencies", "Group by license"]
        )
    
    def _handle_show_statistics(self, query: str, match: re.Match) -> QueryResult:
        """Handle queries requesting statistics."""
        stats = {
            "total_packages": len(self.packages),
            "total_vulnerabilities": len(self.vulnerabilities),
            "licenses": {},
        }
        
        # Count licenses
        for pkg in self.packages:
            license_str = pkg.get("licenseConcluded", "NOASSERTION")
            stats["licenses"][license_str] = stats["licenses"].get(license_str, 0) + 1
        
        # Format answer
        answer = "📊 SBOM Statistics:\n\n"
        answer += f"Total Packages: {stats['total_packages']}\n"
        answer += f"Vulnerabilities: {stats['total_vulnerabilities']}\n\n"
        
        answer += "Top Licenses:\n"
        sorted_licenses = sorted(stats["licenses"].items(), key=lambda x: x[1], reverse=True)
        for license_name, count in sorted_licenses[:5]:
            answer += f"  - {license_name}: {count}\n"
        
        return QueryResult(
            query=query,
            answer=answer,
            data=[stats],
            confidence=0.95,
            suggestions=[]
        )


def interactive_mode(engine: AIQueryEngine) -> None:
    """Run interactive chat interface.
    
    Args:
        engine: Initialized AIQueryEngine instance
    """
    print("🤖 BazBOM AI Query Engine")
    print("=" * 50)
    print("Ask questions about your SBOM in natural language.")
    print("Type 'exit' or 'quit' to end the session.")
    print("=" * 50)
    print()
    
    while True:
        try:
            query = input("💬 You: ").strip()
            
            if not query:
                continue
            
            if query.lower() in ["exit", "quit", "q"]:
                print("\nGoodbye! 👋")
                break
            
            result = engine.query(query)
            
            print(f"\n🤖 Assistant (confidence: {result.confidence:.0%}):")
            print(result.answer)
            
            if result.suggestions:
                print(f"\n💡 Suggestions:")
                for suggestion in result.suggestions:
                    print(f"  - {suggestion}")
            
            print()
            
        except KeyboardInterrupt:
            print("\n\nGoodbye! 👋")
            break
        except Exception as e:
            print(f"\n❌ Error: {e}\n")


def main():
    """Main entry point for AI query engine."""
    parser = argparse.ArgumentParser(
        description="BazBOM AI Query Engine - Natural Language SBOM Queries",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Interactive mode
  bazel run //tools/supplychain:ai_query_engine -- --sbom app.spdx.json

  # Single query
  bazel run //tools/supplychain:ai_query_engine -- \\
    --sbom app.spdx.json \\
    --query "What uses log4j?"

  # Query with JSON output
  bazel run //tools/supplychain:ai_query_engine -- \\
    --sbom app.spdx.json \\
    --query "Show GPL dependencies" \\
    --json
        """
    )
    
    parser.add_argument(
        "--sbom",
        required=True,
        help="Path to SPDX JSON SBOM file"
    )
    
    parser.add_argument(
        "--query",
        help="Single query to execute (omit for interactive mode)"
    )
    
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output results as JSON"
    )
    
    args = parser.parse_args()
    
    try:
        engine = AIQueryEngine(args.sbom)
        
        if args.query:
            # Single query mode
            result = engine.query(args.query)
            
            if args.json:
                # JSON output
                output = {
                    "query": result.query,
                    "answer": result.answer,
                    "data": result.data,
                    "confidence": result.confidence,
                    "suggestions": result.suggestions,
                }
                print(json.dumps(output, indent=2))
            else:
                # Human-readable output
                print(f"Query: {result.query}")
                print(f"Confidence: {result.confidence:.0%}")
                print(f"\nAnswer:\n{result.answer}")
                
                if result.suggestions:
                    print(f"\nSuggestions:")
                    for suggestion in result.suggestions:
                        print(f"  - {suggestion}")
        else:
            # Interactive mode
            interactive_mode(engine)
        
        return 0
        
    except FileNotFoundError as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        return 1
    except ValueError as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"❌ Unexpected error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
