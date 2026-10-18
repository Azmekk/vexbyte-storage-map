use std::path::{Component, Path};

/// Win32 path parsing, which the shell uses, strips trailing dots and spaces from every name, so
/// the shell would act on a different item than the one named.
pub(crate) fn has_name_the_shell_would_rewrite(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(name) => {
            let name = name.to_string_lossy();
            name.ends_with('.') || name.ends_with(' ')
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_ending_in_a_dot_or_space_are_rewritten() {
        assert!(has_name_the_shell_would_rewrite(Path::new(r"C:\notes.")));
        assert!(has_name_the_shell_would_rewrite(Path::new(
            r"C:\folder \inside.jpg"
        )));
        assert!(!has_name_the_shell_would_rewrite(Path::new(
            r"C:\photos\photo.jpg"
        )));
        assert!(!has_name_the_shell_would_rewrite(Path::new(r"C:\.hidden")));
    }
}
