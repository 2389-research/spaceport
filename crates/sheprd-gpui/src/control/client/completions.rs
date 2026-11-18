// ABOUTME: Generates shell completion scripts from the command family catalog.
// ABOUTME: Delivers local Bash, Fish and Zsh scripts without querying the app.
use super::Output;
use crate::cli::control::{FAMILIES, Shell};

pub(super) fn completions(shell: Shell) -> crate::Result<Output> {
    let names = FAMILIES
        .iter()
        .map(|family| family.name)
        .collect::<Vec<_>>()
        .join(" ");
    let script = match shell {
        Shell::Bash => format!(
            "complete -W '{names} --help --json --window --socket --build-info' spaceport\n"
        ),
        Shell::Fish => format!(
            "complete -c spaceport -f -n '__fish_use_subcommand' -a '{names}'\ncomplete -c spaceport -l help\ncomplete -c spaceport -l json\ncomplete -c spaceport -l window -r\ncomplete -c spaceport -l socket -r\ncomplete -c spaceport -l build-info\n"
        ),
        Shell::Zsh => format!(
            "#compdef spaceport\n_spaceport() {{\n  local -a commands\n  commands=({names} --help --json --window --socket --build-info)\n  _describe 'command' commands\n}}\ncompdef _spaceport spaceport\n"
        ),
    };
    Output::data(
        script.trim_end(),
        &serde_json::json!({ "shell": shell.name(), "script": script }),
    )
}
