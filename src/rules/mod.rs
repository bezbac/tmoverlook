use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

mod eval;
mod git;
mod path;

pub trait Evaluatable {
    fn evaluate(&self, paths: &mut BTreeSet<PathBuf>) -> Result<()>;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    Path(self::path::Rule),
    Eval(self::eval::Rule),
    GitRepositories(self::git::Rule),
}

impl Evaluatable for Rule {
    fn evaluate(&self, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
        match &self {
            Rule::Path(rule) => rule.evaluate(paths),
            Rule::Eval(rule) => rule.evaluate(paths),
            Rule::GitRepositories(rule) => rule.evaluate(paths),
        }
    }
}

impl Rule {
    pub fn get_priority(&self) -> usize {
        match &self {
            Rule::Path(_) => 3,
            Rule::Eval(_) => 2,
            Rule::GitRepositories(_) => 1,
        }
    }
}
