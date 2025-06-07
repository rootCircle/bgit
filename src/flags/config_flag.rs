/////////////////////////////////////
/// Workflow Flag Constants
///
/// This module provides constants for all workflow step flags to improve
/// readability and maintainability. Use these constants instead of string
/// literals when accessing configuration flags.
/////////////////////////////////////
pub mod workflows {
    pub mod default {
        pub mod is_sole_contributor {
            pub const OVERRIDE_CHECK_FOR_AUTHORS: &str = "overrideCheckForAuthors";
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steps_module_access() {
        // Test that step modules are accessible through steps module
        assert_eq!(
            workflows::default::is_sole_contributor::OVERRIDE_CHECK_FOR_AUTHORS,
            "overrideCheckForAuthors"
        );
    }
}
