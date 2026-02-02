use std::{
    fs::File as FsFile,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use ignore::WalkBuilder;

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value = ".")]
    input: PathBuf,
    #[arg(short, long, default_value = "dump.md")]
    output: PathBuf,
}

fn get_files(path: &Path) -> impl Iterator<Item = ignore::DirEntry> {
    WalkBuilder::new(path)
        .build()
        .filter_map(|res| res.ok())
        .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_file()))
}

fn write_to_dump<I>(paths: I, output_path: &Path) -> std::io::Result<()>
where
    I: IntoIterator<Item = PathBuf>,
{
    let file = FsFile::create(output_path)?;
    let mut writer = BufWriter::new(file);

    for path in paths {
        if path == output_path {
            continue;
        }

        let mut reader = match FsFile::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Критическая ошибка чтения {:?}: {}", path, e);
                continue;
            }
        };

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        writer.write_all(format!("### {}\n```\n", name).as_bytes())?;

        if let Err(e) = std::io::copy(&mut reader, &mut writer) {
            eprintln!("Ошибка при копировании {:?}: {}", path, e);
        }

        writer.write_all(b"\n```\n\n")?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input_path = std::fs::canonicalize(&args.input)?;

    let mut output_path = args.output.clone();
    if output_path.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            output_path = cwd.join(&output_path)
        }
    }

    let paths = get_files(&input_path).map(|entry| entry.into_path());
    let _ = write_to_dump(paths, &output_path);

    println!("Файлы записаны в {:?}", args.output);
    Ok(())
}
