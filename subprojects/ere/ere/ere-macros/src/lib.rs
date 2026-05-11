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
/// Named structs use field names that match capture group names, with `#[group(0)]` marking the
/// whole-match field. Fields can be declared in any order — the macro resolves bindings by name:
///
/// ```
/// use ere_macros::regex;
///
/// #[derive(Debug, PartialEq, Eq)]
/// #[regex(r"^(?<year>[0-9]{4})-(?:0[1-9]|1[0-2])-(?<day>[0-9]{2})$")]
/// pub struct Date<'a> {
///     #[group(0)]
///     pub matched: &'a str,
///     // day declared before year — order is independent of regex group order
///     pub day: &'a str,
///     pub year: &'a str,
/// }
///
/// assert!(Date::test("2024-03-15"));
/// assert!(!Date::test("2024-13-15"));
///
/// assert_eq!(
///     Date::exec("2024-03-15"),
///     Some(Date { matched: "2024-03-15", day: "15", year: "2024" }),
/// );
/// assert_eq!(Date::exec("2024-13-15"), None);
/// ```
///
/// Optional named groups map to `Option<&'a str>` — `None` when the group did not participate
/// in the match, `Some(...)` when it did:
///
/// ```
/// use ere_macros::regex;
///
/// #[derive(Debug, PartialEq, Eq)]
/// #[regex(r"^(?<country>\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$")]
/// pub struct PhoneNumber<'a> {
///     #[group(0)]
///     pub matched: &'a str,
///     pub country: Option<&'a str>,
/// }
///
/// assert_eq!(
///     PhoneNumber::exec("555-555-5555"),
///     Some(PhoneNumber { matched: "555-555-5555", country: None }),
/// );
/// assert_eq!(
///     PhoneNumber::exec("+1 555-555-5555"),
///     Some(PhoneNumber { matched: "+1 555-555-5555", country: Some("+1 ") }),
/// );
/// ```
///
/// By default, all named capture groups must have a corresponding struct field, but unnamed
/// groups can be left unbound. The `bind` parameter controls this behavior:
///
/// - `bind = Strict` — all capture groups (named and unnamed) must have fields
/// - `bind = Named` — only named groups must have fields (the default)
/// - `bind = None` — no groups are required; only declared fields are populated.
///   Additionally, named struct fields without a matching capture group are assigned `None`
///   (the field must be `Option<T>`).
///
/// If a required group has no corresponding field, compilation fails with an error
/// identifying the unbound group.
///
/// `bind = Named` (the default) skips unnamed groups you don't care about:
///
/// ```
/// use ere_macros::regex;
///
/// #[derive(Debug, PartialEq, Eq)]
/// // bind = Named is the default, so it can be omitted:
/// #[regex(r"^(?<year>[0-9]{4})(-|/)(?<month>[0-9]{2})(-|/)(?<day>[0-9]{2})(T| )(?<hour>[0-9]{2})(:)(?<min>[0-9]{2})(:)(?<sec>[0-9]{2})$")]
/// pub struct Timestamp<'a> {
///     #[group(0)]
///     pub matched: &'a str,
///     pub year: &'a str,
///     pub month: &'a str,
///     pub day: &'a str,
///     pub hour: &'a str,
///     pub min: &'a str,
///     pub sec: &'a str,
///     // Separators like `(-|/)`, `(T| )`, and `(:)` are unnamed and silently ignored.
/// }
///
/// assert_eq!(
///     Timestamp::exec("2024-03-15T09:30:00"),
///     Some(Timestamp {
///         matched: "2024-03-15T09:30:00",
///         year: "2024", month: "03", day: "15",
///         hour: "09", min: "30", sec: "00",
///     }),
/// );
/// assert_eq!(
///     Timestamp::exec("2024/03/15 09:30:00"),
///     Some(Timestamp {
///         matched: "2024/03/15 09:30:00",
///         year: "2024", month: "03", day: "15",
///         hour: "09", min: "30", sec: "00",
///     }),
/// );
/// ```
///
/// `bind = Strict` requires every capture group to have a field — useful when you want
/// the compiler to catch any group you forgot to bind:
///
/// ```
/// use ere_macros::regex;
///
/// #[derive(Debug, PartialEq, Eq)]
/// #[regex(r"^(?<year>[0-9]{4})(-|/)(?<month>[0-9]{2})(-|/)(?<day>[0-9]{2})$", bind = Strict)]
/// pub struct StrictDate<'a> {
///     #[group(0)]
///     pub matched: &'a str,
///     pub year: &'a str,
///     #[group(2)]
///     pub sep1: &'a str,
///     pub month: &'a str,
///     #[group(4)]
///     pub sep2: &'a str,
///     pub day: &'a str,
/// }
///
/// assert_eq!(
///     StrictDate::exec("2024-03-15"),
///     Some(StrictDate {
///         matched: "2024-03-15", year: "2024", sep1: "-",
///         month: "03", sep2: "-", day: "15",
///     }),
/// );
/// ```
///
/// `bind = None` allows any group (including named ones) to go unbound — useful when you
/// only need a few fields from a complex regex:
///
/// ```
/// use ere_macros::regex;
///
/// #[derive(Debug, PartialEq, Eq)]
/// #[regex(r"^(?<year>[0-9]{4})(-|/)(?<month>[0-9]{2})(-|/)(?<day>[0-9]{2})$", bind = None)]
/// pub struct YearOnly<'a> {
///     #[group(0)]
///     pub matched: &'a str,
///     pub year: &'a str,
///     // month, day, and separator groups are all silently ignored.
/// }
///
/// assert_eq!(
///     YearOnly::exec("2024-03-15"),
///     Some(YearOnly { matched: "2024-03-15", year: "2024" }),
/// );
/// ```
///
/// The `engine` parameter selects which matching engine to use. By default, the best
/// engine is chosen automatically. Available engines:
///
/// - `engine = Auto` — automatically select the best engine (default)
/// - `engine = OnePassU8` — one-pass DFA over bytes; single linear scan, no backtracking.
///   Fastest engine, but only works for
///   [one-pass](https://swtch.com/~rsc/regexp/regexp3.html) regexes. Compile error if not applicable.
/// - `engine = DfaU8` — deterministic finite automaton over bytes. Fast, but can fail
///   for complex regexes that would produce too many states.
/// - `engine = FlatLockstepNfaU8` — NFA simulation over bytes via lockstep parallel
///   execution. Handles any ASCII regex.
/// - `engine = FlatLockstepNfa` — NFA simulation over Unicode chars. Most general engine,
///   handles non-ASCII patterns.
/// - `engine = FixedOffset` — extracts captures by fixed string offsets instead of
///   tracking them during matching. Only works when all capture groups are at deterministic
///   positions. Compile error if not applicable.
///
/// ```
/// use ere_macros::regex;
///
/// #[derive(Debug, PartialEq, Eq)]
/// #[regex(r"^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$", engine = OnePassU8)]
/// pub struct HexColor<'a>(&'a str, &'a str, &'a str, &'a str);
///
/// assert_eq!(
///     HexColor::exec("#ff0080"),
///     Some(HexColor("#ff0080", "ff", "00", "80")),
/// );
/// ```
///
/// The `engine` and `bind` parameters can be combined in any order:
///
/// ```
/// use ere_macros::regex;
///
/// #[derive(Debug, PartialEq, Eq)]
/// #[regex(r"^(?<r>[0-9a-f]{2})(?<g>[0-9a-f]{2})(?<b>[0-9a-f]{2})$", engine = DfaU8, bind = Named)]
/// pub struct Color<'a> {
///     #[group(0)]
///     pub matched: &'a str,
///     pub r: &'a str,
///     pub g: &'a str,
///     pub b: &'a str,
/// }
///
/// assert_eq!(
///     Color::exec("ff0080"),
///     Some(Color { matched: "ff0080", r: "ff", g: "00", b: "80" }),
/// );
/// ```
///
/// The `ascii_case_insensitive` flag makes the regex match ASCII letters regardless of case.
/// Note: POSIX character classes like `[:lower:]` and `[:upper:]` are not affected by this flag.
///
/// ```
/// use ere_macros::regex;
///
/// #[derive(Debug, PartialEq, Eq)]
/// #[regex(r"^hello world$", ascii_case_insensitive)]
/// pub struct Greeting<'a>(&'a str);
///
/// assert!(Greeting::test("hello world"));
/// assert!(Greeting::test("Hello World"));
/// assert!(Greeting::test("HELLO WORLD"));
/// assert!(Greeting::test("hElLo WoRlD"));
/// assert!(!Greeting::test("héllo world"));
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
