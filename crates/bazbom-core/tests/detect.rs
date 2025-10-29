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
fn detect_unknown() {
    let dir = tempdir().unwrap();
    assert_eq!(detect_build_system(dir.path()), BuildSystem::Unknown);
}
