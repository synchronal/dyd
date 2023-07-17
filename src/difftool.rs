use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum Difftool {
  Git,
  GitHub,
  Fallthrough(String),
}

impl Default for Difftool {
  fn default() -> Self {
    Self::Git
  }
}

impl std::fmt::Display for Difftool {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Difftool::Git => write!(f, "git difftool -g -y ${{DIFF}}"),
      Difftool::GitHub => todo!(),
      Difftool::Fallthrough(difftool) => write!(f, "{difftool}"),
    }
  }
}
