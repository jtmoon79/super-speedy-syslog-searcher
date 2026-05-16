// subprojects/ere_automator_procmacro/src/lib.rs

/// Procedural macros for creating `ere` regular expressions at build time.

use proc_macro::{
    TokenStream,
};
use proc_macro2::{
    Literal as ProcLiteral,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote,
    ToTokens,
};
use std::collections::{
    HashMap,
    HashSet,
};
use std::io::Write;
use std::sync::{
    Mutex,
    OnceLock,
};
use syn::parse::{
    Parse,
    Parser,
    ParseStream,
};
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use syn::spanned::Spanned;
use syn::{
    Expr,
    ExprCall,
    ExprConst,
    ExprGroup,
    ExprLit,
    ExprMacro,
    ExprMethodCall,
    ExprParen,
    ExprPath,
    File,
    Item,
    ItemConst,
    Ident,
    Lit,
    LitStr,
    Result,
    Token,
};

/// Find all Rust files, i.e. `*.rs`, recursively in a directory.
fn collect_rust_files_recursive(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files_recursive(path.as_path(), out);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Scrape `const` variable declarations from a Rust file.
fn collect_const_exprs_from_file(parsed: &File, out: &mut HashMap<String, Expr>) {
    for item in &parsed.items {
        if let Item::Const(ItemConst { ident, expr, .. }) = item {
            out.insert(ident.to_string(), (**expr).clone());
        }
    }
}

/// Scrape `const` variable declarations from the caller crate.
fn collect_const_exprs_from_caller_crate() -> HashMap<String, Expr> {
    let mut out: HashMap<String, Expr> = HashMap::new();
    let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
        return out;
    };
    let src_dir = std::path::Path::new(&manifest_dir).join("src");
    if !src_dir.is_dir() {
        return out;
    }

    let mut rust_files: Vec<std::path::PathBuf> = Vec::new();
    collect_rust_files_recursive(src_dir.as_path(), &mut rust_files);
    for file_path in rust_files {
        let Ok(source_text) = std::fs::read_to_string(&file_path) else {
            continue;
        };
        let Ok(parsed) = syn::parse_file(&source_text) else {
            continue;
        };
        collect_const_exprs_from_file(&parsed, &mut out);
    }

    out
}

/// Is the rust tokens a concat-like macro path (`concat!` or `concatcp!`)?
fn is_concat_like_path(path: &syn::Path) -> bool {
    let Some(last) = path.segments.last() else {
        return false;
    };
    let last_ident = last.ident.to_string();
    if last_ident != "concat" && last_ident != "concatcp" {
        return false;
    }

    // Accept unqualified forms like `concat!(...)` or `concatcp!(...)`.
    if path.segments.len() == 1 {
        return true;
    }

    // Accept qualified forms such as `::const_str::concat!(...)` and
    // `::const_format::concatcp!(...)`.
    let penultimate = path
        .segments
        .iter()
        .nth(path.segments.len().saturating_sub(2))
        .map(|s| s.ident.to_string());
    matches!(penultimate.as_deref(), Some("const_str") | Some("const_format"))
}

/// Is the rust tokens a `concat_bytes!` macro path (`concat_bytes!` from `const_str`)?
fn is_concat_bytes_path(path: &syn::Path) -> bool {
    let Some(last) = path.segments.last() else {
        return false;
    };
    if last.ident != "concat_bytes" {
        return false;
    }

    // Accept unqualified `concat_bytes!(...)`.
    if path.segments.len() == 1 {
        return true;
    }

    // Accept qualified `::const_str::concat_bytes!(...)`.
    let penultimate = path
        .segments
        .iter()
        .nth(path.segments.len().saturating_sub(2))
        .map(|s| s.ident.to_string());
    matches!(penultimate.as_deref(), Some("const_str"))
}

/// Returns `true` if `haystack` contains `needle` as a contiguous sub-slice.
fn bytes_contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Evaluates an expression to extract the string value, if possible.
fn eval_expr_to_string(
    expr: &Expr,
    consts: &HashMap<String, Expr>,
    visiting: &mut HashSet<String>,
) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) => Some(lit.value()),
        Expr::Group(ExprGroup { expr, .. }) => eval_expr_to_string(expr, consts, visiting),
        Expr::Paren(ExprParen { expr, .. }) => eval_expr_to_string(expr, consts, visiting),
        Expr::Const(ExprConst { block, .. }) => {
            let last_expr = block.stmts.iter().rev().find_map(|stmt| {
                match stmt {
                    syn::Stmt::Expr(expr, _) => Some(expr),
                    _ => None,
                }
            })?;
            eval_expr_to_string(last_expr, consts, visiting)
        }
        Expr::Path(ExprPath { path, .. }) => {
            let ident = path.segments.last()?.ident.to_string();
            if !visiting.insert(ident.clone()) {
                return None;
            }
            let Some(next_expr) = consts.get(&ident) else {
                visiting.remove(&ident);
                return None;
            };
            let resolved = eval_expr_to_string(next_expr, consts, visiting);
            visiting.remove(&ident);
            resolved
        }
        Expr::Call(ExprCall { func, args, .. }) => {
            let Expr::Path(ExprPath { path, .. }) = &**func else {
                return None;
            };
            if !is_concat_like_path(path) {
                return None;
            }

            let mut out = String::new();
            for arg in args {
                out.push_str(eval_expr_to_string(arg, consts, visiting)?.as_str());
            }
            Some(out)
        }
        Expr::Macro(ExprMacro { mac, .. }) => {
            if !is_concat_like_path(&mac.path) {
                return None;
            }

            let args = syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated
                .parse2(mac.tokens.clone())
                .ok()?;

            let mut out = String::new();
            for arg in args {
                out.push_str(eval_expr_to_string(&arg, consts, visiting)?.as_str());
            }
            Some(out)
        }
        _ => None,
    }
}

/// Evaluates an expression to extract a byte string value, if possible.
///
/// Supported forms:
/// - byte string literals, e.g. `b"foo"`
/// - parenthesized/grouped expressions
/// - const blocks with final expression
/// - `const` path references resolved from the caller crate (e.g. `pub const FOO: &[u8] = b"foo"`)
/// - `concat_bytes!(...)` and `::const_str::concat_bytes!(...)` macro/call invocations
fn eval_expr_to_bytes(
    expr: &Expr,
    consts: &HashMap<String, Expr>,
    visiting: &mut HashSet<String>,
) -> Option<Vec<u8>> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::ByteStr(lit), .. }) => Some(lit.value()),
        Expr::Group(ExprGroup { expr, .. }) => eval_expr_to_bytes(expr, consts, visiting),
        Expr::Paren(ExprParen { expr, .. }) => eval_expr_to_bytes(expr, consts, visiting),
        Expr::Const(ExprConst { block, .. }) => {
            let last_expr = block.stmts.iter().rev().find_map(|stmt| {
                match stmt {
                    syn::Stmt::Expr(expr, _) => Some(expr),
                    _ => None,
                }
            })?;
            eval_expr_to_bytes(last_expr, consts, visiting)
        }
        Expr::Path(ExprPath { path, .. }) => {
            let ident = path.segments.last()?.ident.to_string();
            if !visiting.insert(ident.clone()) {
                return None;
            }
            let Some(next_expr) = consts.get(&ident) else {
                visiting.remove(&ident);
                return None;
            };
            let resolved = eval_expr_to_bytes(next_expr, consts, visiting);
            visiting.remove(&ident);
            resolved
        }
        Expr::Call(ExprCall { func, args, .. }) => {
            let Expr::Path(ExprPath { path, .. }) = &**func else {
                return None;
            };
            if !is_concat_bytes_path(path) {
                return None;
            }
            let mut out: Vec<u8> = Vec::new();
            for arg in args {
                out.extend_from_slice(&eval_expr_to_bytes(arg, consts, visiting)?);
            }
            Some(out)
        }
        Expr::Macro(ExprMacro { mac, .. }) => {
            if !is_concat_bytes_path(&mac.path) {
                return None;
            }
            let args = syn::punctuated::Punctuated::<Expr, Token![,]>::parse_terminated
                .parse2(mac.tokens.clone())
                .ok()?;
            let mut out: Vec<u8> = Vec::new();
            for arg in args {
                out.extend_from_slice(&eval_expr_to_bytes(&arg, consts, visiting)?);
            }
            Some(out)
        }
        Expr::MethodCall(ExprMethodCall { receiver, method, args, .. }) => {
            if method != "as_bytes" || !args.is_empty() {
                return None;
            }
            let value = eval_expr_to_string(receiver, consts, visiting)?;
            Some(value.into_bytes())
        }
        _ => None,
    }
}

/// Evaluates an expression to extract the line number as a usize, if possible.
/// e.g. `line!()`
fn eval_expr_to_line_num(expr: &Expr) -> Option<usize> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) => lit.base10_parse::<usize>().ok(),
        Expr::Group(ExprGroup { expr, .. }) => eval_expr_to_line_num(expr),
        Expr::Paren(ExprParen { expr, .. }) => eval_expr_to_line_num(expr),
        Expr::Const(ExprConst { block, .. }) => {
            let last_expr = block.stmts.iter().rev().find_map(|stmt| {
                match stmt {
                    syn::Stmt::Expr(expr, _) => Some(expr),
                    _ => None,
                }
            })?;
            eval_expr_to_line_num(last_expr)
        }
        Expr::Macro(ExprMacro { mac, .. }) => {
            if mac.path.is_ident("line") {
                Some(expr.span().start().line)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Evaluates an expression to extract the file name as a string, if possible.
/// e.g. `file!()`
fn eval_expr_to_file_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) => Some(lit.value()),
        Expr::Group(ExprGroup { expr, .. }) => eval_expr_to_file_name(expr),
        Expr::Paren(ExprParen { expr, .. }) => eval_expr_to_file_name(expr),
        Expr::Const(ExprConst { block, .. }) => {
            let last_expr = block.stmts.iter().rev().find_map(|stmt| {
                match stmt {
                    syn::Stmt::Expr(expr, _) => Some(expr),
                    _ => None,
                }
            })?;
            eval_expr_to_file_name(last_expr)
        }
        Expr::Macro(ExprMacro { mac, .. }) => {
            if mac.path.is_ident("file") {
                Some(expr.span().unwrap().file())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// True if the final segment of `path` is `ident_name`.
fn path_last_ident_is(path: &syn::Path, ident_name: &str) -> bool {
    path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == ident_name)
}

/// Evaluate an expression to an integer value if possible.
///
/// Supported forms:
/// - integer literals, e.g. `1`
/// - parenthesized/grouped expressions
/// - const blocks with final expression
/// - `counter!()` and `counter_last!()` macro invocations
fn eval_expr_to_usize(expr: &Expr) -> Option<usize> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) => lit.base10_parse::<usize>().ok(),
        Expr::Group(ExprGroup { expr, .. }) => eval_expr_to_usize(expr),
        Expr::Paren(ExprParen { expr, .. }) => eval_expr_to_usize(expr),
        Expr::Const(ExprConst { block, .. }) => {
            let last_expr = block.stmts.iter().rev().find_map(|stmt| {
                match stmt {
                    syn::Stmt::Expr(expr, _) => Some(expr),
                    _ => None,
                }
            })?;
            eval_expr_to_usize(last_expr)
        }
        Expr::Macro(ExprMacro { mac, .. }) => {
            if path_last_ident_is(&mac.path, "counter") || path_last_ident_is(&mac.path, "counter_last") {
                let key: String = if mac.tokens.is_empty() {
                    COUNTER_DEFAULT_KEY.to_string()
                } else {
                    let key_expr: Expr = syn::parse2(mac.tokens.clone()).ok()?;
                    let consts: HashMap<String, Expr> = collect_const_exprs_from_caller_crate();
                    let mut visiting: HashSet<String> = HashSet::new();
                    eval_expr_to_string(&key_expr, &consts, &mut visiting)?
                };

                // XXX: duplicates code of `counter!()`
                if path_last_ident_is(&mac.path, "counter") {
                    let mut map = counter_map().lock().ok()?;
                    let next = map.entry(key.clone()).or_insert(0);
                    *next += 1;
                }
                let map = counter_map().lock().ok()?;
                let value = match map.get(&key) {
                    Some(v) => *v,
                    None => 1,
                };
                return Some(value);
            }
            None
        }
        _ => None,
    }
}

/// A struct representing the fields required to create a new ERE regex.
/// Used in `new_ere_regex` procedural macro.
struct EreRegexFieldsStruct {
    /// Positive integer literal identifier for the regex, used for debugging and tracing purposes
    regex_id: Expr,
    /// Bytes representing the regular expression pattern
    pattern_literal: Expr,
    /// ere engine; e.g. `DfaU8`, etc.
    engine: Expr,
    /// File of the declaration site, e.g. `file!()`
    file_name: Expr,
    /// Line number of the declaration, e.g. `line!()`
    line_num: Expr,
    /// Allow `defn!` debug prints in the generated function.
    /// This does not affect whether the `defn!` macro itself is compiled based on release/debug/test profile.
    allow_debug_print: bool,
}

impl Parse for EreRegexFieldsStruct {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let regex_id: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let pattern_literal: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let engine: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let file_name: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let line_num: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let allow_debug_print: bool = if input.is_empty() {
            false
        } else {
            let debug_print_expr: Expr = input.parse()?;
            match debug_print_expr {
                Expr::Lit(ExprLit { lit: Lit::Bool(lit), .. }) => lit.value,
                _ => {
                    return Err(syn::Error::new_spanned(
                        debug_print_expr,
                        "allow_debug_print must be a boolean literal (e.g. `true` or `false`)",
                    ));
                }
            }
        };

        Ok(Self {
            regex_id,
            pattern_literal,
            engine,
            file_name,
            line_num,
            allow_debug_print,
        })
    }
}

// XXX: these must match values in `datetime.rs`
const CGI_YEAR: usize = 0;
const CGI_MONTH: usize = 1;
const CGI_DAY: usize = 2;
const CGI_DAY_IGNORE: usize = 3;
const CGI_HOUR: usize = 4;
const CGI_MINUTE: usize = 5;
const CGI_SECOND: usize = 6;
const CGI_FRACTIONAL: usize = 7;
const CGI_TZ: usize = 8;
const CGI_UPTIME: usize = 9;
const CGI_EPOCH: usize = 10;
const GROUP_NAMES_LEN: usize = 11;

// XXX: these must match values in `datetime.rs`
const GROUP_NAMES_CGP_PARTIAL: &[&str; GROUP_NAMES_LEN] = &[
    "(?<year>",
    "(?<month>",
    "(?<day>",
    "(?<day_ignore>",
    "(?<hour>",
    "(?<minute>",
    "(?<second>",
    "(?<fractional>",
    "(?<tz>",
    "(?<uptime>",
    "(?<epoch>",
];

static COUNTER_REGEX: AtomicUsize = AtomicUsize::new(1);

fn flush_stderr() {
    _ = std::io::stderr().lock().flush();
}

const COUNTER_DEFAULT_KEY: &str = "__default__";
static COUNTER_KEYED: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();

fn counter_map() -> &'static Mutex<HashMap<String, usize>> {
    COUNTER_KEYED.get_or_init(|| Mutex::new(HashMap::new()))
}

fn counter_key_from_input(input: TokenStream) -> Result<String> {
    if input.is_empty() {
        return Ok(COUNTER_DEFAULT_KEY.to_string());
    }

    let expr: Expr = syn::parse(input)?;
    let consts: HashMap<String, Expr> = collect_const_exprs_from_caller_crate();
    let mut visiting: HashSet<String> = HashSet::new();
    match eval_expr_to_string(&expr, &consts, &mut visiting) {
        Some(key) => Ok(key),
        None => Err(syn::Error::new_spanned(
            &expr,
            "expected a string literal or a const &str expression as counter key",
        )),
    }
}

/// Return an incrementing integer literal each time this macro is expanded.
///
/// Example:
/// ```rust
/// let a: usize = counter!(); // 1
/// let b: usize = counter!(); // 2
/// ```
#[proc_macro]
pub fn counter(input: TokenStream) -> TokenStream {
    let key: String = match counter_key_from_input(input) {
        Ok(value) => value,
        Err(err) => return err.to_compile_error().into(),
    };

    let next: usize = {
        let mut map = counter_map().lock().expect("counter map mutex poisoned");
        let entry = map.entry(key).or_insert(0);
        *entry += 1;
        *entry
    };
    let lit = ProcLiteral::usize_unsuffixed(next);

    quote!(#lit).into()
}

/// Returns the last value of `counter!` without incrementing it.
///
/// Example:
/// ```rust
/// let a: usize = counter!(); // 1
/// let b: usize = counter_last!(); // 1
/// ```
#[proc_macro]
pub fn counter_last(input: TokenStream) -> TokenStream {
    let key: String = match counter_key_from_input(input) {
        Ok(value) => value,
        Err(err) => return err.to_compile_error().into(),
    };

    let next: usize = {
        let map = counter_map().lock().expect("counter map mutex poisoned");
        match map.get(&key) {
            Some(value) => *value,
            None => 0,
        }
    };
    let lit = ProcLiteral::usize_unsuffixed(next);

    quote!(#lit).into()
}

/// A procedural macro to create a new
/// [`ere` crate compile-time regular expression].
/// This approach hides the unique struct created by the `#regex` derive
/// macro generated by `ere`. It returns a wrapper function and 
/// the expanded pattern string.
/// This hides the caller from having to handle the unique generated struct.
/// Returns `<match function>`
/// e.g. `(RegexFn, "expanded_pattern_string")`.
///
/// [`ere` crate compile-time regular expression]: https://docs.rs/ere
#[proc_macro]
pub fn new_ere_regex(input: TokenStream) -> TokenStream {
    let EreRegexFieldsStruct {
        regex_id,
        pattern_literal,
        engine,
        file_name,
        line_num,
        allow_debug_print,
    } = syn::parse_macro_input!(input as EreRegexFieldsStruct);
    let count: usize = COUNTER_REGEX.fetch_add(1, Ordering::SeqCst);
    let resolved_regex_id: usize = match eval_expr_to_usize(&regex_id) {
        Some(value) if value < 1 => {
            return syn::Error::new_spanned(
                &regex_id,
                format!("regex_id must be greater than 0; given {value}"),
            )
            .to_compile_error()
            .into();
        }
        Some(value) => value,
        None => {
            return syn::Error::new_spanned(
                &regex_id,
                "regex_id must resolve to an integer literal (e.g. `1`, `counter!()`, or `counter_last!()`)",
            )
            .to_compile_error()
            .into();
        }
    };
    let resolved_file_name = eval_expr_to_file_name(&file_name)
        .unwrap_or_else(|| file_name.to_token_stream().to_string());
    let resolved_line_num = eval_expr_to_line_num(&line_num)
        .unwrap_or_else(|| line_num.span().start().line);
    let resolved_engine = engine.to_token_stream().to_string();
    let print_progress: bool = match std::env::var("S4_BUILD_REGEX_PRINT") {
      Ok(val) => {
        if val == "1" || val == "yes" || val == "ON" {
            true
        } else {
            false
        }
      }
      Err(_) => false,
    };
    let allow_debug_print: bool = allow_debug_print;

    if print_progress {
        eprint!(
            "new_ere_regex: regex #{} (count {}), file {}:{}, engine {}, ",
            resolved_regex_id, count, resolved_file_name, resolved_line_num, resolved_engine,
        );
        flush_stderr();
    }

    let struct_name_id: Ident = format_ident!("ere_match_struct_{}", resolved_regex_id);
    let function_name_id: Ident = format_ident!("ere_match_fn_{}", resolved_regex_id);

    // expand `pattern_literal` into a byte string literal.
    // It may be a byte string literal (`b"..."`), a `const &[u8]`, or
    // `::const_str::concat_bytes!` resolving to byte string consts.
    let source_consts: HashMap<String, Expr> = collect_const_exprs_from_caller_crate();
    let mut visiting: HashSet<String> = HashSet::new();
    let pattern_bytes: Vec<u8> = match eval_expr_to_bytes(&pattern_literal, &source_consts, &mut visiting) {
        Some(value) => value,
        None => {
            return syn::Error::new_spanned(
                &pattern_literal,
                "Expected a byte string literal (`b\"...\"`), a `const &[u8]`, or \
                 `::const_str::concat_bytes!` resolving to byte string consts for `pattern_literal`",
            )
            .to_compile_error()
            .into();
        }
    };
    let mut pattern_lit_bytes_token = ProcLiteral::byte_string(&pattern_bytes);
    pattern_lit_bytes_token.set_span(pattern_literal.span());
    let mut named_fields: Vec<&str> = Vec::new();

    let mut fields_count: usize = 0;
    let has_year: bool = bytes_contains(&pattern_bytes, b"(?<year>");
    if has_year { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_YEAR]); }
    let has_month: bool = bytes_contains(&pattern_bytes, b"(?<month>");
    if has_month { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_MONTH]); }
    let has_day: bool = bytes_contains(&pattern_bytes, b"(?<day>");
    if has_day { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_DAY]); }
    let has_day_ignore: bool = bytes_contains(&pattern_bytes, b"(?<day_ignore>");
    if has_day_ignore { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_DAY_IGNORE]); }
    let has_hour: bool = bytes_contains(&pattern_bytes, b"(?<hour>");
    if has_hour { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_HOUR]); }
    let has_minute: bool = bytes_contains(&pattern_bytes, b"(?<minute>");
    if has_minute { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_MINUTE]); }
    let has_second: bool = bytes_contains(&pattern_bytes, b"(?<second>");
    if has_second { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_SECOND]); }
    let has_fractional: bool = bytes_contains(&pattern_bytes, b"(?<fractional>");
    if has_fractional { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_FRACTIONAL]); }
    let has_tz: bool = bytes_contains(&pattern_bytes, b"(?<tz>");
    if has_tz { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_TZ]); }
    let has_uptime: bool = bytes_contains(&pattern_bytes, b"(?<uptime>");
    if has_uptime { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_UPTIME]); }
    let has_epoch: bool = bytes_contains(&pattern_bytes, b"(?<epoch>");
    if has_epoch { fields_count += 1; named_fields.push(GROUP_NAMES_CGP_PARTIAL[CGI_EPOCH]); }

    let fields_count_token: TokenStream2 = quote! { #fields_count };
    let named_fields_joined: String = named_fields.join("), ") + ")";

    let named_fields_expanded_string = LitStr::new(
        named_fields_joined.as_str(),
        pattern_literal.span(),
    );

    if print_progress {
        eprint!("named fields {}, pattern_bytes «{}»", fields_count, String::from_utf8_lossy(&pattern_bytes));
        flush_stderr();
    }

    // Build per-field instantiations.
    let field_year: Option<TokenStream2>       = has_year.then(||       quote! { year: &'a str, });
    let field_month: Option<TokenStream2>      = has_month.then(||      quote! { month: &'a str, });
    let field_day: Option<TokenStream2>        = has_day.then(||        quote! { day: &'a str, });
    let field_day_ignore: Option<TokenStream2> = has_day_ignore.then(|| quote! { day_ignore: &'a str, });
    let field_hour: Option<TokenStream2>       = has_hour.then(||       quote! { hour: &'a str, });
    let field_minute: Option<TokenStream2>     = has_minute.then(||     quote! { minute: &'a str, });
    let field_second: Option<TokenStream2>     = has_second.then(||     quote! { second: &'a str, });
    let field_fractional: Option<TokenStream2> = has_fractional.then(|| quote! { fractional: &'a str, });
    let field_tz: Option<TokenStream2>         = has_tz.then(||         quote! { tz: &'a str, });
    let field_uptime: Option<TokenStream2>     = has_uptime.then(||     quote! { uptime: &'a str, });
    let field_epoch: Option<TokenStream2>      = has_epoch.then(||      quote! { epoch: &'a str, });
    // Build statements for match function
    let push_year: Option<TokenStream2>        = has_year.then(||       quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.year, haystack), CGI_YEAR)); });
    let push_month: Option<TokenStream2>       = has_month.then(||      quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.month, haystack), CGI_MONTH)); });
    let push_day: Option<TokenStream2>         = has_day.then(||        quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.day, haystack), CGI_DAY)); });
    let push_day_ignore: Option<TokenStream2>  = has_day_ignore.then(|| quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.day_ignore, haystack), CGI_DAY_IGNORE)); });
    let push_hour: Option<TokenStream2>        = has_hour.then(||       quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.hour, haystack), CGI_HOUR)); });
    let push_minute: Option<TokenStream2>      = has_minute.then(||     quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.minute, haystack), CGI_MINUTE)); });
    let push_second: Option<TokenStream2>      = has_second.then(||     quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.second, haystack), CGI_SECOND)); });
    let push_fractional: Option<TokenStream2>  = has_fractional.then(|| quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.fractional, haystack), CGI_FRACTIONAL)); });
    let push_tz: Option<TokenStream2>          = has_tz.then(||         quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.tz, haystack), CGI_TZ)); });
    let push_uptime: Option<TokenStream2>      = has_uptime.then(||     quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.uptime, haystack), CGI_UPTIME)); });
    let push_epoch: Option<TokenStream2>       = has_epoch.then(||      quote! { matches.push(MatchType::new(SpanS4_from_ptrs!(result.epoch, haystack), CGI_EPOCH)); });

    let expanded = quote! {
        {
            {
                fn #function_name_id(haystack: &[u8]) -> MatchesTypeOpt {
                    if #allow_debug_print {
                        defn!("looking for {} named captures: {}", #fields_count_token, #named_fields_expanded_string);
                        defo!("haystack bytes {}", haystack.len());
                    }
                    // here is the declaration of the `ere::regex!` which creates
                    // the ere regular expression engine
                    #[regex(
                        #pattern_lit_bytes_token,
                        engine = #engine,
                        bind = Named
                    )]
                    struct #struct_name_id<'a> {
                        #[group(0)]
                        matched: &'a str,
                        #field_year
                        #field_month
                        #field_day
                        #field_day_ignore
                        #field_hour
                        #field_minute
                        #field_second
                        #field_fractional
                        #field_tz
                        #field_uptime
                        #field_epoch
                    }
                    if let Some(result) = #struct_name_id::exec_bytes(haystack) {
                        let mut matches: MatchesType = MatchesType::with_capacity(#fields_count_token);
                        #push_year
                        #push_month
                        #push_day
                        #push_day_ignore
                        #push_hour
                        #push_minute
                        #push_second
                        #push_fractional
                        #push_tz
                        #push_uptime
                        #push_epoch
                        if #allow_debug_print {
                            defx!("return {} matches", matches.len());
                        }
                        return Some(matches);
                    };

                    if #allow_debug_print {
                        defx!("return None");
                    }
                    None
                }

                #function_name_id as crate::RegexFn
            }
        }
    };
    if print_progress {
        eprintln!();
    }

    expanded.into()
}
