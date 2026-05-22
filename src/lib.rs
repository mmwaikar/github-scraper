pub mod models;

use std::{ffi::{CStr, CString}, os::raw::c_char, str::FromStr};

use log::info;
use scraper::{Html, Selector};

use crate::models::{GitHubRepoStats, GitHubRepoStatsFfi, StatType};

#[unsafe(export_name = "get_github_repo_stats")]
pub extern "C" fn get_github_repo_stats_ffi(url: *const c_char) -> GitHubRepoStatsFfi {
    match get_github_repo_stats(url) {
        Ok(stats) => GitHubRepoStatsFfi {
            stars: to_c_string_ptr(stats.stars),
            watching: to_c_string_ptr(stats.watching),
            forks: to_c_string_ptr(stats.forks),
            releases: to_c_string_ptr(stats.releases),
            used: to_c_string_ptr(stats.used),
            contributors: to_c_string_ptr(stats.contributors),
            error: std::ptr::null_mut(),
            success: 1,
        },
        Err(err) => GitHubRepoStatsFfi {
            error: to_c_string_ptr(err),
            ..Default::default()
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_github_repo_stats(stats: GitHubRepoStatsFfi) {
    free_c_string_ptr(stats.stars);
    free_c_string_ptr(stats.watching);
    free_c_string_ptr(stats.forks);
    free_c_string_ptr(stats.releases);
    free_c_string_ptr(stats.used);
    free_c_string_ptr(stats.contributors);
    free_c_string_ptr(stats.error);
}

pub fn get_github_repo_stats(url: *const c_char) -> Result<GitHubRepoStats, String> {
    if url.is_null() {
        return Err("url pointer is null".to_string());
    }

    let c_str = unsafe { CStr::from_ptr(url) };
    let url_str = c_str
        .to_str()
        .map_err(|e| format!("url is not valid UTF-8: {e}"))?;
    get_github_repo_stats_from_url(url_str)
}

fn get_github_repo_stats_from_url(url_str: &str) -> Result<GitHubRepoStats, String> {
    let mut stats = GitHubRepoStats::default();
    let response = reqwest::blocking::get(url_str)
        .map_err(|e| format!("failed to fetch URL '{url_str}': {e}"))?;
    let html_content = response
        .text()
        .map_err(|e| format!("failed to read response body: {e}"))?;
    // debug!("{html_content}");

    let document = Html::parse_document(&html_content);
    let div_selector = Selector::parse("div.mt-2").unwrap();
    let strong_selector = Selector::parse("strong").unwrap();
    let a_selector = Selector::parse("a.Link").unwrap();

    let stats_divs = document.select(&div_selector);
    // debug!("mt-2 divs: {}", &stats_divs.clone().count());

    for stats_div in stats_divs {
        let a_hrefs = stats_div.select(&a_selector);
        for a_href in a_hrefs {
            let stat_type = a_href
                .inner_html()
                .split_whitespace()
                .last()
                .map(str::to_owned);
            // debug!("stat type: {:?}", stat_type);

            let stat_val = a_href
                .select(&strong_selector)
                .next()
                .map(|a| a.inner_html());
            // debug!("stat value: {:?}", stat_val.clone().unwrap_or_default());

            match (stat_type, stat_val) {
                (None, None) => (),
                (None, Some(_)) => (),
                (Some(_), None) => (),
                (Some(st), Some(sv)) => {
                    if let Ok(st_enum) = StatType::from_str(st.as_str()) {
                        match st_enum {
                            StatType::Stars => stats.stars = sv,
                            StatType::Forks => stats.forks = sv,
                            StatType::Watching => stats.watching = sv,
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    // println!("stats: {:?}", stats);
    get_releases(document, &mut stats);
    // debug!("stats: {:?}", stats);
    Ok(stats)
}

fn get_releases(document: Html, stats: &mut GitHubRepoStats) {
    let a_selector = Selector::parse("a.Link--primary").unwrap();
    let span_selector = Selector::parse("span.Counter").unwrap();

    let a_hrefs = document.select(&a_selector);
    for a_href in a_hrefs {
        let stat_type = a_href
            .inner_html()
            .split_whitespace()
            .next()
            .map(str::to_owned);
        info!("stat type: {:?}", stat_type);

        let stat_val = a_href.select(&span_selector).next().map(|a| a.inner_html());
        info!("stat value: {:?}", stat_val.clone().unwrap_or_default());

        match (stat_type, stat_val) {
            (None, None) => (),
            (None, Some(_)) => (),
            (Some(_), None) => (),
            (Some(st), Some(sv)) => {
                if let Ok(st_enum) = StatType::from_str(st.as_str()) {
                    match st_enum {
                        StatType::Releases => stats.releases = sv,
                        StatType::Used => stats.used = sv,
                        StatType::Contributors => stats.contributors = sv,
                        _ => (),
                    }
                }
            }
        }
    }
}

fn to_c_string_ptr(mut value: String) -> *mut c_char {
    if value.contains('\0') {
        value.retain(|ch| ch != '\0');
    }

    match CString::new(value) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => CString::default().into_raw(),
    }
}

fn free_c_string_ptr(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use pretty_assertions::assert_ne;

    #[test]
    fn test_get_github_repo_stats() {
        let _ = simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .init();

        let url = "https://github.com/rust-lang/rust";
        let stats = get_github_repo_stats_from_url(url).expect("expected stats fetch to succeed");
        info!(
            "stats: {:?}, stars: {}, forks: {}, watching: {}",
            stats,
            stats.get_stars(),
            stats.get_forks(),
            stats.get_watching()
        );
        assert_ne!(stats.get_stars(), 0);
        assert_ne!(stats.get_forks(), 0);
        assert_ne!(stats.get_watching(), 0);
    }

    #[test]
    fn test_get_github_repo_stats_ffi_null_url() {
        let stats = get_github_repo_stats_ffi(std::ptr::null());
        assert_eq!(stats.success, 0);
        assert_ne!(stats.error, std::ptr::null_mut());

        let error = unsafe { CStr::from_ptr(stats.error) }
            .to_str()
            .expect("expected utf-8 error message");
        assert!(error.contains("null"));

        free_github_repo_stats(stats);
    }
}
