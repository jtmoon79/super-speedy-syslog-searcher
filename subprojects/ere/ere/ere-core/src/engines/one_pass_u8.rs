//! This implements an engine for [one-pass](https://swtch.com/~rsc/regexp/regexp3.html#:~:text=Use%20a%20one%2Dpass%20NFA%20if%20possible) regexes.

use crate::working_u8_nfa::U8NFA;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

/// We only need to retain states with outgoing symbol transitions
/// As well as the initial and accept states
fn compute_excluded_states(nfa: &U8NFA) -> Vec<bool> {
    let mut out = vec![true; nfa.states.len()];
    out[0] = false;
    for state in &nfa.states {
        for t in &state.transitions {
            out[t.to] = false;
        }
    }

    return out;
}

#[derive(Clone, PartialEq, Eq)]
struct ThreadUpdates {
    pub state: usize,
    pub update_captures: Vec<(bool, bool)>,
    pub start_only: bool,
    pub end_only: bool,
}
impl ThreadUpdates {
    pub fn new(state: usize, num_captures: usize) -> ThreadUpdates {
        return ThreadUpdates {
            state,
            update_captures: vec![(false, false); num_captures],
            start_only: false,
            end_only: false,
        };
    }
}

/// If the NFA is [one-pass](https://swtch.com/~rsc/regexp/regexp3.html#:~:text=Use%20a%20one%2Dpass%20NFA%20if%20possible), this function will serialize it. Otherwise, returns `None`.
///
/// An NFA/regex is one-pass if there will only ever be one thread active at a time for the NFA.
/// This means each state will only ever end up in at most one next state given an input symbol
/// (has one outgoing symbol transition, including those reachable by epsilon transitions).
///
/// A [`U8NFA`] should be one-pass whenever its corresponding [`WorkingNFA`](`crate::working_nfa::WorkingNFA`) is.
/// There are some optimizations that may make a non-one-pass NFA into a one-pass one:
/// For example, `^(a|a|b)$` into `^(a|b)$`
///
/// Will evaluate to a `const` pair `(test_fn, exec_fn)`.
pub(crate) fn serialize_one_pass_token_stream(nfa: &U8NFA) -> Option<TokenStream> {
    let num_captures = nfa.num_capture_groups();
    let mut symbol_transitions = vec![Vec::new(); nfa.states.len()];
    // states with epsilons into the accept state
    // because the other transitions do any special behavior before the symbol
    // but the space after the last character needs its own
    let mut accept_transitions = vec![Vec::new(); nfa.states.len()];
    for (state_idx, _) in nfa.states.iter().enumerate() {
        let mut stack = vec![ThreadUpdates::new(state_idx, num_captures)];
        let mut reached = vec![ThreadUpdates::new(state_idx, num_captures)];
        while let Some(thread) = stack.pop() {
            if thread.state + 1 == nfa.states.len() {
                accept_transitions[state_idx].push(thread.clone());
            }
            for ep in &nfa.states[thread.state].epsilons {
                let mut new_thread = thread.clone();
                new_thread.state = ep.to;
                match ep.special {
                    crate::working_nfa::EpsilonType::None => (),
                    crate::working_nfa::EpsilonType::StartAnchor => new_thread.start_only = true,
                    crate::working_nfa::EpsilonType::EndAnchor => new_thread.end_only = true,
                    crate::working_nfa::EpsilonType::StartCapture(c) => {
                        new_thread.update_captures[c].0 = true
                    }
                    crate::working_nfa::EpsilonType::EndCapture(c) => {
                        new_thread.update_captures[c].1 = true
                    }
                }
                if !reached.contains(&new_thread) {
                    reached.push(new_thread.clone());
                    stack.push(new_thread);
                }
            }
            for tr in &nfa.states[thread.state].transitions {
                let new_transition = (
                    tr.symbol.0.clone(),
                    ThreadUpdates {
                        state: tr.to,
                        update_captures: thread.update_captures.clone(),
                        start_only: thread.start_only,
                        end_only: thread.end_only,
                    },
                );
                // we can skip if it's exactly the same, otherwise it's a conflict.
                if !symbol_transitions[state_idx].contains(&new_transition) {
                    symbol_transitions[state_idx].push(new_transition);
                }
            }
        }
    }
    for state_transitions in &mut symbol_transitions {
        state_transitions.sort_by_key(|(range, _)| *range.start());
        let overlap = !state_transitions.windows(2).all(|ranges| {
            if let &[(a, _), (b, _)] = &ranges {
                return a.end() < b.start();
            } else {
                unreachable!("Vec::windows does not use const generics so we have to do this.");
            }
        });
        if overlap {
            return None;
        }
    }

    // == codegen ==
    if let Some(_) = nfa.topological_ordering() {
        // There are no loops in this NFA so we can confidently avoid unbounded recursion
        // (plus with the DAG, the state functions seem to get inlined,
        // but with loops it is inconsistent and sometimes produces two inlined copies
        // of the remaining path when it should be converging).
        // Production compiler optimization (i.e. except opt level 0) should minimize all other
        // internal function calls
        return Some(codegen_functional(
            nfa,
            num_captures,
            symbol_transitions,
            accept_transitions,
        ));
    } else {
        // once [`explicit_tail_calls`](https://github.com/rust-lang/rust/issues/112788) is stable,
        // we can add it to [`codegen_functional`] and remove [`codegen_vmlike`].
        return Some(codegen_vmlike(
            nfa,
            num_captures,
            symbol_transitions,
            accept_transitions,
        ));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
enum Tag {
    StartCapture(usize),
    EndCapture(usize),
}

/// Represents a sequence of states that are passed through without
/// any other incoming/outgoing transitions on its internal states
/// (start can have more incoming, and end can have more outgoing).
///
/// It is used to optimize non-branching sequences like `abcd` or `[a-z]{5}`.
/// The benefit comes from the removal of branches, since doing
/// `bytes[x] == b'a' & bytes[x+1] == b'b' & bytes[x+2] == b'c' & bytes[x+3] == b'd'`
/// is more efficient in these cases than allowing short circuiting and other branching behavior.
/// This also potentially allows for vectorization and simd, by organizing related sequences more closely.
#[derive(Clone)]
struct Run {
    start_state: usize,
    symbols: Vec<std::ops::RangeInclusive<u8>>,
    /// `(offset, tag)` where `offset == 0` would refer to the location of the first matched symbol
    tags: Vec<(usize, Tag)>,
    end_state: usize,
}
/// How a state is included in a run (if any).
#[derive(Clone)]
enum StateRunInclusion {
    /// This state is the start of a run.
    Start(Run),
    /// This state is internal to a run,
    /// and thus does not need to be included if the run is implemented.
    Internal,
    /// This state is at the end of a run
    /// (for outgoing transitions, acts the same as [`StateRunInclusion::None`]).
    End,
    /// This state is not part of a run.
    None,
}

/// ## Parameters
/// - `symbol_transitions`: for each state, a list of ranges to match with their corresponding
/// thread updates (the changes to apply if it matches). As a one-pass NFA, the ranges will never
/// overlap within a state.
///
/// ## Returns
/// For each state, if it is the start of a run,
/// A set of runs, where each state in the run only has one incoming and one outgoing transition,
/// except incoming transitions to the first and outgoing transitions to the last (which may
/// have any number).
///
/// e.g. `q0 -h> q1 -e> q2 -l> q3 -l> q4 -o> q5`
fn compute_runs(
    symbol_transitions: &Vec<Vec<(std::ops::RangeInclusive<u8>, ThreadUpdates)>>,
) -> Vec<StateRunInclusion> {
    let mut incoming_count = vec![0; symbol_transitions.len()];
    for state_transitions in symbol_transitions {
        for (_, update) in state_transitions {
            incoming_count[update.state] += 1;
        }
    }

    let run_next: Vec<_> = symbol_transitions
        .iter()
        .map(|state_transitions| {
            let &[(range, update)] = &state_transitions.as_slice() else {
                return None;
            };
            if update.end_only || update.start_only || incoming_count[update.state] != 1 {
                return None;
            }
            let tags: Vec<_> = update
                .update_captures
                .iter()
                .enumerate()
                .flat_map(|(i, set_tags)| match set_tags {
                    (false, false) => Vec::new(),
                    (true, false) => vec![Tag::StartCapture(i)],
                    (false, true) => vec![Tag::EndCapture(i)],
                    (true, true) => vec![Tag::StartCapture(i), Tag::EndCapture(i)],
                })
                .collect();
            return Some((update.state, range, tags));
        })
        .collect();

    // Whether a run starts at this state.
    let mut run_start: Vec<bool> = run_next.iter().map(Option::is_some).collect();
    for next in &run_next {
        if let Some((next, _, _)) = next {
            run_start[*next] = false;
        }
    }

    let mut out = vec![StateRunInclusion::None; symbol_transitions.len()];
    for (start_state, is_start) in run_start.iter().enumerate() {
        if !*is_start {
            continue;
        }

        let mut symbols = Vec::new();
        let mut tags = Vec::new();
        let mut internal = Vec::new(); // also may include end
        let mut state = start_state;
        while let Some(Some((next, range, transition_tags))) = run_next.get(state) {
            if *next == start_state {
                // should never happen since this would be a disconnected part of the NFA
                // that is a single loop with no incoming/outgoing transitions
                // but just in case, we do this check.
                break;
            }
            for tag in transition_tags {
                tags.push((symbols.len(), tag.clone()));
            }
            symbols.push((*range).clone());
            internal.push(*next);
            state = *next;
        }
        tags.sort();

        if state == start_state {
            // Shouldn't happen because it means a loop was allowed or it was an invalid start state,
            // but just in case I'm leaving it here
            continue;
        }
        out[start_state] = StateRunInclusion::Start(Run {
            start_state,
            symbols,
            tags,
            end_state: state,
        });
        for internal_state in internal {
            out[internal_state] = StateRunInclusion::Internal;
        }
        out[state] = StateRunInclusion::End;
    }

    return out;
}

struct ImplVMStateLabel {
    idx: usize,
}
impl ImplVMStateLabel {
    fn new(idx: usize) -> ImplVMStateLabel {
        return ImplVMStateLabel { idx };
    }
}
impl ToTokens for ImplVMStateLabel {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ImplVMStateLabel { idx } = self;
        let label = format!("State{idx}");
        let ident = proc_macro2::Ident::new(&label, proc_macro2::Span::call_site());
        tokens.append(ident);
    }
}

/// [`ToTokens`] implementation for the `test` function with vm-like one-pass implementation.
mod vmlike_test {
    use super::*;

    struct MatchStatement<'a> {
        state_idx: usize,
        symbol_transitions: &'a [(std::ops::RangeInclusive<u8>, ThreadUpdates)],
        excluded_states: &'a [bool],
    }
    impl<'a> ToTokens for MatchStatement<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &MatchStatement {
                state_idx,
                symbol_transitions,
                excluded_states,
            } = self;
            let this_state = ImplVMStateLabel::new(state_idx);
            for (range, thread) in symbol_transitions.iter() {
                if excluded_states[thread.state] {
                    continue; // no point in going here, we included the relevant propogated states already
                }
                if thread.end_only {
                    continue; // only allowed in final match
                }
                let range_start = *range.start();
                let range_end = *range.end();
                let conditions = if thread.start_only {
                    quote! {if i == 0}
                } else {
                    TokenStream::new()
                };
                let to = ImplVMStateLabel::new(thread.state);
                tokens.extend(quote! {
                    (VMStates::#this_state, #range_start..=#range_end) #conditions => {
                        state = VMStates::#to;
                    }
                });
            }
        }
    }

    /// Match statements after the input is fully consumed
    struct MatchStatementFinal<'a> {
        state_idx: usize,
        accept_transitions: &'a [ThreadUpdates],
    }
    impl<'a> ToTokens for MatchStatementFinal<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &MatchStatementFinal {
                state_idx,
                accept_transitions,
            } = self;
            let this_state = ImplVMStateLabel::new(state_idx);
            // Off the top of my head I think it should only ever have at most one, but idk
            for thread in accept_transitions.iter() {
                let conditions = if thread.start_only {
                    quote! {if i == 0}
                } else {
                    TokenStream::new()
                };
                tokens.extend(quote! {
                    VMStates::#this_state #conditions => true,
                });
            }
        }
    }

    /// Implements `ToTokens` to create the `test` function with vm-like one-pass implementation.
    pub(super) struct TestFn<'a> {
        pub(super) symbol_transitions: &'a [Vec<(std::ops::RangeInclusive<u8>, ThreadUpdates)>],
        pub(super) accept_transitions: &'a [Vec<ThreadUpdates>],
        pub(super) excluded_states: &'a [bool],
    }
    impl<'a> ToTokens for TestFn<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &TestFn {
                symbol_transitions,
                accept_transitions,
                excluded_states,
            } = self;
            let enum_states = std::iter::IntoIterator::into_iter(0..symbol_transitions.len())
                .filter(|i| !excluded_states[*i])
                .map(ImplVMStateLabel::new);

            let test_match_statements = symbol_transitions
                .iter()
                .enumerate()
                .filter(|(state_idx, _)| !excluded_states[*state_idx])
                .map(|(state_idx, symbol_transitions)| MatchStatement {
                    state_idx,
                    symbol_transitions: &symbol_transitions,
                    excluded_states: &excluded_states,
                });
            let test_match_statements_final = accept_transitions
                .iter()
                .enumerate()
                .filter(|(state_idx, _)| !excluded_states[*state_idx])
                .map(|(state_idx, accept_transitions)| MatchStatementFinal {
                    state_idx,
                    accept_transitions: &accept_transitions,
                });
            tokens.extend(quote! {
                fn test(text: &str) -> bool {
                    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
                    enum VMStates {
                        #(#enum_states,)*
                    }

                    let mut state: VMStates = VMStates::State0;

                    for (i, c) in text.bytes().enumerate() {
                        match (state, c) {
                            #(#test_match_statements)*
                            _ => return false,
                        }
                    }
                    return match state {
                        #(#test_match_statements_final)*
                        _ => false,
                    };
                }
            });
        }
    }
}
/// [`ToTokens`] implementation for the `exec` function with vm-like one-pass implementation.
mod vmlike_exec {
    use super::*;

    struct MatchStatement<'a> {
        state_idx: usize,
        symbol_transitions: &'a [(std::ops::RangeInclusive<u8>, ThreadUpdates)],
        excluded_states: &'a [bool],
    }
    impl<'a> ToTokens for MatchStatement<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &MatchStatement {
                state_idx,
                symbol_transitions,
                excluded_states,
            } = self;
            let this_state = ImplVMStateLabel::new(state_idx);
            for (range, thread) in symbol_transitions.iter() {
                if excluded_states[thread.state] {
                    continue; // no point in going here, we included the relevant propogated states already
                }
                if thread.end_only {
                    continue; // only allowed in final match
                }
                let range_start = *range.start();
                let range_end = *range.end();
                let conditions = if thread.start_only {
                    quote! {if i == 0}
                } else {
                    TokenStream::new()
                };
                let mut capture_updates = TokenStream::new();
                for (group_num, (start, end)) in thread.update_captures.iter().enumerate() {
                    if *start {
                        capture_updates.extend(quote! {
                            captures[#group_num].0 = i;
                        });
                    }
                    if *end {
                        capture_updates.extend(quote! {
                            captures[#group_num].1 = i;
                        });
                    }
                }
                let to = ImplVMStateLabel::new(thread.state);
                tokens.extend(quote! {
                    (VMStates::#this_state, #range_start..=#range_end) #conditions => {
                        #capture_updates
                        state = VMStates::#to;
                    }
                });
            }
        }
    }

    struct MatchStatementFinal<'a> {
        state_idx: usize,
        accept_transitions: &'a [ThreadUpdates],
    }
    impl<'a> ToTokens for MatchStatementFinal<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &MatchStatementFinal {
                state_idx,
                accept_transitions,
            } = self;
            let this_state = ImplVMStateLabel::new(state_idx);
            for thread in accept_transitions.iter() {
                // end only is always the case here
                let conditions = if thread.start_only {
                    quote! {if i == 0}
                } else {
                    TokenStream::new()
                };
                let mut capture_updates = TokenStream::new();
                for (group_num, (start, end)) in thread.update_captures.iter().enumerate() {
                    if *start {
                        capture_updates.extend(quote! {
                            captures[#group_num].0 = text.len();
                        });
                    }
                    if *end {
                        capture_updates.extend(quote! {
                            captures[#group_num].1 = text.len();
                        });
                    }
                }
                tokens.extend(quote! {
                    VMStates::#this_state #conditions => {
                        #capture_updates
                    }
                });
            }
        }
    }

    /// Implements `ToTokens` to create the `exec` function with vm-like one-pass implementation.
    pub(super) struct ExecFn<'a> {
        pub(super) num_captures: usize,
        pub(super) symbol_transitions: &'a [Vec<(std::ops::RangeInclusive<u8>, ThreadUpdates)>],
        pub(super) accept_transitions: &'a [Vec<ThreadUpdates>],
        pub(super) excluded_states: &'a [bool],
    }
    impl<'a> ToTokens for ExecFn<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &ExecFn {
                num_captures,
                symbol_transitions,
                accept_transitions,
                excluded_states,
            } = self;
            let enum_states = std::iter::IntoIterator::into_iter(0..symbol_transitions.len())
                .filter(|i| !excluded_states[*i])
                .map(ImplVMStateLabel::new);

            let match_statements = symbol_transitions
                .iter()
                .enumerate()
                .filter(|(state_idx, _)| !excluded_states[*state_idx])
                .map(|(state_idx, symbol_transitions)| MatchStatement {
                    state_idx,
                    symbol_transitions: &symbol_transitions,
                    excluded_states: &excluded_states,
                });
            let match_statements_final = accept_transitions
                .iter()
                .enumerate()
                .filter(|(state_idx, _)| !excluded_states[*state_idx])
                .map(|(state_idx, accept_transitions)| MatchStatementFinal {
                    state_idx,
                    accept_transitions: &accept_transitions,
                });

            tokens.extend(quote! {
            fn exec<'a>(text: &'a str) -> ::core::option::Option<[::core::option::Option<&'a str>; #num_captures]> {
                #[derive(Clone, Copy, PartialEq, Eq, Debug)]
                enum VMStates {
                    #(#enum_states,)*
                }

                let mut state: VMStates = VMStates::State0;
                let mut captures: [(usize, usize); #num_captures] = [(usize::MAX, usize::MAX); #num_captures];

                for (i, c) in text.bytes().enumerate() {
                    match (state, c) {
                        #(#match_statements)*
                        _ => return ::core::option::Option::None,
                    }
                }
                match state {
                    #(#match_statements_final)*
                    _ => return ::core::option::Option::None,
                }

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
        })
        }
    }
}

/// Will evaluate to a `const` pair `(test_fn, exec_fn)`.
fn codegen_vmlike(
    nfa: &U8NFA,
    num_captures: usize,
    symbol_transitions: Vec<Vec<(std::ops::RangeInclusive<u8>, ThreadUpdates)>>,
    accept_transitions: Vec<Vec<ThreadUpdates>>,
) -> TokenStream {
    let excluded_states = compute_excluded_states(nfa);

    let test = vmlike_test::TestFn {
        symbol_transitions: &symbol_transitions,
        accept_transitions: &accept_transitions,
        excluded_states: &excluded_states,
    };
    let exec = vmlike_exec::ExecFn {
        num_captures,
        symbol_transitions: &symbol_transitions,
        accept_transitions: &accept_transitions,
        excluded_states: &excluded_states,
    };

    return quote! {{
        #test
        #exec

        (test, exec)
    }}
    .into();
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ImplFunctionalFnIdent {
    state_idx: usize,
}
impl ImplFunctionalFnIdent {
    fn new(state_idx: usize) -> ImplFunctionalFnIdent {
        return ImplFunctionalFnIdent { state_idx };
    }
}
impl ToTokens for ImplFunctionalFnIdent {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ImplFunctionalFnIdent { state_idx } = self;
        let ident = proc_macro2::Ident::new(
            &format!("func_state_{state_idx}"),
            proc_macro2::Span::call_site(),
        );
        tokens.append(ident);
    }
}

mod functional_test {
    use super::*;

    /// Implements the transition function for a state, where the state is the start of a run.
    /// Being the start of a run, it may match many bytes.
    ///
    /// Generally called through [`StateFn`].
    pub(super) struct StateFnWithRun<'a> {
        ident: ImplFunctionalFnIdent,
        run: &'a Run,
    }
    impl<'a> ToTokens for StateFnWithRun<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &StateFnWithRun { ident, run } = self;

            // It's a run so handle special case.
            // TODO: use explicit simd instructions. Currently we just hope LLVM vectorizes.
            let run_length = run.symbols.len();

            let new_state_fn_ident = ImplFunctionalFnIdent::new(run.end_state);

            // TODO: make sure the non-branching path isn't *too* long
            let conditions = run.symbols.iter().enumerate().map(|(i, range)| {
                let lower = range.start();
                let upper = range.end();
                return quote! {
                    (#lower <= run_part[#i]) & (run_part[#i] <= #upper)
                };
            });

            tokens.extend( quote! {
                fn #ident<'a>(mut bytes: ::core::slice::Iter<'a, u8>, start: bool) -> bool {
                    let ::core::option::Option::Some((run_part, rest)) = bytes.as_slice().split_at_checked(#run_length) else {
                        return false;
                    };
                    let result = #(#conditions)&*;

                    return result && #new_state_fn_ident(rest.iter(), false);
                }
            });
        }
    }

    /// Implements the transition function for a state, without a run.
    ///
    /// Generally called through [`StateFn`].
    pub(super) struct StateFnNormal<'a> {
        ident: ImplFunctionalFnIdent,
        accept_transitions: &'a [ThreadUpdates],
        symbol_transitions: &'a [(std::ops::RangeInclusive<u8>, ThreadUpdates)],
    }
    impl<'a> ToTokens for StateFnNormal<'a> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let &StateFnNormal {
                ident,
                accept_transitions,
                symbol_transitions,
            } = self;

            let end_case = if accept_transitions.is_empty() {
                // state is not accepting
                quote! { false }
            } else if accept_transitions.iter().any(|tu| !tu.start_only) {
                // state is accepting, doesn't care if we're at the start.
                quote! { true }
            } else if ident.state_idx == 0 {
                // rare case: needs to be at the start + end to accept
                quote! { start }
            } else {
                // rare case: if we are not at state 0, we cannot be at the start.
                quote! { false }
            };

            let cases = symbol_transitions
            .into_iter()
            .map(|(range, tu)| {
                let start = *range.start();
                let end = *range.end();
                let conditions = if tu.end_only || (tu.start_only && ident.state_idx != 0) {
                    // end_only should already be optimized out, but we are not at the end so this transition should never happen.
                    // since we always start at state 0, start_only is only satisfied when `i == 0`
                    quote! { if false }
                } else if tu.start_only && ident.state_idx == 0 {
                    // if we ever come back to state 0, we may actually need to check start
                    quote! { if start }
                } else {
                    TokenStream::new()
                };

                let new_state_fn_ident = ImplFunctionalFnIdent::new(tu.state);

                return quote! {
                    ::core::option::Option::Some(#start..=#end) #conditions => #new_state_fn_ident(bytes, false),
                };
            });

            tokens.extend(quote! {
                fn #ident<'a>(mut bytes: ::core::slice::Iter<'a, u8>, start: bool) -> bool {
                    return match bytes.next() {
                        ::core::option::Option::None => #end_case,
                        #(#cases)*
                        ::core::option::Option::Some(_) => false,
                    }
                }
            });
        }
    }

    /// Implements the transition function for a state.
    /// Will either use the [`StateFnWithRun`] or [`StateFnNormal`] depending on whether the state has a run
    /// or should just match the next byte.
    ///
    /// ```ignore
    /// fn #ident<'a>(mut bytes: ::core::slice::Iter<'a, u8>, start: bool) -> bool {
    ///     // ...
    /// }
    /// ```
    pub(super) enum StateFn<'a> {
        WithRun(StateFnWithRun<'a>),
        Normal(StateFnNormal<'a>),
    }
    impl<'a> StateFn<'a> {
        pub(super) fn new_run(ident: usize, run: &'a Run) -> Self {
            return Self::WithRun(StateFnWithRun {
                ident: ImplFunctionalFnIdent::new(ident),
                run,
            });
        }
        pub(super) fn new_normal(
            ident: usize,
            accept_transitions: &'a [ThreadUpdates],
            symbol_transitions: &'a [(std::ops::RangeInclusive<u8>, ThreadUpdates)],
        ) -> Self {
            return Self::Normal(StateFnNormal {
                ident: ImplFunctionalFnIdent::new(ident),
                accept_transitions,
                symbol_transitions,
            });
        }
    }
    impl<'a> ToTokens for StateFn<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                StateFn::WithRun(fn_with_run) => {
                    fn_with_run.to_tokens(tokens);
                }
                StateFn::Normal(fn_normal) => {
                    fn_normal.to_tokens(tokens);
                }
            }
        }
    }

    // struct ImplFunctionalTestFn<'a> {
    //     ident: ImplFunctionalFnIdent,
    //     symbol_transitions: &'a [Vec<(std::ops::RangeInclusive<u8>, ThreadUpdates)>],
    //     accept_transitions: &'a [Vec<ThreadUpdates>],
    // }
}

mod functional_exec {
    use super::*;

    /// Implements the transition function for a state, where the state is the start of a run.
    /// Being the start of a run, it may match many bytes.
    ///
    /// Generally called through [`StateFn`].
    pub(super) struct StateFnWithRun<'a> {
        num_captures: usize,
        ident: ImplFunctionalFnIdent,
        run: &'a Run,
    }
    impl<'a> ToTokens for StateFnWithRun<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &StateFnWithRun {
                num_captures,
                ident,
                run,
            } = self;
            // TODO: use explicit simd instructions. Currently we just hope LLVM vectorizes.
            let run_length = run.symbols.len();

            let new_state_fn_ident = ImplFunctionalFnIdent::new(run.end_state);

            // TODO: make sure the non-branching path isn't *too* long
            let conditions = run.symbols.iter().enumerate().map(|(i, range)| {
                let lower = range.start();
                let upper = range.end();
                return quote! {
                    (#lower <= run_part[#i]) & (run_part[#i] <= #upper)
                };
            });

            let apply_tags = run.tags.iter().map(|(offset, tag)| match tag {
                Tag::StartCapture(capture_idx) => {
                    quote! { captures[#capture_idx].0 = #offset + byte_idx; }
                }
                Tag::EndCapture(capture_idx) => {
                    quote! { captures[#capture_idx].1 = #offset + byte_idx; }
                }
            });

            tokens.extend(quote! {
            fn #ident<'a>(
                mut bytes: ::core::slice::Iter<'a, u8>,
                byte_idx: usize,
                mut captures: [(usize, usize); #num_captures],
                start: bool,
            ) -> Option<[(usize, usize); #num_captures]> {
                let ::core::option::Option::Some((run_part, rest)) = bytes.as_slice().split_at_checked(#run_length) else {
                    return ::core::option::Option::None;
                };
                let result = #(#conditions)&*;
                if !result {
                    return ::core::option::Option::None;
                }

                #(#apply_tags)*

                return #new_state_fn_ident(rest.into_iter(), byte_idx + #run_length, captures, false);
            }
        });
        }
    }

    /// Implements the transition function for a state, without a run.
    ///
    /// Generally called through [`StateFn`].
    pub(super) struct StateFnNormal<'a> {
        num_captures: usize,
        ident: ImplFunctionalFnIdent,
        accept_transitions: &'a [ThreadUpdates],
        symbol_transitions: &'a [(std::ops::RangeInclusive<u8>, ThreadUpdates)],
    }
    impl<'a> ToTokens for StateFnNormal<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &StateFnNormal {
                num_captures,
                ident,
                accept_transitions,
                symbol_transitions,
            } = self;

            /// Should have text position `i` and `mut captures` in local context
            fn make_capture_statements(tu: &ThreadUpdates) -> TokenStream {
                fn map_capture(
                    (capture_group, (start, end)): (usize, &(bool, bool)),
                ) -> TokenStream {
                    let mut out = TokenStream::new();
                    if *start {
                        out.extend(quote! {
                            captures[#capture_group].0 = byte_idx;
                        });
                    }
                    if *end {
                        out.extend(quote! {
                            captures[#capture_group].1 = byte_idx;
                        });
                    }
                    return out;
                }
                return tu
                    .update_captures
                    .iter()
                    .enumerate()
                    .map(map_capture)
                    .collect();
            }

            // for end_case, `end` check is implicit
            let end_case = match &accept_transitions {
                &[] => quote! { ::core::option::Option::None },
                &[tu] if tu.start_only && ident.state_idx == 0 => {
                    // Accept if `start`
                    let capture_statements = make_capture_statements(tu);
                    quote! {
                        #capture_statements
                        if start {
                            ::core::option::Option::Some(captures)
                        } else {
                            ::core::option::Option::None
                        }
                    }
                }
                &[tu] if tu.start_only && ident.state_idx != 0 => {
                    quote! { ::core::option::Option::None }
                }
                &[tu] => {
                    // is always accepting
                    let capture_statements = make_capture_statements(tu);
                    quote! {
                        #capture_statements
                        ::core::option::Option::Some(captures)
                    }
                }
                _more => quote! {
                    compile_error!("Should only be one thread update on accept for one-pass");
                },
            };

            let cases = symbol_transitions.into_iter().map(|(range, tu)| {
                let start = *range.start();
                let end = *range.end();
                let conditions = if tu.end_only || (tu.start_only && ident.state_idx != 0) {
                    // end_only should already be optimized out, but we are not at the end so this transition should never happen.
                    // since we always start at state 0, start_only is only satisfied when `i == 0`
                    quote! { if false }
                } else if tu.start_only && ident.state_idx == 0 {
                    // if we ever come back to state 0, we may actually need to check start
                    quote! { if start }
                } else {
                    TokenStream::new()
                };

                let new_state_fn_ident = ImplFunctionalFnIdent::new(tu.state);
                let capture_statements = make_capture_statements(tu);

                return quote! {
                    ::core::option::Option::Some(#start..=#end) #conditions => {
                        #capture_statements
                        #new_state_fn_ident(bytes, byte_idx + 1, captures, false)
                    }
                };
            });

            tokens.extend(quote! {
                fn #ident<'a>(
                    mut bytes: ::core::slice::Iter<'a, u8>,
                    byte_idx: usize,
                    mut captures: [(usize, usize); #num_captures],
                    start: bool,
                ) -> ::core::option::Option<[(usize, usize); #num_captures]> {
                    return match bytes.next() {
                        ::core::option::Option::None => {
                            #end_case
                        }
                        #(#cases)*
                        ::core::option::Option::Some(_) => ::core::option::Option::None,
                    };
                }
            });
        }
    }

    /// Implements the transition function for a state.
    /// Will either use the [`StateFnWithRun`] or [`StateFnNormal`] depending on whether the state has a run
    /// or should just match the next byte.
    ///
    /// ```ignore
    /// fn #ident<'a>(
    ///     mut bytes: ::core::slice::Iter<'a, u8>,
    ///     byte_idx: usize,
    ///     mut captures: [(usize, usize); #num_captures],
    ///     start: bool,
    /// ) -> ::core::option::Option<[(usize, usize); #num_captures]> {
    ///     // ...
    /// }
    /// ```
    pub(super) enum StateFn<'a> {
        WithRun(StateFnWithRun<'a>),
        Normal(StateFnNormal<'a>),
    }
    impl<'a> StateFn<'a> {
        pub(super) fn new_run(num_captures: usize, ident: usize, run: &'a Run) -> Self {
            return Self::WithRun(StateFnWithRun {
                num_captures,
                ident: ImplFunctionalFnIdent::new(ident),
                run,
            });
        }
        pub(super) fn new_normal(
            num_captures: usize,
            ident: usize,
            accept_transitions: &'a [ThreadUpdates],
            symbol_transitions: &'a [(std::ops::RangeInclusive<u8>, ThreadUpdates)],
        ) -> Self {
            return Self::Normal(StateFnNormal {
                num_captures,
                ident: ImplFunctionalFnIdent::new(ident),
                accept_transitions,
                symbol_transitions,
            });
        }
    }
    impl<'a> ToTokens for StateFn<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                StateFn::WithRun(fn_with_run) => {
                    fn_with_run.to_tokens(tokens);
                }
                StateFn::Normal(fn_normal) => {
                    fn_normal.to_tokens(tokens);
                }
            }
        }
    }
}

/// Will evaluate to a `const` pair `(test_fn, exec_fn)`.
///
/// ## Params
/// - `nfa` is the original nfa
/// - `num_captures` is the calculated number of capture groups from `nfa`
/// - `symbol_transitions` is the effective transitions (including other [`ThreadUpdates`] details) for each state.
fn codegen_functional(
    nfa: &U8NFA,
    num_captures: usize,
    symbol_transitions: Vec<Vec<(std::ops::RangeInclusive<u8>, ThreadUpdates)>>,
    accept_transitions: Vec<Vec<ThreadUpdates>>,
) -> TokenStream {
    let U8NFA { states, .. } = nfa;
    assert_eq!(states.len(), symbol_transitions.len());
    assert_eq!(states.len(), accept_transitions.len());

    let excluded_states = compute_excluded_states(nfa);
    assert_eq!(states.len(), excluded_states.len());

    let runs = compute_runs(&symbol_transitions);
    assert_eq!(states.len(), runs.len());

    let test_funcs = runs
        .iter()
        .enumerate()
        .filter(|(i, _)| !excluded_states[*i])
        .filter_map(|(i, run)| match run {
            StateRunInclusion::Start(run) => Some(functional_test::StateFn::new_run(i, run)),
            StateRunInclusion::Internal => None,
            StateRunInclusion::End | StateRunInclusion::None => {
                Some(functional_test::StateFn::new_normal(
                    i,
                    &accept_transitions[i],
                    &symbol_transitions[i],
                ))
            }
        });
    let impl_test_fn = quote! {
        fn test<'a>(text: &'a str) -> bool {
            #(#test_funcs)*
            return func_state_0(text.as_bytes().iter(), true);
        }
    };

    let exec_funcs = runs
        .iter()
        .enumerate()
        .filter(|(i, _)| !excluded_states[*i])
        .filter_map(|(i, run)| match run {
            StateRunInclusion::Start(run) => {
                Some(functional_exec::StateFn::new_run(num_captures, i, run))
            }
            StateRunInclusion::Internal => None,
            StateRunInclusion::End | StateRunInclusion::None => {
                Some(functional_exec::StateFn::new_normal(
                    num_captures,
                    i,
                    &accept_transitions[i],
                    &symbol_transitions[i],
                ))
            }
        });
    let impl_exec_fn = quote! {
        fn exec<'a>(text: &'a str) -> ::core::option::Option<[::core::option::Option<&'a str>; #num_captures]> {
            let captures: [(usize, usize); #num_captures] = [(usize::MAX, usize::MAX); #num_captures];

            #(#exec_funcs)*
            let captures = func_state_0(text.as_bytes().iter(), 0, captures, true)?;

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
    };

    return quote! {{
        #impl_test_fn
        #impl_exec_fn

        (test, exec)
    }}
    .into();
}
