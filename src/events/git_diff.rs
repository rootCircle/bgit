// use super::AtomicEvent;
// use crate::{
//     bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
//     rules::Rule,
// };
// use git2::{Repository, Diff, DiffOptions};
// use std::path::Path;

// pub(crate) struct GitDiff {
//     name: String,
//     pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
// }

// impl AtomicEvent for GitDiff {
//     fn new() -> Self
//     where
//         Self: Sized,
//     {
//         GitDiff {
//             name: "git_diff".to_owned(),
//             pre_check_rules: vec![],
//         }
//     }

//     fn get_name(&self) -> &str {
//         &self.name
//     }

//     fn get_action_description(&self) -> &str {
//         "Show differences between working directory and staging area"
//     }

//     fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
//         self.pre_check_rules.push(rule);
//     }

//     fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
//         &self.pre_check_rules
//     }

//     fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
//         self.diff_working_directory()
//     }
// }

// impl GitDiff {
//     /// Show diff between working directory and staging area (git diff)
//     fn diff_working_directory(&self) -> Result<bool, Box<BGitError>> {
//         let repo = Repository::discover(Path::new(".")).map_err(|e| {
//             Box::new(BGitError::new(
//                 "BGitError",
//                 &format!("Failed to open repository: {}", e),
//                 BGitErrorWorkflowType::AtomicEvent,
//                 NO_EVENT,
//                 &self.name,
//                 NO_RULE,
//             ))
//         })?;

//         let mut diff_opts = DiffOptions::new();
//         diff_opts.include_untracked(false);

//         let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))
//             .map_err(|e| {
//                 Box::new(BGitError::new(
//                     "BGitError",
//                     &format!("Failed to create diff: {}", e),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 ))
//             })?;

//         self.print_diff(&diff)?;
//         Ok(true)
//     }

//     /// Print the diff output
//     fn print_diff(&self, diff: &Diff) -> Result<(), Box<BGitError>> {
//         let stats = diff.stats().map_err(|e| {
//             Box::new(BGitError::new(
//                 "BGitError",
//                 &format!("Failed to get diff stats: {}", e),
//                 BGitErrorWorkflowType::AtomicEvent,
//                 NO_EVENT,
//                 &self.name,
//                 NO_RULE,
//             ))
//         })?;

//         // Print the actual diff
//         diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
//             match line.origin() {
//                 '+' => print!("\x1b[32m+{}\x1b[0m", std::str::from_utf8(line.content()).unwrap_or("")),
//                 '-' => print!("\x1b[31m-{}\x1b[0m", std::str::from_utf8(line.content()).unwrap_or("")),
//                 ' ' => print!(" {}", std::str::from_utf8(line.content()).unwrap_or("")),
//                 _ => print!("{}", std::str::from_utf8(line.content()).unwrap_or("")),
//             }
//             true
//         }).map_err(|e| {
//             Box::new(BGitError::new(
//                 "BGitError",
//                 &format!("Failed to print diff: {}", e),
//                 BGitErrorWorkflowType::AtomicEvent,
//                 NO_EVENT,
//                 &self.name,
//                 NO_RULE,
//             ))
//         })?;

//         if stats.files_changed() == 0 {
//             println!("No differences found.");
//         }

//         Ok(())
//     }
// }