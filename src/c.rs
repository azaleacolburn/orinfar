use crate::{highlight::ORANGE, utility::is_symbol};
use crossterm::style::Color;

const KEYWORDS: &[&str] = &[
    "alignas",
    "alignof",
    "auto",
    "bool",
    "break",
    "case",
    "char",
    "const",
    "constexpr",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "false",
    "float",
    "for",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "nullptr",
    "register",
    "restrict",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "static_assert",
    "struct",
    "switch",
    "thread_local",
    "true",
    "typedef",
    "typeof ",
    "typeof_unqual",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    // "_Alignof",
    // "_Atomic",
    // "_BitInt",
    // "_Bool",
    // "_Complex",
    // "_Decimal128",
    // "_Decimal32",
    // "_Decimal64",
    // "_Generic",
    // "_Imaginary",
    // "_Noreturn",
    // "_Static_assert",
    // "_Thread_local",
];

fn is_c_keyword(str: &str) -> bool {
    KEYWORDS.contains(&str)
}

const OPERATORS: &[&str] = &[
    "+", "-", "/", "*", "++", "--", "+=", "-=", "<", ">", "||", "&&", "<=", ">=", "||=", "&&=",
    "!", "!=", "==", "=", "&", "|", "^", "~", "<<", ">>", "|=", "&=", "~=", "^=", "->",
];

fn is_operator(str: &str) -> bool {
    OPERATORS.contains(&str)
}

pub fn c_node_to_color(node_type: &str, parent_type: &str) -> Option<Color> {
    let color = match node_type {
        "#include" | "#define" | "#ifdef" | "#ifndef" | "#endif" => Color::DarkRed,

        "string_content" | "character" | "\"" | "\'" | "system_lib_string" => Color::Green,
        "identifier"
            if parent_type == "function_declarator" || parent_type == "call_expression" =>
        {
            Color::Green
        }

        "identifier" | "preproc_arg" => Color::Grey,
        "field_identifier" => Color::Blue,

        "primitive_type" => Color::Yellow,
        "type_identifier" => Color::DarkMagenta,

        "comment" | ";" | "." | "," => Color::DarkGrey,
        // Common macros
        // In the future a way to automatically determine
        // which strings are macros would be really cool
        "number_literal" | "true" | "false" | "NULL" => Color::Magenta,
        str if is_operator(str) => ORANGE,
        str if str.chars().all(is_symbol) => Color::Grey,
        str if is_c_keyword(str) => Color::Red,
        _ => return None,
    };

    Some(color)
}
