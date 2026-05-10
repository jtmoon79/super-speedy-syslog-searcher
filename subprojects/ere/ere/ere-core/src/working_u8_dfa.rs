//! Working datastructure for a tagged DFA over `u8`s.
//! Primarily intended for use at compile time, converted from [`crate::U8NFA`].
//!
//! For more information, read https://en.wikipedia.org/wiki/Tagged_Deterministic_Finite_Automaton
//!
//! Additional references:
//! - [NFAs with Tagged Transitions, their Conversion to Deterministic Automata and Application to Regular Expressions](https://laurikari.net/ville/spire2000-tnfa.pdf) by Ville Laurikari, 2000
//! - [Tagged Deterministic Finite Automata with Lookahead](https://arxiv.org/pdf/1907.08837) by Ulya Trofimovich, 2019

use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use crate::{
    epsilon_propogation::{EpsilonPropogation, Tag},
    working_u8_nfa::U8NFA,
};

/// Represents the index of a NFA state from the original U8NFA used to produce a (T)DFA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubNFAStateID(pub usize);

#[derive(Debug)]
pub struct U8TDFATransition {
    pub to: usize,
    pub symbol: RangeInclusive<u8>,
    /// For each of the NFA states the new TDFA state represents, the index of one of the previous TDFA state's NFA states.
    ///
    /// The indexed NFA state's tags will be copied to the new NFA state's tags, before any updates are applied.
    ///
    /// basically, if a TDFA state representing `N` NFA states has `old_tags: [NFAStateTags; N]` then
    /// the new TDFA state representing `copy_tags.len()` NFA states will have
    /// `new_tags: copy_tags.map(|i| old_tags[i])`.
    /// This means index is by local NFA state index, not global NFA state index.
    pub copy_tags: Vec<usize>,
    /// After tags are copied, these tags will be updated.
    ///
    /// Using local NFA state indices on the new TDFA state.
    pub add_tags: Vec<(usize, Tag)>,
}

/// Final epsilon-like transition when at end, allows end anchors within it
pub struct U8TDFAAcceptTransition {
    /// Local NFA state index that we move to accept from (and to copy tags from)
    pub nfa_state: usize,
    pub add_tags: Vec<Tag>,
}
impl U8TDFAAcceptTransition {
    pub fn from_epsilon_prop(local_from_idx: usize, epsilon_prop: &EpsilonPropogation) -> Self {
        return Self {
            nfa_state: local_from_idx,
            add_tags: epsilon_prop.update_tags.clone(),
        };
    }
    /// return 0 if there are no capture groups
    pub fn max_capture_group(&self) -> usize {
        return self
            .add_tags
            .iter()
            .map(Tag::capture_group)
            .max()
            .unwrap_or(0);
    }
}

pub enum U8TDFAAccept {
    /// If there are only end-anchored accept(s), this is the highest priority one.
    Anchored(U8TDFAAcceptTransition),
    /// If there are both end-anchored and non-end-anchored accept(s),
    /// where the highest anchored one is higher priority than the highest non-anchored one:
    ///
    /// Is a pair `(anchored, non_anchored)`
    Both(U8TDFAAcceptTransition, U8TDFAAcceptTransition),
    /// If there is a non-end-anchored accept(s), with no higher priority anchored accept(s).
    Unanchored(U8TDFAAcceptTransition),
    /// If there are no accept(s).
    None,
}
impl U8TDFAAccept {
    /// ## Params
    /// - `local_from_idx` is the local NFA state index that we move to accept *from* (and to copy tags from)
    /// - `epsilon_prop` is the epsilon propogations of the NFA state
    /// - `accept_state_idx` is the index of the accept state in the NFA
    pub fn from_epsilon_prop<'a>(
        local_from_idx: usize,
        epsilon_prop: impl IntoIterator<Item = &'a EpsilonPropogation>,
        accept_state_idx: usize,
    ) -> U8TDFAAccept {
        let accept_transitions: Vec<_> = epsilon_prop
            .into_iter()
            .filter(|ep| ep.state == accept_state_idx)
            .collect();
        let anchored_accept = accept_transitions
            .iter()
            .cloned()
            .enumerate()
            .find(|(_, ep)| ep.end_only);
        let unanchored_accept = accept_transitions
            .iter()
            .cloned()
            .enumerate()
            .find(|(_, ep)| !ep.end_only);

        match (anchored_accept, unanchored_accept) {
            (Some((_, anchored)), None) => U8TDFAAccept::Anchored(
                U8TDFAAcceptTransition::from_epsilon_prop(local_from_idx, anchored),
            ),
            (None, Some((_, unanchored))) => U8TDFAAccept::Unanchored(
                U8TDFAAcceptTransition::from_epsilon_prop(local_from_idx, unanchored),
            ),
            (None, None) => U8TDFAAccept::None,
            (Some((anchored_idx, anchored)), Some((unanchored_idx, unanchored))) => {
                if anchored_idx < unanchored_idx {
                    // anchored is higher priority
                    // So we get the tags for the anchored first (if at end), and otherwise we get the tags for the unanchored
                    U8TDFAAccept::Both(
                        U8TDFAAcceptTransition::from_epsilon_prop(local_from_idx, anchored),
                        U8TDFAAcceptTransition::from_epsilon_prop(local_from_idx, unanchored),
                    )
                } else {
                    // unanchored is higher priority
                    // Since unanchored works even at the end, we don't need an extra anchored transition
                    U8TDFAAccept::Unanchored(U8TDFAAcceptTransition::from_epsilon_prop(
                        local_from_idx,
                        unanchored,
                    ))
                }
            }
        }
    }
    /// Updates the accept transitions with more transitions that take lower priority.
    ///
    /// This is basically equivalent to [`Self::from_epsilon_prop`] with the combined epsilon propogations of the two.
    pub fn update_with_lower_priority(self, other: U8TDFAAccept) -> U8TDFAAccept {
        match (self, other) {
            (U8TDFAAccept::Anchored(a), U8TDFAAccept::Unanchored(b) | U8TDFAAccept::Both(_, b)) => {
                U8TDFAAccept::Both(a, b)
            }
            (this @ (U8TDFAAccept::Both(_, _) | U8TDFAAccept::Unanchored(_)), _) => this,
            (this @ U8TDFAAccept::Anchored(_), U8TDFAAccept::Anchored(_)) => this,
            (U8TDFAAccept::None, other) => other,
            (this, U8TDFAAccept::None) => this,
        }
    }
}

pub struct U8TDFAState {
    /// Each (T)DFA state represents a subset of the NFA states.
    ///
    /// When executed, the TDFA state will store the tags for each of the NFA states it represents.
    ///
    /// Should be sorted by priority order of the NFA state threads it represents.
    /// State uniqueness includes priority order, so multiple TDFA states may represent the same set of NFA states,
    /// just in different orders.
    pub nfa_states: Vec<SubNFAStateID>,
    pub transitions: Vec<U8TDFATransition>,
    /// The highest-priority zero-length (i.e. epsilon) transition(s) to the accept state
    /// from the NFA state(s) this TDFA state represents.
    pub accept: U8TDFAAccept,
}
impl U8TDFAState {
    /// Returns `true` if this state is an immediate accept state.
    /// This means that it has any non-anchored accept transition.
    ///
    /// This only applies to `test` implementations, not `exec` implementations,
    /// as `exec` may need to match more data in higher-priority paths.
    pub fn test_immediate_accept(&self) -> bool {
        return matches!(
            self.accept,
            U8TDFAAccept::Both(_, _) | U8TDFAAccept::Unanchored(_)
        );
    }
    /// Creates a new start state for the TDFA and expands it to create stubs for all the states
    /// it has transitions to. Unlike normal states, the start state's transitions are generated
    /// including transitions in the NFA with start anchors.
    ///
    /// ## Returns
    /// A pair `(start_state, new_states)`
    ///
    /// where `new_states` are the initial set of states and all need to be expanded with [`U8TDFAState::expand`].
    pub fn new_start_state(nfa: &U8NFA) -> (U8TDFAState, Vec<U8TDFAState>) {
        let epsilon_prop: Vec<EpsilonPropogation> =
            EpsilonPropogation::calculate_epsilon_propogations_u8(nfa, 0);
        let accept = U8TDFAAccept::from_epsilon_prop(0, &epsilon_prop, nfa.states.len() - 1);

        let transitions: Vec<_> = epsilon_prop
            .iter()
            .filter(|ep| !ep.end_only)
            .flat_map(|ep| {
                nfa.states[ep.state]
                    .transitions
                    .iter()
                    .map(|tr| (tr.symbol.0.clone(), (ep.clone(), tr)))
            })
            .collect();
        let transitions = transitions
            .iter()
            .map(|(range, value)| (range.clone(), value));
        let mut byte_ranges_transitions = split_ranges_u8(transitions);
        for (_, nfa_tr) in &mut byte_ranges_transitions {
            // remove lower priority transitions to the same nfa state
            nfa_tr.dedup_by_key_all(|tr| tr.1.to);
        }

        let mut new_states: Vec<U8TDFAState> = Vec::new();
        let mut start_state = U8TDFAState {
            nfa_states: vec![SubNFAStateID(0)],
            transitions: Vec::new(),
            accept,
        };
        for (range, nfa_tr) in byte_ranges_transitions {
            let nfa_states = nfa_tr.iter().map(|(_, tr)| SubNFAStateID(tr.to)).collect();

            let new_state_idx = new_states
                .iter()
                .enumerate()
                .find(|&(_, s)| s.nfa_states == nfa_states);
            let new_state_idx = if let Some((i, _)) = new_state_idx {
                i
            } else {
                let new_state = U8TDFAState {
                    nfa_states,
                    transitions: Vec::new(),    // will do when expanded
                    accept: U8TDFAAccept::None, // will do when expanded
                };
                let new_state_idx = new_states.len();
                new_states.push(new_state);
                new_state_idx
            };

            let add_tags = nfa_tr
                .iter()
                .enumerate()
                .flat_map(|(i, (ep, _))| ep.update_tags.iter().map(move |tag| (i, tag.clone())))
                .collect();
            let dfa_tr = U8TDFATransition {
                to: new_state_idx,
                symbol: range,
                copy_tags: vec![0; nfa_tr.len()],
                add_tags,
            };
            start_state.transitions.push(dfa_tr);
        }
        return (start_state, new_states);
    }
    /// Given a state with `nfa_states` set but empty transitions and no accept,
    /// expands the state to include all possible transitions.
    ///
    /// ## Params
    /// - `nfa` is the original nfa
    /// - `curr_dfa_states` is the current list of states in the DFA.
    ///   The returned new states will be appended to the end of this list.
    ///
    /// ## Returns
    /// A list of new states, which will be added to [`U8DFA::states`]. They will only have `nfa_states` set,
    /// and thus will need to have `expand` called on them to get the full set of transitions.
    fn expand(&mut self, nfa: &U8NFA, curr_dfa_states: &[U8TDFAState]) -> Vec<U8TDFAState> {
        assert!(self.transitions.is_empty());
        assert!(matches!(self.accept, U8TDFAAccept::None));

        struct SubNFATransition {
            /// Local NFA state index in the TDFA state we are expanding
            from: usize,
            /// The epsilon propogation before the symbol transition
            ep: EpsilonPropogation,
            /// The index of the destination NFA state in [`U8NFA::states`]
            to: SubNFAStateID,
        }

        let mut transitions = Vec::new();
        for (local_nfa_state_idx, nfa_state_id) in self.nfa_states.iter().enumerate() {
            // maintaining priority: self.nfa_states is sorted by priority

            let epsilon_prop: Vec<EpsilonPropogation> =
                EpsilonPropogation::calculate_epsilon_propogations_u8(nfa, nfa_state_id.0);

            let mut tmp = U8TDFAAccept::None;
            std::mem::swap(&mut tmp, &mut self.accept);
            self.accept = tmp.update_with_lower_priority(U8TDFAAccept::from_epsilon_prop(
                local_nfa_state_idx,
                &epsilon_prop,
                nfa.states.len() - 1,
            ));

            for ep in epsilon_prop {
                if ep.start_only || ep.end_only {
                    continue;
                }
                let nfa_prop_state = &nfa.states[ep.state];
                for tr in &nfa_prop_state.transitions {
                    let symbol = tr.symbol.0.clone();
                    let nfa_tr = SubNFATransition {
                        from: local_nfa_state_idx,
                        ep: ep.clone(),
                        to: SubNFAStateID(tr.to),
                    };
                    transitions.push((symbol, nfa_tr));
                }
            }
        }

        let mut new_states: Vec<U8TDFAState> = Vec::new();
        // now we have all nfa transitions from all the nfa states
        // we need to combine and split them into byte ranges
        let transitions = transitions
            .iter()
            .map(|(range, value)| (range.clone(), value));
        let byte_ranges_transitions = split_ranges_u8(transitions);
        for (range, mut nfa_tr) in byte_ranges_transitions {
            // remove lower priority transitions to the same nfa state
            nfa_tr.dedup_by_key_all(|tr| tr.to);

            let nfa_states = nfa_tr.iter().map(|nfa_tr| nfa_tr.to).collect();
            let new_state_idx = curr_dfa_states
                .iter()
                .enumerate()
                .find(|(_, existing_state)| existing_state.nfa_states == nfa_states)
                .map(|(i, _)| i)
                .or_else(|| {
                    new_states
                        .iter()
                        .enumerate()
                        .find(|(_, new_state)| new_state.nfa_states == nfa_states)
                        .map(|(i, _)| i + curr_dfa_states.len())
                })
                .unwrap_or_else(|| curr_dfa_states.len() + new_states.len());
            if new_state_idx >= new_states.len() + curr_dfa_states.len() {
                // new state needs to be added
                new_states.push(U8TDFAState {
                    nfa_states,
                    transitions: Vec::new(),    // will do when expanded
                    accept: U8TDFAAccept::None, // will do when expanded
                });
            }

            let add_tags = nfa_tr
                .iter()
                .enumerate()
                .flat_map(|(i, nfa_tr)| {
                    nfa_tr
                        .ep
                        .update_tags
                        .iter()
                        .map(move |tag| (i, tag.clone()))
                })
                .collect();
            let copy_tags = nfa_tr.iter().map(|tr| tr.from).collect();
            let dfa_tr = U8TDFATransition {
                to: new_state_idx,
                symbol: range,
                copy_tags,
                add_tags,
            };
            self.transitions.push(dfa_tr);
        }

        return new_states;
    }
}

/// A Tagged DFA over `u8`s, fully constructed. Intended for use at compile time.
pub struct U8TDFA {
    /// The 'start' state is not stored in the `states` vec, and can never be transitioned to.
    /// This allows us to exclude transitions with start anchors from all other states,
    /// while implicitly including them in the start state.
    ///
    /// The start state always begins with one NFA state (0) with no tags.
    pub start_state: U8TDFAState,
    /// Unique by [`U8TDFAState::nfa_states`], including priority order.
    pub states: Vec<U8TDFAState>,
}
impl U8TDFA {
    /// Creates a TDFA from a TNFA. This should be the primary way to create a `U8TDFA`.
    ///
    /// Since the size of a DFA is worst-case exponential in the number of NFA states,
    /// the maximum number of states is `max_states`.
    /// If the number of states exceeds `max_states` then `None` will be returned.
    pub fn from_nfa(nfa: &U8NFA, max_states: usize) -> Option<Self> {
        let mut states; // Created TDFA states (except start)
        let mut stack = Vec::new(); // TDFA states to expand out from. added on creation

        let start_state = {
            let (start_state, new_states) = U8TDFAState::new_start_state(nfa);
            stack.extend(0..new_states.len());
            states = new_states;

            if states.len() > max_states {
                return None;
            }

            start_state
        };

        // other states
        while let Some(dfa_state_id) = stack.pop() {
            let mut dfa_state = U8TDFAState {
                nfa_states: states[dfa_state_id].nfa_states.clone(),
                transitions: Default::default(),
                accept: U8TDFAAccept::None,
            };
            std::mem::swap(&mut dfa_state, &mut states[dfa_state_id]);
            let new_states = dfa_state.expand(nfa, &states);
            states[dfa_state_id] = dfa_state;

            for new_state in new_states {
                if states.iter().any(|s| s.nfa_states == new_state.nfa_states) {
                    continue;
                }
                stack.push(states.len());
                states.push(new_state);

                if states.len() > max_states {
                    return None;
                }
            }
        }

        return Some(U8TDFA {
            start_state,
            states,
        });
    }

    pub fn num_capture_groups(&self) -> usize {
        let mut max = 0;

        for state in self.states.iter().chain(std::iter::once(&self.start_state)) {
            for tr in &state.transitions {
                for (_, tag) in &tr.add_tags {
                    max = std::cmp::max(max, tag.capture_group());
                }
            }
            match &state.accept {
                U8TDFAAccept::Anchored(t) | U8TDFAAccept::Unanchored(t) => {
                    max = std::cmp::max(max, t.max_capture_group());
                }
                U8TDFAAccept::Both(t1, t2) => {
                    max = std::cmp::max(max, t1.max_capture_group());
                    max = std::cmp::max(max, t2.max_capture_group());
                }
                U8TDFAAccept::None => {}
            }
        }
        return max + 1;
    }

    /// Returns the default bound for the TDFA.
    ///
    /// Should probably be tuned.
    pub fn default_bound(nfa_states: usize) -> usize {
        return std::cmp::max(100, nfa_states * 2);
    }
}

impl U8TDFA {
    /// Writes a LaTeX TikZ representation to visualize the graph.
    ///
    /// If `include_doc` is `true`, will include the headers.
    /// Otherwise, you should include `\usepackage{tikz}` and `\usetikzlibrary{automata, positioning}`.
    pub fn to_tikz(&self, include_doc: bool) -> String {
        let accept_state = crate::visualization::LatexGraphState {
            label: "accept".to_string(),
            transitions: Vec::new(),
            initial: false,
            accept: true,
        };
        fn make_label(nfa_indices: &[SubNFAStateID]) -> String {
            let nfa_indices: Vec<_> = nfa_indices.iter().map(|s| s.0.to_string()).collect();
            return nfa_indices.join(",");
        }
        let map_state =
            |(i, state): (usize, &U8TDFAState)| -> crate::visualization::LatexGraphState {
                let transitions =
                    state
                        .transitions
                        .iter()
                        .map(|t| crate::visualization::LatexGraphTransition {
                            label: crate::visualization::escape_latex(
                                DisplayRange(t.symbol.clone()).to_string(),
                            ),
                            to: t.to + 1,
                        });
                let accept = match &state.accept {
                    U8TDFAAccept::Anchored(_) => Some(crate::visualization::LatexGraphTransition {
                        to: self.states.len() + 1,
                        label: "\\$".to_string(),
                    }),
                    U8TDFAAccept::Both(_, _) => Some(crate::visualization::LatexGraphTransition {
                        to: self.states.len() + 1,
                        label: String::new(),
                    }),
                    U8TDFAAccept::Unanchored(_) => {
                        Some(crate::visualization::LatexGraphTransition {
                            to: self.states.len() + 1,
                            label: String::new(),
                        })
                    }
                    U8TDFAAccept::None => None,
                };

                let transitions = transitions.chain(accept).collect();

                return crate::visualization::LatexGraphState {
                    label: make_label(&state.nfa_states),
                    transitions,
                    initial: i == 0,
                    accept: false,
                };
            };

        let graph = crate::visualization::LatexGraph {
            states: std::iter::once(&self.start_state)
                .chain(self.states.iter())
                .enumerate()
                .map(map_state)
                .chain(std::iter::once(accept_state))
                .collect(),
        };
        return graph.to_tikz(include_doc);
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct U8DFATransition {
    pub to: usize,
    pub symbol: RangeInclusive<u8>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum U8DFAAccept {
    Anchored,
    Always,
    None,
}

/// Un-tagged DFA state. Only useful for `test`, not `exec`.
#[derive(PartialEq, Eq, Hash)]
pub struct U8DFAState {
    pub accept: U8DFAAccept,
    /// Sorted by and and with non-overlapping symbol ranges
    pub transitions: Vec<U8DFATransition>,
}
impl U8DFAState {
    fn from_tagged(tdfa_state: &U8TDFAState) -> U8DFAState {
        let accept = match tdfa_state.accept {
            U8TDFAAccept::Anchored(_) => U8DFAAccept::Anchored,
            U8TDFAAccept::Both(_, _) | U8TDFAAccept::Unanchored(_) => U8DFAAccept::Always,
            U8TDFAAccept::None => U8DFAAccept::None,
        };
        let mut transitions: Vec<_> = tdfa_state
            .transitions
            .iter()
            .map(|tr| U8DFATransition {
                to: tr.to,
                symbol: tr.symbol.clone(),
            })
            .collect();
        transitions.sort_by_key(|tr| *tr.symbol.start());
        return U8DFAState {
            accept,
            transitions,
        };
    }
}

/// Un-tagged DFA. Only useful for `test`, not `exec`.
pub struct U8DFA {
    pub start_state: U8DFAState,
    pub states: Vec<U8DFAState>,
}
impl U8DFA {
    /// Converts from the tagged [`U8TDFA`] and performs DFA minimization.
    /// Primarily intended for an intermediate step for constructing `test` implementations.
    ///
    /// ## Errors
    /// Only if the passed [`U8TDFA`] was invalid (e.g. invalid index references)
    pub fn from_tagged(tdfa: &U8TDFA) -> Result<U8DFA, ()> {
        let start_state = U8DFAState::from_tagged(&tdfa.start_state);
        let states = tdfa.states.iter().map(U8DFAState::from_tagged).collect();
        let mut dfa = U8DFA {
            start_state,
            states,
        };
        dfa.validate()?;

        // We should have already removed unreachable/dead states for the NFA
        // Now we need to merge redundant states:
        loop {
            let mut merge = Vec::new();
            let mut unique = HashMap::new();

            for (i, state) in dfa.states.iter().enumerate() {
                match unique.entry(state) {
                    std::collections::hash_map::Entry::Occupied(occupied_entry) => {
                        merge.push((i, *occupied_entry.get()));
                    }
                    std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(i);
                    }
                }
            }
            drop(unique);

            if merge.is_empty() {
                break;
            }
            debug_assert!(merge.is_sorted());
            for (merge, keep) in merge.into_iter().rev() {
                debug_assert!(keep < merge);
                // by going in reverse order, `merge` will always decrease (and `keep < merge`)
                // so previous mergings in this pass will not have changed the index
                // of the states referred to by `merge` and `keep`.
                dfa.merge_states(keep, merge).unwrap_or_else(|()| {
                    unreachable!(
                        "We just got these indices and they should not have been invalidated"
                    )
                });
            }
        }

        debug_assert!(
            dfa.validate().is_ok(),
            "DFA minimization should not cause a valid DFA to become invalid",
        );
        return Ok(dfa);
    }

    pub fn validate(&self) -> Result<(), ()> {
        for state in std::iter::once(&self.start_state).chain(&self.states) {
            let mut prev_end: i16 = -1;
            for tr in &state.transitions {
                if tr.to >= self.states.len() {
                    return Err(());
                }
                if *tr.symbol.start() as i16 <= prev_end {
                    // either out of order or overlapping
                    return Err(());
                }
                prev_end = *tr.symbol.end() as i16;
            }
        }

        return Ok(());
    }
    /// Writes a LaTeX TikZ representation to visualize the graph.
    ///
    /// If `include_doc` is `true`, will include the headers.
    /// Otherwise, you should include `\usepackage{tikz}` and `\usetikzlibrary{automata, positioning}`.
    pub fn to_tikz(&self, include_doc: bool) -> String {
        let accept_state = crate::visualization::LatexGraphState {
            label: "accept".to_string(),
            transitions: Vec::new(),
            initial: false,
            accept: true,
        };
        fn make_label(nfa_indices: &[SubNFAStateID]) -> String {
            let nfa_indices: Vec<_> = nfa_indices.iter().map(|s| s.0.to_string()).collect();
            return nfa_indices.join(",");
        }
        let map_state =
            |(i, state): (usize, &U8DFAState)| -> crate::visualization::LatexGraphState {
                let transitions =
                    state
                        .transitions
                        .iter()
                        .map(|t| crate::visualization::LatexGraphTransition {
                            label: crate::visualization::escape_latex(
                                DisplayRange(t.symbol.clone()).to_string(),
                            ),
                            to: t.to + 1,
                        });
                let accept = match &state.accept {
                    U8DFAAccept::Anchored => Some(crate::visualization::LatexGraphTransition {
                        to: self.states.len() + 1,
                        label: "\\$".to_string(),
                    }),
                    U8DFAAccept::Always => Some(crate::visualization::LatexGraphTransition {
                        to: self.states.len() + 1,
                        label: String::new(),
                    }),
                    U8DFAAccept::None => None,
                };

                let transitions = transitions.chain(accept).collect();

                return crate::visualization::LatexGraphState {
                    label: i.to_string(),
                    transitions,
                    initial: i == 0,
                    accept: false,
                };
            };

        let graph = crate::visualization::LatexGraph {
            states: std::iter::once(&self.start_state)
                .chain(self.states.iter())
                .enumerate()
                .map(map_state)
                .chain(std::iter::once(accept_state))
                .collect(),
        };
        return graph.to_tikz(include_doc);
    }

    /// Merge two states, changing all transitions into `merge` to transitions to `keep`,
    /// and removing `merge` state (changing all higher indices as well)
    ///
    /// ## Errors
    /// If `keep` or `merge` are invalid indices
    fn merge_states(&mut self, keep: usize, merge: usize) -> Result<(), ()> {
        if merge >= self.states.len() || keep >= self.states.len() {
            return Err(());
        }

        self.states.remove(merge);
        for state in std::iter::once(&mut self.start_state).chain(&mut self.states) {
            for tr in state.transitions.iter_mut() {
                if tr.to == merge {
                    tr.to = keep;
                } else if tr.to > merge {
                    tr.to -= 1;
                }
            }
        }

        return Ok(());
    }
}

/// Splits overlapping ranges so they are fully overlapping and/or non-overlapping.
/// This essentially makes the ranges disjoint, while maintaining the associated values for each u8.
///
/// E.g. `[(0..=5, 'a'), (3..=10, 'b')]` becomes `[(0..=2, ['a']), (3..=5, ['a', 'b']), (6..=10, ['b'])]`
///
/// Within each range, the items keep their order.
fn split_ranges_u8<'a, T>(
    items: impl IntoIterator<Item = (RangeInclusive<u8>, &'a T)>,
) -> Vec<(RangeInclusive<u8>, Vec<&'a T>)> {
    fn same<'a, T>(a: &[&'a T], b: &[&'a T]) -> bool {
        return a.len() == b.len() && std::iter::zip(a, b).all(|(&a, &b)| std::ptr::eq(a, b));
    }
    let mut value_ranges: Box<[_; 256]> = vec![Vec::new(); 256]
        .into_boxed_slice()
        .try_into()
        .unwrap_or_else(|_| unreachable!("Just allocated with size"));
    for (range, value) in items {
        for i in range {
            value_ranges[i as usize].push(value);
        }
    }

    let mut out = Vec::new();
    let mut prev_items = Vec::new();
    let mut prev_start = u8::MIN;
    for (i, items) in value_ranges.into_iter().enumerate() {
        let i = i as u8;
        if !same(&items, &prev_items) {
            if !prev_items.is_empty() {
                out.push((prev_start..=i - 1, prev_items));
            }
            prev_items = items;
            prev_start = i;
        }
    }
    if !prev_items.is_empty() {
        out.push((prev_start..=u8::MAX, prev_items));
    }

    debug_assert!(out.iter().all(|(_, items)| items.len() > 0));
    debug_assert!(out.windows(2).all(|w| w[0].0.end() < w[1].0.start()));
    return out;
}

trait VecExt<T> {
    /// Deduplicates the vector by key, keeping the first occurrence of each key.
    /// Unlike [`Vec::dedup_by_key`], this method removes all duplicates, not just adjacent ones.
    fn dedup_by_key_all<K: Eq + std::hash::Hash>(&mut self, key: impl Fn(&T) -> K);
}
impl<T> VecExt<T> for Vec<T> {
    fn dedup_by_key_all<K: Eq + std::hash::Hash>(&mut self, key: impl Fn(&T) -> K) {
        // TODO: we could use a vec for smaller sizes
        let mut seen = HashSet::new();
        self.retain(|x| {
            let k = key(x);
            if seen.contains(&k) {
                return false;
            }
            seen.insert(k);
            true
        });
    }
}

/// Newtype for displaying bytes as characters.
/// - Printable ascii characters are printed as themselves (or their escaped versions)
/// - Other characters are printed as their hex value
struct DisplayByteChar(u8);
impl std::fmt::Display for DisplayByteChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            b if b.is_ascii_graphic() => write!(f, "{:?}", b as char),
            b'\t' => write!(f, "'\\t'"),
            b'\n' => write!(f, "'\\n'"),
            b'\r' => write!(f, "'\\r'"),
            b' ' => write!(f, "' '"),
            b if b.is_ascii_whitespace() => write!(f, "{:?}", b as char),
            _ => write!(f, "0x{:02x}", self.0),
        }
    }
}

/// Newtype for displaying a byte range for transitions.
/// See [`DisplayByteChar`] for details.
struct DisplayRange(RangeInclusive<u8>);
impl std::fmt::Display for DisplayRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.0.start();
        let end = self.0.end();
        if start == end {
            return write!(f, "{}", DisplayByteChar(*start));
        } else {
            return write!(f, "{}..={}", DisplayByteChar(*start), DisplayByteChar(*end));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::Config, parse_tree::ERE, simplified_tree::SimplifiedTreeNode,
        working_nfa::WorkingNFA,
    };

    use super::*;

    #[test]
    fn phone_number() {
        let ere = ERE::parse_str(r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$").unwrap();
        let (tree, _) = SimplifiedTreeNode::from_ere(&ere, &Config::default());
        // assert_eq!(capture_groups, 2);
        let nfa = WorkingNFA::new(&tree);
        let nfa = U8NFA::new(&nfa);
        let nfa = U8DFA::from_nfa(&nfa, 100).unwrap();

        // println!("{}", nfa.to_tikz(true));
    }
}
