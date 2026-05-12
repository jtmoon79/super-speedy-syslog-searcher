//! This is a highly-efficient implementation
//! for regexes where capture groups always have the same offset and the same length (in bytes).
//! This also means the text has a fixed length.
//!
//! This means that no variable quantifiers are allowed,
//! and alternations must have the same length in each case.
//! Additionally, capture groups cannot occur within alternations.
//!
//! The strategy is essentially to only ever run `test` then to index fixed offsets
//!
//! ## Examples
//! - `^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$` for hex colors
//! - `^\+1 ([0-9]{3})-([0-9]{3})-([0-9]{4})$` for US phone numbers

use quote::quote;

use crate::working_u8_nfa::U8NFA;

/// If the offsets are fixed, returns them for each capture group.
/// Value is [`usize::MAX`] if a capture group never matches (usually shouldn't happen)
pub(crate) fn get_fixed_offsets(nfa: &U8NFA) -> Option<Vec<(usize, usize)>> {
    let Some(_) = nfa.topological_ordering() else {
        // there are loops
        return None;
    };

    let num_capture_groups = nfa.num_capture_groups();
    let mut final_capture_groups = None;
    let mut stack = vec![(
        0usize,
        0usize,
        vec![(usize::MAX, usize::MAX); num_capture_groups],
    )];

    while let Some((state_idx, offset, capture_offsets)) = stack.pop() {
        let state = &nfa.states[state_idx];

        // If at accept state
        if state_idx + 1 == nfa.states.len() {
            match &final_capture_groups {
                None => {
                    final_capture_groups = Some(capture_offsets.clone());
                }
                Some(final_capture_groups) if *final_capture_groups != capture_offsets => {
                    return None;
                }
                _ => {}
            }
        }

        // Handle transitions
        for tr in &state.transitions {
            stack.push((tr.to, offset + 1, capture_offsets.clone()));
        }
        for ep in &state.epsilons {
            let mut capture_offsets = capture_offsets.clone();
            match ep.special {
                crate::working_nfa::EpsilonType::None => {}
                crate::working_nfa::EpsilonType::StartAnchor => {}
                crate::working_nfa::EpsilonType::EndAnchor => {}
                crate::working_nfa::EpsilonType::StartCapture(group) => {
                    capture_offsets[group].0 = offset
                }
                crate::working_nfa::EpsilonType::EndCapture(group) => {
                    capture_offsets[group].1 = offset
                }
            }
            stack.push((ep.to, offset, capture_offsets));
        }
    }
    return final_capture_groups;
}

/// Uses the `test` function from an inner engine for the exec
/// and extracts capture groups with fixed offsets
pub(crate) fn serialize_fixed_offset_token_stream(
    inner_engine: proc_macro2::TokenStream,
    offsets: Vec<(usize, usize)>,
    capture_groups: usize,
) -> proc_macro2::TokenStream {
    let extract_offsets = offsets.iter().map(|(start, end)| {
        if *start == usize::MAX || *end == usize::MAX {
            return quote! { ::core::option::Option::None };
        }
        return quote! { ::core::option::Option::Some(&text[#start..#end]) };
    });

    return quote! {{
        const TEST_FN: fn(&str) -> bool = #inner_engine.0;
        #[inline]
        fn exec<'a>(text: &'a str) -> ::core::option::Option<[::core::option::Option<&'a str>; #capture_groups]> {
            if !(TEST_FN)(text) {
                return ::core::option::Option::None;
            }
            return ::core::option::Option::Some(
                [#(#extract_offsets),*]
            );
        }
        (TEST_FN, exec)
    }};
}
