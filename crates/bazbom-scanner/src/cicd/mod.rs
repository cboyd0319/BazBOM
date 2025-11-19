pub mod github_actions;
// Future: pub mod gitlab_ci;
// Future: pub mod circleci;
// Future: pub mod azure_pipelines;

pub use github_actions::detect_github_actions;
