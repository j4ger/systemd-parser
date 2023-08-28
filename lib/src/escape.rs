pub(crate) fn escape<S: AsRef<str>>(input: S) -> String {
    let mut result = String::new();
    let mut prev = false;
    for char in input
        .as_ref()
        .trim_start_matches('/')
        .trim_end_matches('/')
        .chars()
    {
        match char {
            '/' => {
                if prev {
                    continue;
                } else {
                    prev = true;
                    result.push('-');
                }
            }
            _ => {
                prev = false;
                result.push(char);
            }
        }
    }
    result
}

pub(crate) fn unescape_path<S: AsRef<str>>(input: S) -> String {
    format!("/{}", input.as_ref().replace('-', "/"))
}

pub(crate) fn unescape_non_path<S: AsRef<str>>(input: S) -> String {
    input.as_ref().replace('-', "/")
}

#[test]
fn test_escape() {
    assert_eq!(escape("/dev//sda"), "dev-sda".to_string());
    assert_eq!(escape("/foo//bar/baz/"), "foo-bar-baz".to_string());
}

#[test]
fn test_unescape_path() {
    assert_eq!(unescape_path("dev-sda"), "/dev/sda".to_string());
}

#[test]
fn test_unescape_non_path() {
    assert_eq!(
        unescape_non_path("normal-escaped-string"),
        "normal/escaped/string".to_string()
    );
}
