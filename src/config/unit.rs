use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

/// How a file is assigned to a unit: a set of files that may relate to one another.
#[derive(Debug, Clone)]
pub enum Unit {
    Directory,
    Source,
    Roots(GlobSet),
}

impl Unit {
    /// Reads folder patterns where a star does not reach across a slash.
    pub(super) fn roots(patterns: &[String]) -> Result<Unit, globset::Error> {
        let mut roots = GlobSetBuilder::new();
        for pattern in patterns {
            roots.add(GlobBuilder::new(pattern).literal_separator(true).build()?);
        }
        Ok(Unit::Roots(roots.build()?))
    }

    /// The folder of the source, relative to it, that is the unit of a file.
    pub fn of(&self, file: &Path) -> PathBuf {
        let folder = file.parent().unwrap_or(Path::new("")).to_path_buf();
        match self {
            Unit::Directory => folder,
            Unit::Source => PathBuf::new(),
            Unit::Roots(roots) => {
                let mut ancestor = PathBuf::new();
                for component in folder.components() {
                    ancestor.push(component);
                    if roots.is_match(&ancestor) {
                        return ancestor;
                    }
                }
                folder
            }
        }
    }
}
