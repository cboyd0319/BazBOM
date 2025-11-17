#!/usr/bin/env bash
#
# BazBOM Monorepo Diagnostics Script
#
# This script collects comprehensive information about your monorepo to help
# tune BazBOM for optimal performance with large, complex projects.
#
# Usage:
#   cd /path/to/your/monorepo
#   /path/to/BazBOM/scripts/monorepo-diagnostics.sh
#
# Output:
#   Creates ./bazbom-diagnostics/ directory with all diagnostic files
#   Creates ./bazbom-diagnostics.tar.gz for easy sharing
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TIMEOUT_SHORT=30
TIMEOUT_MEDIUM=60
TIMEOUT_LONG=120
MAX_FILE_SAMPLES=20
MAX_LINES_SAMPLE=100

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

section_header() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  $1"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Check if we're in a git repository
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    log_error "Not in a git repository. Please run this script from your monorepo root."
    exit 1
fi

REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

log_info "Starting BazBOM monorepo diagnostics..."
log_info "Repository root: $REPO_ROOT"

# Create diagnostics directory
DIAG_DIR="$REPO_ROOT/bazbom-diagnostics"
rm -rf "$DIAG_DIR"
mkdir -p "$DIAG_DIR"

log_success "Created diagnostics directory: $DIAG_DIR"

# ============================================================================
# 1. REPOSITORY SIZE & STRUCTURE
# ============================================================================
section_header "1/10 Repository Size & Structure"

{
    echo "=== Repository Size ==="
    du -sh . 2>/dev/null || echo "Could not calculate size"
    echo ""

    echo "=== Total File Count ==="
    find . -type f 2>/dev/null | wc -l | xargs echo "Total files:"
    echo ""

    echo "=== File Counts by Language/Type ==="
    find . -type f -name "*.java" 2>/dev/null | wc -l | xargs echo "Java files:"
    find . -type f -name "*.kt" 2>/dev/null | wc -l | xargs echo "Kotlin files:"
    find . -type f -name "*.scala" 2>/dev/null | wc -l | xargs echo "Scala files:"
    find . -type f \( -name "*.js" -o -name "*.ts" -o -name "*.jsx" -o -name "*.tsx" \) 2>/dev/null | wc -l | xargs echo "JavaScript/TypeScript files:"
    find . -type f -name "*.py" 2>/dev/null | wc -l | xargs echo "Python files:"
    find . -type f -name "*.go" 2>/dev/null | wc -l | xargs echo "Go files:"
    find . -type f -name "*.rb" 2>/dev/null | wc -l | xargs echo "Ruby files:"
    find . -type f -name "*.php" 2>/dev/null | wc -l | xargs echo "PHP files:"
    find . -type f -name "*.rs" 2>/dev/null | wc -l | xargs echo "Rust files:"
    find . -type f -name "*.cpp" -o -name "*.cc" -o -name "*.cxx" 2>/dev/null | wc -l | xargs echo "C++ files:"
    find . -type f -name "*.c" 2>/dev/null | wc -l | xargs echo "C files:"
    echo ""

    echo "=== Lines of Code (Top 10 Languages) ==="
    if command -v cloc >/dev/null 2>&1; then
        timeout $TIMEOUT_MEDIUM cloc --quiet --csv . 2>/dev/null | head -12 || echo "cloc timed out"
    else
        echo "cloc not installed (optional - install with: brew install cloc)"
    fi
} > "$DIAG_DIR/01-repo-structure.txt"

log_success "Completed: Repository structure analysis"

# ============================================================================
# 2. DIRECTORY TREE
# ============================================================================
section_header "2/10 Directory Tree"

{
    echo "=== Directory Structure (3 levels deep) ==="
    if command -v tree >/dev/null 2>&1; then
        tree -L 3 -d --gitignore 2>/dev/null | head -500
    else
        find . -maxdepth 3 -type d 2>/dev/null | head -200
        echo ""
        echo "(Install 'tree' for better output: brew install tree)"
    fi
} > "$DIAG_DIR/02-directory-tree.txt"

log_success "Completed: Directory tree"

# ============================================================================
# 3. BUILD SYSTEMS DETECTION
# ============================================================================
section_header "3/10 Build Systems Detection"

{
    echo "=== Build Systems Present ==="
    echo ""

    # Maven
    echo "━━━ Maven ━━━"
    MAVEN_COUNT=$(find . -name "pom.xml" -type f 2>/dev/null | wc -l | tr -d ' ')
    echo "Total pom.xml files: $MAVEN_COUNT"
    if [ "$MAVEN_COUNT" -gt 0 ]; then
        echo "Sample pom.xml locations (first $MAX_FILE_SAMPLES):"
        find . -name "pom.xml" -type f 2>/dev/null | head -$MAX_FILE_SAMPLES | sed 's/^/  /'
    fi
    echo ""

    # Gradle
    echo "━━━ Gradle ━━━"
    GRADLE_BUILD_COUNT=$(find . \( -name "build.gradle" -o -name "build.gradle.kts" \) -type f 2>/dev/null | wc -l | tr -d ' ')
    GRADLE_SETTINGS_COUNT=$(find . \( -name "settings.gradle" -o -name "settings.gradle.kts" \) -type f 2>/dev/null | wc -l | tr -d ' ')
    echo "Total build.gradle files: $GRADLE_BUILD_COUNT"
    echo "Total settings.gradle files: $GRADLE_SETTINGS_COUNT"
    if [ "$GRADLE_BUILD_COUNT" -gt 0 ]; then
        echo "Sample build.gradle locations (first $MAX_FILE_SAMPLES):"
        find . \( -name "build.gradle" -o -name "build.gradle.kts" \) -type f 2>/dev/null | head -$MAX_FILE_SAMPLES | sed 's/^/  /'
    fi
    if [ -f "gradle/wrapper/gradle-wrapper.properties" ]; then
        echo "Gradle wrapper version:"
        grep "distributionUrl" gradle/wrapper/gradle-wrapper.properties | sed 's/^/  /'
    fi
    echo ""

    # Bazel
    echo "━━━ Bazel ━━━"
    BUILD_COUNT=$(find . \( -name "BUILD" -o -name "BUILD.bazel" \) -type f 2>/dev/null | wc -l | tr -d ' ')
    echo "Total BUILD files: $BUILD_COUNT"

    if [ -f ".bazelversion" ]; then
        echo "Bazel version (from .bazelversion): $(cat .bazelversion)"
    fi

    if [ -f "MODULE.bazel" ]; then
        echo "Build system: Bzlmod (MODULE.bazel) - Bazel 6+"
        echo "MODULE.bazel size: $(wc -l < MODULE.bazel) lines"
    elif [ -f "WORKSPACE.bazel" ]; then
        echo "Build system: Legacy WORKSPACE.bazel"
        echo "WORKSPACE.bazel size: $(wc -l < WORKSPACE.bazel) lines"
    elif [ -f "WORKSPACE" ]; then
        echo "Build system: Legacy WORKSPACE"
        echo "WORKSPACE size: $(wc -l < WORKSPACE) lines"
    else
        echo "No Bazel workspace file found"
    fi

    if [ "$BUILD_COUNT" -gt 0 ]; then
        echo "Sample BUILD file locations (first $MAX_FILE_SAMPLES):"
        find . \( -name "BUILD" -o -name "BUILD.bazel" \) -type f 2>/dev/null | head -$MAX_FILE_SAMPLES | sed 's/^/  /'
    fi
    echo ""

    # sbt
    echo "━━━ sbt (Scala Build Tool) ━━━"
    SBT_COUNT=$(find . -name "build.sbt" -type f 2>/dev/null | wc -l | tr -d ' ')
    echo "Total build.sbt files: $SBT_COUNT"
    echo ""

    # Other package managers
    echo "━━━ Other Package Managers ━━━"
    find . -name "package.json" -type f 2>/dev/null | wc -l | xargs echo "npm/yarn (package.json):"
    find . \( -name "requirements.txt" -o -name "Pipfile" -o -name "pyproject.toml" -o -name "setup.py" \) -type f 2>/dev/null | wc -l | xargs echo "Python:"
    find . -name "go.mod" -type f 2>/dev/null | wc -l | xargs echo "Go (go.mod):"
    find . -name "Cargo.toml" -type f 2>/dev/null | wc -l | xargs echo "Rust (Cargo.toml):"
    find . -name "Gemfile" -type f 2>/dev/null | wc -l | xargs echo "Ruby (Gemfile):"
    find . -name "composer.json" -type f 2>/dev/null | wc -l | xargs echo "PHP (composer.json):"

} > "$DIAG_DIR/03-build-systems.txt"

log_success "Completed: Build systems detection"

# ============================================================================
# 4. SAMPLE BUILD FILES
# ============================================================================
section_header "4/10 Sample Build Files"

{
    # Root Maven POM
    if [ -f "pom.xml" ]; then
        echo "=== Root pom.xml (first $MAX_LINES_SAMPLE lines) ==="
        head -$MAX_LINES_SAMPLE pom.xml
        echo ""
        echo ""
    fi

    # Root Gradle build
    if [ -f "build.gradle" ]; then
        echo "=== Root build.gradle (first $MAX_LINES_SAMPLE lines) ==="
        head -$MAX_LINES_SAMPLE build.gradle
        echo ""
        echo ""
    elif [ -f "build.gradle.kts" ]; then
        echo "=== Root build.gradle.kts (first $MAX_LINES_SAMPLE lines) ==="
        head -$MAX_LINES_SAMPLE build.gradle.kts
        echo ""
        echo ""
    fi

    # Gradle settings
    if [ -f "settings.gradle" ]; then
        echo "=== Root settings.gradle (first $MAX_LINES_SAMPLE lines) ==="
        head -$MAX_LINES_SAMPLE settings.gradle
        echo ""
        echo ""
    elif [ -f "settings.gradle.kts" ]; then
        echo "=== Root settings.gradle.kts (first $MAX_LINES_SAMPLE lines) ==="
        head -$MAX_LINES_SAMPLE settings.gradle.kts
        echo ""
        echo ""
    fi

    # Bazel MODULE.bazel (modern)
    if [ -f "MODULE.bazel" ]; then
        echo "=== MODULE.bazel (Bazel 6+ Bzlmod) - first 150 lines ==="
        head -150 MODULE.bazel
        echo ""
        echo ""
    fi

    # Legacy WORKSPACE
    if [ -f "WORKSPACE" ]; then
        echo "=== WORKSPACE (first $MAX_LINES_SAMPLE lines) ==="
        head -$MAX_LINES_SAMPLE WORKSPACE
        echo ""
        echo ""
    elif [ -f "WORKSPACE.bazel" ]; then
        echo "=== WORKSPACE.bazel (first $MAX_LINES_SAMPLE lines) ==="
        head -$MAX_LINES_SAMPLE WORKSPACE.bazel
        echo ""
        echo ""
    fi

    # .bazelrc
    if [ -f ".bazelrc" ]; then
        echo "=== .bazelrc (first $MAX_LINES_SAMPLE lines) ==="
        head -$MAX_LINES_SAMPLE .bazelrc
        echo ""
        echo ""
    fi

} > "$DIAG_DIR/04-build-file-samples.txt"

log_success "Completed: Sample build files"

# ============================================================================
# 5. BAZEL DETAILED ANALYSIS
# ============================================================================
section_header "5/10 Bazel Detailed Analysis"

if [ -f "MODULE.bazel" ] || [ -f "WORKSPACE" ] || [ -f "WORKSPACE.bazel" ]; then
    {
        echo "=== Bazel Installation ==="
        if command -v bazel >/dev/null 2>&1; then
            bazel version 2>&1 | head -10
        else
            echo "Bazel not found in PATH"
            echo "Note: Some queries will be skipped"
        fi
        echo ""

        if command -v bazel >/dev/null 2>&1; then
            echo "=== Bazel Info ==="
            timeout $TIMEOUT_SHORT bazel info 2>&1 | grep -E "(output_base|execution_root|workspace|bazel-bin)" || echo "bazel info failed or timed out"
            echo ""

            # Bzlmod module dependencies
            if [ -f "MODULE.bazel" ]; then
                echo "=== Bazel Module System (Bzlmod) ==="
                echo "Using MODULE.bazel (Bazel 6+ module system)"
                echo ""

                echo "Module dependency graph (first 50 lines):"
                timeout $TIMEOUT_MEDIUM bazel mod graph 2>&1 | head -50 || echo "bazel mod graph failed or timed out"
                echo ""
            fi

            # Target counts
            echo "=== Bazel Target Counts ==="
            log_info "Counting all Bazel targets (may take a while for large repos)..."
            timeout $TIMEOUT_LONG bazel query "//..." 2>/dev/null | wc -l | xargs echo "Total Bazel targets:" || echo "Query timed out after ${TIMEOUT_LONG}s"

            log_info "Counting Java targets..."
            timeout $TIMEOUT_MEDIUM bazel query 'kind("java_.*", //...)' 2>/dev/null | wc -l | xargs echo "Java targets (java_library, java_binary, etc.):" || echo "Query timed out"

            timeout $TIMEOUT_MEDIUM bazel query 'kind("java_library", //...)' 2>/dev/null | wc -l | xargs echo "  java_library:" || echo "  Query timed out"
            timeout $TIMEOUT_MEDIUM bazel query 'kind("java_binary", //...)' 2>/dev/null | wc -l | xargs echo "  java_binary:" || echo "  Query timed out"
            timeout $TIMEOUT_MEDIUM bazel query 'kind("java_test", //...)' 2>/dev/null | wc -l | xargs echo "  java_test:" || echo "  Query timed out"

            log_info "Counting Kotlin targets..."
            timeout $TIMEOUT_MEDIUM bazel query 'kind("kt_.*", //...)' 2>/dev/null | wc -l | xargs echo "Kotlin targets:" || echo "Query timed out"

            log_info "Counting test targets..."
            timeout $TIMEOUT_MEDIUM bazel query 'kind(".*_test", //...)' 2>/dev/null | wc -l | xargs echo "Test targets (all types):" || echo "Query timed out"

            log_info "Counting proto targets..."
            timeout $TIMEOUT_MEDIUM bazel query 'kind("proto_library", //...)' 2>/dev/null | wc -l | xargs echo "proto_library:" || echo "Query timed out"
            echo ""

            # Sample targets
            echo "=== Sample Bazel Targets (first 50) ==="
            timeout $TIMEOUT_MEDIUM bazel query "//..." 2>/dev/null | head -50 || echo "Query failed or timed out"
            echo ""

            # Maven dependency management
            echo "=== Maven Dependency Management in Bazel ==="
            if [ -f "MODULE.bazel" ]; then
                echo "Checking MODULE.bazel for Maven dependencies..."
                grep -c "maven" MODULE.bazel 2>/dev/null | xargs echo "Mentions of 'maven' in MODULE.bazel:" || echo "0"
                echo ""

                echo "maven.install declarations:"
                grep -A 10 "maven.install" MODULE.bazel 2>/dev/null | head -50 || echo "No maven.install found"
                echo ""
            fi

            if [ -f "WORKSPACE" ] || [ -f "WORKSPACE.bazel" ]; then
                echo "Checking WORKSPACE for rules_jvm_external..."
                grep -A 10 "rules_jvm_external" WORKSPACE* 2>/dev/null | head -50 || echo "No rules_jvm_external found"
                echo ""
            fi

            # Maven lockfiles
            echo "Maven dependency lockfiles:"
            MAVEN_LOCKS=$(find . -name "maven_install.json" -type f 2>/dev/null)
            if [ -n "$MAVEN_LOCKS" ]; then
                echo "$MAVEN_LOCKS" | wc -l | xargs echo "Found maven_install.json files:"
                echo "$MAVEN_LOCKS" | head -10
                echo ""

                # Sample first lockfile
                FIRST_LOCK=$(echo "$MAVEN_LOCKS" | head -1)
                if [ -n "$FIRST_LOCK" ]; then
                    echo "Sample maven_install.json (first 100 lines): $FIRST_LOCK"
                    head -100 "$FIRST_LOCK"
                fi
            else
                echo "No maven_install.json files found"
            fi
        fi

    } > "$DIAG_DIR/05-bazel-analysis.txt"

    log_success "Completed: Bazel analysis"
else
    echo "No Bazel workspace found (skipping Bazel analysis)" > "$DIAG_DIR/05-bazel-analysis.txt"
    log_warning "No Bazel workspace found - skipped Bazel analysis"
fi

# ============================================================================
# 6. MODULE/PROJECT STRUCTURE
# ============================================================================
section_header "6/10 Module/Project Structure"

{
    echo "=== Project/Module Count ==="
    echo ""

    # Maven modules
    if [ -f "pom.xml" ]; then
        echo "━━━ Maven Modules ━━━"
        MODULE_COUNT=$(grep -c "<module>" pom.xml 2>/dev/null || echo "0")
        echo "Modules declared in root pom.xml: $MODULE_COUNT"
        if [ "$MODULE_COUNT" -gt 0 ]; then
            echo "Modules (first 30):"
            grep "<module>" pom.xml 2>/dev/null | head -30 | sed 's/^/  /'
        fi
        echo ""
    fi

    # Gradle subprojects
    if [ -f "settings.gradle" ] || [ -f "settings.gradle.kts" ]; then
        echo "━━━ Gradle Subprojects ━━━"
        SUBPROJECT_COUNT=$(grep -c "include" settings.gradle* 2>/dev/null || echo "0")
        echo "Subprojects declared in settings.gradle: $SUBPROJECT_COUNT"
        if [ "$SUBPROJECT_COUNT" -gt 0 ]; then
            echo "Subprojects (first 30):"
            grep "include" settings.gradle* 2>/dev/null | head -30 | sed 's/^/  /'
        fi
        echo ""
    fi

    # Estimate total independent projects
    echo "━━━ Estimated Total Projects/Modules ━━━"
    TOTAL_PROJECTS=$(find . \( -name "pom.xml" -o -name "build.gradle" -o -name "build.gradle.kts" -o -name "package.json" \) -type f 2>/dev/null | wc -l | tr -d ' ')
    echo "Total build files (pom.xml + build.gradle + package.json): $TOTAL_PROJECTS"
    echo ""

    # Package structure
    echo "━━━ Package/Namespace Structure ━━━"
    echo "Top-level Java packages (sample):"
    find . -type d -path "*/src/main/java/*" 2>/dev/null | sed 's|.*/src/main/java/||' | cut -d'/' -f1 | sort -u | head -20 | sed 's/^/  /'
    echo ""

} > "$DIAG_DIR/06-module-structure.txt"

log_success "Completed: Module/project structure"

# ============================================================================
# 7. DEPENDENCY PATTERNS
# ============================================================================
section_header "7/10 Dependency Patterns"

{
    echo "=== Dependency Declaration Patterns ==="
    echo ""

    # Maven dependencies
    if [ -f "pom.xml" ]; then
        echo "━━━ Maven Dependencies (from root pom.xml) ━━━"
        echo "Sample dependencies:"
        grep -A 5 "<dependency>" pom.xml 2>/dev/null | head -50 || echo "No dependencies found"
        echo ""

        echo "Dependency management:"
        grep -A 10 "dependencyManagement" pom.xml 2>/dev/null | head -30 || echo "No dependency management found"
        echo ""
    fi

    # Gradle dependencies
    GRADLE_FILE=$([ -f "build.gradle" ] && echo "build.gradle" || echo "build.gradle.kts")
    if [ -f "$GRADLE_FILE" ]; then
        echo "━━━ Gradle Dependencies (from root $GRADLE_FILE) ━━━"
        echo "Sample dependencies:"
        grep -E "(implementation|api|compile|testImplementation|runtimeOnly)" "$GRADLE_FILE" 2>/dev/null | head -30 || echo "No dependencies found"
        echo ""

        echo "Platforms/BOMs:"
        grep "platform(" "$GRADLE_FILE" 2>/dev/null | head -10 || echo "No platforms found"
        echo ""
    fi

    # Version catalogs (Gradle)
    echo "━━━ Gradle Version Catalogs ━━━"
    CATALOG_FILES=$(find . -path "*/gradle/libs.versions.toml" 2>/dev/null)
    if [ -n "$CATALOG_FILES" ]; then
        echo "Found version catalog files:"
        echo "$CATALOG_FILES"
        echo ""
        echo "Sample catalog (first file, first 50 lines):"
        head -50 "$(echo "$CATALOG_FILES" | head -1)"
    else
        echo "No version catalogs found"
    fi
    echo ""

    # Common dependency groups
    echo "━━━ Common Dependency Patterns ━━━"
    echo "Spring Boot usage:"
    grep -r "spring-boot" --include="pom.xml" --include="build.gradle*" . 2>/dev/null | wc -l | xargs echo "  Files mentioning spring-boot:"

    echo "Kotlin usage:"
    grep -r "kotlin" --include="pom.xml" --include="build.gradle*" . 2>/dev/null | wc -l | xargs echo "  Files mentioning kotlin:"

    echo "gRPC/Protobuf usage:"
    grep -r "grpc\|protobuf" --include="pom.xml" --include="build.gradle*" . 2>/dev/null | wc -l | xargs echo "  Files mentioning grpc/protobuf:"

} > "$DIAG_DIR/07-dependency-patterns.txt"

log_success "Completed: Dependency patterns"

# ============================================================================
# 8. GIT REPOSITORY STATS
# ============================================================================
section_header "8/10 Git Repository Stats"

{
    echo "=== Git Repository Statistics ==="
    echo ""

    git log --oneline 2>/dev/null | wc -l | xargs echo "Total commits:"
    git branch -r 2>/dev/null | wc -l | xargs echo "Remote branches:"
    git ls-files 2>/dev/null | wc -l | xargs echo "Tracked files:"
    echo ""

    echo "Total lines in tracked files:"
    git ls-files | xargs wc -l 2>/dev/null | tail -1 || echo "Could not count lines"
    echo ""

    echo "Recent commits (last 20):"
    git log --oneline -20 2>/dev/null
    echo ""

    echo "Active contributors (last 6 months):"
    git log --since="6 months ago" --format='%an' 2>/dev/null | sort -u | wc -l | xargs echo "Unique contributors:"

    echo ""
    echo "Repository age:"
    git log --reverse --format='%ai' | head -1 | xargs echo "First commit:"
    git log --format='%ai' | head -1 | xargs echo "Latest commit:"

} > "$DIAG_DIR/08-git-stats.txt"

log_success "Completed: Git repository stats"

# ============================================================================
# 9. SPECIAL CONFIGURATIONS
# ============================================================================
section_header "9/10 Special Configurations"

{
    echo "=== BazBOM Configuration ==="
    if [ -f ".bazbomrc" ] || [ -f "bazbom.toml" ]; then
        echo "Found BazBOM configuration files:"
        find . -maxdepth 2 \( -name ".bazbomrc" -o -name "bazbom.toml" \) 2>/dev/null
        echo ""
        if [ -f "bazbom.toml" ]; then
            echo "bazbom.toml contents:"
            cat bazbom.toml
        fi
    else
        echo "No BazBOM configuration found (.bazbomrc or bazbom.toml)"
    fi
    echo ""

    echo "=== CI/CD Configurations ==="
    find . -path "*/.github/workflows/*.yml" -o -path "*/.github/workflows/*.yaml" 2>/dev/null | wc -l | xargs echo "GitHub Actions workflows:"
    [ -f ".gitlab-ci.yml" ] && echo "GitLab CI: Found" || echo "GitLab CI: Not found"
    find . -name "Jenkinsfile" 2>/dev/null | wc -l | xargs echo "Jenkinsfiles:"
    [ -f ".circleci/config.yml" ] && echo "CircleCI: Found" || echo "CircleCI: Not found"
    [ -f ".travis.yml" ] && echo "Travis CI: Found" || echo "Travis CI: Not found"
    echo ""

    echo "=== Containerization ==="
    find . -name "Dockerfile" -type f 2>/dev/null | wc -l | xargs echo "Dockerfiles:"
    find . -name "docker-compose.yml" -o -name "docker-compose.yaml" 2>/dev/null | wc -l | xargs echo "docker-compose files:"
    [ -f "skaffold.yaml" ] && echo "Skaffold: Found" || echo "Skaffold: Not found"
    [ -f "tilt.config" ] && echo "Tilt: Found" || echo "Tilt: Not found"
    echo ""

    echo "=== Kubernetes ==="
    find . -path "*/k8s/*.yaml" -o -path "*/kubernetes/*.yaml" 2>/dev/null | wc -l | xargs echo "Kubernetes YAML files:"
    [ -d "charts" ] && echo "Helm charts directory: Found" || echo "Helm charts directory: Not found"
    echo ""

    echo "=== Code Quality Tools ==="
    [ -f ".editorconfig" ] && echo "EditorConfig: Found" || echo "EditorConfig: Not found"
    [ -f ".prettierrc" ] || [ -f "prettier.config.js" ] && echo "Prettier: Found" || echo "Prettier: Not found"
    [ -f ".eslintrc" ] || [ -f ".eslintrc.js" ] && echo "ESLint: Found" || echo "ESLint: Not found"
    find . -name "checkstyle.xml" 2>/dev/null | wc -l | xargs echo "Checkstyle configs:"
    find . -name "spotbugs.xml" -o -name "findbugs.xml" 2>/dev/null | wc -l | xargs echo "SpotBugs/FindBugs configs:"
    echo ""

} > "$DIAG_DIR/09-special-configs.txt"

log_success "Completed: Special configurations"

# ============================================================================
# 10. PERFORMANCE INDICATORS
# ============================================================================
section_header "10/10 Performance Indicators"

{
    echo "=== Performance Indicators ==="
    echo ""

    echo "File listing performance:"
    /usr/bin/time -p find . -type f >/dev/null 2>&1 || echo "Could not measure"
    echo ""

    echo "Largest build files (top 10 by line count):"
    find . \( -name "pom.xml" -o -name "build.gradle" -o -name "build.gradle.kts" \) -type f -exec wc -l {} + 2>/dev/null | sort -rn | head -10 || echo "Could not measure"
    echo ""

    echo "Deepest directory nesting:"
    find . -type d 2>/dev/null | awk '{print gsub(/\//, "")}' | sort -rn | head -1 | xargs echo "Maximum depth:"
    echo ""

    echo "Git repository size:"
    du -sh .git 2>/dev/null || echo "Could not measure"
    echo ""

    echo "Ignored files (.gitignore patterns):"
    git ls-files --others --ignored --exclude-standard 2>/dev/null | wc -l | xargs echo "Ignored files count:"
    echo ""

    echo "Build output directories (estimated size):"
    du -sh target 2>/dev/null | xargs echo "Maven target/:" || echo "Maven target/: Not found"
    du -sh build 2>/dev/null | xargs echo "Gradle build/:" || echo "Gradle build/: Not found"
    du -sh bazel-* 2>/dev/null | head -5 || echo "Bazel outputs: Not found"

} > "$DIAG_DIR/10-performance-indicators.txt"

log_success "Completed: Performance indicators"

# ============================================================================
# SUMMARY REPORT
# ============================================================================
section_header "Generating Summary Report"

{
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                                                                ║"
    echo "║         BazBOM Monorepo Diagnostics - Summary Report          ║"
    echo "║                                                                ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Generated: $(date)"
    echo "Repository: $REPO_ROOT"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "QUICK STATS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Repository size
    du -sh . 2>/dev/null | awk '{print "Repository size: " $1}'

    # File counts
    find . -type f 2>/dev/null | wc -l | xargs printf "Total files: %'d\n"
    git ls-files 2>/dev/null | wc -l | xargs printf "Tracked files: %'d\n"

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "BUILD SYSTEMS DETECTED"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Maven
    MAVEN_COUNT=$(find . -name "pom.xml" -type f 2>/dev/null | wc -l | tr -d ' ')
    [ "$MAVEN_COUNT" -gt 0 ] && echo "✓ Maven: $MAVEN_COUNT pom.xml files"

    # Gradle
    GRADLE_COUNT=$(find . \( -name "build.gradle" -o -name "build.gradle.kts" \) -type f 2>/dev/null | wc -l | tr -d ' ')
    [ "$GRADLE_COUNT" -gt 0 ] && echo "✓ Gradle: $GRADLE_COUNT build files"

    # Bazel
    BUILD_COUNT=$(find . \( -name "BUILD" -o -name "BUILD.bazel" \) -type f 2>/dev/null | wc -l | tr -d ' ')
    if [ "$BUILD_COUNT" -gt 0 ]; then
        if [ -f "MODULE.bazel" ]; then
            echo "✓ Bazel: $BUILD_COUNT BUILD files (Bzlmod/MODULE.bazel)"
        else
            echo "✓ Bazel: $BUILD_COUNT BUILD files (Legacy WORKSPACE)"
        fi
    fi

    # Others
    NPM_COUNT=$(find . -name "package.json" -type f 2>/dev/null | wc -l | tr -d ' ')
    [ "$NPM_COUNT" -gt 0 ] && echo "✓ npm/yarn: $NPM_COUNT package.json files"

    GO_COUNT=$(find . -name "go.mod" -type f 2>/dev/null | wc -l | tr -d ' ')
    [ "$GO_COUNT" -gt 0 ] && echo "✓ Go: $GO_COUNT go.mod files"

    CARGO_COUNT=$(find . -name "Cargo.toml" -type f 2>/dev/null | wc -l | tr -d ' ')
    [ "$CARGO_COUNT" -gt 0 ] && echo "✓ Rust: $CARGO_COUNT Cargo.toml files"

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "KEY RECOMMENDATIONS FOR BAZBOM"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    TOTAL_BUILD_FILES=$(( MAVEN_COUNT + GRADLE_COUNT + BUILD_COUNT ))

    if [ "$TOTAL_BUILD_FILES" -gt 1000 ]; then
        echo "⚠ VERY LARGE MONOREPO ($TOTAL_BUILD_FILES build files)"
        echo "  → Recommend using --limit parameter for testing"
        echo "  → Enable caching for repeated scans"
        echo "  → Consider incremental scanning"
    elif [ "$TOTAL_BUILD_FILES" -gt 100 ]; then
        echo "⚠ LARGE MONOREPO ($TOTAL_BUILD_FILES build files)"
        echo "  → May benefit from --limit parameter"
        echo "  → Enable caching recommended"
    else
        echo "✓ MODERATE SIZE ($TOTAL_BUILD_FILES build files)"
        echo "  → Should scan efficiently without special tuning"
    fi

    echo ""

    if [ "$BUILD_COUNT" -gt 0 ]; then
        echo "📦 Bazel-based repository detected"
        echo "  → Review 05-bazel-analysis.txt for target counts"
        echo "  → Check maven_install.json for dependency lockfiles"
        if [ -f "MODULE.bazel" ]; then
            echo "  → Using Bzlmod (modern Bazel 6+ modules)"
        fi
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "DIAGNOSTIC FILES GENERATED"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    ls -lh "$DIAG_DIR"/*.txt 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
    echo ""
    echo "Total diagnostic data: $(du -sh "$DIAG_DIR" | awk '{print $1}')"
    echo ""

} > "$DIAG_DIR/00-SUMMARY.txt"

# Display summary to console
cat "$DIAG_DIR/00-SUMMARY.txt"

# ============================================================================
# CREATE TARBALL
# ============================================================================
section_header "Creating Archive"

cd "$REPO_ROOT"
TARBALL="bazbom-diagnostics.tar.gz"
tar -czf "$TARBALL" bazbom-diagnostics/

log_success "Created archive: $TARBALL ($(du -sh "$TARBALL" | awk '{print $1}'))"

# ============================================================================
# FINAL INSTRUCTIONS
# ============================================================================
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
log_success "Diagnostics complete!"
echo ""
echo "📁 Output location:"
echo "   Directory: $DIAG_DIR/"
echo "   Archive:   $TARBALL"
echo ""
echo "📤 To share diagnostics:"
echo "   1. Review files in $DIAG_DIR/ to ensure no sensitive data"
echo "   2. Share the entire directory or the $TARBALL file"
echo "   3. Start with 00-SUMMARY.txt for quick overview"
echo ""
echo "🔍 Key files to review:"
echo "   00-SUMMARY.txt           - Quick overview and recommendations"
echo "   03-build-systems.txt     - Build systems detected"
echo "   05-bazel-analysis.txt    - Bazel target counts and structure"
echo "   06-module-structure.txt  - Project/module organization"
echo "   07-dependency-patterns.txt - How dependencies are managed"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
