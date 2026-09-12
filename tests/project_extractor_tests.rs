use bytelog::activity::ProjectExtractor;

#[test]
fn test_vscode_project_extraction() {
    // 3-part title with file and workspace
    assert_eq!(
        ProjectExtractor::extract("code.exe", Some("main.rs - byte-log - Visual Studio Code")),
        Some("byte-log".to_string())
    );

    // 2-part title with workspace only
    assert_eq!(
        ProjectExtractor::extract("code.exe", Some("byte-log - Visual Studio Code")),
        Some("byte-log".to_string())
    );

    // 2-part title with standalone file (should be rejected for privacy)
    assert_eq!(
        ProjectExtractor::extract("code.exe", Some("passwords.env - Visual Studio Code")),
        None
    );
    assert_eq!(
        ProjectExtractor::extract("code.exe", Some("secret.rs - Visual Studio Code")),
        None
    );

    // Cursor editor
    assert_eq!(
        ProjectExtractor::extract("cursor.exe", Some("Cargo.toml - my-cool-app - Cursor")),
        Some("my-cool-app".to_string())
    );
}

#[test]
fn test_jetbrains_project_extraction() {
    assert_eq!(
        ProjectExtractor::extract(
            "idea64.exe",
            Some("byte-log [C:\\Users\\Azeem\\projects\\byte-log] - IntelliJ IDEA")
        ),
        Some("byte-log".to_string())
    );

    assert_eq!(
        ProjectExtractor::extract("pycharm64.exe", Some("data-pipeline – main.py")),
        Some("data-pipeline".to_string())
    );

    assert_eq!(
        ProjectExtractor::extract("rustrover64.exe", Some("kernel [~/src/kernel]")),
        Some("kernel".to_string())
    );
}

#[test]
fn test_vim_and_sublime_project_extraction() {
    assert_eq!(
        ProjectExtractor::extract("nvim.exe", Some("byte-log - NVIM")),
        Some("byte-log".to_string())
    );

    assert_eq!(
        ProjectExtractor::extract("sublime_text.exe", Some("app.js - frontend - Sublime Text")),
        Some("frontend".to_string())
    );
}

#[test]
fn test_terminal_project_extraction() {
    assert_eq!(
        ProjectExtractor::extract(
            "windowsterminal.exe",
            Some("C:\\Users\\Azeem\\Documents\\github\\byte-log")
        ),
        Some("byte-log".to_string())
    );

    assert_eq!(
        ProjectExtractor::extract("alacritty.exe", Some("/home/user/code/my-rust-project")),
        Some("my-rust-project".to_string())
    );

    // Generic terminal shell titles must be ignored
    assert_eq!(
        ProjectExtractor::extract("powershell.exe", Some("Windows PowerShell")),
        None
    );
    assert_eq!(
        ProjectExtractor::extract("cmd.exe", Some("Command Prompt")),
        None
    );
}

#[test]
fn test_unrecognized_apps() {
    assert_eq!(
        ProjectExtractor::extract("firefox.exe", Some("Google - Mozilla Firefox")),
        None
    );
    assert_eq!(
        ProjectExtractor::extract("slack.exe", Some("General - ByteLog Slack")),
        None
    );
}
