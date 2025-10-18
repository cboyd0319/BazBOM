#!/usr/bin/env python3
"""Build system abstraction layer for BazBOM.

This module provides a unified interface for resolving dependencies across
different build systems (Maven, Gradle, Bazel, etc.). This enables BazBOM
to work with any JVM project, not just Bazel.
"""

import os
import subprocess
import sys
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Dict, List, Optional


class Dependency:
    """Represents a resolved dependency."""
    
    def __init__(
        self,
        name: str,
        version: str,
        group_id: Optional[str] = None,
        artifact_id: Optional[str] = None,
        scope: str = "compile",
        purl: Optional[str] = None,
    ):
        """Initialize dependency.
        
        Args:
            name: Package name
            version: Package version
            group_id: Maven group ID (for Maven/Gradle)
            artifact_id: Maven artifact ID (for Maven/Gradle)
            scope: Dependency scope (compile, runtime, test, etc.)
            purl: Package URL if available
        """
        self.name = name
        self.version = version
        self.group_id = group_id
        self.artifact_id = artifact_id
        self.scope = scope
        self.purl = purl or self._generate_purl()
    
    def _generate_purl(self) -> str:
        """Generate PURL from dependency information."""
        if self.group_id and self.artifact_id:
            return f"pkg:maven/{self.group_id}/{self.artifact_id}@{self.version}"
        return f"pkg:generic/{self.name}@{self.version}"
    
    def to_dict(self) -> Dict[str, str]:
        """Convert to dictionary representation."""
        return {
            "name": self.name,
            "version": self.version,
            "group_id": self.group_id or "",
            "artifact_id": self.artifact_id or "",
            "scope": self.scope,
            "purl": self.purl,
        }
    
    def __repr__(self) -> str:
        """String representation."""
        return f"Dependency({self.purl})"
    
    def __eq__(self, other) -> bool:
        """Equality comparison."""
        if not isinstance(other, Dependency):
            return False
        return self.purl == other.purl
    
    def __hash__(self) -> int:
        """Hash for use in sets/dicts."""
        return hash(self.purl)


class BuildSystem(ABC):
    """Abstract base class for build system implementations."""
    
    @abstractmethod
    def detect(self, path: Path) -> bool:
        """Check if this build system can handle the project.
        
        Args:
            path: Project directory path
            
        Returns:
            True if this build system is detected
        """
        pass
    
    @abstractmethod
    def resolve_dependencies(
        self,
        path: Path,
        include_test_deps: bool = False
    ) -> List[Dependency]:
        """Resolve all dependencies for the project.
        
        Args:
            path: Project directory path
            include_test_deps: Whether to include test dependencies
            
        Returns:
            List of resolved dependencies
            
        Raises:
            RuntimeError: If dependency resolution fails
        """
        pass
    
    @abstractmethod
    def get_name(self) -> str:
        """Get the name of this build system.
        
        Returns:
            Build system name (e.g., "Maven", "Gradle", "Bazel")
        """
        pass


class MavenBuildSystem(BuildSystem):
    """Maven build system implementation."""
    
    def detect(self, path: Path) -> bool:
        """Check if project uses Maven."""
        return (path / "pom.xml").exists()
    
    def get_name(self) -> str:
        """Get build system name."""
        return "Maven"
    
    def resolve_dependencies(
        self,
        path: Path,
        include_test_deps: bool = False
    ) -> List[Dependency]:
        """Resolve Maven dependencies using mvn dependency:list.
        
        Args:
            path: Project directory with pom.xml
            include_test_deps: Whether to include test scope dependencies
            
        Returns:
            List of resolved dependencies
            
        Raises:
            RuntimeError: If Maven command fails
        """
        if not self.detect(path):
            raise RuntimeError(f"No pom.xml found in {path}")
        
        # Check if mvn is available
        if not self._check_maven_available():
            raise RuntimeError(
                "Maven not found in PATH. Please install Maven.\n"
                "See: https://maven.apache.org/install.html"
            )
        
        try:
            # Run mvn dependency:list to get dependencies
            cmd = [
                "mvn",
                "dependency:list",
                "-DoutputFile=/dev/stdout",
                "-DoutputAbsoluteArtifactFilename=false",
                "-DincludeScope=compile,runtime",
            ]
            
            if include_test_deps:
                cmd[-1] = "-DincludeScope=compile,runtime,test"
            
            result = subprocess.run(
                cmd,
                cwd=path,
                capture_output=True,
                text=True,
                timeout=300,  # 5 minute timeout
            )
            
            if result.returncode != 0:
                raise RuntimeError(
                    f"Maven dependency resolution failed:\n{result.stderr}"
                )
            
            # Parse output to extract dependencies
            dependencies = self._parse_maven_output(result.stdout)
            return dependencies
            
        except subprocess.TimeoutExpired:
            raise RuntimeError(
                "Maven dependency resolution timed out after 5 minutes"
            )
        except Exception as e:
            raise RuntimeError(f"Failed to resolve Maven dependencies: {str(e)}")
    
    def _check_maven_available(self) -> bool:
        """Check if Maven is installed."""
        try:
            subprocess.run(
                ["mvn", "--version"],
                capture_output=True,
                timeout=10,
            )
            return True
        except (subprocess.SubprocessError, FileNotFoundError):
            return False
    
    def _parse_maven_output(self, output: str) -> List[Dependency]:
        """Parse Maven dependency:list output.
        
        Args:
            output: Maven command output
            
        Returns:
            List of dependencies
        """
        dependencies = []
        
        for line in output.splitlines():
            line = line.strip()
            
            # Look for dependency lines (format: groupId:artifactId:type:version:scope)
            if ":" in line and not line.startswith("["):
                parts = line.split(":")
                
                # Maven dependency format: groupId:artifactId:packaging:version:scope
                if len(parts) >= 4:
                    group_id = parts[0].strip()
                    artifact_id = parts[1].strip()
                    # parts[2] is packaging (jar, war, etc.)
                    version = parts[3].strip()
                    scope = parts[4].strip() if len(parts) > 4 else "compile"
                    
                    # Skip invalid entries
                    if not group_id or not artifact_id or not version:
                        continue
                    
                    dep = Dependency(
                        name=f"{group_id}:{artifact_id}",
                        version=version,
                        group_id=group_id,
                        artifact_id=artifact_id,
                        scope=scope,
                    )
                    dependencies.append(dep)
        
        return dependencies


class GradleBuildSystem(BuildSystem):
    """Gradle build system implementation."""
    
    def detect(self, path: Path) -> bool:
        """Check if project uses Gradle."""
        return (
            (path / "build.gradle").exists() or
            (path / "build.gradle.kts").exists()
        )
    
    def get_name(self) -> str:
        """Get build system name."""
        return "Gradle"
    
    def resolve_dependencies(
        self,
        path: Path,
        include_test_deps: bool = False
    ) -> List[Dependency]:
        """Resolve Gradle dependencies using gradle dependencies.
        
        Args:
            path: Project directory with build.gradle
            include_test_deps: Whether to include test dependencies
            
        Returns:
            List of resolved dependencies
            
        Raises:
            RuntimeError: If Gradle command fails
        """
        if not self.detect(path):
            raise RuntimeError(f"No build.gradle found in {path}")
        
        # Check if gradlew or gradle is available
        gradlew = path / "gradlew"
        if gradlew.exists():
            gradle_cmd = str(gradlew)
        elif self._check_gradle_available():
            gradle_cmd = "gradle"
        else:
            raise RuntimeError(
                "Gradle not found. Please install Gradle or use gradlew.\n"
                "See: https://gradle.org/install/"
            )
        
        try:
            # Determine which configurations to include
            configurations = ["runtimeClasspath", "compileClasspath"]
            if include_test_deps:
                configurations.extend(["testRuntimeClasspath", "testCompileClasspath"])
            
            # Run gradle dependencies for each configuration
            all_dependencies = set()
            
            for config in configurations:
                cmd = [gradle_cmd, "dependencies", f"--configuration={config}"]
                
                result = subprocess.run(
                    cmd,
                    cwd=path,
                    capture_output=True,
                    text=True,
                    timeout=300,
                )
                
                # Gradle may fail if configuration doesn't exist, that's okay
                if result.returncode == 0:
                    deps = self._parse_gradle_output(result.stdout)
                    all_dependencies.update(deps)
            
            return list(all_dependencies)
            
        except subprocess.TimeoutExpired:
            raise RuntimeError(
                "Gradle dependency resolution timed out after 5 minutes"
            )
        except Exception as e:
            raise RuntimeError(f"Failed to resolve Gradle dependencies: {str(e)}")
    
    def _check_gradle_available(self) -> bool:
        """Check if Gradle is installed."""
        try:
            subprocess.run(
                ["gradle", "--version"],
                capture_output=True,
                timeout=10,
            )
            return True
        except (subprocess.SubprocessError, FileNotFoundError):
            return False
    
    def _parse_gradle_output(self, output: str) -> List[Dependency]:
        """Parse Gradle dependencies output.
        
        Args:
            output: Gradle command output
            
        Returns:
            List of dependencies
        """
        dependencies = []
        
        for line in output.splitlines():
            line = line.strip()
            
            # Look for dependency lines (format: group:name:version)
            # Example: "+--- com.google.guava:guava:31.1-jre"
            if "---" in line and ":" in line:
                # Extract the dependency part (after ---)
                dep_part = line.split("---", 1)[1].strip()
                
                # Handle version selection markers like " -> 2.0.0"
                if " -> " in dep_part:
                    dep_part = dep_part.split(" -> ")[1].strip()
                
                # Remove any trailing markers like (*), (c), etc.
                dep_part = dep_part.split()[0] if " " in dep_part else dep_part
                
                # Parse group:artifact:version
                parts = dep_part.split(":")
                if len(parts) >= 3:
                    group_id = parts[0]
                    artifact_id = parts[1]
                    version = parts[2]
                    
                    dep = Dependency(
                        name=f"{group_id}:{artifact_id}",
                        version=version,
                        group_id=group_id,
                        artifact_id=artifact_id,
                        scope="compile",
                    )
                    dependencies.append(dep)
        
        return dependencies


class BazelBuildSystem(BuildSystem):
    """Bazel build system implementation (existing functionality)."""
    
    def detect(self, path: Path) -> bool:
        """Check if project uses Bazel."""
        return (
            (path / "WORKSPACE").exists() or
            (path / "WORKSPACE.bazel").exists() or
            (path / "MODULE.bazel").exists()
        )
    
    def get_name(self) -> str:
        """Get build system name."""
        return "Bazel"
    
    def resolve_dependencies(
        self,
        path: Path,
        include_test_deps: bool = False
    ) -> List[Dependency]:
        """Resolve Bazel dependencies (delegates to existing tooling).
        
        Note: For Bazel, BazBOM uses aspect-based dependency discovery.
        This method is a placeholder that indicates Bazel is detected.
        
        Args:
            path: Project directory
            include_test_deps: Whether to include test dependencies
            
        Returns:
            Empty list (actual resolution happens via Bazel aspects)
            
        Raises:
            RuntimeError: If Bazel is not available
        """
        if not self.detect(path):
            raise RuntimeError(f"No Bazel WORKSPACE found in {path}")
        
        # For Bazel, we rely on the existing aspect-based tooling
        # This is just to indicate that Bazel was detected
        print(
            "INFO: Bazel detected. Use 'bazel build //:sbom_all' for SBOM generation.",
            file=sys.stderr
        )
        return []


def detect_build_system(path: Path) -> Optional[BuildSystem]:
    """Auto-detect the build system for a project.
    
    Args:
        path: Project directory path
        
    Returns:
        Detected build system or None if not detected
    """
    # Order matters: check Bazel first (most specific), then others
    systems = [
        BazelBuildSystem(),
        MavenBuildSystem(),
        GradleBuildSystem(),
    ]
    
    for system in systems:
        if system.detect(path):
            return system
    
    return None


def main():
    """Main entry point for testing build system detection."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Detect and resolve dependencies for a project'
    )
    parser.add_argument(
        'path',
        type=str,
        help='Project directory path'
    )
    parser.add_argument(
        '--include-test',
        action='store_true',
        help='Include test dependencies'
    )
    
    args = parser.parse_args()
    
    path = Path(args.path).resolve()
    
    if not path.exists():
        print(f"ERROR: Path does not exist: {path}", file=sys.stderr)
        sys.exit(1)
    
    if not path.is_dir():
        print(f"ERROR: Path is not a directory: {path}", file=sys.stderr)
        sys.exit(1)
    
    # Detect build system
    build_system = detect_build_system(path)
    
    if not build_system:
        print("ERROR: Could not detect build system", file=sys.stderr)
        print("Supported: Maven (pom.xml), Gradle (build.gradle), Bazel (WORKSPACE)",
              file=sys.stderr)
        sys.exit(1)
    
    print(f"Detected build system: {build_system.get_name()}")
    
    # Resolve dependencies
    try:
        dependencies = build_system.resolve_dependencies(
            path,
            include_test_deps=args.include_test
        )
        
        print(f"\nFound {len(dependencies)} dependencies:")
        for dep in dependencies:
            print(f"  - {dep.purl}")
            
    except RuntimeError as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
