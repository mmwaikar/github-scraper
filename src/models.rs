use std::{os::raw::c_char, str::FromStr};

#[derive(Debug, PartialEq)]
pub enum StatType {
    None,
    Stars,
    Forks,
    Watching,
    Releases,
    Used,
    Contributors
}

impl FromStr for StatType {
    type Err = ();

    fn from_str(input: &str) -> Result<StatType, Self::Err> {
        match input {
            "stars" => Ok(StatType::Stars),
            "forks" => Ok(StatType::Forks),
            "watching" => Ok(StatType::Watching),
            "Releases" => Ok(StatType::Releases),
            "Used" => Ok(StatType::Used),
            "Contributors" => Ok(StatType::Contributors),
            _ => Ok(StatType::None),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GitHubRepo {
    pub name: String,
    pub url: String,
    pub stats: GitHubRepoStats,
}

impl GitHubRepo {
    pub fn new(name: &str, url: &str) -> GitHubRepo {
        GitHubRepo {
            name: name.to_string(),
            url: url.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GitHubRepoStats {
    pub stars: String,
    pub watching: String,
    pub forks: String,
    pub releases: String,
    pub used: String,
    pub contributors: String,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct GitHubRepoStatsFfi {
    pub stars: *mut c_char,
    pub watching: *mut c_char,
    pub forks: *mut c_char,
    pub releases: *mut c_char,
    pub used: *mut c_char,
    pub contributors: *mut c_char,
    pub error: *mut c_char,
    pub success: i32,
}

impl Default for GitHubRepoStatsFfi {
    fn default() -> Self {
        Self {
            stars: std::ptr::null_mut(),
            watching: std::ptr::null_mut(),
            forks: std::ptr::null_mut(),
            releases: std::ptr::null_mut(),
            used: std::ptr::null_mut(),
            contributors: std::ptr::null_mut(),
            error: std::ptr::null_mut(),
            success: 0,
        }
    }
}

impl GitHubRepoStats {
    pub fn get_stars(&self) -> u64 {
        GitHubRepoStats::get_count(self.stars.as_str())
    }

    pub fn get_watching(&self) -> u64 {
        GitHubRepoStats::get_count(self.watching.as_str())
    }

    pub fn get_forks(&self) -> u64 {
        GitHubRepoStats::get_count(self.forks.as_str())
    }

    pub fn get_releases(&self) -> u64 {
        GitHubRepoStats::get_count(self.releases.as_str())
    }

    pub fn get_used(&self) -> u64 {
        GitHubRepoStats::get_count(self.used.as_str())
    }

    pub fn get_contributors(&self) -> u64 {
        GitHubRepoStats::get_count(self.contributors.as_str())
    }

    fn get_count(num_str: &str) -> u64 {
        if num_str.ends_with("k") {
            let num = num_str[0..num_str.len() - 1].parse::<f64>().unwrap_or(0.0);
            (num * 1000.0) as u64
        } else {
            num_str.parse::<u64>().unwrap_or(0)
        }
    }
}

#[cfg(test)]
mod test_super {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_default_count() {
        let stats = GitHubRepoStats::default();
        assert_eq!(stats.get_stars(), 0);
        assert_eq!(stats.get_watching(), 0);
        assert_eq!(stats.get_forks(), 0);
        assert_eq!(stats.get_releases(), 0);
        assert_eq!(stats.get_used(), 0);
        assert_eq!(stats.get_contributors(), 0);
    }

    #[test]
    fn test_get_count() {
        let stats = GitHubRepoStats {
            stars: "1.5k".to_string(),
            watching: "500".to_string(),
            forks: "2k".to_string(),
            releases: "10".to_string(),
            used: "3.2k".to_string(),
            contributors: "100".to_string(),
        };
        assert_eq!(stats.get_stars(), 1500);
        assert_eq!(stats.get_watching(), 500);
        assert_eq!(stats.get_forks(), 2000);
        assert_eq!(stats.get_releases(), 10);
        assert_eq!(stats.get_used(), 3200);
        assert_eq!(stats.get_contributors(), 100);
    }
}
