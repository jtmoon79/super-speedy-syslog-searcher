//! Implements an nfa-like regex engine for over `char`s.
//! The engine keeps all threads in lockstep (all threads are at the same input index),
//! and the NFA's epsilon transitions are flattened to a single epsilon transition between symbols
//! (including handling anchors and capture tags).
//!
//! Currently we flatten all epsilon transitions for the VM so that epsilon transitions are at most a single step between symbols.
//! I'll have to review to ensure we avoid this causing large binary size overhead,
//! but it should be worst-case `O(n^2)` in the number of states, and far fewer on average.

use crate::{
    epsilon_propogation::{EpsilonPropogation, Tag},
    nfa_static,
    working_nfa::{WorkingNFA, WorkingTransition},
};
use quote::{quote, ToTokens, TokenStreamExt};
use std::fmt::Write;

#[derive(Clone)]
pub struct Thread<const N: usize, S: Send + Sync + Copy + Eq> {
    pub state: S,
    pub captures: [(usize, usize); N],
}
impl<const N: usize, S: Send + Sync + Copy + Eq + std::fmt::Debug> std::fmt::Debug
    for Thread<N, S>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct CapturesDebug<'a, const N: usize>(&'a [(usize, usize); N]);
        impl<'a, const N: usize> std::fmt::Debug for CapturesDebug<'a, N> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_char('[')?;
                for (i, endpoints) in self.0.iter().enumerate() {
                    if i != 0 {
                        f.write_str(", ")?;
                    }
                    match endpoints {
                        (usize::MAX, usize::MAX) => f.write_str("(_, _)")?,
                        (start, usize::MAX) => write!(f, "({start}, _)")?,
                        (usize::MAX, end) => write!(f, "(_, {end})")?,
                        (start, end) => write!(f, "({start}, {end})")?,
                    }
                }
                return f.write_char(']');
            }
        }
        return f
            .debug_struct("Thread")
            .field("state", &self.state)
            .field("captures", &CapturesDebug(&self.captures))
            .finish();
    }
}

/// The NFA and some precomputed data to go with it.
struct CachedNFA<'a> {
    nfa: &'a WorkingNFA,
    excluded_states: Vec<bool>,
    capture_groups: usize,
}
impl<'a> CachedNFA<'a> {
    fn new(nfa: &'a WorkingNFA) -> CachedNFA<'a> {
        let excluded_states = compute_excluded_states(nfa);
        assert_eq!(nfa.states.len(), excluded_states.len());
        let capture_groups = nfa.num_capture_groups();
        return CachedNFA {
            nfa,
            excluded_states,
            capture_groups,
        };
    }
}

/// Since we are shortcutting the epsilon transitions, we can skip printing
/// states that have only epsilon transitions and are not the start/end states
fn compute_excluded_states(nfa: &WorkingNFA) -> Vec<bool> {
    let mut out = vec![true; nfa.states.len()];
    out[0] = false;
    out[nfa.states.len() - 1] = false;
    for (from, state) in nfa.states.iter().enumerate() {
        for t in &state.transitions {
            out[from] = false;
            out[t.to] = false;
        }
    }

    return out;
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ImplVMStateLabel(usize);
impl ToTokens for ImplVMStateLabel {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ImplVMStateLabel(idx) = self;
        let label = format!("State{idx}");
        tokens.append(proc_macro2::Ident::new(
            &label,
            proc_macro2::Span::call_site(),
        ));
    }
}

mod impl_test {
    use quote::ToTokens;

    use super::*;

    /// Implements symbol transitions for a single state
    struct ImplTransitionStateSymbol<'a> {
        transition: &'a WorkingTransition,
    }
    impl<'a> ToTokens for ImplTransitionStateSymbol<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let ImplTransitionStateSymbol { transition } = self;
            let WorkingTransition { symbol, to } = transition;
            let symbol = nfa_static::AtomStatic::serialize_as_token_stream(symbol);
            tokens.extend(quote! {{
                let symbol = #symbol;
                if symbol.check(c) {
                    new_list[#to] = true;
                }
            }});
        }
    }

    /// Assumes the `VMStates` enum is already created locally in the token stream
    ///
    /// Creates the function `transition_symbols_test` for running symbol transitions on the engine
    pub(super) struct TransitionSymbols<'a>(pub &'a CachedNFA<'a>);
    impl<'a> ToTokens for TransitionSymbols<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let TransitionSymbols(nfa) = self;
            let CachedNFA {
                nfa,
                excluded_states,
                ..
            } = nfa;

            let transition_symbols_defs_test = nfa
                .states
                .iter()
                .enumerate()
                .filter(|(i, _)| !excluded_states[*i])
                .map(|(i, state)| {
                    let state_transitions = state
                        .transitions
                        .iter()
                        .map(|t| ImplTransitionStateSymbol { transition: t });

                    return quote! {
                        if list[#i] {
                            #(#state_transitions)*
                        }
                    };
                });

            tokens.extend(quote! {
                fn transition_symbols_test(
                    list: &[bool],
                    new_list: &mut [bool],
                    c: char,
                ) {
                    #(#transition_symbols_defs_test)*
                }
            });
        }
    }

    /// Implements epsilon transitions for a single state
    ///
    /// Becomes:
    /// ```ignore
    /// if list[#from_state] {
    ///     // ...
    /// }
    /// ```
    pub(super) struct ImplTransitionStateEpsilon<'a> {
        pub(super) from_state: ImplVMStateLabel,
        pub(super) thread_updates: &'a [ThreadUpdates],
    }
    impl<'a> ToTokens for ImplTransitionStateEpsilon<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &ImplTransitionStateEpsilon {
                from_state,
                thread_updates,
            } = self;
            let ImplVMStateLabel(from_state) = from_state;

            // Write epsilon-propogation of threads to the token stream for test
            let start_end_threads = thread_updates
                .iter()
                .filter(|t| t.0.start_only && t.0.end_only)
                .map(ThreadUpdates::serialize_thread_update_test);
            let start_threads = thread_updates
                .iter()
                .filter(|t| t.0.start_only && !t.0.end_only)
                .map(ThreadUpdates::serialize_thread_update_test);
            let end_threads = thread_updates
                .iter()
                .filter(|t| !t.0.start_only && t.0.end_only)
                .map(ThreadUpdates::serialize_thread_update_test);
            let normal_threads = thread_updates
                .iter()
                .filter(|t| !t.0.start_only && !t.0.end_only)
                .map(ThreadUpdates::serialize_thread_update_test);

            tokens.extend(quote! {
                if list[#from_state] {
                    if is_start && is_end {
                        #(#start_end_threads)*
                    }
                    if is_start {
                        #(#start_threads)*
                    }
                    if is_end {
                        #(#end_threads)*
                    }
                    #(#normal_threads)*
                }
            });
        }
    }

    /// Implements a function that runs all epsilon transitions for all threads.
    ///
    /// ```ignore
    /// fn transition_epsilons_test(
    ///     list: &mut [bool],
    ///     idx: usize,
    ///     len: usize,
    /// ) {
    ///     // ...
    /// }
    /// ```
    pub(super) struct TransitionEpsilons<'a>(pub &'a CachedNFA<'a>);
    impl<'a> ToTokens for TransitionEpsilons<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let TransitionEpsilons(nfa) = self;
            let CachedNFA {
                nfa,
                excluded_states,
                ..
            } = nfa;
            assert_eq!(nfa.states.len(), excluded_states.len());
            let num_states = nfa.states.len();

            let states_epsilon_transitions = std::iter::zip(nfa.states.iter(), excluded_states)
                .enumerate()
                .filter(|(_, (_, &excluded))| !excluded)
                .map(|(i, _)| {
                    // all reachable states with next transition as epsilon
                    let mut new_threads = calculate_epsilon_propogations(nfa, i);
                    new_threads.retain(|t| {
                        !nfa.states[t.0.state].transitions.is_empty() || t.0.state + 1 == num_states
                    });

                    let label: ImplVMStateLabel = ImplVMStateLabel(i);

                    let state_epsilon_transitions_test = impl_test::ImplTransitionStateEpsilon {
                        from_state: label,
                        thread_updates: &new_threads,
                    };
                    state_epsilon_transitions_test.to_token_stream()
                });

            tokens.extend(quote! {
                fn transition_epsilons_test(
                    list: &mut [bool],
                    idx: usize,
                    len: usize,
                ) {
                    let is_start = idx == 0;
                    let is_end = idx == len;
                    #(#states_epsilon_transitions)*
                }
            });
        }
    }

    pub(crate) struct TestFn<'a>(pub &'a CachedNFA<'a>);
    impl<'a> ToTokens for TestFn<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let TestFn(nfa) = self;
            let CachedNFA {
                nfa: u8_nfa,
                excluded_states,
                ..
            } = nfa;

            let enum_states = excluded_states
                .iter()
                .enumerate()
                .filter_map(|(i, excluded)| match excluded {
                    true => None,
                    false => Some(ImplVMStateLabel(i)),
                });
            let state_count = u8_nfa.states.len();
            let accept_state = ImplVMStateLabel(state_count - 1);

            let transition_symbols_test = TransitionSymbols(nfa);
            let transition_epsilons_test = TransitionEpsilons(nfa);

            tokens.extend(quote! {
                fn test(text: &str) -> bool {
                    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
                    enum VMStates {
                        #(#enum_states,)*
                    }

                    #transition_symbols_test
                    #transition_epsilons_test

                    let mut list = [false; #state_count];
                    let mut new_list = [false; #state_count];
                    list[0] = true;

                    transition_epsilons_test(&mut list, 0, text.len());
                    for (i, c) in text.char_indices() {
                        transition_symbols_test(&list, &mut new_list, c);
                        if new_list.iter().all(|b| !b) {
                            return false;
                        }
                        ::std::mem::swap(&mut list, &mut new_list);
                        transition_epsilons_test(&mut list, i + c.len_utf8(), text.len());
                        new_list.fill(false);
                    }

                    return list[#state_count - 1];
                }
            });
        }
    }
}

mod impl_exec {
    use quote::ToTokens;

    use super::*;

    struct ImplTransition<'a> {
        transition: &'a WorkingTransition,
    }
    impl<'a> ToTokens for ImplTransition<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let ImplTransition { transition } = self;
            let WorkingTransition { symbol, to } = transition;
            let symbol = nfa_static::AtomStatic::serialize_as_token_stream(symbol);
            let to_label = ImplVMStateLabel(*to);
            tokens.extend(quote! {{
                let symbol = #symbol;
                if symbol.check(c) {
                    new_threads.push(
                        ::ere::flat_lockstep_nfa::Thread {
                            state: VMStates::#to_label,
                            captures: thread.captures.clone(),
                        },
                    );
                }
            }});
        }
    }

    /// Assumes the `VMStates` enum is already created locally in the token stream
    ///
    /// Creates the function `transition_symbols_exec` for running symbol transitions on the flat lockstep NFA
    /// - expects `new_threads` to be empty
    ///
    /// ```ignore
    /// fn transition_symbols_exec(
    ///     threads: &[::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>],
    ///     new_threads: &mut ::std::vec::Vec<::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>>,
    ///     c: char,
    /// ) {
    ///     // ...
    /// }
    /// ```
    pub(super) struct TransitionSymbols<'a>(pub &'a CachedNFA<'a>);
    impl<'a> ToTokens for TransitionSymbols<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let TransitionSymbols(nfa) = self;
            let CachedNFA {
                nfa,
                capture_groups,
                excluded_states,
            } = nfa;

            let transition_symbols_defs_exec = nfa
                .states
                .iter()
                .enumerate()
                .filter(|(i, _)| !excluded_states[*i])
                .map(|(i, state)| {
                    let label = ImplVMStateLabel(i);
                    let state_transitions = state
                        .transitions
                        .iter()
                        .map(|t| ImplTransition { transition: t });

                    return quote! {
                        VMStates::#label => {
                            #(#state_transitions)*
                        }
                    };
                });

            tokens.extend(quote! {
                fn transition_symbols_exec(
                    threads: &[::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>],
                    new_threads: &mut ::std::vec::Vec<::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>>,
                    c: char,
                ) {
                    for thread in threads {
                        match thread.state {
                            #(#transition_symbols_defs_exec)*
                        }
                    }
                }
            });
        }
    }

    /// Implements epsilon transitions for a single state
    ///
    /// Becomes:
    /// ```ignore
    /// VMStates::#from_state => {
    ///     // ...
    /// }
    /// ```
    pub(super) struct ImplTransitionStateEpsilon<'a> {
        pub(super) from_state: ImplVMStateLabel,
        pub(super) thread_updates: &'a [ThreadUpdates],
    }
    impl<'a> ToTokens for ImplTransitionStateEpsilon<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let &ImplTransitionStateEpsilon {
                from_state,
                thread_updates,
            } = self;

            // Write epsilon-propogation of threads to the token stream for exec
            let start_end_threads = thread_updates
                .iter()
                .filter(|t| t.0.start_only && t.0.end_only)
                .map(ThreadUpdates::serialize_thread_update_exec);
            let start_threads = thread_updates
                .iter()
                .filter(|t| t.0.start_only && !t.0.end_only)
                .map(ThreadUpdates::serialize_thread_update_exec);
            let end_threads = thread_updates
                .iter()
                .filter(|t| !t.0.start_only && t.0.end_only)
                .map(ThreadUpdates::serialize_thread_update_exec);
            let normal_threads = thread_updates
                .iter()
                .filter(|t| !t.0.start_only && !t.0.end_only)
                .map(ThreadUpdates::serialize_thread_update_exec);

            tokens.extend(quote! {
                VMStates::#from_state => {
                    if is_start && is_end {
                        #(#start_end_threads)*
                    }
                    if is_start {
                        #(#start_threads)*
                    }
                    if is_end {
                        #(#end_threads)*
                    }
                    #(#normal_threads)*
                }
            });
        }
    }

    /// Implements a function that runs all epsilon transitions for all threads.
    ///
    /// ```ignore
    /// fn transition_epsilons_exec(
    ///     threads: &[::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>],
    ///     new_threads: &mut ::std::vec::Vec<::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>>,
    ///     idx: usize,
    ///     len: usize,
    /// ) {
    ///     // ...
    /// }
    /// ```
    pub(super) struct TransitionEpsilons<'a>(pub &'a CachedNFA<'a>);
    impl<'a> ToTokens for TransitionEpsilons<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let TransitionEpsilons(nfa) = self;
            let CachedNFA {
                nfa,
                capture_groups,
                excluded_states,
            } = nfa;
            assert_eq!(nfa.states.len(), excluded_states.len());
            let num_states = nfa.states.len();

            let states_epsilon_transitions = std::iter::zip(nfa.states.iter(), excluded_states)
                .enumerate()
                .filter(|(_, (_, &excluded))| !excluded)
                .map(|(i, _)| {
                    // all reachable states with next transition as epsilon
                    let mut new_threads = calculate_epsilon_propogations(nfa, i);
                    new_threads.retain(|t| {
                        !nfa.states[t.0.state].transitions.is_empty() || t.0.state + 1 == num_states
                    });

                    let label: ImplVMStateLabel = ImplVMStateLabel(i);

                    let state_epsilon_transitions_exec = impl_exec::ImplTransitionStateEpsilon {
                        from_state: label,
                        thread_updates: &new_threads,
                    };
                    state_epsilon_transitions_exec.to_token_stream()
                });

            tokens.extend(quote! {
                fn transition_epsilons_exec(
                    threads: &[::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>],
                    new_threads: &mut ::std::vec::Vec<::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>>,
                    idx: usize,
                    len: usize,
                ) {
                    let is_start = idx == 0;
                    let is_end = idx == len;
                    let mut occupied_states = ::std::vec![false; #num_states];
                    for thread in threads {
                        match thread.state {
                            #(#states_epsilon_transitions)*
                        }
                    }
                }
            });
        }
    }

    pub(crate) struct ExecFn<'a>(pub &'a CachedNFA<'a>);
    impl<'a> ToTokens for ExecFn<'a> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let ExecFn(nfa) = self;
            let CachedNFA {
                nfa: u8_nfa,
                excluded_states,
                capture_groups,
            } = nfa;

            let enum_states = excluded_states
                .iter()
                .enumerate()
                .filter_map(|(i, excluded)| match excluded {
                    true => None,
                    false => Some(ImplVMStateLabel(i)),
                });
            let state_count = u8_nfa.states.len();
            let accept_state = ImplVMStateLabel(state_count - 1);

            let transition_symbols_exec = impl_exec::TransitionSymbols(&nfa);
            let transition_epsilons_exec = impl_exec::TransitionEpsilons(&nfa);

            tokens.extend(quote! {
                fn exec<'a>(text: &'a str) -> Option<[Option<&'a str>; #capture_groups]> {
                    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
                    enum VMStates {
                        #(#enum_states,)*
                    }

                    #transition_symbols_exec
                    #transition_epsilons_exec

                    let mut threads = ::std::vec::Vec::<::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>>::new();
                    let mut new_threads = ::std::vec::Vec::<::ere::flat_lockstep_nfa::Thread<#capture_groups, VMStates>>::new();
                    threads.push(::ere::flat_lockstep_nfa::Thread {
                        state: VMStates::State0,
                        captures: [(usize::MAX, usize::MAX); #capture_groups],
                    });

                    transition_epsilons_exec(&threads, &mut new_threads, 0, text.len());
                    ::std::mem::swap(&mut threads, &mut new_threads);

                    for (i, c) in text.char_indices() {
                        new_threads.clear();
                        transition_symbols_exec(&threads, &mut new_threads, c);
                        ::std::mem::swap(&mut threads, &mut new_threads);

                        new_threads.clear();
                        transition_epsilons_exec(&threads, &mut new_threads, i + c.len_utf8(), text.len());
                        ::std::mem::swap(&mut threads, &mut new_threads);

                        if threads.is_empty() {
                            return None;
                        }
                    }

                    let final_capture_bounds = threads
                        .into_iter()
                        .find(|t| t.state == VMStates::#accept_state)?
                        .captures;
                    let mut captures = [::core::option::Option::None; #capture_groups];
                    for (i, (start, end)) in final_capture_bounds.into_iter().enumerate() {
                        if start != usize::MAX {
                            assert_ne!(end, usize::MAX);
                            // assert!(start <= end);
                            captures[i] = text.get(start..end);
                            assert!(captures[i].is_some());
                        } else {
                            assert_eq!(end, usize::MAX);
                        }
                    }
                    return Some(captures);
                }
            });
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct ThreadUpdates(EpsilonPropogation);
impl ThreadUpdates {
    /// Creates a block which takes `list: &mut [bool; STATE_NUM]` from its local context, updates it in-place using `self` (compile-time).
    pub fn serialize_thread_update_test(&self) -> proc_macro2::TokenStream {
        let new_state = self.0.state;
        return quote! {{
            list[#new_state] = true;
        }};
    }
    /// Creates a block which takes `thread` from its local context, updates it using `self` (compile-time),
    /// and appends it to `new_threads` from its local context.
    pub fn serialize_thread_update_exec(&self) -> proc_macro2::TokenStream {
        let new_state_idx = self.0.state;
        let new_state = ImplVMStateLabel(self.0.state);
        let capture_updates = self.0.update_tags.iter().map(|tag| match tag {
            Tag::StartCapture(capture_group) => quote! {
                new_thread.captures[#capture_group].0 = idx;
            },
            Tag::EndCapture(capture_group) => quote! {
                new_thread.captures[#capture_group].1 = idx;
            },
        });

        return quote! {
            if !occupied_states[#new_state_idx] {
                let mut new_thread = thread.clone();
                new_thread.state = VMStates::#new_state;

                #(#capture_updates)*

                new_threads.push(new_thread);
                occupied_states[#new_state_idx] = true;
            }
        };
    }
}

fn calculate_epsilon_propogations(nfa: &WorkingNFA, state: usize) -> Vec<ThreadUpdates> {
    let prop = EpsilonPropogation::calculate_epsilon_propogations_char(nfa, state);
    return prop.into_iter().map(ThreadUpdates).collect();
}

/// Converts a [`WorkingNFA`] into a format that, when returned by a proc macro, will
/// create the corresponding engine.
///
/// Will evaluate to a `const` pair `(test_fn, exec_fn)`.
pub(crate) fn serialize_flat_lockstep_nfa_token_stream(
    nfa: &WorkingNFA,
) -> proc_macro2::TokenStream {
    let nfa = CachedNFA::new(nfa);

    let test_fn = impl_test::TestFn(&nfa);
    let exec_fn = impl_exec::ExecFn(&nfa);

    return quote! {{
        #test_fn
        #exec_fn

        (test, exec)
    }};
}
