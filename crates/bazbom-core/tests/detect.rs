use bazbom_core::{detect_build_system, BuildSystem};
use std::fs;
use tempfile::tempdir;

#[test]
fn detect_maven() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("pom.xml"), "<project/>").unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Maven);
}

#[test]
fn detect_gradle() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("build.gradle.kts"), "plugins{}").unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Gradle);
}

#[test]
fn detect_bazel() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("MODULE.bazel"), "module()").unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Bazel);
}

#[test]
fn detect_ant() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("build.xml"), "<project/>").unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Ant);
}

#[test]
fn detect_buildr_buildfile() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("buildfile"), "# Buildr buildfile").unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Buildr);
}

#[test]
fn detect_buildr_rakefile() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Rakefile"),
        "require 'buildr'\n\nBuildr.application"
    ).unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Buildr);
}

#[test]
fn detect_buildr_rakefile_double_quotes() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Rakefile"),
        "require \"buildr\"\n\ntask :build"
    ).unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Buildr);
}

#[test]
fn detect_unknown() {
    let dir = tempdir().unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Unknown);
}

#[test]
fn detect_priority_maven_over_ant() {
    // If both pom.xml and build.xml exist, Maven should be detected (higher priority)
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("pom.xml"), "<project/>").unwrap();
    fs::write(dir.path().join("build.xml"), "<project/>").unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Maven);
}

#[test]
fn detect_rakefile_without_buildr() {
    // Regular Rakefile without Buildr should not be detected as Buildr
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Rakefile"),
        "task :default do\n  puts 'Hello'\nend"
    ).unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Unknown);
}
