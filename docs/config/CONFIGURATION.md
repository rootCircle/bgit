# BGit Configuration

BGit uses a configuration file to customize rule behavior and workflow step flags. This document explains how to configure BGit for your project.

## Configuration File Location

BGit looks for its configuration file at:

```
<repository_root>/.bgit/config.toml
```

The configuration file is automatically discovered by:

1. Finding the Git repository root using `git2::Repository::discover()`
2. Looking for config.toml in that root directory
3. If not in a Git repository, it looks in the current directory

## Configuration File Format

The configuration file uses TOML format and supports two main sections:

### 1. Rules Configuration

Configure rule behavior using the `[rules]` section:

```toml
[rules.workflow_name]
rule_name = "Level"
```

**Structure:**

- `workflow_name` - Name of the workflow (e.g., `default`)

**Available Rule Levels:**

- `"Skip"` - Skip the rule check entirely
- `"Warning"` - Emit a warning if rule fails, try to fix, but continue
- `"Error"` - Emit an error if rule fails, try to fix, but stop if not fixable

**Example:**

```toml
[rules.default]
a01_git_install = "Warning"
a02_git_name_email_setup = "Error"
a12_no_secrets_staged = "Skip"
a16_no_large_file = "Warning"
a17_conventional_commit_message = "Error"
```

### 2. Workflow Configuration

Configure workflow step flags using the `[workflow]` section:

```toml
[workflow.workflow_name.step_name]
flag_name = value
```

**Structure:**

- `workflow_name` - Name of the workflow (e.g., `default`)
- `step_name` - Name of the step within the workflow (e.g., `git_commit`, `git_push`, `is_sole_contributor`)
- `flag_name` - Name of the flag to override
- `value` - Value to set (can be boolean, integer, string, etc.)

**Example:**

```toml
[workflow.default.is_sole_contributor]
overrideCheckForAuthors = ["Name <email@gmail.com>"]
```

## Complete Configuration Example

```toml
# Rules configuration
[rules.default]
a01_git_install = "Warning"
a02_git_name_email_setup = "Error"
a12_no_secrets_staged = "Skip"
a14_big_repo_size = "Warning"
a16_no_large_file = "Error"
a17_conventional_commit_message = "Warning"

# Workflow configurations
[workflow.default.is_sole_contributor]
overrideCheckForAuthors = ["Name <email@gmail.com>"]
```

## Default Behavior

- If the configuration file doesn't exist, BGit uses sensible defaults
- Rules without explicit configuration use their default levels
- Workflow flags without configuration use their default values
- All configuration sections are optional

## File Creation

You can also create the file manually:

```bash
mkdir -p .bgit
touch .bgit/config.toml
```

## Best Practices

1. **Start Small**: Begin with a minimal configuration and add settings as needed
2. **Team Configuration**: Commit the config.toml file to share settings with your team
3. **Environment-Specific**: Use different configurations for different environments
4. **Documentation**: Comment your configuration choices for team members

## Troubleshooting

### Configuration Not Loading

- Ensure you're in a Git repository or the file is in the current directory
- Check file permissions (must be readable)
- Verify TOML syntax using a TOML validator

### Invalid Configuration Values

- Rule levels must be exactly: `"Skip"`, `"Warning"`, or `"Error"`
- Flag values must match expected types (boolean, integer, string)
- Workflow and step names must match exactly (case-sensitive)
