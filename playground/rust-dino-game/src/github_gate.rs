use std::{env, process::Command};

const OWNER: &str = "Yeachan-Heo";
const REPO: &str = "oh-my-codex";
const REPO_URL: &str = "github.com/Yeachan-Heo/oh-my-codex";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportMode {
    Starred { login: String },
    NotStarred { login: String },
    Unavailable { message: String },
}

impl SupportMode {
    pub fn detect() -> Self {
        if let Ok(mode) = env::var("DINO_SUPPORT_MODE") {
            return match mode.as_str() {
                "starred" => Self::Starred {
                    login: "override".to_owned(),
                },
                "unstarred" => Self::NotStarred {
                    login: "override".to_owned(),
                },
                _ => Self::Unavailable {
                    message: "support override active".to_owned(),
                },
            };
        }

        let auth_output = match Command::new("gh").args(["auth", "status"]).output() {
            Ok(output) => output,
            Err(_) => {
                return Self::Unavailable {
                    message: "gh CLI not found".to_owned(),
                };
            }
        };

        if !auth_output.status.success() {
            return Self::Unavailable {
                message: "gh login required to verify GitHub star".to_owned(),
            };
        }

        let auth_text = String::from_utf8_lossy(&auth_output.stdout).to_string()
            + &String::from_utf8_lossy(&auth_output.stderr);
        let login = extract_login(&auth_text).unwrap_or_else(|| "gh-user".to_owned());

        let star_output = match Command::new("gh")
            .args(["api", "-i", &format!("/user/starred/{OWNER}/{REPO}")])
            .output()
        {
            Ok(output) => output,
            Err(_) => {
                return Self::Unavailable {
                    message: "failed to run gh api".to_owned(),
                };
            }
        };

        let star_text = String::from_utf8_lossy(&star_output.stdout).to_string()
            + &String::from_utf8_lossy(&star_output.stderr);

        match parse_star_status(&star_text) {
            Some(204) => Self::Starred { login },
            Some(404) => Self::NotStarred { login },
            _ => Self::Unavailable {
                message: "unable to verify repo star status".to_owned(),
            },
        }
    }

    pub fn repo_url(&self) -> &'static str {
        REPO_URL
    }

    pub fn is_penalty_mode(&self) -> bool {
        matches!(self, Self::NotStarred { .. })
    }

    pub fn scene_tint(&self) -> Option<(u8, u8, u8, u8)> {
        None
    }

    pub fn status_heading(&self) -> String {
        match self {
            Self::Starred { login } => format!("{login}: repository star detected"),
            Self::NotStarred { login } => format!("{login}: repository star not detected"),
            Self::Unavailable { .. } => "GitHub star status unavailable".to_owned(),
        }
    }

    pub fn status_detail(&self) -> String {
        match self {
            Self::Starred { .. } => "Supporter mode: normal obstacle pacing unlocked".to_owned(),
            Self::NotStarred { .. } => "You have not starred this repository, so this feature is unavailable until you star it.".to_owned(),
            Self::Unavailable { message } => format!("Verification unavailable: {message}"),
        }
    }
}

fn extract_login(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let marker = "account ";
        let idx = line.find(marker)?;
        let tail = &line[idx + marker.len()..];
        tail.split_whitespace().next().map(|s| s.to_owned())
    })
}

fn parse_star_status(text: &str) -> Option<u16> {
    for line in text.lines() {
        if let Some(status_line) = line.strip_prefix("HTTP/") {
            let parts: Vec<_> = status_line.split_whitespace().collect();
            if let Some(code) = parts.get(1) {
                if let Ok(value) = code.parse::<u16>() {
                    return Some(value);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{extract_login, parse_star_status};

    #[test]
    fn extracts_login_from_gh_auth_status_output() {
        let sample = "github.com\n  ✓ Logged in to github.com account HaD0Yun (keyring)\n";
        assert_eq!(extract_login(sample).as_deref(), Some("HaD0Yun"));
    }

    #[test]
    fn parses_starred_status_code() {
        let sample = "HTTP/2.0 204 No Content\nDate: today\n";
        assert_eq!(parse_star_status(sample), Some(204));
    }

    #[test]
    fn parses_not_starred_status_code() {
        let sample = "HTTP/2.0 404 Not Found\nDate: today\n";
        assert_eq!(parse_star_status(sample), Some(404));
    }
}
