#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("step '{step_id}' failed: {reason}")]
    StepFailed { step_id: String, reason: String },

    #[error("step '{step_id}' timed out")]
    StepTimeout { step_id: String },

    #[error("workflow has cyclic step dependencies")]
    CyclicDependency,

    #[error("workflow definition invalid: {0}")]
    InvalidDefinition(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    Core(#[from] winforge_core::CoreError),
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;
