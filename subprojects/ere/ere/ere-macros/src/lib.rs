use ere_core;
use proc_macro::TokenStream;

extern crate proc_macro;

/// This is the primary entrypoint to the `ere` crate.
/// Checks and compiles a regular expression into a [`Regex<N>`](`ere_core::Regex<N>`).
///
/// This compilation happens during build using proc macros,
/// resulting in rust code equivalent to your regex.
/// This code can then by further optimized by rustc/llvm when compiled directly into the binary.
///
/// The generic `const N: usize` will be the number of capture groups present in the regular expression
/// (including capture group 0 which is the entire matched text).
/// You will need to properly specify this in the generics for the regex (default if unspecified is 1).
/// When using [`Regex<N>::exec`](`ere_core::Regex<N>::exec`), this is the length of the captures returned.
///
/// ```ignore
/// use ere::Regex;
/// use ere::compile_regex;
///
/// const MY_REGEX: Regex<2> = compile_regex!("a(b?)c");
/// ```
#[proc_macro]
pub fn compile_regex(stream: TokenStream) -> TokenStream {
    return ere_core::__compile_regex(stream);
}

/// Checks and compiles a regular expression into a into a [`ere_core::Regex<N>`] with the [`ere_core::dfa_u8`] engine.
/// Unless you specifically want this engine, you might want to use [`compile_regex!`] instead.
///
/// This compilation happens during build using proc macros,
/// resulting in rust code equivalent to your regex.
/// This code can then by further optimized by rustc/llvm when compiled directly into the binary.
///
/// The generic `const N: usize` will be the number of capture groups present in the regular expression
/// (including capture group 0 which is the entire matched text).
/// You will need to properly specify this in the generics for the regex (default if unspecified is 1).
/// When using [`Regex<N>::exec`](`ere_core::Regex<N>::exec`), this is the length of the captures returned.
///
/// ```ignore
/// use ere::Regex;
/// use ere::compile_regex_dfa_u8;
///
/// const MY_REGEX: Regex<2> = compile_regex_dfa_u8!("a(b?)c");
/// ```
#[proc_macro]
pub fn compile_regex_dfa_u8(stream: TokenStream) -> TokenStream {
    return ere_core::__compile_regex_engine_dfa_u8(stream);
}

/// Checks and compiles a regular expression into a into a [`ere_core::Regex<N>`] with the [`ere_core::flat_lockstep_nfa`] engine.
/// Unless you specifically want this engine, you might want to use [`compile_regex!`] instead.
///
/// This compilation happens during build using proc macros,
/// resulting in rust code equivalent to your regex.
/// This code can then by further optimized by rustc/llvm when compiled directly into the binary.
///
/// The generic `const N: usize` will be the number of capture groups present in the regular expression
/// (including capture group 0 which is the entire matched text).
/// You will need to properly specify this in the generics for the regex (default if unspecified is 1).
/// When using [`Regex<N>::exec`](`ere_core::Regex<N>::exec`), this is the length of the captures returned.
///
/// ```ignore
/// use ere::Regex;
/// use ere::compile_regex_flat_lockstep_nfa;
///
/// const MY_REGEX: Regex<2> = compile_regex_flat_lockstep_nfa!("a(b?)c");
/// ```
#[proc_macro]
pub fn compile_regex_flat_lockstep_nfa(stream: TokenStream) -> TokenStream {
    return ere_core::__compile_regex_engine_flat_lockstep_nfa(stream);
}

/// Checks and compiles a regular expression into a into a [`ere_core::Regex<N>`] with the [`ere_core::flat_lockstep_nfa_u8`] engine.
/// Unless you specifically want this engine, you might want to use [`compile_regex!`] instead.
///
/// This compilation happens during build using proc macros,
/// resulting in rust code equivalent to your regex.
/// This code can then by further optimized by rustc/llvm when compiled directly into the binary.
///
/// The generic `const N: usize` will be the number of capture groups present in the regular expression
/// (including capture group 0 which is the entire matched text).
/// You will need to properly specify this in the generics for the regex (default if unspecified is 1).
/// When using [`Regex<N>::exec`](`ere_core::Regex<N>::exec`), this is the length of the captures returned.
///
/// ```ignore
/// use ere::Regex;
/// use ere::compile_regex_flat_lockstep_nfa_u8;
///
/// const MY_REGEX: Regex<2> = compile_regex_flat_lockstep_nfa_u8!("a(b?)c");
/// ```
#[proc_macro]
pub fn compile_regex_flat_lockstep_nfa_u8(stream: TokenStream) -> TokenStream {
    return ere_core::__compile_regex_engine_flat_lockstep_nfa_u8(stream);
}

/// Checks and compiles a regular expression into a [`ere_core::Regex<N>`] with the [`ere_core::one_pass_u8`] engine.
/// Unless you specifically want this engine, you might want to use [`compile_regex!`] instead.
///
/// This compilation happens during build using proc macros,
/// resulting in rust code equivalent to your regex.
/// This code can then by further optimized by rustc/llvm when compiled directly into the binary.
///
/// The generic `const N: usize` will be the number of capture groups present in the regular expression
/// (including capture group 0 which is the entire matched text).
/// You will need to properly specify this in the generics for the regex (default if unspecified is 1).
/// When using [`Regex<N>::exec`](`ere_core::Regex<N>::exec`), this is the length of the captures returned.
///
/// ```ignore
/// use ere::Regex;
/// use ere::compile_regex_u8onepass;
///
/// const MY_REGEX: Regex<2> = compile_regex_u8onepass!("^a(b?)c$");
/// ```
///
/// ---
///
/// Note that this engine does not support all valid regular expressions,
/// and will raise a compile error if necessary.
/// For example, unanchored regexes are generally not one-pass.
///
#[proc_macro]
pub fn compile_regex_u8onepass(stream: TokenStream) -> TokenStream {
    return ere_core::__compile_regex_engine_one_pass_u8(stream);
}

/// Checks and compiles a regular expression into a [`ere_core::Regex<N>`] with the [`ere_core::fixed_offset`] engine.
/// Unless you specifically want this engine, you might want to use [`compile_regex!`] instead.
///
/// This compilation happens during build using proc macros,
/// resulting in rust code equivalent to your regex.
/// This code can then by further optimized by rustc/llvm when compiled directly into the binary.
///
/// The generic `const N: usize` will be the number of capture groups present in the regular expression
/// (including capture group 0 which is the entire matched text).
/// You will need to properly specify this in the generics for the regex (default if unspecified is 1).
/// When using [`Regex<N>::exec`](`ere_core::Regex<N>::exec`), this is the length of the captures returned.
///
/// ```ignore
/// use ere::Regex;
/// use ere::compile_regex_fixed_offset;
///
/// const MY_REGEX: Regex<2> = compile_regex_fixed_offset!("^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$");
/// ```
///
/// ---
///
/// Note that this engine does not support all valid regular expressions,
/// and will raise a compile error if necessary..
///
#[proc_macro]
pub fn compile_regex_fixed_offset(stream: TokenStream) -> TokenStream {
    return ere_core::__compile_regex_engine_fixed_offset(stream);
}

/// EXPERIMENTAL: this attribute provides an alternate syntax with finer control for creating regexes.
///
/// Compared with [`compile_regex!`], this allows the type system to know which capture groups
/// should be optional and which should not.
///
/// For example:
///
/// ```
/// use ere_macros::regex;
///
/// #[derive(Debug, PartialEq, Eq)]
/// #[regex(r"^#?([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})?$")]
/// pub struct HexColor<'a>(
///     pub &'a str,
///     pub &'a str,
///     pub &'a str,
///     pub &'a str,
///     pub Option<&'a str>,
/// );
///
/// assert!(HexColor::test("#1F1F1F"));
/// assert!(HexColor::test("#1F1F1F80"));
/// assert!(HexColor::test("20202020"));
///
/// assert_eq!(
///     HexColor::exec("#112233"),
///     Some(HexColor(
///         "#112233",
///         "11",
///         "22",
///         "33",
///         None,
///     )),
/// );
/// assert_eq!(
///     HexColor::exec("#11223344"),
///     Some(HexColor(
///         "#11223344",
///         "11",
///         "22",
///         "33",
///         Some("44"),
///     )),
/// );
/// ```
///
/// ---
///
/// Note that it is required to specify the fields with the proper type
/// (i.e. `&'a str` or `Option<&'a str>` depending on the capture group)
/// and the lifetime should be the first generic argument on the struct.
///
/// The field for the 0th capture group should never be an `Option` since if there is a match,
/// it will always contain the entire match (and otherwise `exec` returns `None`).
#[cfg(feature = "unstable-attr-regex")]
#[proc_macro_attribute]
pub fn regex(attr: TokenStream, input: TokenStream) -> TokenStream {
    return ere_core::__compile_regex_attr(attr, input);
}
