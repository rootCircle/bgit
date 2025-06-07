use crate::config::BGitConfig;
use crate::step::{ActionStep, Step, Task};
use crate::workflow_queue::WorkflowQueue;
use crate::workflows::default::action::ta01_is_git_repo::IsGitRepo;

pub(crate) fn default_cmd_workflow(bgit_config: BGitConfig) {
    let default_workflow_rules_config = bgit_config.get_workflow_rules("default");
    let default_workflow_config_flags = bgit_config.get_workflow_steps("default");

    let workflow_queue = WorkflowQueue::new(Step::Start(Task::ActionStepTask(Box::new(
        IsGitRepo::new(),
    ))));
    match workflow_queue.execute(default_workflow_config_flags, default_workflow_rules_config) {
        Ok(_) => {}
        Err(err) => err.print_error(),
    };
}
