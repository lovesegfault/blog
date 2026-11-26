//! Addresses, and whether they lead anywhere.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::output::{File, Files};

const ADDRESSES: [&str; 4] = [" href=\"", ":href=\"", " src=\"", " poster=\""];

fn is_external(url: &str) -> bool {
    let scheme = url.split_once(':').map_or("", |(scheme, _)| scheme);
    url.starts_with("//")
        || !scheme.is_empty() && scheme.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"+-".contains(&byte))
}

/// `url` from the root of the site, for a link on the page at `base`.
pub fn resolve(base: &str, url: &str) -> String {
    if is_external(url) || url.starts_with(['/', '#']) {
        return url.to_owned();
    }
    let (mut folder, mut rest) = (&base[..=base.rfind('/').unwrap_or(0)], url);
    while let Some((dots @ ("." | ".."), after)) = Some(rest.split_once('/').unwrap_or((rest, ""))) {
        if dots == ".." {
            folder = &folder[..=folder.trim_end_matches('/').rfind('/').unwrap_or(0)];
        }
        rest = after;
    }
    format!("{folder}{rest}")
}

/// `html` from the page at `base`, with addresses that work away from the site.
pub fn absolute(html: &str, site: &str, base: &str) -> String {
    let mut out = String::new();
    for part in html.split_inclusive("=\"") {
        if ADDRESSES.iter().any(|attribute| out.ends_with(attribute)) && !is_external(part) {
            out.extend([site, if part.starts_with('#') { base } else { "" }, &resolve(base, part)]);
        } else {
            out += part;
        }
    }
    out
}

pub fn encode(text: &str) -> String {
    let escape = |byte: u8| if byte.is_ascii_alphanumeric() { char::from(byte).into() } else { format!("%{byte:02X}") };
    text.bytes().map(escape).collect()
}

/// Undoes `%20` and the like.
pub fn decode(url: &str) -> String {
    let mut parts = url.split('%');
    let mut bytes = parts.next().unwrap_or_default().as_bytes().to_vec();
    for part in parts {
        let byte = part.get(..2).and_then(|hex| u8::from_str_radix(hex, 16).ok());
        bytes.push(byte.unwrap_or(b'%'));
        bytes.extend(part.bytes().skip(if byte.is_some() { 2 } else { 0 }));
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// What a server would send for `path`.
pub fn find<'a>(files: &'a Files, path: &str) -> Option<(&'a String, &'a File)> {
    files.get_key_value(path).or_else(|| files.get_key_value(&format!("{}/index.html", path.trim_end_matches('/'))))
}

/// Text never has bare quotes in it, so whatever follows `opening` is the value of an attribute.
pub fn values<'a>(html: &'a str, opening: &'a str) -> impl Iterator<Item = String> {
    html.split(opening).skip(1).filter_map(|rest| rest.split('"').next()).map(|value| value.replace("&amp;", "&"))
}

/// Every link within the site that is broken.
pub fn check(files: &Files, site: &str) -> BTreeSet<String> {
    let page = |(path, file): (&'_ String, &File)| match file {
        File::Made(html) if path.ends_with(".html") => Some((path.clone(), String::from_utf8_lossy(html).into_owned())),
        _ => None,
    };
    let pages: HashMap<String, String> = files.iter().filter_map(page).collect();
    let (mut problems, mut anchors) = (BTreeSet::new(), HashSet::new());
    for (path, html) in &pages {
        for id in values(html, " id=\"") {
            if let Some((_, id)) = anchors.replace((path, id)) {
                problems.insert(format!("{path}: more than one thing is called `{id}`"));
            }
        }
    }
    for (path, html) in &pages {
        for url in ADDRESSES.iter().flat_map(|attribute| values(html, attribute)).filter(|url| !is_external(url)) {
            let (address, anchor) = url.split_once('#').unwrap_or((&url, ""));
            let target = match address.split('?').next().unwrap() {
                "" => Some(path),
                address => find(files, &decode(&resolve(path, address))).map(|(target, _)| target),
            };
            let has_anchor = |target| !pages.contains_key(target) || anchors.contains(&(target, decode(anchor)));
            if !target.is_some_and(|target| anchor.is_empty() || has_anchor(target)) {
                problems.insert(format!("{path}: `{url}` leads nowhere"));
            }
        }
    }
    // The feed has pictures that the pages only have prints of.
    if let Some(File::Made(feed)) = files.get("/index.xml") {
        let feed = String::from_utf8_lossy(feed).replace("&quot;", "\"").replace("&amp;", "&");
        for url in values(&feed, &format!("src=\"{site}")) {
            if find(files, &decode(url.split(['?', '#']).next().unwrap())).is_none() {
                problems.insert(format!("/index.xml: `{url}` leads nowhere"));
            }
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_addresses_are_resolved() {
        for (url, expected) in [("./a.jpg", "/p/x/a.jpg"), ("../y/#z", "/p/y/#z"), ("../../..", "/"), ("", "/p/x/")] {
            assert_eq!(resolve("/p/x/", url), expected);
        }
        for url in ["#fn:1", "/about/", "mailto:a@b", "//c.d"] {
            assert_eq!(resolve("/p/x/", url), url);
        }
        assert_eq!(
            absolute(r##"<a id="a" href="#b"><img src="./c"><a href="/?d="><a href="e:f">"##, "//s", "/p/"),
            r##"<a id="a" href="//s/p/#b"><img src="//s/p/c"><a href="//s/?d="><a href="e:f">"##
        );
        assert_eq!(decode("a%20%C3%A9%"), "a é%");
        assert_eq!(decode(&encode("C# & é")), "C# & é");
    }

    #[test]
    fn broken_links_are_found() {
        let html = r##"<p id="a"><a href="#a"><a href="#b"><a href="/x"><a href="y"><a href="/x/#c"><i id="a">"##;
        let files = [("/index.html", html), ("/index.xml", "src=&quot;//s/z&quot;"), ("/x/index.html", "")];
        let files = files.map(|(path, html)| (path.to_owned(), File::Made(html.into()))).into();
        let found = check(&files, "//s");
        assert!(found.iter().filter_map(|problem| problem.split('`').nth(1)).eq(["#b", "/x/#c", "y", "a", "/z"]));
    }
}
