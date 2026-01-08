use clap::Parser;
use std::fs;

#[derive(Parser)]
#[command(name = "lat-plaintext")]
#[command(about = "cat for LLMs - plaintext version")]
struct Args {
    /// File path (e.g., file.txt or file.txt#more:100)
    file: String,

    /// Maximum character count
    #[arg(short = 'c', long = "chars")]
    chars: Option<usize>,
}

fn parse_more_anchor(path: &str) -> Option<(&str, usize)> {
    if let Some(pos) = path.find("#more:") {
        let file_path = &path[..pos];
        let line_str = &path[pos + 6..];
        let line: usize = line_str.parse().ok()?;
        Some((file_path, line))
    } else {
        None
    }
}

fn truncate_to_line_boundary(s: &str, max_chars: usize) -> (&str, usize) {
    if s.len() <= max_chars {
        return (s, 0);
    }
    let truncated = &s[..max_chars];
    let last_newline = truncated.rfind('\n').unwrap_or(max_chars);
    let result = &truncated[..last_newline];
    let line_count = result.chars().filter(|c| *c == '\n').count() + 1;
    (result, line_count)
}

fn main() {
    let args = Args::parse();

    let (file_path, start_line) = parse_more_anchor(&args.file).unwrap_or((&args.file, 1));

    let content = fs::read_to_string(file_path).expect("Failed to read file");

    let lines: Vec<&str> = content.lines().collect();
    let output_lines = if start_line <= 1 {
        &lines[..]
    } else if start_line > lines.len() {
        &lines[0..0]
    } else {
        &lines[start_line - 1..]
    };

    let output = output_lines.join("\n");

    match args.chars {
        Some(max_chars) if output.len() > max_chars => {
            let (truncated, line_count) = truncate_to_line_boundary(&output, max_chars);
            let next_line = start_line + line_count;
            println!("{}", truncated);
            println!("#more:{}", next_line);
        }
        _ => {
            println!("{}", output);
        }
    }
}
