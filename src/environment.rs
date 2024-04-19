use once_cell::sync::OnceCell;
use std::{path::PathBuf, str::FromStr};

pub static OVERWRITTEN_PREFIX_DIR: OnceCell<Option<String>> = OnceCell::new();
pub static OVERWRITTEN_HOME_DIR: OnceCell<Option<String>> = OnceCell::new();

pub fn expand_and_prefix_path<P: AsRef<str>>(
    input: &P,
    prefix_dir: Option<String>,
    home_dir: Option<String>,
) -> std::result::Result<PathBuf, core::convert::Infallible> {
    let expanded = shellexpand::tilde_with_context(input, || home_dir);

    let output = prefix_dir
        .map(|prefix_dir| {
            let expanded_without_prefix = expanded.strip_prefix(&prefix_dir).unwrap_or(&expanded);
            format!("{prefix_dir}{expanded_without_prefix}")
        })
        .unwrap_or(expanded.to_string());

    PathBuf::from_str(&output)
}

pub fn expand_path<P: AsRef<str>>(
    input: &P,
) -> std::result::Result<PathBuf, core::convert::Infallible> {
    let home_dir = {
        let default_home_dir = dirs::home_dir().and_then(|d| d.to_str().map(|s| s.to_string()));

        OVERWRITTEN_HOME_DIR
            .get()
            .unwrap_or(&default_home_dir)
            .clone()
    };

    let prefix_dir = OVERWRITTEN_PREFIX_DIR.get().cloned().flatten();

    expand_and_prefix_path(input, prefix_dir, home_dir)

    // TODO: Resolve symlinks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_and_prefix_path_valid() {
        assert_eq!(
            expand_and_prefix_path(&String::from("/"), None, None)
                .map(|path| path.to_str().unwrap().to_string()),
            Ok(String::from("/"))
        );

        assert_eq!(
            expand_and_prefix_path(
                &String::from("~/Downloads"),
                None,
                Some(String::from("/Users/bob"))
            )
            .map(|path| path.to_str().unwrap().to_string()),
            Ok(String::from("/Users/bob/Downloads"))
        );

        assert_eq!(
            expand_and_prefix_path(
                &String::from("/Volume/My SSD"),
                None,
                Some(String::from("/Users/bob"))
            )
            .map(|path| path.to_str().unwrap().to_string()),
            Ok(String::from("/Volume/My SSD"))
        );

        assert_eq!(
            expand_and_prefix_path(
                &String::from("~/Downloads"),
                Some(String::from("/some/prefix/dir")),
                Some(String::from("/Users/bob"))
            )
            .map(|path| path.to_str().unwrap().to_string()),
            Ok(String::from("/some/prefix/dir/Users/bob/Downloads"))
        );

        assert_eq!(
            expand_and_prefix_path(
                &String::from("/Volume/My SSD"),
                Some(String::from("/some/prefix/dir")),
                Some(String::from("/Users/bob"))
            )
            .map(|path| path.to_str().unwrap().to_string()),
            Ok(String::from("/some/prefix/dir/Volume/My SSD"))
        );
    }
}
