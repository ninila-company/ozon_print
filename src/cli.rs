use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "ozon-print",
    about = "Терминальное приложение для печати штрихкодов в формате PDF",
    version,
    author
)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE", help = "Путь к конфигурационному файлу")]
    pub config: Option<PathBuf>,

    #[arg(
        short = 'd',
        long = "directory",
        value_name = "DIR",
        help = "Временная папка для работы"
    )]
    pub directory: Option<PathBuf>,

    #[arg(
        short = 'p',
        long = "printer",
        value_name = "NAME",
        help = "Имя принтера"
    )]
    pub printer: Option<String>,

    #[arg(short, long, help = "Подробный вывод")]
    pub verbose: bool,

    #[arg(
        short = 'b',
        long = "barcode-dir",
        value_name = "DIR",
        help = "Директория со штрихкодами"
    )]
    pub barcode_dir: Option<PathBuf>,
}