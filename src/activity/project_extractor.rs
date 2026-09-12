/// Safe, privacy-preserving workspace and project name extractor.
///
/// Extracts project names from focused IDE, editor, and terminal window titles.
/// Adheres strictly to ByteLog's privacy guarantee:
/// 1. File names, source code titles, and full paths are never retained.
/// 2. Only the standalone workspace/project directory name is extracted.
/// 3. If a title only contains a file name without a workspace or is unrecognized,
///    it returns `None` (unattributed).

pub struct ProjectExtractor;

impl ProjectExtractor {
    pub fn extract(executable_name: &str, window_title: Option<&str>) -> Option<String> {
        let title = window_title?.trim();
        if title.is_empty() {
            return None;
        }

        let exec = executable_name.to_lowercase();
        let exec_stem = exec.strip_suffix(".exe").unwrap_or(&exec);

        match exec_stem {
            // VS Code, Cursor, VSCodium, Windsurf
            "code" | "cursor" | "codium" | "vscodium" | "windsurf" | "code - oss" => {
                Self::extract_vscode(title)
            }

            // JetBrains IDEs
            "idea64" | "pycharm64" | "pycharm" | "rustrover64" | "rustrover"
            | "clion64" | "clion" | "webstorm64" | "webstorm" | "goland64"
            | "goland" | "rider64" | "rider" | "studio64" => {
                Self::extract_jetbrains(title)
            }

            // Visual Studio
            "devenv" => Self::extract_visual_studio(title),

            // Sublime Text
            "sublime_text" => Self::extract_sublime(title),

            // Vim / Neovim
            "nvim" | "vim" | "gvim" => Self::extract_vim(title),

            // Terminals
            "windowsterminal" | "alacritty" | "wezterm" | "wezterm-gui" | "kitty"
            | "foot" | "ghostty" | "powershell" | "cmd" => Self::extract_terminal(title),

            _ => None,
        }
    }

    fn extract_vscode(title: &str) -> Option<String> {
        // VS Code formats:
        // "[file] - [workspace] - Visual Studio Code" -> 3 parts, workspace is parts[1]
        // "[workspace] - Visual Studio Code" -> 2 parts, workspace is parts[0]
        // "[file] - Visual Studio Code" -> 2 parts, but parts[0] is a file (has ext)
        let parts: Vec<&str> = title.split(" - ").collect();

        if parts.len() >= 3 {
            // Example: "main.rs - byte-log - Visual Studio Code"
            let candidate = parts[parts.len() - 2].trim();
            if Self::is_valid_project_name(candidate) {
                return Some(candidate.to_string());
            }
        } else if parts.len() == 2 {
            // Example: "byte-log - Visual Studio Code"
            let candidate = parts[0].trim();
            // If it contains a file extension (e.g. "main.rs"), it is NOT a workspace!
            if !candidate.contains('.') && Self::is_valid_project_name(candidate) {
                return Some(candidate.to_string());
            }
        }

        None
    }

    fn extract_jetbrains(title: &str) -> Option<String> {
        // JetBrains formats:
        // "project_name – file.rs [project_name]"
        // "project_name [C:\path\to\project] - ... - IntelliJ IDEA"
        // "project_name – main.rs"
        let normalized = title.replace('–', "-");
        let parts: Vec<&str> = normalized.split(" - ").collect();

        if !parts.is_empty() {
            let first = parts[0].trim();
            // Could be "project_name [path]"
            let project_candidate = if let Some(idx) = first.find('[') {
                first[..idx].trim()
            } else {
                first
            };

            if Self::is_valid_project_name(project_candidate) {
                return Some(project_candidate.to_string());
            }
        }

        None
    }

    fn extract_visual_studio(title: &str) -> Option<String> {
        // Example: "byte-log - Microsoft Visual Studio"
        let parts: Vec<&str> = title.split(" - ").collect();
        if parts.len() >= 2 {
            let candidate = parts[0].trim();
            if !candidate.contains('.') && Self::is_valid_project_name(candidate) {
                return Some(candidate.to_string());
            }
        }
        None
    }

    fn extract_sublime(title: &str) -> Option<String> {
        // Example: "main.rs - byte-log - Sublime Text"
        let parts: Vec<&str> = title.split(" - ").collect();
        if parts.len() >= 3 {
            let candidate = parts[parts.len() - 2].trim();
            if Self::is_valid_project_name(candidate) {
                return Some(candidate.to_string());
            }
        }
        None
    }

    fn extract_vim(title: &str) -> Option<String> {
        // Example: "byte-log - NVIM" or "[byte-log] NVIM"
        let trimmed = title
            .trim_end_matches(" - NVIM")
            .trim_end_matches(" - VIM")
            .trim();

        if trimmed != title && Self::is_valid_project_name(trimmed) {
            return Some(trimmed.to_string());
        }

        None
    }

    fn extract_terminal(title: &str) -> Option<String> {
        // Terminals often set the title to cwd:
        // e.g. "C:\Users\Azeem\Documents\github\my projects\byte-log"
        // or "~/github/byte-log"
        // Discard generic shell titles or non-path titles
        let lower = title.to_lowercase();
        if lower == "powershell"
            || lower == "windows powershell"
            || lower == "cmd"
            || lower == "command prompt"
            || lower == "terminal"
            || lower == "bash"
            || lower == "zsh"
        {
            return None;
        }

        // Terminals must contain path separators to reliably extract project directory
        if !title.contains('\\') && !title.contains('/') {
            return None;
        }

        // If title is a path, grab last directory component
        let path_sep = if title.contains('\\') { '\\' } else { '/' };
        let segments: Vec<&str> = title
            .split(path_sep)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if let Some(last) = segments.last() {
            // Filter out root paths like "C:" or "~"
            if last.ends_with(':') || *last == "~" {
                return None;
            }
            if Self::is_valid_project_name(last) {
                return Some((*last).to_string());
            }
        }

        None
    }

    /// Validation to ensure only safe project names are stored.
    /// Excludes filenames with common extensions, paths, and generic strings.
    fn is_valid_project_name(name: &str) -> bool {
        let trimmed = name.trim();
        if trimmed.len() < 2 || trimmed.len() > 64 {
            return false;
        }

        // Must not contain path separators
        if trimmed.contains('/') || trimmed.contains('\\') {
            return false;
        }

        // Exclude common file extensions
        const FILE_EXTS: &[&str] = &[
            ".rs", ".js", ".ts", ".jsx", ".tsx", ".py", ".go", ".c", ".cpp", ".h",
            ".java", ".kt", ".rb", ".php", ".html", ".css", ".md", ".json", ".toml",
            ".yaml", ".yml", ".txt", ".sql", ".sh", ".bat", ".ps1", ".lock",
        ];
        let lower = trimmed.to_lowercase();
        for ext in FILE_EXTS {
            if lower.ends_with(ext) {
                return false;
            }
        }

        // Exclude generic window title suffixes
        const GENERIC_EXCLUDES: &[&str] = &[
            "visual studio code",
            "visual studio",
            "cursor",
            "sublime text",
            "untitled",
            "administrator",
            "command prompt",
            "powershell",
            "windows powershell",
            "terminal",
            "new tab",
        ];
        if GENERIC_EXCLUDES.contains(&lower.as_str()) {
            return false;
        }

        // Must contain at least one alphanumeric character
        trimmed.chars().any(|c| c.is_alphanumeric())
    }
}
