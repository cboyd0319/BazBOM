workspace(name = "bazbom")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# bazel_skylib for common Starlark utilities (json encoding, etc.)
http_archive(
    name = "bazel_skylib",
    sha256 = "bc283cdfcd526a52c3201279cda4bc298652efa898b10b4db0837dc51652756f",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/1.7.1/bazel-skylib-1.7.1.tar.gz",
        "https://github.com/bazelbuild/bazel-skylib/releases/download/1.7.1/bazel-skylib-1.7.1.tar.gz",
    ],
)

load("@bazel_skylib//:workspace.bzl", "bazel_skylib_workspace")

bazel_skylib_workspace()

# rules_java for Java support and JavaInfo provider
http_archive(
    name = "rules_java",
    urls = [
        "https://github.com/bazelbuild/rules_java/releases/download/7.11.1/rules_java-7.11.1.tar.gz",
    ],
    sha256 = "6f3ce0e9fba979a844faba2d60467843fbf5191d8ca61fa3d2ea17655b56bb8c",
)

load("@rules_java//java:repositories.bzl", "rules_java_dependencies", "rules_java_toolchains")

rules_java_dependencies()
rules_java_toolchains()

# rules_jvm_external for Maven dependency management
http_archive(
    name = "rules_jvm_external",
    strip_prefix = "rules_jvm_external-6.0",
    sha256 = "85fd6bad58ac76cc3a27c8e051e4255ff9ccd8c92ba879670d195622e7c0a9b7",
    url = "https://github.com/bazelbuild/rules_jvm_external/releases/download/6.0/rules_jvm_external-6.0.tar.gz",
)

load("@rules_jvm_external//:repositories.bzl", "rules_jvm_external_deps")

rules_jvm_external_deps()

load("@rules_jvm_external//:setup.bzl", "rules_jvm_external_setup")

rules_jvm_external_setup()

# Maven dependencies for examples
load("@rules_jvm_external//:defs.bzl", "maven_install")

maven_install(
    name = "maven",
    artifacts = [
        "com.google.guava:guava:31.1-jre",
    ],
    repositories = [
        "https://repo1.maven.org/maven2",
    ],
    maven_install_json = "//:maven_install.json",
    fail_if_repin_required = False,
)

load("@maven//:defs.bzl", "pinned_maven_install")

pinned_maven_install()
