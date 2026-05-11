//! This crate provides the core functionality to the `ere` crate.

use proc_macro::TokenStream;
use quote::quote;
extern crate proc_macro;

pub mod config;
mod engines;
mod epsilon_propogation;
pub mod parse_tree;
pub mod simplified_tree;
pub mod visualization;
pub mod working_nfa;
pub mod working_u8_dfa;
pub mod working_u8_nfa;

pub use engines::*;

/// A regular expression (specifically, a [POSIX ERE](https://en.wikibooks.org/wiki/Regular_Expressions/POSIX-Extended_Regular_Expressions)).
///
/// Internally, this may contain one of several engines depending on the expression.
///
/// The const generic `N` represents the number of capture groups (including capture group 0 which is the entire expression).
/// It defaults to `1` (for just capture group 0), but you will need to specify it in the type for expressions with more capture groups.
pub struct Regex<const N: usize = 1> {
    test_fn: Option<fn(&str) -> bool>,
    exec_fn: Option<for<'a> fn(&'a str) -> Option<[Option<&'a str>; N]>>,
    test_bytes_fn: Option<fn(&[u8]) -> bool>,
    exec_bytes_fn: Option<for<'a> fn(&'a [u8]) -> Option<[Option<&'a [u8]>; N]>>,
}
impl<const N: usize> Regex<N> {
    /// Returns whether or not the text is matched by the regular expression.
    #[inline]
    pub fn test(&self, text: &str) -> bool {
        if let Some(test_fn) = self.test_fn {
            return test_fn(text);
        }
        let Some(test_bytes_fn) = self.test_bytes_fn else {
            return false;
        };
        return test_bytes_fn(text.as_bytes());
    }
    /// Executes the regular expression against `text` and returns the captures if it matches.
    ///
    /// The returned array has `N` elements: index `0` is the entire match, and indices `1..N`
    /// are the individual capture groups in the order they appear in the pattern.
    /// Each element is `Some(&str)` when the group participated in the match and `None` when it
    /// did not (for example, an optional group that was not taken).
    ///
    /// Returns `None` when the regular expression does not match `text`.
    #[inline]
    pub fn exec<'a>(&self, text: &'a str) -> Option<[Option<&'a str>; N]> {
        if let Some(exec_fn) = self.exec_fn {
            return exec_fn(text);
        }
        let exec_bytes_fn = self.exec_bytes_fn?;
        let captures = exec_bytes_fn(text.as_bytes())?;
        let mut out = [None; N];
        for (i, capture) in captures.into_iter().enumerate() {
            out[i] = match capture {
                None => None,
                Some(bytes) => std::str::from_utf8(bytes).ok(),
            };
        }
        return Some(out);
    }

    /// Returns whether or not the bytes are matched by the regular expression.
    #[inline]
    pub fn test_bytes(&self, text: &[u8]) -> bool {
        if let Some(test_bytes_fn) = self.test_bytes_fn {
            return test_bytes_fn(text);
        }
        let Some(test_fn) = self.test_fn else {
            return false;
        };
        let Ok(text) = std::str::from_utf8(text) else {
            return false;
        };
        return test_fn(text);
    }

    /// Executes the regular expression against bytes and returns byte-slice captures if it matches.
    #[inline]
    pub fn exec_bytes<'a>(&self, text: &'a [u8]) -> Option<[Option<&'a [u8]>; N]> {
        if let Some(exec_bytes_fn) = self.exec_bytes_fn {
            return exec_bytes_fn(text);
        }
        let exec_fn = self.exec_fn?;
        let text_str = std::str::from_utf8(text).ok()?;
        let captures = exec_fn(text_str)?;
        let mut out = [None; N];
        for (i, capture) in captures.into_iter().enumerate() {
            out[i] = capture.map(str::as_bytes);
        }
        return Some(out);
    }
}

/// Intended to be used in macros only.
#[inline]
pub const fn __construct_regex<const N: usize>(
    fn_pair: (
        fn(&str) -> bool,
        for<'a> fn(&'a str) -> Option<[Option<&'a str>; N]>,
    ),
) -> Regex<N> {
    return Regex {
        test_fn: Some(fn_pair.0),
        exec_fn: Some(fn_pair.1),
        test_bytes_fn: None,
        exec_bytes_fn: None,
    };
}

/// Intended to be used in macros only.
#[inline]
pub const fn __construct_regex_u8<const N: usize>(
    fn_pair: (
        fn(&[u8]) -> bool,
        for<'a> fn(&'a [u8]) -> Option<[Option<&'a [u8]>; N]>,
    ),
) -> Regex<N> {
    return Regex {
        test_fn: None,
        exec_fn: None,
        test_bytes_fn: Some(fn_pair.0),
        exec_bytes_fn: Some(fn_pair.1),
    };
}

/// Intended to be used in macros only.
#[inline]
pub const fn __construct_regex_with_bytes<const N: usize>(
    str_fn_pair: (
        fn(&str) -> bool,
        for<'a> fn(&'a str) -> Option<[Option<&'a str>; N]>,
    ),
    u8_fn_pair: (
        fn(&[u8]) -> bool,
        for<'a> fn(&'a [u8]) -> Option<[Option<&'a [u8]>; N]>,
    ),
) -> Regex<N> {
    return Regex {
        test_fn: Some(str_fn_pair.0),
        exec_fn: Some(str_fn_pair.1),
        test_bytes_fn: Some(u8_fn_pair.0),
        exec_bytes_fn: Some(u8_fn_pair.1),
    };
}

fn serialize_u8_engine_as_str(
    u8_engine: proc_macro2::TokenStream,
    capture_groups: usize,
) -> proc_macro2::TokenStream {
    return quote! {{
        const __ERE_TEST_BYTES: fn(&[u8]) -> bool = #u8_engine.0;
        const __ERE_EXEC_BYTES: for<'a> fn(&'a [u8]) -> Option<[Option<&'a [u8]>; #capture_groups]> = #u8_engine.1;

        #[inline]
        fn test(text: &str) -> bool {
            return __ERE_TEST_BYTES(text.as_bytes());
        }

        #[inline]
        fn exec<'a>(text: &'a str) -> Option<[Option<&'a str>; #capture_groups]> {
            let captures = __ERE_EXEC_BYTES(text.as_bytes())?;
            return Some(captures.map(|capture| {
                capture.and_then(|bytes| ::core::str::from_utf8(bytes).ok())
            }));
        }

        (test, exec)
    }};
}

fn build_u8_search_nfa(tree: &simplified_tree::SimplifiedTreeNode) -> working_u8_nfa::U8NFA {
    let nfa = working_nfa::WorkingNFA::new_loop_opt(tree, false, false);
    return working_u8_nfa::U8NFA::new_loop_opt(&nfa, true, true);
}

/// Tries to pick the best engine that doesn't rely on sub-engines.
///
/// Returns a stream that evaluates to a pair `(test_fn, exec_fn)`
fn pick_base_engine(
    ere: parse_tree::ERE,
) -> (
    proc_macro2::TokenStream,
    simplified_tree::SimplifiedTreeNode,
    working_nfa::WorkingNFA,
    working_u8_nfa::U8NFA,
    &'static str,
) {
    let tree = simplified_tree::SimplifiedTreeNode::from(ere);
    let nfa = working_nfa::WorkingNFA::new(&tree);

    // Currently use a conservative check: only use u8 engines when it will only match ascii strings
    fn is_state_ascii(state: &working_nfa::WorkingState) -> bool {
        return state
            .transitions
            .iter()
            .flat_map(|t| t.symbol.to_ranges())
            .all(|range| range.end().is_ascii());
    }
    let is_ascii = nfa.states.iter().all(is_state_ascii);

    let u8_nfa = working_u8_nfa::U8NFA::new(&nfa);

    const ONE_PASS_U8_DESC: &str = "This regular expression is [one-pass](https://swtch.com/~rsc/regexp/regexp3.html#:~:text=Let%27s%20define%20a%20%E2%80%9Cone%2Dpass%20regular%20expression%E2%80%9D).
This allows us to use an efficient [`::ere::one_pass_u8`] implementation.";
    const FLAT_LOCKSTEP_NFA_U8_DESC: &str =
        "Uses a general-case [`::ere::flat_lockstep_nfa_u8`] implementatation over `u8`s.";
    const FLAT_LOCKSTEP_NFA_DESC: &str =
        "Uses a general-case [`::ere::flat_lockstep_nfa`] implementatation over `char`s.";
    const DFA_DESC: &str = "Uses a [`::ere::dfa::U8DFA`] implementation over `u8`s.";

    let pick_base_engine_inner = || -> (_, &str) {
        if let Some(engine) = one_pass_u8::serialize_one_pass_token_stream(&u8_nfa) {
            return (engine, ONE_PASS_U8_DESC);
        }
        let dfa_bound = working_u8_dfa::U8TDFA::default_bound(u8_nfa.states.len());
        if let Some(dfa) = working_u8_dfa::U8TDFA::from_nfa(&u8_nfa, dfa_bound) {
            return (dfa_u8::serialize_u8_dfa_token_stream(&dfa), DFA_DESC);
        }
        if is_ascii {
            let u8_engine =
                flat_lockstep_nfa_u8::serialize_flat_lockstep_nfa_u8_token_stream(&u8_nfa);
            return (
                serialize_u8_engine_as_str(u8_engine, u8_nfa.num_capture_groups()),
                FLAT_LOCKSTEP_NFA_U8_DESC,
            );
        }
        return (
            flat_lockstep_nfa::serialize_flat_lockstep_nfa_token_stream(&nfa),
            FLAT_LOCKSTEP_NFA_DESC,
        );
    };

    let (base_engine, description) = pick_base_engine_inner();
    return (base_engine, tree, nfa, u8_nfa, description);
}

/// Tries to pick the best engine that doesn't rely on sub-engines.
///
/// Returns a stream that evaluates to a pair `(test_fn, exec_fn)`
fn pick_engine(ere: parse_tree::ERE) -> (proc_macro2::TokenStream, String) {
    let (base_engine, _, _, u8_nfa, base_description) = pick_base_engine(ere);

    // Consider nested engines
    if let Some(offsets) = fixed_offset::get_fixed_offsets(&u8_nfa) {
        let engine = fixed_offset::serialize_fixed_offset_token_stream(
            base_engine,
            offsets,
            u8_nfa.num_capture_groups(),
        );
        return (
            engine,
            format!(
                "This regular expression's capture groups are always at fixed offsets.
Because of this, we can skip a complex `exec` implementation, and instead simply run `test` then index into the string.

### Details on the `test` implementation:

{base_description}"
            ),
        );
    };
    return (base_engine, base_description.to_string());
}

/// Tries to pick the best engine.
pub fn __compile_regex(stream: TokenStream) -> TokenStream {
    let ere: parse_tree::ERE = syn::parse_macro_input!(stream);
    let (str_fn_pair, _) = pick_engine(ere.clone());

    let tree = simplified_tree::SimplifiedTreeNode::from(ere);
    let nfa = build_u8_search_nfa(&tree);
    let u8_fn_pair = flat_lockstep_nfa_u8::serialize_flat_lockstep_nfa_u8_token_stream(&nfa);
    return quote! {
        {
            ::ere::__construct_regex_with_bytes(#str_fn_pair, #u8_fn_pair)
        }
    }
    .into();
}

/// Always uses the [`dfa_u8`] engine
pub fn __compile_regex_engine_dfa_u8(stream: TokenStream) -> TokenStream {
    let ere: parse_tree::ERE = syn::parse_macro_input!(stream);
    let tree = simplified_tree::SimplifiedTreeNode::from(ere);
    let nfa = working_nfa::WorkingNFA::new(&tree);
    let nfa = working_u8_nfa::U8NFA::new(&nfa);
    let dfa_state_limit = working_u8_dfa::U8TDFA::default_bound(nfa.states.len());
    let dfa = working_u8_dfa::U8TDFA::from_nfa(&nfa, dfa_state_limit);
    let Some(dfa) = dfa else {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "Failed to convert NFA into DFA: exceeded DFA state limit of {},
                compilation time and binary size could be exponential.",
                dfa_state_limit
            ),
        )
        .into_compile_error()
        .into();
    };
    let fn_pair = dfa_u8::serialize_u8_dfa_token_stream(&dfa);
    return quote! {
        ::ere::__construct_regex(#fn_pair)
    }
    .into();
}

/// Always uses the [`flat_lockstep_nfa`] engine
pub fn __compile_regex_engine_flat_lockstep_nfa(stream: TokenStream) -> TokenStream {
    let ere: parse_tree::ERE = syn::parse_macro_input!(stream);
    let tree = simplified_tree::SimplifiedTreeNode::from(ere);
    let nfa = working_nfa::WorkingNFA::new(&tree);
    let fn_pair = flat_lockstep_nfa::serialize_flat_lockstep_nfa_token_stream(&nfa);
    return quote! {
        ::ere::__construct_regex(#fn_pair)
    }
    .into();
}

/// Always uses the [`flat_lockstep_nfa_u8`] engine
pub fn __compile_regex_engine_flat_lockstep_nfa_u8(stream: TokenStream) -> TokenStream {
    let ere: parse_tree::ERE = syn::parse_macro_input!(stream);
    let tree = simplified_tree::SimplifiedTreeNode::from(ere);
    let nfa = build_u8_search_nfa(&tree);
    let fn_pair = flat_lockstep_nfa_u8::serialize_flat_lockstep_nfa_u8_token_stream(&nfa);
    return quote! {
        ::ere::__construct_regex_u8(#fn_pair)
    }
    .into();
}

/// Always uses the [`one_pass_u8`]
///
/// Will return a compiler error if regex was not one-pass and could not be optimized to become one-pass.
pub fn __compile_regex_engine_one_pass_u8(stream: TokenStream) -> TokenStream {
    let ere: parse_tree::ERE = syn::parse_macro_input!(stream);
    let tree = simplified_tree::SimplifiedTreeNode::from(ere);
    let nfa = working_nfa::WorkingNFA::new(&tree);
    let nfa = working_u8_nfa::U8NFA::new(&nfa);
    let Some(fn_pair) = one_pass_u8::serialize_one_pass_token_stream(&nfa) else {
        return syn::parse::Error::new(
            proc_macro2::Span::call_site(),
            "Regex was not one-pass and could not be optimized to become one pass. 
Try using a different engine.",
        )
        .to_compile_error()
        .into();
    };
    return quote! {
        ::ere::__construct_regex(#fn_pair)
    }
    .into();
}

/// Always uses the [`fixed_offset`]
///
/// Will return a compiler error if regex was not fixed offset.
pub fn __compile_regex_engine_fixed_offset(stream: TokenStream) -> TokenStream {
    let ere: parse_tree::ERE = syn::parse_macro_input!(stream);
    let (base_engine, _, _, u8_nfa, _) = pick_base_engine(ere);

    let Some(offsets) = fixed_offset::get_fixed_offsets(&u8_nfa) else {
        return syn::parse::Error::new(
            proc_macro2::Span::call_site(),
            "Regex capture groups were not fixed offset. Try using a different engine.",
        )
        .to_compile_error()
        .into();
    };
    let fn_pair = fixed_offset::serialize_fixed_offset_token_stream(
        base_engine,
        offsets,
        u8_nfa.num_capture_groups(),
    );
    return quote! {
        ::ere::__construct_regex(#fn_pair)
    }
    .into();
}

/// Which matching engine to use for the compiled regex.
#[cfg(feature = "unstable-attr-regex")]
#[derive(Clone, Copy)]
pub enum Engine {
    /// Automatically select the best engine (default).
    Auto,
    /// One-pass DFA over bytes. Single linear scan, no backtracking.
    OnePassU8,
    /// Deterministic finite automaton over bytes.
    DfaU8,
    /// NFA simulation over bytes via lockstep parallel execution.
    FlatLockstepNfaU8,
    /// NFA simulation over Unicode chars via lockstep parallel execution.
    FlatLockstepNfa,
    /// Extracts captures by fixed string offsets. Wraps the auto-selected base engine.
    FixedOffset,
}

/// Controls how strictly capture groups must be bound to struct fields.
#[cfg(feature = "unstable-attr-regex")]
#[derive(Clone, Copy)]
pub enum GroupBind {
    /// All capture groups (named and unnamed) must have a corresponding field.
    Strict,
    /// Only named capture groups must have a corresponding field.
    Named,
    /// No capture groups are required to have a corresponding field.
    None,
}

#[cfg(feature = "unstable-attr-regex")]
struct RegexAttr {
    ere_litstr: syn::LitStr,
    bind: GroupBind,
    engine: Engine,
}

#[cfg(feature = "unstable-attr-regex")]
impl syn::parse::Parse for RegexAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Accept either a string literal ("...") or a byte string literal (b"...").
        // Byte strings must be valid UTF-8; their content is used identically to a
        // string literal once parsed.
        let ere_litstr: syn::LitStr = if input.peek(syn::LitByteStr) {
            let byte_lit: syn::LitByteStr = input.parse()?;
            let bytes = byte_lit.value();
            let s = std::str::from_utf8(&bytes).map_err(|e| {
                syn::Error::new(byte_lit.span(), format!("byte string pattern is not valid UTF-8: {e}"))
            })?;
            syn::LitStr::new(s, byte_lit.span())
        } else {
            input.parse()?
        };
        let mut bind = GroupBind::Named;
        let mut engine = Engine::Auto;

        while input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            if input.is_empty() {
                break; // trailing comma
            }
            let key: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            let value: syn::Ident = input.parse()?;
            if key == "bind" {
                bind = match value.to_string().as_str() {
                    "Strict" => GroupBind::Strict,
                    "Named" => GroupBind::Named,
                    "None" => GroupBind::None,
                    _ => return Err(syn::Error::new_spanned(&value, "Expected `Strict`, `Named`, or `None`.")),
                };
            } else if key == "engine" {
                engine = match value.to_string().as_str() {
                    "Auto" => Engine::Auto,
                    "OnePassU8" => Engine::OnePassU8,
                    "DfaU8" => Engine::DfaU8,
                    "FlatLockstepNfaU8" => Engine::FlatLockstepNfaU8,
                    "FlatLockstepNfa" => Engine::FlatLockstepNfa,
                    "FixedOffset" => Engine::FixedOffset,
                    _ => return Err(syn::Error::new_spanned(&value, "Expected `Auto`, `OnePassU8`, `DfaU8`, `FlatLockstepNfaU8`, `FlatLockstepNfa`, or `FixedOffset`.")),
                };
            } else {
                return Err(syn::Error::new_spanned(&key, format!("Unknown attribute `{key}`. Expected `bind` or `engine`.")));
            }
        }

        Ok(RegexAttr { ere_litstr, bind, engine })
    }
}

#[cfg(feature = "unstable-attr-regex")]
pub fn __compile_regex_attr(attr: TokenStream, input: TokenStream) -> TokenStream {
    let RegexAttr { ere_litstr, bind, engine } = syn::parse_macro_input!(attr as RegexAttr);
    let ere_str = ere_litstr.value();
    let ere = match parse_tree::ERE::parse_str_syn(&ere_str, ere_litstr.span()) {
        Ok(ere) => ere,
        Err(compile_err) => return compile_err.into_compile_error().into(),
    };

    let tree = simplified_tree::SimplifiedTreeNode::from(ere.clone());
    let nfa = working_nfa::WorkingNFA::new(&tree);
    let u8_nfa = build_u8_search_nfa(&tree);

    let capture_groups = nfa.num_capture_groups();
    let optional_captures: Vec<bool> = (0..capture_groups)
        .map(|group_num| nfa.capture_group_is_optional(group_num))
        .collect();

    let mut regex_struct: syn::DeriveInput = syn::parse_macro_input!(input);
    let syn::Data::Struct(ref mut data_struct) = regex_struct.data else {
        return syn::parse::Error::new_spanned(
            regex_struct,
            "Attribute regexes currently only support structs.",
        )
        .to_compile_error()
        .into();
    };

    let (fn_pair, description) = match engine {
        Engine::Auto => pick_engine(ere.clone()),
        Engine::OnePassU8 => {
            let Some(fp) = one_pass_u8::serialize_one_pass_token_stream(&u8_nfa) else {
                return syn::parse::Error::new_spanned(
                    &ere_litstr,
                    "Regex is not one-pass and could not be optimized to become one-pass. Try a different engine.",
                )
                .to_compile_error()
                .into();
            };
            (fp, "Uses the `one_pass_u8` engine.".to_string())
        }
        Engine::DfaU8 => {
            let dfa_state_limit = working_u8_dfa::U8TDFA::default_bound(u8_nfa.states.len());
            let Some(dfa) = working_u8_dfa::U8TDFA::from_nfa(&u8_nfa, dfa_state_limit) else {
                return syn::parse::Error::new_spanned(
                    &ere_litstr,
                    format!("Failed to convert NFA into DFA: exceeded DFA state limit of {dfa_state_limit}. Try a different engine."),
                )
                .to_compile_error()
                .into();
            };
            (dfa_u8::serialize_u8_dfa_token_stream(&dfa), "Uses the `dfa_u8` engine.".to_string())
        }
        Engine::FlatLockstepNfaU8 => {
            let u8_fp = flat_lockstep_nfa_u8::serialize_flat_lockstep_nfa_u8_token_stream(&u8_nfa);
            let fp = serialize_u8_engine_as_str(u8_fp, capture_groups);
            (fp, "Uses the `flat_lockstep_nfa_u8` engine.".to_string())
        }
        Engine::FlatLockstepNfa => {
            let fp = flat_lockstep_nfa::serialize_flat_lockstep_nfa_token_stream(&nfa);
            (fp, "Uses the `flat_lockstep_nfa` engine.".to_string())
        }
        Engine::FixedOffset => {
            let (base_engine, _, _, u8_nfa, _) = pick_base_engine(ere.clone());
            let Some(offsets) = fixed_offset::get_fixed_offsets(&u8_nfa) else {
                return syn::parse::Error::new_spanned(
                    &ere_litstr,
                    "Regex capture groups are not at fixed offsets. Try a different engine.",
                )
                .to_compile_error()
                .into();
            };
            let fp = fixed_offset::serialize_fixed_offset_token_stream(
                base_engine,
                offsets,
                u8_nfa.num_capture_groups(),
            );
            (fp, "Uses the `fixed_offset` engine.".to_string())
        }
    };
    let fn_pair_bytes = flat_lockstep_nfa_u8::serialize_flat_lockstep_nfa_u8_token_stream(&u8_nfa);
    let struct_name = regex_struct.ident.clone();
    let ere_display_doc = format!("`{ere_str}`");
    let struct_name_link_doc = format!("[`{}`]", struct_name.to_string());

    let constructor = match &mut data_struct.fields {
        syn::Fields::Unnamed(fields) => {
            if fields.unnamed.len() != optional_captures.len() {
                return syn::parse::Error::new_spanned(
                    &fields.unnamed,
                    format!(
                        "Expected struct to have {} unnamed fields, based on number of captures in regular expression.",
                        optional_captures.len()
                    ),
                )
                .to_compile_error()
                .into();
            }
            let args: proc_macro2::TokenStream = optional_captures
                .iter()
                .enumerate()
                .map(|(group_num, opt)| if *opt {
                    quote! { result[#group_num], }
                } else {
                    quote! {
                        result[#group_num]
                        .expect(
                            "If you are seeing this, there is probably an internal bug in the `ere-core` crate where a capture group was mistakenly marked as non-optional. Please report the bug."
                        ),
                    }
                })
                .collect();
            quote! { #struct_name(#args) }
        }
        syn::Fields::Named(ref mut fields) => {
            let group_names = ere.group_names();
            let name_to_group: std::collections::HashMap<String, usize> = group_names
                .iter()
                .enumerate()
                .filter_map(|(i, name)| name.as_ref().map(|n| (n.clone(), i + 1)))
                .collect();

            let mut field_args = Vec::new();
            let mut used_groups = std::collections::HashSet::new();
            used_groups.insert(0usize); // group 0 is implicitly always present
            for field in fields.named.iter_mut() {
                let ident = field.ident.as_ref().unwrap();

                // Parse #[group(N)] attribute if present, then strip it
                let explicit_group: Option<usize> = field.attrs.iter()
                    .find(|a| a.path().is_ident("group"))
                    .and_then(|a| a.parse_args::<syn::LitInt>().ok())
                    .and_then(|lit| lit.base10_parse().ok());
                field.attrs.retain(|a| !a.path().is_ident("group"));

                let group_num = if let Some(n) = explicit_group {
                    n
                } else {
                    let name = ident.to_string();
                    match name_to_group.get(&name) {
                        Some(&n) => n,
                        None => {
                            return syn::parse::Error::new_spanned(
                                ident,
                                format!("No capture group named `{name}` found in the regular expression."),
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                };
                if group_num >= capture_groups {
                    return syn::parse::Error::new_spanned(
                        ident,
                        format!(
                            "#[group({group_num})] is out of range: the regular expression only has {} capture group(s) (groups 0..{}).",
                            capture_groups,
                            capture_groups,
                        ),
                    )
                    .to_compile_error()
                    .into();
                }
                used_groups.insert(group_num);
                let opt = optional_captures[group_num];
                let arg = if opt {
                    quote! { #ident: result[#group_num], }
                } else {
                    quote! {
                        #ident: result[#group_num]
                            .expect("If you are seeing this, there is probably an internal bug in the `ere-core` crate where a capture group was mistakenly marked as non-optional. Please report the bug."),
                    }
                };
                field_args.push(arg);
            }

            // Check for unbound capture groups based on the bind mode.
            for group_num in 0..capture_groups {
                if used_groups.contains(&group_num) {
                    continue;
                }
                let is_named = name_to_group.iter().find(|(_, &g)| g == group_num);
                match (bind, is_named) {
                    (GroupBind::None, _) | (GroupBind::Named, Option::None) => {}
                    (GroupBind::Named | GroupBind::Strict, Some((name, _))) => {
                        return syn::parse::Error::new_spanned(
                            &ere_litstr,
                            format!("Named capture group `{name}` has no corresponding field in the struct."),
                        )
                        .to_compile_error()
                        .into();
                    }
                    (GroupBind::Strict, Option::None) => {
                        return syn::parse::Error::new_spanned(
                            &ere_litstr,
                            format!("Capture group {group_num} has no corresponding field in the struct. Add a field like `#[group({group_num})] captured: &'a str`."),
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }

            let args: proc_macro2::TokenStream = field_args.into_iter().collect();
            quote! { #struct_name { #args } }
        }
        syn::Fields::Unit => {
            return syn::parse::Error::new_spanned(
                &struct_name,
                "Attribute regexes require a struct with fields.",
            )
            .to_compile_error()
            .into();
        }
    };

    let implementation = quote! {
        impl<'a> #struct_name<'a> {
            const ENGINE: (
                fn(&str) -> bool,
                fn(&'a str) -> ::core::option::Option<[::core::option::Option<&'a str>; #capture_groups]>,
            ) = #fn_pair;
            const ENGINE_BYTES: (
                fn(&[u8]) -> bool,
                fn(&'a [u8]) -> ::core::option::Option<[::core::option::Option<&'a [u8]>; #capture_groups]>,
            ) = #fn_pair_bytes;
            /// Returns `true` if the regular expression
            #[doc = #ere_display_doc]
            /// matches the string.
            /// Otherwise, returns `false`
            ///
            /// ## Implementation
            #[doc = #description]
            #[inline]
            pub fn test(text: &str) -> bool {
                return (Self::ENGINE.0)(text);
            }
            /// Returns `true` if the regular expression
            #[doc = #ere_display_doc]
            /// matches the bytes.
            /// Otherwise, returns `false`.
            #[inline]
            pub fn test_bytes(text: &[u8]) -> bool {
                return (Self::ENGINE_BYTES.0)(text);
            }
            /// Returns an instance of
            #[doc = #struct_name_link_doc]
            /// containing capture groups if
            #[doc = #ere_display_doc]
            /// matches the string.
            /// Otherwise, returns `None`.
            ///
            /// ## Implementation
            #[doc = #description]
            pub fn exec(text: &'a str) -> ::core::option::Option<#struct_name<'a>> {
                let result: [::core::option::Option<&'a str>; #capture_groups] = (Self::ENGINE.1)(text)?;
                return ::core::option::Option::<#struct_name<'a>>::Some(#constructor);
            }
            /// Returns an instance of
            #[doc = #struct_name_link_doc]
            /// containing capture groups if
            #[doc = #ere_display_doc]
            /// matches the bytes and all captured slices are valid UTF-8.
            /// Otherwise, returns `None`.
            pub fn exec_bytes(text: &'a [u8]) -> ::core::option::Option<#struct_name<'a>> {
                let result_bytes: [::core::option::Option<&'a [u8]>; #capture_groups] = (Self::ENGINE_BYTES.1)(text)?;
                let result: [::core::option::Option<&'a str>; #capture_groups] = result_bytes.map(|capture| {
                    capture.and_then(|bytes| ::core::str::from_utf8(bytes).ok())
                });
                return ::core::option::Option::<#struct_name<'a>>::Some(#constructor);
            }
        }
    };
    return quote! {
        #regex_struct
        #implementation
    }.into();
}
