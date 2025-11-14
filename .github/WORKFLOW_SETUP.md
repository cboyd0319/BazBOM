# GitHub Actions Workflow Setup

## Version Bump Automation Setup

The version bump automation workflow requires additional repository configuration to function properly.

### Issue

GitHub Actions workflows using `GITHUB_TOKEN` cannot create pull requests by default due to a security restriction. This prevents automated workflows from triggering additional workflow runs.

**Error message:**
```
Error: GitHub Actions is not permitted to create or approve pull requests.
```

### Solution Options

You have two options to fix this issue:

#### Option 1: Enable Repository Setting (Recommended)

This is the simplest solution and works for most use cases.

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Actions** → **General**
3. Scroll down to **Workflow permissions**
4. Check the box: **"Allow GitHub Actions to create and approve pull requests"**
5. Click **Save**

**Affected workflows:**
- `version-bump.yml` - Version bump automation
- `dependency-updates.yml` - Automated dependency updates

#### Option 2: Use a Personal Access Token (PAT)

If you cannot or do not want to enable the repository setting, you can use a Personal Access Token instead.

1. Create a Personal Access Token (classic) with `repo` scope:
   - Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Click "Generate new token (classic)"
   - Give it a descriptive name like "BazBOM Workflow Automation"
   - Select the `repo` scope (full control of private repositories)
   - Set an appropriate expiration date
   - Click "Generate token" and copy the token

2. Add the token as a repository secret:
   - Go to your repository Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `PAT_TOKEN`
   - Value: Paste your personal access token
   - Click "Add secret"

3. The workflows are already configured to use `PAT_TOKEN` if available, falling back to `GITHUB_TOKEN` otherwise.

### Verification

After applying either solution, test the workflow:

1. Go to **Actions** → **Version Bump Automation**
2. Click **Run workflow**
3. Enter a test version number (e.g., `0.2.1`)
4. The workflow should now successfully create a pull request

### Security Considerations

- **Option 1** allows any workflow in your repository to create PRs, which is generally safe for trusted repositories
- **Option 2** provides more granular control but requires token management and periodic renewal
- Both options maintain the security boundary by requiring explicit configuration

### Troubleshooting

If you continue to experience issues:

1. Verify the workflow has the correct permissions in the YAML file:
   ```yaml
   permissions:
     contents: write
     pull-requests: write
   ```

2. Check that the `peter-evans/create-pull-request` action is up to date (currently using v7)

3. Ensure your repository is not restricted by organization-level policies that override these settings

4. Review the workflow run logs for specific error messages

### Additional Resources

- [GitHub Actions: Workflow permissions](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository#setting-the-permissions-of-the-github_token-for-your-repository)
- [peter-evans/create-pull-request documentation](https://github.com/peter-evans/create-pull-request)
- [Creating a personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
