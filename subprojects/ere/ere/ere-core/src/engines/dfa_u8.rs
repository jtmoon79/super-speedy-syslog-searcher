//! Implements a statically-built DFA over `u8`s.
//! Executing a DFA is much faster than running an NFA due to dealing with only a single thread.
//! However, since the upper bound of a DFA's size is exponential in the number of NFA states,
//! we may need to cancel static construction if the DFA becomes too large.
//!
//! For more information, read https://en.wikipedia.org/wiki/Powerset_construction

use crate::working_u8_dfa::{U8TDFAAccept, U8TDFAState, U8TDFATransition, U8DFA, U8TDFA};
use quote::{quote, ToTokens, TokenStreamExt as _};

/// If `idx == usize::MAX`, then it is the start state
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum VMStateLabel {
    Start,
    Normal(usize),
}
impl ToTokens for VMStateLabel {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let _label;
        let label = match self {
            VMStateLabel::Start => "StateStart",
            VMStateLabel::Normal(idx) => {
                _label = format!("State{idx}");
                &_label
            }
        };
        let ident = proc_macro2::Ident::new(&label, proc_macro2::Span::call_site());
        tokens.append(ident);
    }
}

mod impl_test {
    use crate::working_u8_dfa::{U8DFAAccept, U8DFAState, U8DFATransition, U8DFA};

    use super::*;

    pub struct TestFn<'a>(pub &'a U8DFA);
    impl<'a> ToTokens for TestFn<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &TestFn(dfa) = self;

            if matches!(dfa.start_state.accept, U8DFAAccept::Always) {
                // this regular expression accepts all strings.
                // note that U8DFAAccept::Anchored from start is accounted for with state_accepts_at_end,
                // where we will only still be on start state if length is 0
                tokens.extend(quote! {
                    fn test(text: &str) -> bool {
                        return true;
                    }
                });
                return;
            }

            let enum_states = dfa
                .states
                .iter()
                .enumerate()
                .map(|(i, _)| VMStateLabel::Normal(i));

            let normal_match_statements = dfa
                .states
                .iter()
                .enumerate()
                .map(|(i, state)| (VMStateLabel::Normal(i), state))
                .chain(std::iter::once((VMStateLabel::Start, &dfa.start_state)))
                .map(NormalStateSymbolMatchStatements::from_pair);

            let state_accepts_at_end = dfa
                .states
                .iter()
                .chain(std::iter::once(&dfa.start_state))
                .map(|state| !matches!(&state.accept, U8DFAAccept::None));
            let enum_states_for_accepts_at_end = enum_states
                .clone()
                .chain(std::iter::once(VMStateLabel::Start));

            tokens.extend(quote! {
                fn test(text: &str) -> bool {
                    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
                    enum VMStates {
                        StateStart,
                        #(#enum_states,)*
                    }

                    let mut state: VMStates = VMStates::StateStart;

                    for c in text.bytes() {
                        match (state, c) {
                            #(#normal_match_statements)*
                            _ => return false,
                        }
                    }
                    return match state {
                        #(VMStates::#enum_states_for_accepts_at_end => #state_accepts_at_end,)*
                    };
                }
            });
        }
    }

    enum NormalStateSymbolMatchStatements<'a> {
        ImmediateAccept(VMStateLabel),
        SymbolTransitions(VMStateLabel, &'a [U8DFATransition]),
    }
    impl<'a> NormalStateSymbolMatchStatements<'a> {
        fn from_pair((label, state): (VMStateLabel, &'a U8DFAState)) -> Self {
            if let U8DFAAccept::Always = &state.accept {
                // arriving at this state means we can directly accept
                // we don't have to worry about anchored state accept, since symbol transitions are never at end
                return NormalStateSymbolMatchStatements::ImmediateAccept(label);
            }
            return NormalStateSymbolMatchStatements::SymbolTransitions(label, &state.transitions);
        }
    }
    impl<'a> ToTokens for NormalStateSymbolMatchStatements<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                NormalStateSymbolMatchStatements::ImmediateAccept(from) => {
                    tokens.extend(quote! {
                        (VMStates::#from, _) => {
                            return true;
                        }
                    });
                }
                &NormalStateSymbolMatchStatements::SymbolTransitions(from, dfa_tr) => {
                    for tr in dfa_tr {
                        let to = VMStateLabel::Normal(tr.to);
                        let symbol_start = tr.symbol.start();
                        let symbol_end = tr.symbol.end();
                        tokens.extend(quote! {
                            (VMStates::#from, #symbol_start..=#symbol_end) => {
                                state = VMStates::#to;
                            }
                        });
                    }
                }
            }
        }
    }
}

mod impl_exec {
    use crate::{epsilon_propogation::Tag, working_u8_dfa::U8TDFAAcceptTransition};

    use super::*;

    pub struct ExecFn<'a>(pub &'a U8TDFA);
    impl<'a> ToTokens for ExecFn<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &ExecFn(dfa) = self;
            let num_captures = dfa.num_capture_groups();
            let def_enum_states = dfa
                .states
                .iter()
                .enumerate()
                .map(|(i, _)| VMStateLabel::Normal(i));
            let def_enum_states_nfa_count = dfa.states.iter().map(|state| state.nfa_states.len());

            // match dfa.start_state.accept {
            //     U8TDFAAccept::Both(anchored, unanchored) => {}
            // }

            let normal_match_statements = dfa
                .states
                .iter()
                .enumerate()
                .map(|(i, state)| (VMStateLabel::Normal(i), state))
                .chain(std::iter::once((VMStateLabel::Start, &dfa.start_state)))
                .map(NormalStateSymbolMatchStatements::from_pair);

            let state_accepts_at_end = dfa
                .states
                .iter()
                .map(|state| state)
                .chain(std::iter::once(&dfa.start_state))
                .map(|state| {
                    let U8TDFAAcceptTransition {
                        nfa_state,
                        add_tags,
                    } = match &state.accept {
                        U8TDFAAccept::None => return quote! { ::core::option::Option::None },
                        U8TDFAAccept::Anchored(accept) => accept,
                        U8TDFAAccept::Both(accept, _) => accept,
                        U8TDFAAccept::Unanchored(accept) => accept,
                    };

                    let add_tags = add_tags.iter().map(|tag| match tag {
                        Tag::StartCapture(group_idx) => {
                            quote! { new_tags[#group_idx].0 = text.len(); }
                        }
                        Tag::EndCapture(group_idx) => {
                            quote! { new_tags[#group_idx].1 = text.len(); }
                        }
                    });

                    return quote! {
                        let mut new_tags = old_tags[#nfa_state];
                        #(#add_tags)*
                        ::core::option::Option::Some(new_tags)
                    };
                });
            let enum_states_for_accepts_at_end = def_enum_states
                .clone()
                .chain(std::iter::once(VMStateLabel::Start));

            tokens.extend(quote! {
                fn exec<'a>(
                    text: &'a str,
                ) -> ::core::option::Option<[::core::option::Option<&'a str>; #num_captures]> {
                    fn exec_inner<'a>(
                        text: &'a str,
                    ) -> ::core::option::Option<[(usize, usize); #num_captures]> {
                        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
                        enum VMStates {
                            StateStart([[(usize, usize); #num_captures]; 1]),
                            #(#def_enum_states(
                                [[(usize, usize); #num_captures]; #def_enum_states_nfa_count]
                            ),)*
                        }

                        let mut state: VMStates = VMStates::StateStart(
                            [[(usize::MAX, usize::MAX); #num_captures]; 1]
                        );

                        for (i, c) in text.bytes().enumerate() {
                            match (state, c) {
                                #(#normal_match_statements)*
                                _ => return ::core::option::Option::None,
                            }
                        }
                        return match state {
                            #(VMStates::#enum_states_for_accepts_at_end(old_tags) => {
                                #state_accepts_at_end
                            })*
                        };
                    }

                    let captures = exec_inner(text)?;
                    let mut capture_strs = [::core::option::Option::None; #num_captures];
                    for (i, (start, end)) in captures.into_iter().enumerate() {
                        if start != usize::MAX {
                            assert_ne!(end, usize::MAX);
                            // assert!(start <= end);
                            capture_strs[i] = text.get(start..end);
                            assert!(capture_strs[i].is_some());
                        } else {
                            assert_eq!(end, usize::MAX);
                        }
                    }
                    return ::core::option::Option::Some(capture_strs);
                }
            });
        }
    }

    enum NormalStateSymbolMatchStatements<'a> {
        // ImmediateAccept(VMStateLabel, &'a U8TDFAState),
        SymbolTransitions(VMStateLabel, &'a U8TDFAState),
    }
    impl<'a> NormalStateSymbolMatchStatements<'a> {
        fn from_pair((label, state): (VMStateLabel, &'a U8TDFAState)) -> Self {
            // TODO: early return, we need to follow the ambiguous submatching rules
            // if let U8TDFAAccept::Unanchored(_) | U8TDFAAccept::Both(_, _) = &state.accept {
            //     // arriving at this state means we can directly accept
            //     // we don't have to worry about anchored state accept, since symbol transitions are never at end
            //     return NormalStateSymbolMatchStatements::ImmediateAccept(label, state);
            // }
            return NormalStateSymbolMatchStatements::SymbolTransitions(label, state);
        }
    }
    impl<'a> ToTokens for NormalStateSymbolMatchStatements<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                // NormalStateSymbolMatchStatements::ImmediateAccept(from) => {
                //     tokens.extend(quote! {
                //         (#from, _) => {
                //             return true;
                //         }
                //     });
                // }
                &NormalStateSymbolMatchStatements::SymbolTransitions(from, state) => {
                    for tr in &state.transitions {
                        NormalTransitionMatchStatement { from, tr }.to_tokens(tokens);
                    }
                }
            }
        }
    }

    struct NormalTransitionMatchStatement<'a> {
        from: VMStateLabel,
        tr: &'a U8TDFATransition,
    }
    impl<'a> ToTokens for NormalTransitionMatchStatement<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let from = self.from;
            let to = VMStateLabel::Normal(self.tr.to);
            let symbol_start = self.tr.symbol.start();
            let symbol_end = self.tr.symbol.end();

            let copy_tags = &self.tr.copy_tags;
            let add_tags = self
                .tr
                .add_tags
                .iter()
                .map(|(local_nfa_idx, tag)| match tag {
                    Tag::StartCapture(group_idx) => {
                        quote! { new_tags[#local_nfa_idx][#group_idx].0 = i; }
                    }
                    Tag::EndCapture(group_idx) => {
                        quote! { new_tags[#local_nfa_idx][#group_idx].1 = i; }
                    }
                });
            tokens.extend(quote! {
                (VMStates::#from(old_tags), #symbol_start..=#symbol_end) => {
                    let mut new_tags = [#(old_tags[#copy_tags]),*];
                    #(#add_tags)*
                    state = VMStates::#to(new_tags);
                }
            });
        }
    }
}

/// Converts a [`U8TDFA`] into a format that, when returned by a proc macro, will
/// create the corresponding engine.
///
/// Will use [`U8DFA`] as an intermediate representation for `test` implementation.
///
/// Will evaluate to a `const` pair `(test_fn, exec_fn)`.
pub(crate) fn serialize_u8_dfa_token_stream(tdfa: &U8TDFA) -> proc_macro2::TokenStream {
    let Ok(dfa) = U8DFA::from_tagged(tdfa) else {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "Failed to convert TDFA to DFA. This is probably an internal error. Please report this.",
        ).into_compile_error();
    };
    let test_fn = impl_test::TestFn(&dfa);
    let exec_fn = impl_exec::ExecFn(&tdfa);

    return quote! {{
        #test_fn
        #exec_fn

        (test, exec)
    }};
}
