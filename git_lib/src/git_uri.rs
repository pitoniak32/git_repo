use git_url_parse::GitUrl;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantNames};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitUri {
    pub host: Option<String>,
    pub name: String,
    pub owner: Option<String>,
    pub organization: Option<String>,
    pub fullname: String,
    pub scheme: Scheme,
    pub user: Option<String>,
    pub token: Option<String>,
    pub port: Option<u16>,
    pub path: String,
    pub git_suffix: bool,
    pub scheme_prefix: bool,
}

/// Supported uri schemes for parsing
#[derive(
    Debug, PartialEq, Eq, EnumString, VariantNames, Clone, Display, Copy, Serialize, Deserialize,
)]
#[strum(serialize_all = "kebab_case")]
pub enum Scheme {
    /// Represents `file://` url scheme
    File,
    /// Represents `ftp://` url scheme
    Ftp,
    /// Represents `ftps://` url scheme
    Ftps,
    /// Represents `git://` url scheme
    Git,
    /// Represents `git+ssh://` url scheme
    #[strum(serialize = "git+ssh")]
    GitSsh,
    /// Represents `http://` url scheme
    Http,
    /// Represents `https://` url scheme
    Https,
    /// Represents `ssh://` url scheme
    Ssh,
    /// Represents No url scheme
    Unspecified,
}

impl Scheme {
    fn from(value: git_url_parse::Scheme) -> Self {
        match value {
            git_url_parse::Scheme::File => Scheme::File,
            git_url_parse::Scheme::Ftp => Scheme::Ftp,
            git_url_parse::Scheme::Ftps => Scheme::Ftps,
            git_url_parse::Scheme::Git => Scheme::Git,
            git_url_parse::Scheme::GitSsh => Scheme::GitSsh,
            git_url_parse::Scheme::Http => Scheme::Http,
            git_url_parse::Scheme::Https => Scheme::Https,
            git_url_parse::Scheme::Ssh => Scheme::Ssh,
            git_url_parse::Scheme::Unspecified => Scheme::Unspecified,
        }
    }
}

impl From<GitUrl> for GitUri {
    fn from(value: GitUrl) -> Self {
        GitUri {
            host: value.host,
            name: value.name,
            owner: value.owner,
            organization: value.organization,
            fullname: value.fullname,
            scheme: Scheme::from(value.scheme),
            user: value.user,
            token: value.token,
            port: value.port,
            path: value.path,
            git_suffix: value.git_suffix,
            scheme_prefix: value.scheme_prefix,
        }
    }
}
