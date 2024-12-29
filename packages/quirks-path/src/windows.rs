use nom::{
    bytes::complete::tag,
    character::complete::{self, satisfy},
    combinator::peek,
    error::{context, ContextError, Error, ErrorKind, ParseError},
    sequence::pair,
    AsChar, IResult, InputIter, InputTakeAtPosition,
};

use crate::Prefix;

fn non_slash(input: &str) -> IResult<&str, &str> {
    input.split_at_position_complete(|item| item != '/')
}

pub fn parse_drive(path: &str) -> IResult<&str, char> {
    context("drive", satisfy(char::is_alpha))(path).map(|a: (&str, char)| a)
}

pub fn parse_drive_exact(path: &str) -> IResult<&str, char> {
    context("drive_exact", pair(parse_drive, complete::char(':')))(path)
        .map(|(path, (drive, _))| (path, drive))
}

pub fn is_windows_verbatim_sep(c: char) -> bool {
    c == '\\'
}

pub fn is_windows_sep(c: char) -> bool {
    c == '\\' || c == '/'
}

pub fn parse_windows_next_component(path: &str, verbatim: bool) -> (&str, &str, &str) {
    let separator = if verbatim {
        is_windows_verbatim_sep
    } else {
        is_windows_sep
    };
    let p = path.as_bytes();
    match p.position(|x| separator(x as char)) {
        Some(separator_start) => {
            let separator_end = separator_start + 1;
            let component = &path[0..separator_start];
            let path_with_sep = &path[separator_start..];
            let path_without_sep = &path[separator_end..];

            (component, path_with_sep, path_without_sep)
        }
        None => (path, "", ""),
    }
}

fn context_verify_error<'a>(input: &'a str, context: &'static str) -> nom::Err<Error<&'a str>> {
    nom::Err::Error(Error::add_context(
        input,
        context,
        Error::from_error_kind(input, ErrorKind::Verify),
    ))
}

pub fn parse_windows_path_prefix(raw_path: &str) -> IResult<&str, Prefix<'_>> {
    if let Ok((path, _)) = tag(r"\\")(raw_path) as IResult<&str, &str> {
        if let Ok((path, _)) = tag(r"?\")(path) as IResult<&str, &str> {
            if let Ok((path, _)) = peek(non_slash)(path) as IResult<&str, &str> {
                if let Ok((path, _)) = tag(r"UNC\")(path) as IResult<&str, &str> {
                    let (server, _, other) = parse_windows_next_component(path, true);
                    let (share, next_input, _) = parse_windows_next_component(other, true);

                    return Ok((next_input, Prefix::VerbatimUNC { server, share }));
                } else if let Ok((path, drive)) = parse_drive_exact(path) {
                    return Ok((path, Prefix::VerbatimDisk { drive }));
                } else {
                    let (prefix, next_input, _) = parse_windows_next_component(path, true);
                    return Ok((next_input, Prefix::Verbatim { prefix }));
                }
            }
        }
        if let Ok((path, _)) = tag(r".\")(path) as IResult<&str, &str> {
            let (prefix, next_input, _) = parse_windows_next_component(path, false);
            return Ok((next_input, Prefix::DeviceNS { device: prefix }));
        }
        let (server, _, other) = parse_windows_next_component(path, false);
        let (share, next_input, _) = parse_windows_next_component(other, false);

        if !server.is_empty() && !share.is_empty() {
            return Ok((next_input, Prefix::UNC { server, share }));
        }
        Err(context_verify_error(raw_path, "windows path prefix"))
    } else if let Ok((path, drive)) = parse_drive_exact(raw_path) {
        Ok((path, Prefix::Disk { drive }))
    } else {
        Err(context_verify_error(raw_path, "windows path prefix"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_windows_path_prefix() {
        use super::*;
        assert_eq!(
            parse_windows_path_prefix(r"\\?\UNC\server\share\path"),
            Ok((
                r"\path",
                Prefix::VerbatimUNC {
                    server: "server",
                    share: "share"
                }
            ))
        );
        assert_eq!(
            parse_windows_path_prefix(r"\\?\C:\path"),
            Ok((r"\path", Prefix::VerbatimDisk { drive: 'C' }))
        );
        assert_eq!(
            parse_windows_path_prefix(r"\\server\share\path"),
            Ok((
                r"\path",
                Prefix::UNC {
                    server: "server",
                    share: "share"
                }
            ))
        );
        assert_eq!(
            parse_windows_path_prefix(r"C:\path"),
            Ok((r"\path", Prefix::Disk { drive: 'C' }))
        );
        assert_eq!(
            parse_windows_path_prefix(r"\\.\device\path"),
            Ok((r"\path", Prefix::DeviceNS { device: "device" }))
        );
        assert_eq!(
            parse_windows_path_prefix(r"\\?\abc\path"),
            Ok((r"\path", Prefix::Verbatim { prefix: "abc" }))
        )
    }
}
