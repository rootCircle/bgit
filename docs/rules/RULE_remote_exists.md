# 📘 Git Rule Specification: Git Remote Exists

**Rule ID**: `RULE_remote_exists`  
**Status**: Draft  
**Author**: rootCircle | GitHub Copilot  
**Created**: 2025-08-04  
**Updated**: 2025-08-04  
**Version**: v1.0.0  
**RuleLevel**: Error

---

## 1. Summary

> Ensures that required Git remotes (especially 'origin') exist before performing operations that depend on them like `git pull` or `git push`.

## 2. Scope

### Applies To

- [x] Developers (local)
- [x] CI/CD pipelines
- [ ] GitHub/GitLab Web UI
- [x] Hooks (pre-commit, pre-push, etc.)
- [ ] Git config/templates

### Affects

- [ ] Commits  
- [ ] Branching  
- [ ] Merges  
- [x] Pushes  
- [ ] Repository layout
- [x] Miscellaneous

### Trigger Point (When to Check)

Before performing `git pull`, `git push`, or any other remote operations.

This rule must run **before** remote operations to prevent confusing error messages when remotes are missing.

---

## 3. Motivation

### Problem Statement

Developers often encounter confusing error messages when attempting to pull or push changes in repositories where no remotes have been configured. This leads to workflow interruptions and unclear guidance on how to resolve the issue.

### Objectives

- Prevent confusing runtime errors during push/pull operations
- Provide clear, actionable error messages when remotes are missing
- Ensure Git workflows don't fail at critical steps due to missing remote configuration
- Help developers understand repository setup requirements

### Common Pitfall

A developer clones a repository or initializes a new one but forgets to add the origin remote, then attempts to push changes and receives cryptic error messages about missing remotes.

---

## 4. Rule Definition

### Description

This rule validates that essential Git remotes are configured in the repository before attempting operations that require them. The most common remote is 'origin', but this rule can be configured to check for other remotes as needed.

**Allowed:**  

- Repositories with properly configured 'origin' remote
- Repositories with the required remote(s) for the specific operation

**Forbidden:**  

- Attempting push/pull operations when no 'origin' remote exists
- Remote operations without the necessary remote configuration

---

## 5. Examples

### ✅ Correct Usage

#### Repository with origin remote configured

```bash
$ git remote -v
origin  git@github.com:user/repo.git (fetch)
origin  git@github.com:user/repo.git (push)
```

#### Repository with multiple remotes

```bash
$ git remote -v
origin     git@github.com:user/repo.git (fetch)
origin     git@github.com:user/repo.git (push)
upstream   git@github.com:original/repo.git (fetch)
upstream   git@github.com:original/repo.git (push)
```

### ❌ Incorrect Usage

#### Repository with no remotes

```bash
$ git remote -v
# (no output - no remotes configured)
```

#### Repository missing origin remote

```bash
$ git remote -v
upstream   git@github.com:original/repo.git (fetch)
upstream   git@github.com:original/repo.git (push)
# origin remote is missing
```

---

## 6. Impact Assessment

### Frequency of Violation

- [ ] Rare  
- [x] Occasional  
- [ ] Frequent  

### Severity When Violated

- [ ] Pedantic (minor)  
- [ ] Low  
- [x] Medium  
- [ ] High  
- [ ] Critical

---

## 7. Enforcement Strategy

### Pseudocode / Workflow

```bash
# Check if origin remote exists
if ! git remote | grep -q "^origin$"; then
  echo "ERROR: Required remote 'origin' does not exist."
  echo "Please add a remote using: git remote add origin <url>"
  exit 1
fi

# Optional: Check for other required remotes
REQUIRED_REMOTES=("origin")
for remote in "${REQUIRED_REMOTES[@]}"; do
  if ! git remote | grep -q "^${remote}$"; then
    echo "ERROR: Required remote '${remote}' does not exist."
    exit 1
  fi
done
```

### Suggested Tooling

- Pre-push/pre-pull validation hooks
- Repository setup verification scripts
- CI/CD pipeline checks
- bgit workflow pre-checks

---

## 8. Possible Fixes

### Manual Fix

Add the missing remote manually:

```bash
# Add origin remote
git remote add origin <repository-url>

# Example with SSH
git remote add origin git@github.com:user/repo.git

# Example with HTTPS
git remote add origin https://github.com/user/repo.git

# Verify the remote was added
git remote -v
```

### Automated Fix Suggestion

The rule can detect missing remotes but typically cannot automatically fix them, as it requires user input for the correct repository URL. However, it can provide clear instructions and suggest common patterns based on the repository context.

---

## 9. Exceptions & Edge Cases

- Local-only repositories where remotes are not needed
- Repositories using non-standard remote names (e.g., 'upstream' instead of 'origin')
- Temporary workflows where remotes will be added later
- Special deployment scenarios with custom remote configurations

---

## 10. Drawbacks

> May be too strict for local-only development workflows or experimental repositories where remotes are not immediately required.

---

## 11. Related Rules / RFCs

- `RULE_git_remote_http_ssh` - Validates remote URL schemes
- `RULE_github_credentials_http` - Ensures proper HTTP authentication
- `RULE_github_credentials_ssh` - Ensures proper SSH authentication
- `RULE_git_default_config` - Basic Git configuration validation

---

## 12. Revision History

| Date       | Version | Author           | Notes                        |
|------------|---------|------------------|------------------------------|
| 2025-08-04 | 1.0.0   | rootCircle & GitHub Copilot   | Initial draft                |

---

## 13. Glossary

| Term          | Definition                                                  |
|---------------|--------------------------------------------------------------|
| Remote        | A reference to a repository hosted elsewhere (e.g., GitHub, GitLab) |
| Origin        | The default name for the primary remote repository |
| Pre-check     | Validation performed before executing an operation |

---

## 14. References

- <https://git-scm.com/docs/git-remote>
- <https://git-scm.com/book/en/v2/Git-Basics-Working-with-Remotes>
- <https://docs.github.com/en/get-started/getting-started-with-git/about-remote-repositories>

---
