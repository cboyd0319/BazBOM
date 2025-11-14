# Shell Completions for BazBOM

BazBOM CLI commands can be made easier with shell completions for Bash, Zsh, Fish, PowerShell, and Elvish.

---

## Quick Install

### Bash

```bash
# macOS
brew install bash-completion

# Linux
sudo mkdir -p /etc/bash_completion.d/
# Generate completions (requires clap_complete feature)
# Coming soon - completions will be auto-generated in future release
```

### Zsh

```bash
# macOS
brew install zsh-completions

# Manual install
mkdir -p ~/.zsh/completion
# Add to ~/.zshrc:
fpath=(~/.zsh/completion $fpath)
autoload -Uz compinit && compinit
```

### Fish

```bash
# Manual install
mkdir -p ~/.config/fish/completions/
# Completions will be auto-generated in future release
```

---

## Manual Completion Setup (Temporary Workaround)

Until automatic completion generation is implemented, you can use these manual completions:

### Bash Completion

Create `~/.bash_completion.d/bazbom`:

```bash
#!/bin/bash

_bazbom_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Top-level commands
    local commands="scan policy fix db license install-hooks init explore dashboard team report help"

    # Flags for scan command
    local scan_flags="--reachability --fast --format --out-dir --bazel-targets-query --bazel-targets --bazel-affected-by-files --bazel-universe --cyclonedx --with-semgrep --with-codeql --autofix --containers --no-upload --target --incremental --base --benchmark --ml-risk --help"

    # Format options
    local formats="spdx cyclonedx"

    # CodeQL suites
    local codeql_suites="default security-extended"

    # Autofix modes
    local autofix_modes="off dry-run pr"

    # Container strategies
    local container_strategies="auto syft bazbom"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${commands}" -- ${cur}) )
        return 0
    fi

    case "${prev}" in
        scan)
            COMPREPLY=( $(compgen -W "${scan_flags}" -- ${cur}) )
            ;;
        --format)
            COMPREPLY=( $(compgen -W "${formats}" -- ${cur}) )
            ;;
        --with-codeql)
            COMPREPLY=( $(compgen -W "${codeql_suites}" -- ${cur}) )
            ;;
        --autofix)
            COMPREPLY=( $(compgen -W "${autofix_modes}" -- ${cur}) )
            ;;
        --containers)
            COMPREPLY=( $(compgen -W "${container_strategies}" -- ${cur}) )
            ;;
        --out-dir|--base)
            COMPREPLY=( $(compgen -d -- ${cur}) )
            ;;
        policy)
            COMPREPLY=( $(compgen -W "check init validate --help" -- ${cur}) )
            ;;
        fix)
            COMPREPLY=( $(compgen -W "--suggest --apply --pr --interactive --ml-prioritize --llm --llm-provider --llm-model --help" -- ${cur}) )
            ;;
        db)
            COMPREPLY=( $(compgen -W "sync --help" -- ${cur}) )
            ;;
        license)
            COMPREPLY=( $(compgen -W "obligations compatibility contamination --help" -- ${cur}) )
            ;;
        *)
            COMPREPLY=( $(compgen -f -- ${cur}) )
            ;;
    esac
}

complete -F _bazbom_completions bazbom
```

Source it in your `~/.bashrc`:

```bash
echo 'source ~/.bash_completion.d/bazbom' >> ~/.bashrc
source ~/.bashrc
```

### Zsh Completion

Create `~/.zsh/completion/_bazbom`:

```bash
#compdef bazbom

_bazbom() {
    local -a commands
    commands=(
        'scan:Scan a project and generate SBOM + findings'
        'policy:Apply policy checks and output SARIF/JSON verdicts'
        'fix:Show remediation suggestions or apply fixes'
        'db:Advisory database operations (offline sync)'
        'license:License compliance operations'
        'install-hooks:Install git pre-commit hooks for vulnerability scanning'
        'init:Interactive setup wizard for new projects'
        'explore:Interactive dependency graph explorer (TUI)'
        'dashboard:Start web dashboard server'
        'team:Team coordination and assignment management'
        'report:Generate security and compliance reports'
        'help:Print help message'
    )

    _arguments -C \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe 'command' commands
            ;;
        args)
            case $words[1] in
                scan)
                    _arguments \
                        '--reachability[Enable reachability analysis]' \
                        '--fast[Fast mode: skip reachability analysis]' \
                        '--format=[Output format]:format:(spdx cyclonedx)' \
                        '--out-dir=[Output directory]:directory:_files -/' \
                        '--bazel-targets-query=[Bazel query expression]:query:' \
                        '--bazel-targets=[Explicit Bazel targets]:targets:' \
                        '--bazel-affected-by-files=[Files for incremental scan]:files:_files' \
                        '--cyclonedx[Also emit CycloneDX SBOM]' \
                        '--with-semgrep[Run Semgrep analysis]' \
                        '--with-codeql=[Run CodeQL analysis]:suite:(default security-extended)' \
                        '--autofix=[Generate OpenRewrite recipes]:mode:(off dry-run pr)' \
                        '--containers=[Container SBOM strategy]:strategy:(auto syft bazbom)' \
                        '--incremental[Enable incremental analysis]' \
                        '--base=[Git base reference]:ref:' \
                        '--benchmark[Enable performance benchmarking]' \
                        '--ml-risk[Use ML-enhanced risk scoring]' \
                        '*:path:_files -/'
                    ;;
                policy)
                    _arguments \
                        '1: :(check init validate)'
                    ;;
                fix)
                    _arguments \
                        '--suggest[Suggest fixes without applying]' \
                        '--apply[Apply fixes automatically]' \
                        '--pr[Create pull request with fixes]' \
                        '--interactive[Interactive mode]' \
                        '--ml-prioritize[Use ML-enhanced prioritization]' \
                        '--llm[Use LLM-powered fix generation]' \
                        '--llm-provider=[LLM provider]:provider:(ollama anthropic openai)' \
                        '--llm-model=[LLM model]:model:'
                    ;;
                db)
                    _arguments \
                        '1: :(sync)'
                    ;;
                license)
                    _arguments \
                        '1: :(obligations compatibility contamination)'
                    ;;
            esac
            ;;
    esac
}

_bazbom "$@"
```

Add to `~/.zshrc`:

```bash
fpath=(~/.zsh/completion $fpath)
autoload -Uz compinit && compinit
```

### Fish Completion

Create `~/.config/fish/completions/bazbom.fish`:

```fish
# bazbom completions for Fish shell

# Commands
complete -c bazbom -f -n '__fish_use_subcommand' -a 'scan' -d 'Scan a project and generate SBOM + findings'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'policy' -d 'Apply policy checks'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'fix' -d 'Show remediation suggestions or apply fixes'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'db' -d 'Advisory database operations'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'license' -d 'License compliance operations'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'install-hooks' -d 'Install git pre-commit hooks'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'init' -d 'Interactive setup wizard'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'explore' -d 'Interactive dependency graph explorer'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'dashboard' -d 'Start web dashboard server'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'team' -d 'Team coordination and assignment'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'report' -d 'Generate security reports'

# Scan command options
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l reachability -d 'Enable reachability analysis'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l fast -d 'Fast mode: skip reachability analysis'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l format -a 'spdx cyclonedx' -d 'Output format'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l out-dir -r -F -d 'Output directory'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l cyclonedx -d 'Also emit CycloneDX SBOM'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l with-semgrep -d 'Run Semgrep analysis'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l with-codeql -a 'default security-extended' -d 'Run CodeQL analysis'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l autofix -a 'off dry-run pr' -d 'Generate OpenRewrite recipes'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l containers -a 'auto syft bazbom' -d 'Container SBOM strategy'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l incremental -d 'Enable incremental analysis'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l benchmark -d 'Enable performance benchmarking'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l ml-risk -d 'Use ML-enhanced risk scoring'

# Policy subcommands
complete -c bazbom -n '__fish_seen_subcommand_from policy' -a 'check' -d 'Run policy checks'
complete -c bazbom -n '__fish_seen_subcommand_from policy' -a 'init' -d 'Initialize a policy template'
complete -c bazbom -n '__fish_seen_subcommand_from policy' -a 'validate' -d 'Validate a policy file'

# Fix command options
complete -c bazbom -n '__fish_seen_subcommand_from fix' -l suggest -d 'Suggest fixes without applying'
complete -c bazbom -n '__fish_seen_subcommand_from fix' -l apply -d 'Apply fixes automatically'
complete -c bazbom -n '__fish_seen_subcommand_from fix' -l pr -d 'Create pull request with fixes'
complete -c bazbom -n '__fish_seen_subcommand_from fix' -l interactive -d 'Interactive mode'
complete -c bazbom -n '__fish_seen_subcommand_from fix' -l ml-prioritize -d 'Use ML-enhanced prioritization'
complete -c bazbom -n '__fish_seen_subcommand_from fix' -l llm -d 'Use LLM-powered fix generation'
complete -c bazbom -n '__fish_seen_subcommand_from fix' -l llm-provider -a 'ollama anthropic openai' -d 'LLM provider'

# DB subcommands
complete -c bazbom -n '__fish_seen_subcommand_from db' -a 'sync' -d 'Sync local advisory mirrors'

# License subcommands
complete -c bazbom -n '__fish_seen_subcommand_from license' -a 'obligations' -d 'Generate license obligations report'
complete -c bazbom -n '__fish_seen_subcommand_from license' -a 'compatibility' -d 'Check license compatibility'
complete -c bazbom -n '__fish_seen_subcommand_from license' -a 'contamination' -d 'Detect copyleft contamination'
```

---

## Future: Automatic Completion Generation

BazBOM will include automatic shell completion generation in a future release using `clap_complete`. This will add:

```bash
# Generate completions
bazbom completions bash > /etc/bash_completion.d/bazbom
bazbom completions zsh > ~/.zsh/completion/_bazbom
bazbom completions fish > ~/.config/fish/completions/bazbom.fish
bazbom completions powershell > bazbom.ps1
bazbom completions elvish > bazbom.elv
```

---

## Testing Completions

After installing completions, test them:

```bash
# Type and press TAB
bazbom <TAB>        # Should show: scan, policy, fix, db, license, ...
bazbom scan <TAB>   # Should show: --reachability, --fast, --format, ...
bazbom scan --format <TAB>   # Should show: spdx, cyclonedx
```

---

## Package Manager Status

BazBOM does not yet ship via Homebrew or any other package manager, so completions are not auto-installed. Use the manual snippets above and regenerate them after you rebuild the CLI.

---

## Troubleshooting

### Completions Not Working

**Bash:**
```bash
# Check if bash-completion is installed
dpkg -l | grep bash-completion  # Debian/Ubuntu
brew list | grep bash-completion  # macOS

# Source completions manually
source ~/.bash_completion.d/bazbom
```

**Zsh:**
```bash
# Check fpath includes completion directory
echo $fpath

# Rebuild completion cache
rm ~/.zcompdump
autoload -Uz compinit && compinit
```

**Fish:**
```bash
# Check completions path
echo $fish_complete_path

# Reload completions
fish_update_completions
```

### Permission Issues

```bash
# Make completion files readable
chmod +r ~/.bash_completion.d/bazbom
chmod +r ~/.zsh/completion/_bazbom
chmod +r ~/.config/fish/completions/bazbom.fish
```

---

## Contributing

To add or improve shell completions:

1. Edit completion scripts above
2. Test with all supported shells
3. Submit PR with examples and tests

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for details.

---

## References

- [Clap Documentation](https://docs.rs/clap/latest/clap/)
- [Bash Completion Tutorial](https://github.com/scop/bash-completion)
- [Zsh Completion Guide](https://zsh.sourceforge.net/Doc/Release/Completion-System.html)
- [Fish Completion Guide](https://fishshell.com/docs/current/completions.html)
