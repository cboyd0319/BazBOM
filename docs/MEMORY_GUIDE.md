# Memory System Guide for BazBOM

This document explains the Claude Code memory system setup for BazBOM and how to maintain it effectively.

---

## Overview

Claude Code uses a hierarchical memory system:

- **User Memory** (`~/.claude/CLAUDE.md`) - Personal preferences across all projects
- **Project Memory** (`./claude/CLAUDE.md`) - Team-shared project-specific knowledge

---

## Project Memory Location

**File:** `/Users/chad/Documents/GitHub/BazBOM/.claude/CLAUDE.md`

### What's Included

#### Build and Development Commands
- Complete cargo commands for building, testing, linting
- Installation procedures
- Test execution commands

#### Architecture and Code Organization
- Crate structure overview
- Key files and their purposes
- Critical code patterns (Bazel detection, logging)

#### Coding Standards
- Rust style guidelines
- Error handling patterns
- Documentation requirements
- Logging best practices

#### Common Workflows
- Making changes to Bazel support
- Adding new build system support
- Debugging scans
- Release process

#### Testing Infrastructure
- Test repository locations
- Key test scripts
- Expected results
- Running instructions

#### Important Historical Context
- Critical bug fixes (with dates and details)
- Lessons learned from each fix
- Architectural decisions and why they were made

#### Known Issues and Workarounds
- Current limitations
- Temporary solutions
- Future improvement plans

#### Quick Reference
- Most commonly used commands
- Performance expectations
- Documentation structure

### When to Update Project Memory

✅ **Update when:**
- Adding new features or modules
- Fixing significant bugs
- Changing build/test procedures
- Establishing new coding patterns
- Adding new dependencies
- Modifying architecture

❌ **Don't update for:**
- Routine bug fixes
- Minor refactoring
- Temporary experiments
- Work-in-progress features

---

## User Memory Location

**File:** `~/.claude/CLAUDE.md`

### What's Included

#### Development Philosophy
- Problem-solving approach
- Bug investigation patterns
- Code quality preferences

#### Technical Preferences
- Rust development standards
- Testing strategy
- Documentation style
- Git workflow

#### System Configuration
- Platform details (macOS, Apple Silicon)
- Development directory structure
- Tool preferences

#### Patterns That Work Well
- Efficient development cycles
- Debugging strategies
- Big change management

#### Communication Preferences
- Direct, technical style
- No corporate speak
- Humor and tone guidelines

#### Project-Specific Shortcuts
- Quick aliases for common tasks
- Environment variables
- Common paths

### When to Update User Memory

✅ **Update when:**
- Discovering new personal preferences
- Establishing new workflow patterns
- Adding project shortcuts/aliases
- Changing development environment
- Learning better approaches

❌ **Don't update for:**
- Project-specific details (put those in project memory)
- Temporary preferences
- Team-wide standards

---

## Using Memory Effectively

### Importing Additional Files

Both memory types support imports using `@path/to/file` syntax:

```markdown
# In project memory
@~/Documents/BazBOM_Testing/README.md

# In user memory
@~/.config/personal-dev-prefs.md
```

### Checking Current Memory

```bash
# View what Claude is currently loading
/memory
```

### Quick Memory Updates

```bash
# Quick addition to memory
# Type in chat:
Add to memory: Always use tracing::info! instead of println! for debug output
```

### Extensive Memory Edits

```bash
# Edit memory files directly
code ~/.claude/CLAUDE.md
code ~/Documents/GitHub/BazBOM/.claude/CLAUDE.md
```

---

## Memory Maintenance Best Practices

### Keep Information Current
- Review project memory quarterly
- Update after major changes
- Remove outdated information
- Verify commands still work

### Organize with Structure
- Use clear markdown headings
- Group related information
- Keep sections focused
- Use bullet points for clarity

### Be Specific
```markdown
# ❌ Vague
- Use good coding style

# ✅ Specific
- Use `tracing` crate for logging, not `eprintln!`
- Enable with `RUST_LOG=debug bazbom scan .`
```

### Include Examples
```markdown
# Bazel Detection Pattern
\`\`\`rust
if system == bazbom_core::BuildSystem::Bazel {
    let maven_install_json = workspace.join("maven_install.json");
    // ...
}
\`\`\`
```

### Document "Why" Not Just "What"
```markdown
# ❌ Just "what"
- Update scan.rs and scan_orchestrator.rs

# ✅ Include "why"
- Update both scan.rs AND scan_orchestrator.rs
  (Both paths handle scans - need consistency)
```

---

## Integration with Version Control

### Project Memory
✅ **Should be committed to Git**
- Part of repository
- Shared with team
- Version controlled
- Reviewed in PRs

### User Memory
❌ **Should NOT be committed**
- Personal preferences
- Lives in home directory
- Not project-specific
- Private to individual

---

## Example Workflow: Adding New Feature

1. **Check Project Memory** - Review relevant sections
2. **Implement Feature** - Follow established patterns
3. **Update Project Memory** - Add new patterns/commands
4. **Update User Memory** (if needed) - Add personal shortcuts
5. **Test** - Verify memory is accurate
6. **Commit** - Include project memory updates in PR

---

## Memory Hierarchy

```
Claude Code Memory System
│
├── User Memory (~/.claude/CLAUDE.md)
│   ├── Personal communication style
│   ├── Development philosophy
│   ├── Technical preferences (all projects)
│   ├── System configuration
│   └── Project shortcuts
│
└── Project Memory (./.claude/CLAUDE.md)
    ├── Build commands
    ├── Architecture
    ├── Coding standards
    ├── Common workflows
    ├── Testing infrastructure
    ├── Historical context
    └── Quick reference
```

---

## Current Memory Status

### Project Memory
- ✅ Created: 2025-11-18
- ✅ Comprehensive coverage of BazBOM project
- ✅ Includes recent bug fix documentation
- ✅ Ready for team use

### User Memory
- ✅ Updated: 2025-11-18
- ✅ Enhanced with structured technical preferences
- ✅ Includes workflow patterns discovered during recent work
- ✅ Maintains Chad's communication style

---

## Quick Memory Commands

```bash
# View current memory
/memory

# Quick add to memory
# (type in chat)
Add to memory: [your information here]

# Edit project memory
code ~/Documents/GitHub/BazBOM/.claude/CLAUDE.md

# Edit user memory
code ~/.claude/CLAUDE.md

# Initialize new project memory
/init

# Check what files are loaded
/memory
```

---

## Tips for Effective Memory Usage

### 1. Use Consistent Formatting
- Markdown headings for organization
- Bullet points for lists
- Code blocks for commands/patterns
- Tables for comparisons

### 2. Keep Commands Executable
- Test commands before adding
- Include full paths when needed
- Show expected output
- Add error cases

### 3. Link to Documentation
```markdown
See `docs/FIXES_SUMMARY.md` for full technical details
Reference: [ARCHITECTURE.md](./ARCHITECTURE.md)
```

### 4. Update After Learning
When you discover something important:
1. Add it immediately
2. Be specific
3. Include context
4. Test the information

### 5. Review Regularly
- Before major changes
- After completing features
- When onboarding team members
- Quarterly maintenance

---

## Common Mistakes to Avoid

❌ **Don't:**
- Put temporary information in memory
- Include work-in-progress details
- Add untested commands
- Mix personal and team preferences
- Forget to update after major changes

✅ **Do:**
- Keep information current
- Test all commands
- Separate user vs project memory
- Document "why" decisions were made
- Update after significant changes

---

## Memory as Knowledge Base

Think of memory as your project's institutional knowledge:

- **New Team Members** - Quick onboarding reference
- **Future You** - Remember why decisions were made
- **Code Reviews** - Ensure consistency with standards
- **Debugging** - Quick access to troubleshooting patterns
- **Documentation** - Living reference that evolves with project

---

## Verification Checklist

After updating memory:

- [ ] Commands work as documented
- [ ] Information is current and accurate
- [ ] Organization is clear and logical
- [ ] Examples are correct
- [ ] Links to docs work
- [ ] Personal vs project info separated correctly
- [ ] Committed to git (project memory only)

---

## Support and Questions

- **Claude Code Docs:** https://code.claude.com/docs/en/memory
- **Project Issues:** https://github.com/cboyd0319/BazBOM/issues
- **Memory Location:** Check with `/memory` command

---

**Last Updated:** 2025-11-18
**Status:** ✅ Fully configured and documented

