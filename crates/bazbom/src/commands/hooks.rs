use anyhow::Result;
use bazbom::hooks::{install_hooks, HooksConfig};

/// Handle the `bazbom install-hooks` command
pub fn handle_install_hooks(policy: String, fast: bool) -> Result<()> {
    println!("[bazbom] installing pre-commit hooks");
    let config = HooksConfig {
        policy_file: policy,
        fast_mode: fast,
    };
    install_hooks(&config)
}
