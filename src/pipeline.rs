use std::fmt;

use toml::Value as Toml;

use crate::stages::{DeclareError, Stage};

/// The ordered stages a watch applies to the files it claims.
#[derive(Debug, Clone)]
pub struct Pipeline {
    stages: Vec<Stage>,
}

/// Why a pipeline was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    Syntax(String),
    NotAList,
    Declare { index: usize, error: DeclareError },
    PureAfterEffect { stage: String },
    PathAfterNext { stage: String },
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::Syntax(message) => write!(f, "not valid TOML: {message}"),
            PipelineError::NotAList => write!(f, "`stages` must be a list"),
            PipelineError::Declare { index, error } => write!(f, "stage {}: {error}", index + 1),
            PipelineError::PureAfterEffect { stage } => {
                write!(
                    f,
                    "`{stage}` only rewrites the plan, so it cannot follow a stage that moves files"
                )
            }
            PipelineError::PathAfterNext { stage } => {
                write!(
                    f,
                    "`{stage}` changes the folder, so it must come before `next`, which reads that folder"
                )
            }
        }
    }
}

impl std::error::Error for PipelineError {}

impl Pipeline {
    /// Reads a list of stage declarations and checks their order.
    pub fn declare(values: &[Toml]) -> Result<Pipeline, PipelineError> {
        let stages = values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                Stage::declare(value).map_err(|error| PipelineError::Declare { index, error })
            })
            .collect::<Result<Vec<_>, _>>()?;
        check_order(&stages)?;
        Ok(Pipeline { stages })
    }

    /// Reads the `stages` list of a TOML document.
    pub fn from_toml(source: &str) -> Result<Pipeline, PipelineError> {
        let document: toml::Table = source
            .parse()
            .map_err(|error: toml::de::Error| PipelineError::Syntax(error.to_string()))?;
        match document.get("stages") {
            Some(Toml::Array(values)) => Pipeline::declare(values),
            _ => Err(PipelineError::NotAList),
        }
    }

    /// The declared stages in the order they were written.
    pub fn stages(&self) -> &[Stage] {
        &self.stages
    }

    /// Whether the pipeline ends in a stage that touches the filesystem.
    pub fn has_effect(&self) -> bool {
        self.stages.iter().any(Stage::is_effect)
    }
}

fn check_order(stages: &[Stage]) -> Result<(), PipelineError> {
    let mut seen_effect = false;
    let mut seen_next = false;
    for stage in stages {
        if stage.is_effect() {
            seen_effect = true;
        } else if seen_effect {
            return Err(PipelineError::PureAfterEffect {
                stage: stage.name().to_string(),
            });
        }
        if stage.is_path() && seen_next {
            return Err(PipelineError::PathAfterNext {
                stage: stage.name().to_string(),
            });
        }
        seen_next |= matches!(stage, Stage::Next(_));
    }
    Ok(())
}
