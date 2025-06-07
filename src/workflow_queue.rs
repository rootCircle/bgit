use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE, NO_STEP};
use crate::config::{WorkflowRules, WorkflowSteps};
use crate::step::Task::{ActionStepTask, PromptStepTask};
use crate::step::{Step, Task};
use colored::Colorize;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use std::time::Duration;
use std::time::Instant;

const HATCHING_CHICK_EMOJI: &str = "🐣";

pub(crate) struct WorkflowQueue {
    init_step: Step,
    pb: ProgressBar,
}

impl WorkflowQueue {
    pub(crate) fn new(init_step: Step) -> Self {
        // Initialize spinner for progress indication
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(200));
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.cyan/blue.bold} [{pos:.yellow}/?] Executing step: {wide_msg:.green}",
            )
            .unwrap(),
        );

        WorkflowQueue { init_step, pb }
    }

    fn run_step_and_traverse(
        &self,
        workflow_config_flags: Option<&WorkflowSteps>,
        workflow_rules_config: Option<&WorkflowRules>,
        task: &Task,
    ) -> Result<Step, Box<BGitError>> {
        match task {
            ActionStepTask(action_step_task) => {
                eprintln!(
                    "{} Running Action Step: {}",
                    HATCHING_CHICK_EMOJI,
                    action_step_task.get_name().cyan().bold()
                );
                self.pb.set_message(format!(
                    "Step '{}' in progress...",
                    action_step_task.get_name().bold()
                ));

                let action_step_config_flags = workflow_config_flags
                    .and_then(|flags| flags.get_step_flags(action_step_task.get_name()));

                let action_step_result =
                    action_step_task.execute(action_step_config_flags, workflow_rules_config)?;

                self.pb.inc(1);
                self.pb.tick();

                Ok(action_step_result)
            }
            PromptStepTask(prompt_step_task) => {
                self.pb.disable_steady_tick();
                eprintln!(
                    "{} Running Prompt Step: {}",
                    HATCHING_CHICK_EMOJI,
                    prompt_step_task.get_name().cyan().bold()
                );

                self.pb.set_message(format!(
                    "Step '{}' in progress...",
                    prompt_step_task.get_name().bold()
                ));

                let prompt_step_config_flags = workflow_config_flags
                    .and_then(|flags| flags.get_step_flags(prompt_step_task.get_name()));

                let prompt_step_result =
                    prompt_step_task.execute(prompt_step_config_flags, workflow_rules_config)?;
                self.pb.enable_steady_tick(Duration::from_millis(200));

                self.pb.inc(1);
                self.pb.tick();

                Ok(prompt_step_result)
            }
        }
    }

    pub(crate) fn execute(
        &self,
        workflow_config_flags: Option<&WorkflowSteps>,
        workflow_rules_config: Option<&WorkflowRules>,
    ) -> Result<bool, Box<BGitError>> {
        match &self.init_step {
            Step::Start(task) => {
                let started = Instant::now();

                let mut next_step: Step =
                    self.run_step_and_traverse(workflow_config_flags, workflow_rules_config, task)?;

                while next_step != Step::Stop {
                    if let Step::Start(_) = next_step {
                        return Err(Box::new(BGitError::new(
                            "next_step must not be a Start Task!",
                            "next_step must not be a Start Task! This is a bug in the code",
                            BGitErrorWorkflowType::WorkflowQueue,
                            NO_STEP,
                            NO_EVENT,
                            NO_RULE,
                        )));
                    }

                    match next_step {
                        Step::Task(task) => {
                            next_step = self.run_step_and_traverse(
                                workflow_config_flags,
                                workflow_rules_config,
                                &task,
                            )?;
                        }
                        _ => {
                            unreachable!("This code is unreachable")
                        }
                    }
                }

                self.pb.finish_with_message("Workflow complete");

                if next_step == Step::Stop {
                    println!("Done in {}", HumanDuration(started.elapsed()));
                    Ok(true)
                } else {
                    Err(Box::new(BGitError::new(
                        "final_step must be a Stop Task!",
                        "final_step must be a Stop Task! This is a bug in the code",
                        BGitErrorWorkflowType::WorkflowQueue,
                        NO_STEP,
                        NO_EVENT,
                        NO_RULE,
                    )))
                }
            }
            _ => Err(Box::new(BGitError::new(
                "init_step must be a Start Task!",
                "init_step must be a Start Task! This is a bug in the code",
                BGitErrorWorkflowType::WorkflowQueue,
                NO_STEP,
                NO_EVENT,
                NO_RULE,
            ))),
        }
    }
}
