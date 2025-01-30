use std::error;

use libps::{ErrorKind, Lexer, Object};

#[test]
fn test_lex_comment() {
    let mut lexer = Lexer::from("% this is a comment".chars());
    let obj = lexer.next();

    assert!(obj.is_none());

    let cases = [
        ("10% this is a comment", Object::Integer(10)),
        ("16#FFFE% this is a comment", Object::Integer(0xFFFE)),
        ("1.0% this is a comment", Object::Real(1.0)),
        ("1.0e7% this is a comment", Object::Real(1.0e7)),
    ];

    for (input, expect) in cases {
        let objs: Vec<Object> = Lexer::from(input.chars()).filter_map(|o| o.ok()).collect();

        assert_eq!(1, objs.len());
        assert_eq!(expect, objs[0]);
    }
}

#[test]
fn test_lex_bad_numeric() -> Result<(), Box<dyn error::Error>> {
    let inputs = ["1x0", "1.x0"];

    for input in inputs {
        let mut lexer = Lexer::from(input.chars());

        let Some(Ok(Object::Name(name))) = lexer.next() else {
            return Err("expected name object".into());
        };

        assert_eq!(input, name);
    }

    Ok(())
}

#[test]
fn test_lex_numeric() -> Result<(), Box<dyn error::Error>> {
    let cases = [
        ("1", Object::Integer(1)),
        ("-1", Object::Integer(-1)),
        ("1234567890", Object::Integer(1234567890)),
        ("2147483648", Object::Real(2147483648.0)),
        (".1", Object::Real(0.1)),
        ("-.1", Object::Real(-0.1)),
        ("1.234567890", Object::Real(1.234567890)),
        ("1.2E7", Object::Real(1.2e7)),
        ("1.2e7", Object::Real(1.2e7)),
        ("-1.2e7", Object::Real(-1.2e7)),
        ("1.2e-7", Object::Real(1.2e-7)),
        ("-1.2e-7", Object::Real(-1.2e-7)),
        ("2#1000", Object::Integer(0b1000)),
        ("8#1777", Object::Integer(0o1777)),
        ("16#fffe", Object::Integer(0xFFFE)),
        ("16#FFFE", Object::Integer(0xFFFE)),
        ("16#ffFE", Object::Integer(0xFFFE)),
    ];

    for (input, expect) in cases {
        let mut lexer = Lexer::from(input.chars());

        let Some(Ok(obj)) = lexer.next() else {
            return Err("expected object".into());
        };

        assert_eq!(expect, obj);
    }

    Ok(())
}

#[test]
fn test_lex_bad_string() -> Result<(), Box<dyn error::Error>> {
    let inputs = ["(this is a string", "(this is a string\\)"];

    for input in inputs {
        let mut lexer = Lexer::from(input.chars());

        let Some(Err(e)) = lexer.next() else {
            return Err("expected error".into());
        };

        assert!(e.kind() == ErrorKind::Syntax);
    }

    Ok(())
}

#[test]
fn test_lex_string() -> Result<(), Box<dyn error::Error>> {
    let cases = [
        ("(this is a string)", "this is a string"),
        (
            "(this is a multiline\nstring)",
            "this is a multiline\nstring",
        ),
        (
            "(this is a multiline\r\nstring)",
            "this is a multiline\r\nstring",
        ),
        (
            "(this has (nested) parenthesis)",
            "this has (nested) parenthesis",
        ),
    ];

    for (input, expect) in cases {
        let mut lexer = Lexer::from(input.chars());

        let Some(Ok(Object::String(string))) = lexer.next() else {
            return Err("expected string object".into());
        };

        assert_eq!(expect, string);
    }

    Ok(())
}

#[test]
fn test_lex_escaped_string() -> Result<(), Box<dyn error::Error>> {
    let cases = [
        ("()", ""),
        ("(\\n)", "\n"),
        ("(\\r)", "\r"),
        ("(\\t)", "\t"),
        ("(\\b)", "\x08"),
        ("(\\f)", "\x0C"),
        ("(\\\\)", "\\"),
        ("(\\()", "("),
        ("(\\))", ")"),
        ("(\\\n)", ""),
        ("(\\\r)", ""),
        ("(\\\r\n)", ""),
    ];

    for (input, expect) in cases {
        let mut lexer = Lexer::from(input.chars());

        let Some(Ok(Object::String(string))) = lexer.next() else {
            return Err("expected token".into());
        };

        assert_eq!(expect, string);
    }

    Ok(())
}

#[test]
fn test_lex_ignore_escaped_string() -> Result<(), Box<dyn error::Error>> {
    let input = "(\\ii)";
    let mut lexer = Lexer::from(input.chars());

    let Some(Ok(Object::String(string))) = lexer.next() else {
        return Err("expected string object".into());
    };

    assert_eq!("ii", string);

    Ok(())
}

#[test]
fn test_lex_octal_string() -> Result<(), Box<dyn error::Error>> {
    let cases = [("(\\000)", "\0"), ("(\\377)", "ÿ")];

    for (input, expect) in cases {
        let mut lexer = Lexer::from(input.chars());

        let Some(Ok(Object::String(string))) = lexer.next() else {
            return Err("expected string object".into());
        };

        assert_eq!(expect, string);
    }

    Ok(())
}

#[test]
fn test_lex_hex_string() -> Result<(), Box<dyn error::Error>> {
    let cases = [
        ("<736F6D65>", "some"),
        ("<736f6d65>", "some"),
        ("<736f6D65>", "some"),
        ("<73 6F 6D 65>", "some"),
        ("<70756D7>", "pump"),
        ("<70756D70>", "pump"),
    ];

    for (input, expect) in cases {
        let mut lexer = Lexer::from(input.chars());

        let Some(Ok(Object::String(string))) = lexer.next() else {
            return Err("expected string object".into());
        };

        assert_eq!(expect, string);
    }

    Ok(())
}

#[test]
fn test_lex_base85_string() -> Result<(), Box<dyn error::Error>> {
    let input = "<~FD,B0+DGm>F)Po,+EV1>F8~>";
    let expect = "this is some text";
    let mut lexer = Lexer::from(input.chars());

    let Some(Ok(Object::String(string))) = lexer.next() else {
        return Err("expected string object".into());
    };

    assert_eq!(expect, string);

    Ok(())
}

#[test]
fn test_lex_multiple_string() -> Result<(), Box<dyn error::Error>> {
    let input = "(this is a literal string) <7468697320697320612068657820737472696E67> <~FD,B0+DGm>@3B#fF(I<g+EMXFBl7P~>";
    let expect = [
        "this is a literal string",
        "this is a hex string",
        "this is a base85 string",
    ];

    let mut lexer = Lexer::from(input.chars());

    for e in expect {
        let Some(Ok(Object::String(string))) = lexer.next() else {
            return Err("expected string object".into());
        };
        assert_eq!(e, string);
    }

    Ok(())
}

#[test]
fn test_lex_name() -> Result<(), Box<dyn error::Error>> {
    let inputs = [
        "abc",
        "Offset",
        "$$",
        "23A",
        "13-456",
        "a.b",
        "$MyDict",
        "@pattern",
        "16#FFFF.LMAO",
    ];

    for input in inputs {
        let mut lexer = Lexer::from(input.chars());

        let Some(Ok(Object::Name(name))) = lexer.next() else {
            return Err("expected name object".into());
        };

        assert_eq!(input, name);
    }

    Ok(())
}

#[test]
fn test_lex_self_deliminating() -> Result<(), Box<dyn error::Error>> {
    let inputs = [
        ("mid[dle", "["),
        ("mid]dle", "]"),
        ("mid{dle", "{"),
        ("mid}dle", "}"),
        ("mid<<dle", "<<"),
        ("mid>>dle", ">>"),
        ("1[2", "["),
        ("1]2", "]"),
        ("1{2", "{"),
        ("1}2", "}"),
        ("1<<2", "<<"),
        ("1>>2", ">>"),
        ("1.2[3", "["),
        ("1.2]3", "]"),
        ("1.2{3", "{"),
        ("1.2}3", "}"),
        ("1.2<<3", "<<"),
        ("1.2>>3", ">>"),
        ("16#FF[FF", "["),
        ("16#FF]FF", "]"),
        ("16#FF{FF", "{"),
        ("16#FF}FF", "}"),
        ("16#FF<<FF", "<<"),
        ("16#FF>>FF", ">>"),
    ];

    for (input, expect) in inputs {
        let mut lexer = Lexer::from(input.chars());
        let _ = lexer.next();

        let Some(Ok(Object::Name(name))) = lexer.next() else {
            return Err("expected name object".into());
        };
        assert_eq!(expect, name);
    }

    Ok(())
}

#[test]
fn test_lex_all() -> Result<(), Box<dyn error::Error>> {
    let input = "
myStr (i have a string right here)
myOtherStr (and
another \
right \
here)
% this is a comment
myInt 1234567890
myNegativeInt -1234567890
myReal 3.1456
myNegativeReal -3.1456
        ";

    let expect = [
        Object::Name("myStr".to_string()),
        Object::String("i have a string right here".to_string()),
        Object::Name("myOtherStr".to_string()),
        Object::String("and\nanother right here".to_string()),
        Object::Name("myInt".to_string()),
        Object::Integer(1234567890),
        Object::Name("myNegativeInt".to_string()),
        Object::Integer(-1234567890),
        Object::Name("myReal".to_string()),
        Object::Real(3.1456),
        Object::Name("myNegativeReal".to_string()),
        Object::Real(-3.1456),
    ];

    let mut lexer = Lexer::from(input.chars());

    for obj in expect {
        let Some(Ok(received)) = lexer.next() else {
            return Err("expected object".into());
        };

        assert_eq!(obj, received);
    }

    Ok(())
}
