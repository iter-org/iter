use dialoguer::{Input, theme::ColorfulTheme, console::style};

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

pub fn load_config() -> Result<crate::config::IterConfig, anyhow::Error> {
    // get the config file in current directory
    let config_file = std::env::current_dir()?.join("iter.json");

    // check if the config file exists
    if !config_file.exists() {
        eprintln!(
            "{} {}",
            style("?").yellow().bold(),
            style("run `iter init` to create a new iter.json project file in the current directory").bold(),
        );
        return Err(anyhow::anyhow!(format!("iter.json project file not found in current directory: {}", config_file.display())));
    }

    // read the config file
    let config_file = std::fs::File::open(&config_file)?;
    let config: crate::config::IterConfig = serde_json::from_reader(config_file)?;

    Ok(config)
}