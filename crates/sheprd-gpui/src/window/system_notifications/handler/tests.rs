use super::*;

#[test]
fn protocol_toasts_require_the_current_executable_as_the_handler() {
    let exe = std::path::Path::new(r"C:\Program Files\Spaceport\spaceport.exe");
    assert!(owns_handler(
        r#""C:\Program Files\Spaceport\spaceport.exe" "%1""#,
        exe
    ));
    for command in [
        r#""C:\Program Files\Sheprd\spaceport.exe" "%1""#,
        r#""C:\Program Files\Spaceport\spaceport.exe" "--run" "%1""#,
        r#""C:\Program Files\Spaceport\spaceport.exe" "%1" --extra"#,
        r#"C:\Program Files\Spaceport\spaceport.exe "%1""#,
        "",
    ] {
        assert!(!owns_handler(command, exe), "{command}");
    }
}
