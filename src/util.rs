//TODO: util files/modules are inherently disorganized, this should be broken
//TODO: into multiple organized files/modules soon.

use std::fs;

// Gets the provided line, along with the one before and the one after it. Used for error messages.
pub fn get_lines_from(source: &str, line: usize) -> Vec<(usize, String)> {
    let itr = source.lines().enumerate();

    let mut lines: Vec<(usize, String)> = vec![];

    let line = line.checked_sub(2).unwrap_or(0);

    itr.skip(line)
        .take(3)
        .for_each(|(index, str_)| lines.push((index + 1, str_.to_string())));

    lines
}

// Gets the provided line, along with the one before and the one after it. Used for error messages.
pub fn get_lines(file_path: &str, line: usize) -> Vec<(usize, String)> {
    get_lines_from(
        fs::read_to_string(file_path)
            .expect(format!("failed to read file {file_path}").as_str())
            .as_str(),
        line,
    )
}
