use dialoguer::{Input, theme::ColorfulTheme};

pub fn unwrap_or_prompt(arg: Option<String>, prompt: &str) -> Result<String, anyhow::Error> {
    match arg {
        Some(arg) => Ok(arg),
        None => request_missing_arg(prompt),
    }
}

pub fn request_missing_arg(prompt: &str) -> Result<String, anyhow::Error> {
    return Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()
        .map_err(|e| anyhow::anyhow!(e));
}
