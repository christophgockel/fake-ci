pub fn combine_lines(lines: &Vec<String>) -> String {
    let mut all_lines = vec![];

    for line in lines {
        all_lines.push(wrap_itself_with_echo(line));
        all_lines.push(line.clone());
    }

    all_lines.join(";")
}

pub fn wrap_itself_with_echo(command: &str) -> String {
    format!("echo -e \"\\e[1;32m{}\\e[0m\"", command)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn wraps_command_with_shell_escape_sequence() {
        let bla = wrap_itself_with_echo("cat file.txt");

        assert_eq!(bla, "echo -e \"\\e[1;32mcat file.txt\\e[0m\"");
    }

    #[test]
    fn combines_script_lines_into_single_string_with_echoed_commands() {
        let script = vec!["cat file.txt".to_string()];
        let combined = combine_lines(&script);

        // Instead of duplicating the full resulting string these assertions
        // match the semantics of what `combine()` does.
        // Containing the actual command twice ("echo wrapped" command and the command itself).
        assert_eq!(combined.matches("cat file.txt").count(), 2);
        // And each wrapped and actual command is separated by a semicolon.
        assert_eq!(combined.matches(";cat file.txt").count(), 1);
    }
}
