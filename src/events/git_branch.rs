// use super::AtomicEvent;
// use crate::{
//     bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
//     rules::Rule,
// };
// use git2::{BranchType, Repository};
// use std::path::Path;

// #[derive(Debug, Clone)]
// pub(crate) enum BranchOperation {
//     Create,
//     Delete,
//     List,
// }

// pub(crate) struct GitBranch {
//     name: String,
//     pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
//     branch_name: Option<String>,
//     operation: BranchOperation,
//     force_delete: bool,
// }

// impl GitBranch {
//     pub fn create_branch(branch_name: String) -> Self {
//         GitBranch {
//             name: "git_branch".to_owned(),
//             pre_check_rules: vec![],
//             branch_name: Some(branch_name),
//             operation: BranchOperation::Create,
//             force_delete: false,
//         }
//     }

//     pub fn delete_branch(branch_name: String) -> Self {
//         GitBranch {
//             name: "git_branch".to_owned(),
//             pre_check_rules: vec![],
//             branch_name: Some(branch_name),
//             operation: BranchOperation::Delete,
//             force_delete: false,
//         }
//     }

//     pub fn force_delete_branch(branch_name: String) -> Self {
//         GitBranch {
//             name: "git_branch".to_owned(),
//             pre_check_rules: vec![],
//             branch_name: Some(branch_name),
//             operation: BranchOperation::Delete,
//             force_delete: true,
//         }
//     }

//     pub fn list_branches() -> Self {
//         GitBranch {
//             name: "git_branch".to_owned(),
//             pre_check_rules: vec![],
//             branch_name: None,
//             operation: BranchOperation::List,
//             force_delete: false,
//         }
//     }

//     pub fn set_branch_name(&mut self, branch_name: String) {
//         self.branch_name = Some(branch_name);
//     }

//     pub fn set_operation(&mut self, operation: BranchOperation) {
//         self.operation = operation;
//     }

//     pub fn set_force_delete(&mut self, force: bool) {
//         self.force_delete = force;
//     }
// }

// impl AtomicEvent for GitBranch {
//     fn new() -> Self
//     where
//         Self: Sized,
//     {
//         GitBranch {
//             name: "git_branch".to_owned(),
//             pre_check_rules: vec![],
//             branch_name: None,
//             operation: BranchOperation::List,
//             force_delete: false,
//         }
//     }

//     fn get_name(&self) -> &str {
//         &self.name
//     }

//     fn get_action_description(&self) -> &str {
//         match self.operation {
//             BranchOperation::Create => "Create a new branch",
//             BranchOperation::Delete => "Delete a branch",
//             BranchOperation::List => "List all branches",
//         }
//     }

//     fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
//         self.pre_check_rules.push(rule);
//     }

//     fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
//         &self.pre_check_rules
//     }

//     fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
//         // Open the repository at the current directory
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

//         match self.operation {
//             BranchOperation::Create => self.create_branch_impl(&repo),
//             BranchOperation::Delete => self.delete_branch_impl(&repo),
//             BranchOperation::List => self.list_branches_impl(&repo),
//         }
//     }
// }

// impl GitBranch {
//     fn create_branch_impl(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
//         // Check if branch name is provided
//         let branch_name = match &self.branch_name {
//             Some(name) => name,
//             None => {
//                 return Err(Box::new(BGitError::new(
//                     "BGitError",
//                     "Branch name not provided for create operation",
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 )));
//             }
//         };

//         // Get the current HEAD commit
//         let head = repo.head().map_err(|e| {
//             Box::new(BGitError::new(
//                 "BGitError",
//                 &format!("Failed to get HEAD: {}", e),
//                 BGitErrorWorkflowType::AtomicEvent,
//                 NO_EVENT,
//                 &self.name,
//                 NO_RULE,
//             ))
//         })?;

//         let target_commit = head.peel_to_commit().map_err(|e| {
//             Box::new(BGitError::new(
//                 "BGitError",
//                 &format!("Failed to get target commit: {}", e),
//                 BGitErrorWorkflowType::AtomicEvent,
//                 NO_EVENT,
//                 &self.name,
//                 NO_RULE,
//             ))
//         })?;

//         // Check if branch already exists
//         match repo.find_branch(branch_name, BranchType::Local) {
//             Ok(_) => {
//                 return Err(Box::new(BGitError::new(
//                     "BGitError",
//                     &format!("Branch '{}' already exists", branch_name),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 )));
//             }
//             Err(_) => {
//                 // Branch doesn't exist, which is what we want
//             }
//         }

//         // Create the new branch
//         repo.branch(branch_name, &target_commit, false)
//             .map_err(|e| {
//                 Box::new(BGitError::new(
//                     "BGitError",
//                     &format!("Failed to create branch '{}': {}", branch_name, e),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 ))
//             })?;

//         println!("Created branch '{}'", branch_name);
//         Ok(true)
//     }

//     fn delete_branch_impl(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
//         // Check if branch name is provided
//         let branch_name = match &self.branch_name {
//             Some(name) => name,
//             None => {
//                 return Err(Box::new(BGitError::new(
//                     "BGitError",
//                     "Branch name not provided for delete operation",
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 )));
//             }
//         };

//         // Get current HEAD to prevent deleting the current branch
//         let current_branch = match repo.head() {
//             Ok(head) => {
//                 if head.is_branch() {
//                     head.shorthand().map(|s| s.to_string())
//                 } else {
//                     None
//                 }
//             }
//             Err(_) => None,
//         };

//         // Check if trying to delete the current branch
//         if let Some(ref current) = current_branch {
//             if current == branch_name {
//                 return Err(Box::new(BGitError::new(
//                     "BGitError",
//                     &format!(
//                         "Cannot delete the currently checked out branch '{}'",
//                         branch_name
//                     ),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 )));
//             }
//         }

//         // Find the branch to delete
//         let mut branch = repo
//             .find_branch(branch_name, BranchType::Local)
//             .map_err(|e| {
//                 Box::new(BGitError::new(
//                     "BGitError",
//                     &format!("Branch '{}' not found: {}", branch_name, e),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 ))
//             })?;

//         // Check if branch is merged (unless force delete is enabled)
//         if !self.force_delete {
//             let is_merged = self.is_branch_merged(repo, &branch)?;
//             if !is_merged {
//                 return Err(Box::new(BGitError::new(
//                     "BGitError",
//                     &format!(
//                         "Branch '{}' is not fully merged. Use force delete to override.",
//                         branch_name
//                     ),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 )));
//             }
//         }

//         // Delete the branch
//         branch.delete().map_err(|e| {
//             Box::new(BGitError::new(
//                 "BGitError",
//                 &format!("Failed to delete branch '{}': {}", branch_name, e),
//                 BGitErrorWorkflowType::AtomicEvent,
//                 NO_EVENT,
//                 &self.name,
//                 NO_RULE,
//             ))
//         })?;

//         println!("Deleted branch '{}'", branch_name);
//         Ok(true)
//     }

//     fn list_branches_impl(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
//         // Get current branch for marking
//         let current_branch = match repo.head() {
//             Ok(head) => {
//                 if head.is_branch() {
//                     head.shorthand().map(|s| s.to_string())
//                 } else {
//                     None
//                 }
//             }
//             Err(_) => None,
//         };

//         // Get all local branches
//         let branches = repo.branches(Some(BranchType::Local)).map_err(|e| {
//             Box::new(BGitError::new(
//                 "BGitError",
//                 &format!("Failed to list branches: {}", e),
//                 BGitErrorWorkflowType::AtomicEvent,
//                 NO_EVENT,
//                 &self.name,
//                 NO_RULE,
//             ))
//         })?;

//         println!("Local branches:");
//         let mut branch_count = 0;

//         for branch_result in branches {
//             let (branch, _branch_type) = branch_result.map_err(|e| {
//                 Box::new(BGitError::new(
//                     "BGitError",
//                     &format!("Failed to process branch: {}", e),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 ))
//             })?;

//             if let Some(branch_name) = branch.name().map_err(|e| {
//                 Box::new(BGitError::new(
//                     "BGitError",
//                     &format!("Failed to get branch name: {}", e),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 ))
//             })? {
//                 let marker = if Some(branch_name.to_string()) == current_branch {
//                     "* "
//                 } else {
//                     "  "
//                 };
//                 println!("{}{}", marker, branch_name);
//                 branch_count += 1;
//             }
//         }

//         if branch_count == 0 {
//             println!("No local branches found.");
//         }

//         Ok(true)
//     }

//     fn is_branch_merged(
//         &self,
//         repo: &Repository,
//         branch: &git2::Branch,
//     ) -> Result<bool, Box<BGitError>> {
//         // Get the branch commit
//         let branch_commit = branch.get().peel_to_commit().map_err(|e| {
//             Box::new(BGitError::new(
//                 "BGitError",
//                 &format!("Failed to get branch commit: {}", e),
//                 BGitErrorWorkflowType::AtomicEvent,
//                 NO_EVENT,
//                 &self.name,
//                 NO_RULE,
//             ))
//         })?;

//         // Get HEAD commit
//         let head_commit = repo
//             .head()
//             .and_then(|head| head.peel_to_commit())
//             .map_err(|e| {
//                 Box::new(BGitError::new(
//                     "BGitError",
//                     &format!("Failed to get HEAD commit: {}", e),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 ))
//             })?;

//         // Check if branch commit is ancestor of HEAD (i.e., merged)
//         let is_ancestor = repo
//             .graph_descendant_of(head_commit.id(), branch_commit.id())
//             .map_err(|e| {
//                 Box::new(BGitError::new(
//                     "BGitError",
//                     &format!("Failed to check merge status: {}", e),
//                     BGitErrorWorkflowType::AtomicEvent,
//                     NO_EVENT,
//                     &self.name,
//                     NO_RULE,
//                 ))
//             })?;

//         Ok(is_ancestor)
//     }
// }
