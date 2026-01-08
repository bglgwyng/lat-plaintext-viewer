use clap::Parser;
use std::fs;

#[derive(Parser)]
#[command(name = "lat-plaintext")]
#[command(about = "cat for LLMs - plaintext version")]
struct Args {
    /// File path
    file: String,

    /// Maximum character count
    #[arg(short = 'c', long = "chars")]
    chars: Option<usize>,

    /// Line ranges to focus on (e.g., 10:20,30:40 or #more:50)
    #[arg(short, long, value_delimiter = ',')]
    focus: Option<Vec<String>>,
}

fn parse_more_anchor(path: &str) -> Option<usize> {
    if let Some(pos) = path.find("#more:") {
        let line_str = &path[pos + 6..];
        line_str.parse().ok()
    } else {
        None
    }
}

fn parse_range(range: &str) -> Option<(usize, Option<usize>)> {
    if let Some(start_line) = parse_more_anchor(range) {
        return Some((start_line, None));
    }
    if let Some(colon_pos) = range.find(':') {
        let start: usize = range[..colon_pos].parse().ok()?;
        let end: usize = range[colon_pos + 1..].parse().ok()?;
        Some((start, Some(end)))
    } else {
        let line: usize = range.parse().ok()?;
        Some((line, Some(line)))
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

fn extract_lines(lines: &[&str], start: usize, end: Option<usize>) -> String {
    let start_idx = if start <= 1 { 0 } else { start - 1 };
    let end_idx = match end {
        Some(e) => e.min(lines.len()),
        None => lines.len(),
    };
    if start_idx >= lines.len() {
        return String::new();
    }
    lines[start_idx..end_idx].join("\n")
}

fn main() {
    let args = Args::parse();

    let content = fs::read_to_string(&args.file).expect("Failed to read file");
    let lines: Vec<&str> = content.lines().collect();

    let output = if let Some(ref focuses) = args.focus {
        let mut results = Vec::new();
        for focus in focuses {
            let (start, end) = parse_range(focus).unwrap_or((1, None));
            let extracted = extract_lines(&lines, start, end);
            let header = match end {
                Some(e) => format!("// {}:{}", start, e),
                None => format!("// #more:{}", start),
            };
            results.push(format!("{}\n{}", header, extracted));
        }
        results.join("\n\n")
    } else {
        lines.join("\n")
    };

    match args.chars {
        Some(max_chars) if output.len() > max_chars => {
            let (truncated, line_count) = truncate_to_line_boundary(&output, max_chars);
            let next_line = 1 + line_count;
            println!("{}", truncated);
            println!("#more:{}", next_line);
        }
        _ => {
            println!("{}", output);
        }
    }
}
