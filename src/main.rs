mod app;
mod cli;
mod config;
mod error;
mod preview;
mod printer;
mod scanner;
mod tui;
mod ui;

use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

use crate::app::App;
use crate::cli::Cli;
use crate::config::Config;
use crate::error::AppError;

fn main() {
    if let Err(e) = run() {
        eprintln!("Ошибка: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .init();
    }

    let (config, config_path) = load_config(&cli)?;
    
    let config_path = if let Some(config_path_arg) = &cli.config {
        config_path_arg.clone()
    } else {
        config_path
    };
    
    config.validate().map_err(AppError::from)?;

    let mut app = App::new(config, config_path).map_err(AppError::from)?;
    app.run().map_err(AppError::from)?;
    
    Ok(())
}

fn load_config(cli: &Cli) -> Result<(Config, PathBuf), AppError> {
    let config_path = if let Some(config_path) = &cli.config {
        config_path.clone()
    } else {
        get_default_config_path()
    };
    
    let config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        let config = Config::default();
        config.save(&config_path)?;
        config
    };

    let config = config.merge_with_args(
        cli.barcode_dir.clone(),
        cli.printer.clone(),
    );

    if let Some(dir) = &cli.directory {
        let mut cfg = config;
        cfg.temp_directory = dir.clone();
        return Ok((cfg, config_path));
    }

    Ok((config, config_path))
}

fn get_default_config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config_for_ozon_print.json")
}