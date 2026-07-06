use std::{
    fs::File as FsFile,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use ignore::WalkBuilder;

const UNKNOWN_NAME: &str = "unknown";

#[derive(Parser, Debug)]
#[command(about = "Дампит файлы из директории в один Markdown-файл")]
struct Args {
    #[arg(default_value = ".", help = "Директории для сканирования")]
    input: Vec<PathBuf>,

    #[arg(
        short,
        long,
        default_value = "dump.md",
        help = "Путь к файлу результата"
    )]
    output: PathBuf,
}

fn walk_source_files(root: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    WalkBuilder::new(root)
        .build()
        .filter_map(|res| {
            if let Err(e) = &res {
                eprintln!("Предупреждение: пропуск записи при обходе: {}", e);
            }
            res.ok()
        })
        .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_file()))
        .map(|entry| entry.into_path())
}

fn display_name<'a>(path: &'a Path, root: &Path) -> &'a str {
    path.strip_prefix(root)
        .ok()
        .and_then(|p| p.to_str())
        .or_else(|| path.to_str())
        .unwrap_or(UNKNOWN_NAME)
}

fn write_dump<I>(files: I, output_path: &Path, root: &Path) -> Result<()>
where
    I: IntoIterator<Item = PathBuf>,
{
    let file = FsFile::create(output_path)
        .with_context(|| format!("Не удалось создать файл дампа: {:?}", output_path))?;
    let mut writer = BufWriter::new(file);

    for path in files {
        if path == output_path {
            continue;
        }

        let mut reader = match FsFile::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Предупреждение: не удалось открыть {:?}: {}", path, e);
                continue;
            }
        };

        let name = display_name(&path, root);

        write!(writer, "### {}\n```\n", name)?;

        if let Err(e) = std::io::copy(&mut reader, &mut writer) {
            eprintln!("Предупреждение: ошибка при копировании {:?}: {}", path, e);
        }

        write!(writer, "\n```\n\n")?;
    }

    writer.flush().context("Не удалось очистить буфер записи")?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let cwd = std::env::current_dir().context("Не удалось получить текущую директорию")?;
    let output_path = if args.output.is_relative() {
        cwd.join(&args.output)
    } else {
        args.output.clone()
    };

    let mut files = Vec::new();
    for input in &args.input {
        let input_path = std::fs::canonicalize(input)
            .with_context(|| format!("Входная директория не найдена: {:?}", input))?;

        files.extend(walk_source_files(&input_path));
    }

    write_dump(files, &output_path, &cwd).context("Не удалось записать дамп")?;

    println!("Файлы записаны в {:?}", args.output);
    Ok(())
}
