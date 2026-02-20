mod trait_def;
mod gitea;
mod github;

pub use trait_def::GitClient;
pub use gitea::GiteaClient;
pub use github::GitHubClient;
