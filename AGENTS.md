# AGENTS.md

This file provides specialized guidance for AI agents working with the bgit codebase.

## Project Architecture Summary

bgit is a sophisticated Git workflow automation tool built in Rust with a trait-based, step-driven architecture. The system orchestrates complex Git workflows through interactive steps while enforcing smart rules to prevent common Git mistakes.

### Core Architecture Components

1. **Step-Based Execution Engine** (`src/step.rs` + `src/workflow_queue.rs`)
2. **Rules Validation System** (`src/rules/`)
3. **Atomic Git Events** (`src/events/`)
4. **Cross-Platform Authentication** (`src/auth/` with SSH agent management)
5. **Two-Tier Configuration** (local + global config system)
6. **Hook Execution Framework** (platform-specific implementations)

## Key Traits and Interfaces

### Workflow Execution Traits
```rust
// Core step execution
pub(crate) trait ActionStep {
    fn execute(&self) -> Result<Step, Box<BGitError>>;
}

pub(crate) trait PromptStep {
    fn execute(&self) -> Result<Step, Box<BGitError>>;
}

// Git operations
pub(crate) trait AtomicEvent<'a> {
    fn execute(&self) -> Result<bool, Box<BGitError>>;
    fn pre_execute_hook(&self) -> Result<bool, Box<BGitError>>;
    fn post_execute_hook(&self) -> Result<bool, Box<BGitError>>;
}

// Validation rules
pub(crate) trait Rule {
    fn check(&self) -> Result<RuleOutput, Box<BGitError>>;
    fn try_fix(&self) -> Result<bool, Box<BGitError>>;
}
```

## Development Patterns and Conventions

### Naming Conventions
- **ActionSteps**: `ta##_descriptive_name.rs` (e.g., `ta01_is_git_repo.rs`)
- **PromptSteps**: `pa##_ask_descriptive_action.rs` (e.g., `pa05_ask_to_add.rs`)
- **Rules**: `a##_rule_description.rs` (e.g., `a12_no_secrets_staged.rs`)
- **Events**: `git_operation.rs` (e.g., `git_commit.rs`, `git_push.rs`)

### File Structure Patterns
- Each module typically has a single public struct implementing the relevant trait
- Error handling uses `BGitError` with contextual information
- Configuration access through `BGitConfig` and `BGitGlobalConfig`
- Cross-platform implementations use conditional compilation (`#[cfg(unix)]`, `#[cfg(windows)]`)

### Code Organization
```
src/
├── main.rs                          # Entry point and CLI routing
├── step.rs                          # Core execution model
├── workflow_queue.rs                # Workflow execution engine
├── bgit_error.rs                    # Structured error handling
├── rules.rs + rules/                # validation rules
├── events.rs + events/              # Git operations
├── auth/ + auth/ssh/                # Authentication and SSH management
├── config/                          # Two-tier configuration system
├── workflows/default/action/        # ActionStep implementations
├── workflows/default/prompt/        # PromptStep implementations
├── hook_executor/                   # Cross-platform hook execution
├── llm_tools/                       # AI integration components
└── cmd/                             # CLI command implementations
```

## Implementation Guidelines for AI Agents

### When Adding New Rules
1. **Rule Structure**: Implement `Rule` trait in `src/rules/aXX_rule_name.rs`
2. **Error Handling**: Use `BGitError` with appropriate workflow type
3. **Configuration**: Add rule to `WorkflowRules` enum if configurable
4. **Auto-fixing**: Implement `try_fix()` for recoverable failures
5. **Testing**: Focus on edge cases and cross-platform compatibility

### When Adding New Workflow Steps
1. **Step Type**: Choose `ActionStep` (automated) vs `PromptStep` (interactive)
2. **Step Progression**: Define clear next step logic in `execute()` return
3. **State Management**: Use Git repository state to determine flow
4. **Error Recovery**: Handle failures gracefully with appropriate error context
5. **User Experience**: For prompts, provide clear choices and guidance

### When Adding New Git Events
1. **Event Wrapper**: Implement `AtomicEvent` trait with proper Git2 integration
2. **Rule Integration**: Call appropriate rules in `pre_execute_hook()`
3. **Hook Support**: Implement pre/post hook execution points
4. **Authentication**: Integrate with existing auth system when needed
5. **Platform Support**: Ensure cross-platform compatibility

### Configuration System Usage
```rust
// Local configuration (repository-specific)
let config = BGitConfig::load()?;
let rule_level = config.workflow_rules.get_rule_level("default", "a12_no_secrets_staged");

// Global configuration (user-wide)
let global_config = BGitGlobalConfig::load_global()?;
let auth_preference = global_config.auth_preference;
```

### Error Handling Patterns
```rust
// Structured error creation
BGitError::new(
    "Rule Check Failed".to_string(),
    format!("Secret detected in file: {}", file_path),
    BGitErrorWorkflowType::Rules,
    step_name.to_string(),
    NO_EVENT.to_string(),
    rule_name.to_string(),
)

// Rule execution with auto-fixing
match rule.check() {
    Ok(RuleOutput::Success) => Ok(true),
    Ok(RuleOutput::Exception(msg)) => {
        if rule.try_fix()? {
            rule.check() // Re-check after fix
        } else {
            Err(error)
        }
    }
    Err(e) => Err(e),
}
```

### Secret Detection Implementation Notes
The `a12_no_secrets_staged` rule demonstrates sophisticated pattern matching:
- **Regex Detection**: patterns for different secret types
- **Entropy Analysis**: Statistical randomness measurement
- **Context Validation**: Custom validators for specific secret formats
- **False Positive Reduction**: Whitelist common words, detect patterns

### SSH Management Implementation Notes
The SSH system (`src/auth/ssh/`) provides:
- **Agent Lifecycle**: Automatic creation, key loading, cleanup
- **Cross-Platform**: Unix sockets vs Windows named pipes
- **Key Discovery**: Automatic `~/.ssh/id_*` detection
- **Retry Logic**: Configurable attempts with exponential backoff
- **Stale Cleanup**: Process validation and socket cleanup

## Testing Strategies

### Unit Testing Focus Areas
1. **Rule Logic**: Test detection, fixing, and edge cases
2. **Step Progression**: Verify correct workflow routing
3. **Configuration**: Test parsing, defaults, and overrides
4. **Cross-Platform**: Validate platform-specific implementations
5. **Error Handling**: Ensure proper error propagation and context

### Integration Testing Patterns
- Use `tempfile` for temporary Git repositories
- Use `assert_cmd` for CLI testing
- Mock external dependencies (SSH agents, remote repositories)
- Test complete workflow scenarios end-to-end

## Performance Considerations

### Optimization Points
1. **Rule Execution**: Cache expensive checks when possible
2. **Git Operations**: Minimize repository state queries
3. **SSH Agent**: Reuse connections, avoid redundant key loading
4. **Configuration**: Load once and pass references
5. **Pattern Matching**: Compile regex patterns once

### Memory Management
- Use `Box<dyn Trait>` for trait objects to enable dynamic dispatch
- Prefer `&str` over `String` for temporary string operations
- Use `Arc<>` for shared configuration data across threads

## Security Considerations

### Sensitive Data Handling
1. **Credentials**: Base64 encode in configuration files
2. **Memory**: Clear sensitive data after use when possible
3. **Logging**: Never log credentials or secrets
4. **File Permissions**: Respect SSH key file permissions
5. **Process Isolation**: Clean up SSH agents and temporary files

### Secret Detection Guidelines
- Maintain high sensitivity for security rules
- Balance false positives vs false negatives
- Provide clear remediation guidance
- Support whitelist mechanisms for known safe patterns

## Common Development Tasks

### Adding a New Authentication Method
1. Implement auth strategy in `src/auth/`
2. Update `PreferredAuth` enum
3. Integrate with existing event system
4. Add configuration options
5. Test cross-platform compatibility

### Extending the Rules System
1. Create new rule file following naming convention
2. Implement `Rule` trait with proper error handling
3. Add to rule registry in `src/rules.rs`
4. Update configuration schema if needed
5. Add comprehensive tests

### Adding New Git Operations
1. Create event file in `src/events/`
2. Implement `AtomicEvent` trait
3. Integrate with rules system
4. Add hook support points
5. Update workflow steps as needed

## AI Agent Specific Guidelines

### Code Analysis Strategies
1. **Understand Traits First**: Focus on understanding the core traits before diving into implementations
2. **Follow Data Flow**: Trace how steps, rules, and events interact
3. **Check Cross-Platform Code**: Pay attention to `#[cfg]` attributes for platform differences
4. **Review Error Paths**: Understand how errors propagate through the system
5. **Study Configuration**: Understand how behavior is customized through config

### Code Generation Best Practices
1. **Follow Existing Patterns**: Match naming conventions and code structure
2. **Implement Error Handling**: Always include proper `BGitError` context
3. **Consider Cross-Platform**: Use appropriate platform abstractions
4. **Add Documentation**: Include clear docstrings for public interfaces
5. **Test Thoroughly**: Consider edge cases and error conditions

### Logging Conventions
1. **No Emoji Symbols**: All logging output must be emoji-free for compatibility across terminals and systems
2. **Clear Status Messages**: Use plain text for success/failure indicators
   - Use "Successfully" instead of "✓"
   - Use "Failed" instead of "✗"
   - Use "Warning" instead of "⚠"
3. **Debug vs User Output**: Debug logs (`debug!()`) are for developers, user output (`println!()`) should be concise
4. **Consistent Formatting**: Maintain consistent message structure across similar operations
5. **Cross-Platform Text**: Ensure all text displays correctly on Unix, Windows, and different terminal emulators

```rust
// Good: Emoji-free logging
debug!("Successfully created persistent agent");
println!("SSH key '{}' added successfully!", key_name);

// Bad: Contains emoji symbols
debug!("✓ Successfully created persistent agent");
println!("✓ SSH key '{}' added successfully!", key_name);
```

### Common Pitfalls to Avoid
1. **Ignoring Platform Differences**: Not handling Windows vs Unix differences
2. **Poor Error Context**: Generic error messages without proper context
3. **Rule Bypass**: Forgetting to integrate rules into new operations
4. **Memory Leaks**: Not properly cleaning up SSH agents or temporary resources
5. **Configuration Conflicts**: Not respecting user configuration overrides

This guide should help AI agents understand the bgit codebase architecture and contribute effectively while maintaining the existing code quality and design patterns.
