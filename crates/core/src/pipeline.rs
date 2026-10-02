use std::fmt;

use toml::Value as Toml;

use crate::stages::{DeclareError, Declared, Filter};

/// The most stages a pipeline may hold, because a declaration may come from downloaded content.
const MAX_STAGES: usize = 64;

/// The ordered stages a route applies to the files it claims; with none, it takes every file as it is.
#[derive(Debug, Clone, Default)]
pub struct Pipeline {
    stages: Vec<Declared>,
}

/// Why a pipeline was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    Syntax(String),
    NotAList,
    TooManyStages,
    Declare { index: usize, error: DeclareError },
    PathAfterNext { stage: String },
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::Syntax(message) => write!(f, "not valid TOML: {message}"),
            PipelineError::NotAList => write!(f, "`stages` must be a list"),
            PipelineError::TooManyStages => {
                write!(f, "a pipeline may hold at most {MAX_STAGES} stages")
            }
            PipelineError::Declare { index, error } => write!(f, "stage {}: {error}", index + 1),
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
        if values.len() > MAX_STAGES {
            return Err(PipelineError::TooManyStages);
        }
        let stages = values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                Declared::read(value).map_err(|error| PipelineError::Declare { index, error })
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
    pub fn stages(&self) -> &[Declared] {
        &self.stages
    }

    /// The pipeline in two parts: the filters it opens with, which say what it claims, and the stages after them.
    pub(crate) fn split_at_claim(&self) -> (Vec<&Filter>, &[Declared]) {
        let filters: Vec<&Filter> = self
            .stages
            .iter()
            .map_while(|stage| match stage {
                Declared::Filter(filter) => Some(filter),
                _ => None,
            })
            .collect();
        let rest = &self.stages[filters.len()..];
        (filters, rest)
    }
}

fn check_order(stages: &[Declared]) -> Result<(), PipelineError> {
    let mut seen_next = false;
    for stage in stages {
        if stage.is_path() && seen_next {
            return Err(PipelineError::PathAfterNext {
                stage: stage.name().to_string(),
            });
        }
        seen_next |= matches!(stage, Declared::Next(_));
    }
    Ok(())
}
