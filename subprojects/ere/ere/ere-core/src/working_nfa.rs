//! Implements the primary compile-time intermediate [`WorkingNFA`] structure for optimization.

use crate::parse_tree::Atom;
use crate::simplified_tree::SimplifiedTreeNode;
use quote::{quote, ToTokens};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpsilonType {
    None,
    StartAnchor,
    EndAnchor,
    StartCapture(usize),
    EndCapture(usize),
}
impl ToTokens for EpsilonType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            EpsilonType::None => tokens.extend(quote! { ::ere::working_nfa::EpsilonType::None }),
            EpsilonType::StartAnchor => {
                tokens.extend(quote! { ::ere::working_nfa::EpsilonType::StartAnchor })
            }
            EpsilonType::EndAnchor => {
                tokens.extend(quote! { ::ere::working_nfa::EpsilonType::EndAnchor })
            }
            EpsilonType::StartCapture(group_num) => tokens.extend(quote! {
                ::ere::working_nfa::EpsilonType::StartCapture(#group_num)
            }),
            EpsilonType::EndCapture(group_num) => tokens.extend(quote! {
                ::ere::working_nfa::EpsilonType::EndCapture(#group_num)
            }),
        };
    }
}

/// An epsilon transition for the [`WorkingNFA`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpsilonTransition {
    pub(crate) to: usize,
    pub(crate) special: EpsilonType,
}
impl EpsilonTransition {
    pub(crate) const fn new(to: usize) -> EpsilonTransition {
        return EpsilonTransition {
            to,
            special: EpsilonType::None,
        };
    }
    pub(crate) const fn with_offset(self, offset: usize) -> EpsilonTransition {
        return EpsilonTransition {
            to: self.to + offset,
            special: self.special,
        };
    }
    pub(crate) fn inplace_offset(&mut self, offset: usize) {
        self.to += offset;
    }
    pub(crate) const fn add_offset(&self, offset: usize) -> EpsilonTransition {
        return EpsilonTransition {
            to: self.to + offset,
            special: self.special,
        };
    }
    /// Only intended for internal use by macros.
    pub const fn __load(to: usize, special: EpsilonType) -> EpsilonTransition {
        return EpsilonTransition { to, special };
    }
}
impl std::fmt::Display for EpsilonTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "-> {}", self.to);
    }
}
impl ToTokens for EpsilonTransition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let EpsilonTransition { to, special } = self;
        tokens.extend(quote! {
            ere_core::working_nfa::EpsilonTransition::__load(
                #to,
                #special,
            )
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkingTransition {
    pub(crate) to: usize,
    pub(crate) symbol: Atom,
}
impl WorkingTransition {
    pub fn new(to: usize, symbol: Atom) -> WorkingTransition {
        return WorkingTransition { to, symbol };
    }
    pub fn with_offset(mut self, offset: usize) -> WorkingTransition {
        self.inplace_offset(offset);
        return self;
    }
    pub fn inplace_offset(&mut self, offset: usize) {
        self.to += offset;
    }
    pub fn add_offset(&self, offset: usize) -> WorkingTransition {
        return WorkingTransition {
            to: self.to + offset,
            symbol: self.symbol.clone(),
        };
    }
}
impl std::fmt::Display for WorkingTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "-({})> {}", self.symbol, self.to);
    }
}

/// The symbol transitions are assumed to have priority before epsilon transitions,
/// meaning that of all the propogated next symbol transitions available,
/// those going out from the previous transition's destination will come first.
///
/// So, if the symbol transitions should have a lower priority than those found
/// via an epsilon transition, they should be in a new state with a lower priority
/// epsilon transition to it.
/// Due to the way we construct NFAs, this should initially be the case--
/// No state has both incoming and outgoing symbol transitions
/// (they are always separated by at least one epsilon transition) and priority is maintained.
/// We then can optimize where there are no competing transitions.
#[derive(Debug, Clone)]
pub struct WorkingState {
    pub(crate) transitions: Vec<WorkingTransition>,
    pub(crate) epsilons: Vec<EpsilonTransition>,
}
impl WorkingState {
    pub const fn new() -> WorkingState {
        return WorkingState {
            transitions: Vec::new(),
            epsilons: Vec::new(),
        };
    }
    pub fn with_transition(mut self, to: usize, symbol: Atom) -> WorkingState {
        self.transitions.push(WorkingTransition::new(to, symbol));
        return self;
    }
    pub fn with_epsilon(mut self, to: usize) -> WorkingState {
        self.epsilons.push(EpsilonTransition::new(to));
        return self;
    }
    pub fn with_epsilon_special(mut self, to: usize, special: EpsilonType) -> WorkingState {
        self.epsilons.push(EpsilonTransition { to, special });
        return self;
    }
    pub fn with_offset(mut self, offset: usize) -> WorkingState {
        self.inplace_offset(offset);
        return self;
    }
    pub fn inplace_offset(&mut self, offset: usize) {
        for t in &mut self.transitions {
            t.inplace_offset(offset);
        }
        for e in &mut self.epsilons {
            e.inplace_offset(offset);
        }
    }
    pub fn add_offset(&self, offset: usize) -> WorkingState {
        return WorkingState {
            transitions: self
                .transitions
                .iter()
                .map(|t| t.add_offset(offset))
                .collect(),
            epsilons: self.epsilons.iter().map(|e| e.add_offset(offset)).collect(),
        };
    }
}
impl std::fmt::Display for WorkingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in &self.transitions {
            writeln!(f, "  {t}")?;
        }
        for e in &self.epsilons {
            writeln!(f, "  {e}")?;
        }
        return Ok(());
    }
}

/// Each NFA has one start state (`0`) and one accept state (`states.len() - 1`)
#[derive(Debug, Clone)]
pub struct WorkingNFA {
    pub(crate) states: Vec<WorkingState>,
}
impl WorkingNFA {
    /// Makes an NFA that matches with zero length.
    fn nfa_empty() -> WorkingNFA {
        let states = vec![WorkingState::new()];
        return WorkingNFA { states };
    }
    /// Makes an NFA matching a some symbol.
    fn nfa_symbol(c: &Atom) -> WorkingNFA {
        let states = vec![
            WorkingState::new().with_transition(1, c.clone()),
            WorkingState::new(),
        ];
        return WorkingNFA { states };
    }
    /// Makes a union of NFAs.
    fn nfa_union(nodes: &[WorkingNFA]) -> WorkingNFA {
        let states_count = 2 + nodes.iter().map(|n| n.states.len()).sum::<usize>();
        let mut states = vec![WorkingState::new()];
        for nfa in nodes {
            let sub_nfa_start = states.len();
            states[0]
                .epsilons
                .push(EpsilonTransition::new(sub_nfa_start));
            states.extend(
                nfa.states
                    .iter()
                    .map(|state| state.add_offset(sub_nfa_start)),
            );
            states
                .last_mut()
                .unwrap()
                .epsilons
                .push(EpsilonTransition::new(states_count - 1));
        }
        states.push(WorkingState::new());
        assert_eq!(states_count, states.len());

        return WorkingNFA { states };
    }
    fn build_union(nodes: &[SimplifiedTreeNode]) -> WorkingNFA {
        let sub_nfas: Vec<WorkingNFA> = nodes.iter().map(WorkingNFA::build).collect();
        return WorkingNFA::nfa_union(&sub_nfas);
    }
    /// Wraps an NFA part in a capture group.
    fn nfa_capture(nfa: &WorkingNFA, group_num: usize) -> WorkingNFA {
        let states_count = 2 + nfa.states.len();
        let mut states: Vec<WorkingState> = std::iter::once(
            WorkingState::new().with_epsilon_special(1, EpsilonType::StartCapture(group_num)),
        )
        .chain(nfa.states.iter().map(|state| state.add_offset(1)))
        .chain(std::iter::once(WorkingState::new()))
        .collect();
        assert_eq!(states_count, states.len());
        states[states_count - 2].epsilons.push(EpsilonTransition {
            to: states_count - 1,
            special: EpsilonType::EndCapture(group_num),
        });

        return WorkingNFA { states };
    }
    fn build_capture(tree: &SimplifiedTreeNode, group_num: usize) -> WorkingNFA {
        let nfa = WorkingNFA::build(tree);
        return WorkingNFA::nfa_capture(&nfa, group_num);
    }
    /// Makes an NFA that matches a concatenation of NFAs.
    fn nfa_concat<T: IntoIterator<Item = WorkingNFA>>(nodes: T) -> WorkingNFA {
        let mut states = vec![WorkingState::new().with_epsilon(1)];

        for nfa in nodes {
            let states_count = states.len();
            states.extend(
                nfa.states
                    .into_iter()
                    .map(|state| state.with_offset(states_count)),
            );
            let states_count = states.len();
            states
                .last_mut()
                .unwrap()
                .epsilons
                .push(EpsilonTransition::new(states_count));
        }

        states.push(WorkingState::new());
        return WorkingNFA { states };
    }
    fn build_concat<'a, T: IntoIterator<Item = &'a SimplifiedTreeNode>>(nodes: T) -> WorkingNFA {
        return WorkingNFA::nfa_concat(nodes.into_iter().map(WorkingNFA::build));
    }
    /// Makes an NFA that matches some NFA concatenated with itself multiple times.
    fn nfa_repeat(nfa: &WorkingNFA, times: usize) -> WorkingNFA {
        return WorkingNFA::nfa_concat(std::iter::repeat(nfa).cloned().take(times));
    }
    fn build_repeat(tree: &SimplifiedTreeNode, times: usize) -> WorkingNFA {
        let nfa = WorkingNFA::build(tree);
        return WorkingNFA::nfa_repeat(&nfa, times);
    }
    /// Makes an NFA that matches some NFA concatenated with itself up to some number of times.
    fn nfa_upto(nfa: &WorkingNFA, times: usize, longest: bool) -> WorkingNFA {
        let end_state_idx = 1 + (nfa.states.len() + 1) * times;

        let state0 = if longest {
            WorkingState::new()
                .with_epsilon(1)
                .with_epsilon(end_state_idx - 1)
        } else {
            WorkingState::new()
                .with_epsilon(end_state_idx - 1)
                .with_epsilon(1)
        };
        let mut states = vec![state0];
        for i in 0..times {
            let states_count = states.len();
            states.extend(
                nfa.states
                    .iter()
                    .map(|state| state.add_offset(states_count)),
            );
            let transition_state_idx = states.len();
            states
                .last_mut()
                .unwrap()
                .epsilons
                .push(EpsilonTransition::new(transition_state_idx));
            let mut transition_state = WorkingState::new();
            if i + 1 != times {
                if longest {
                    transition_state
                        .epsilons
                        .push(EpsilonTransition::new(states.len() + 1));
                }

                transition_state
                    .epsilons
                    .push(EpsilonTransition::new(end_state_idx - 1));
                if !longest {
                    transition_state
                        .epsilons
                        .push(EpsilonTransition::new(states.len() + 1));
                }
            }
            states.push(transition_state);
        }

        return WorkingNFA { states };
    }
    fn build_upto(tree: &SimplifiedTreeNode, times: usize, longest: bool) -> WorkingNFA {
        let nfa = WorkingNFA::build(tree);
        return WorkingNFA::nfa_upto(&nfa, times, longest);
    }
    /// Makes an NFA that matches some NFA concatenated with itself any number of times.
    fn nfa_star(nfa: WorkingNFA, longest: bool) -> WorkingNFA {
        let end_state_idx = 1 + nfa.states.len();
        let mut start_state = WorkingState::new();
        if !longest {
            start_state
                .epsilons
                .push(EpsilonTransition::new(end_state_idx));
        }
        start_state.epsilons.push(EpsilonTransition::new(1));
        if longest {
            start_state
                .epsilons
                .push(EpsilonTransition::new(end_state_idx));
        }
        let mut states: Vec<WorkingState> = std::iter::once(start_state)
            .chain(nfa.states.into_iter().map(|state| state.with_offset(1)))
            .chain(std::iter::once(WorkingState::new()))
            .collect();
        states[end_state_idx - 1]
            .epsilons
            .push(EpsilonTransition::new(0));
        return WorkingNFA { states };
    }
    fn build_star(tree: &SimplifiedTreeNode, longest: bool) -> WorkingNFA {
        let nfa = WorkingNFA::build(tree);
        return WorkingNFA::nfa_star(nfa, longest);
    }
    /// Makes an NFA that matches zero length but only at the text start
    fn nfa_start() -> WorkingNFA {
        let states = vec![
            WorkingState::new().with_epsilon_special(1, EpsilonType::StartAnchor),
            WorkingState::new(),
        ];
        return WorkingNFA { states };
    }
    /// Makes an NFA that matches zero length but only at the text end
    fn nfa_end() -> WorkingNFA {
        let states = vec![
            WorkingState::new().with_epsilon_special(1, EpsilonType::EndAnchor),
            WorkingState::new(),
        ];
        return WorkingNFA { states };
    }
    /// Makes an NFA that never matches.
    fn nfa_never() -> WorkingNFA {
        let states = vec![WorkingState::new(), WorkingState::new()];
        return WorkingNFA { states };
    }
    /// Recursively builds an inefficient but valid NFA based loosely on Thompson's Algorithm.
    ///
    /// Should be optimized using [`WorkingNFA::optimize_pass`]
    pub fn build(tree: &SimplifiedTreeNode) -> WorkingNFA {
        return match tree {
            SimplifiedTreeNode::Empty => WorkingNFA::nfa_empty(),
            SimplifiedTreeNode::Symbol(c) => WorkingNFA::nfa_symbol(c),
            SimplifiedTreeNode::Union(nodes) => WorkingNFA::build_union(nodes),
            SimplifiedTreeNode::Capture(tree, group_num) => {
                WorkingNFA::build_capture(&tree, *group_num)
            }
            SimplifiedTreeNode::Concat(nodes) => WorkingNFA::build_concat(nodes),
            SimplifiedTreeNode::Repeat(tree, times) => WorkingNFA::build_repeat(tree, times.get()),
            SimplifiedTreeNode::UpTo(tree, times, longest) => {
                WorkingNFA::build_upto(tree, times.get(), *longest)
            }
            SimplifiedTreeNode::Star(tree, longest) => WorkingNFA::build_star(tree, *longest),
            SimplifiedTreeNode::Start => WorkingNFA::nfa_start(),
            SimplifiedTreeNode::End => WorkingNFA::nfa_end(),
            SimplifiedTreeNode::Never => WorkingNFA::nfa_never(),
        };
    }

    /// Creates an NFA with the default `.*?` loops at the start and end (though they may be optimized away if not needed).
    pub fn new(tree: &SimplifiedTreeNode) -> WorkingNFA {
        return Self::new_loop_opt(tree, true, true);
    }
    /// Creates an NFA but allowing specification of whether to include the `.*?` loops at the start and end.
    pub fn new_loop_opt(tree: &SimplifiedTreeNode, start_loop: bool, end_loop: bool) -> WorkingNFA {
        let mut nfa = WorkingNFA::build(tree);

        nfa.clean_start_anchors();
        nfa.clean_end_anchors();

        // add loops at start and end in case we lack anchors
        if start_loop {
            nfa = WorkingNFA::nfa_concat([
                WorkingNFA::nfa_star(
                    WorkingNFA::nfa_symbol(&Atom::NonmatchingList(Vec::new())),
                    false,
                ),
                nfa,
            ]);
        }
        if end_loop {
            nfa = WorkingNFA::nfa_concat([
                nfa,
                WorkingNFA::nfa_star(
                    WorkingNFA::nfa_symbol(&Atom::NonmatchingList(Vec::new())),
                    false,
                ),
            ]);
        }

        let zero_symbol_states: Vec<bool> =
            std::iter::zip(nfa.nodes_after_end(), nfa.nodes_before_start())
                .map(|(a, b)| a || b)
                .collect();
        for (from, state) in nfa.states.iter_mut().enumerate() {
            if zero_symbol_states[from] {
                state.transitions = Vec::new();
            }
        }

        // nfa.remove_unreachable();
        // Finally, do normal optimization passes
        // println!("{}", nfa.to_tikz(true));
        while nfa.optimize_pass() {
            // println!("{}", nfa.to_tikz(true));
        }
        nfa.remove_unreachable();
        return nfa;
    }

    /// Removes start anchors that will never be satisfied
    /// (basically turning them into a `Never` to allow further optimization)
    fn clean_start_anchors(&mut self) {
        let mut zero_len_reachable = vec![false; self.states.len()];
        zero_len_reachable[0] = true;
        let mut stack = vec![0];
        while let Some(state) = stack.pop() {
            for e in &self.states[state].epsilons {
                if !zero_len_reachable[e.to] {
                    stack.push(e.to);
                }
                zero_len_reachable[e.to] = true;
            }
        }

        for (i, state) in self.states.iter_mut().enumerate() {
            state
                .epsilons
                .retain(|e| e.special != EpsilonType::StartAnchor || zero_len_reachable[i]);
        }
    }

    /// Removes end anchors that will never be satisfied
    /// (basically turning them into a `Never` to allow further optimization)    
    fn clean_end_anchors(&mut self) {
        let mut zero_len_reachable = vec![false; self.states.len()];
        zero_len_reachable[self.states.len() - 1] = true;

        let mut reverse_epsilons = vec![Vec::new(); self.states.len()];
        for (i, state) in self.states.iter().enumerate() {
            for e in &state.epsilons {
                reverse_epsilons[e.to].push(i);
            }
        }

        let mut stack = vec![self.states.len() - 1];
        while let Some(state) = stack.pop() {
            for src in &reverse_epsilons[state] {
                if !zero_len_reachable[*src] {
                    stack.push(*src);
                }
                zero_len_reachable[*src] = true;
            }
        }

        for state in self.states.iter_mut() {
            state
                .epsilons
                .retain(|e| e.special != EpsilonType::EndAnchor || zero_len_reachable[e.to]);
        }
    }
    /// Finds all nodes that are only ever visited after a `$`.
    fn nodes_after_end(&self) -> Vec<bool> {
        let mut nodes = vec![true; self.states.len()];
        nodes[0] = false;

        let mut stack = vec![0];
        while let Some(from) = stack.pop() {
            for e in self.states[from].epsilons.iter() {
                if nodes[e.to] && e.special != EpsilonType::EndAnchor {
                    nodes[e.to] = false;
                    stack.push(e.to);
                }
            }
            for t in self.states[from].transitions.iter() {
                if nodes[t.to] {
                    nodes[t.to] = false;
                    stack.push(t.to);
                }
            }
        }
        return nodes;
    }
    /// Finds all nodes that are only ever visited before a `^`.
    fn nodes_before_start(&self) -> Vec<bool> {
        let mut reverse = vec![Vec::new(); self.states.len()];
        for (i, state) in self.states.iter().enumerate() {
            for e in &state.epsilons {
                if e.special != EpsilonType::StartAnchor {
                    reverse[e.to].push(i);
                }
            }
            for t in &state.transitions {
                reverse[t.to].push(i);
            }
        }

        let mut nodes = vec![true; self.states.len()];
        nodes[self.states.len() - 1] = false;

        let mut stack = vec![self.states.len() - 1];
        while let Some(to) = stack.pop() {
            for from in &reverse[to] {
                if nodes[*from] {
                    nodes[*from] = false;
                    stack.push(*from);
                }
            }
        }
        return nodes;
    }

    /// Helper function for removing a set of states.
    ///
    /// These states should have no incoming transitions.
    fn remove_dead_states<T: IntoIterator<Item = bool>>(&mut self, dead_states: T) {
        let state_map: Vec<usize> = dead_states
            .into_iter()
            .scan(0, |s, dead| {
                if dead {
                    return Some(usize::MAX);
                } else {
                    let out = *s;
                    *s += 1;
                    return Some(out);
                }
            })
            .collect();
        self.states = self
            .states
            .iter()
            .enumerate()
            .filter(|(i, _)| state_map[*i] != usize::MAX)
            .map(|(_, state)| state)
            .cloned()
            .collect();

        for state in &mut self.states {
            for t in &mut state.transitions {
                t.to = state_map[t.to];
            }
            for t in &mut state.epsilons {
                t.to = state_map[t.to];
            }
        }
    }

    /// De-duplicates identical transitions
    /// (`a -e> b`, `a -e> b`) -> (`a -e> b`)
    ///
    /// Returns `true` if changes were made.
    /// The highest-priority transition will be kept.
    ///
    /// ---
    ///
    /// Typically these are caused by optimizations that merge paths.
    fn dedupe_transitions(&mut self) -> bool {
        let mut changed = false;

        for state in &mut self.states {
            // state transitions
            let keep: Vec<bool> = state
                .transitions
                .iter()
                .enumerate()
                .map(|(i, e)| state.transitions[..=i].contains(e))
                .collect();
            let prev_len = state.transitions.len();
            let mut i = 0;
            state.transitions.retain(|_| {
                let idx = i;
                i += 1;
                return keep[idx];
            });
            if state.transitions.len() != prev_len {
                changed = true;
            }

            // epsilon transitions
            let keep: Vec<bool> = state
                .epsilons
                .iter()
                .enumerate()
                .map(|(i, e)| !state.epsilons[..i].contains(e))
                .collect();
            let prev_len = state.epsilons.len();
            let mut i = 0;
            state.epsilons.retain(|_| {
                let idx = i;
                i += 1;
                return keep[idx];
            });
            if state.epsilons.len() != prev_len {
                changed = true;
            }
        }

        return changed;
    }

    /// Various operations to optimize the NFA graph.
    ///
    /// Returns `true` if changes were made (meaning another pass should be tried).
    fn optimize_pass(&mut self) -> bool {
        let mut changed = false;
        let state_count = self.states.len();
        debug_assert!(state_count >= 2);

        let mut dead_states = vec![false; self.states.len()];

        // Skip redundant states
        // Special transitions (anchors + capture groups) are treated similar to non-epsilon transitions
        'state_loop: for state_idx in 1..state_count - 1 {
            // merge states with same outgoing
            for other_idx in 0..state_count - 1 {
                if self.states[state_idx].epsilons == self.states[other_idx].epsilons
                    && self.states[state_idx].transitions == self.states[other_idx].transitions
                    && state_idx != other_idx
                    && (!self.states[state_idx].epsilons.is_empty()
                        || !self.states[state_idx].transitions.is_empty())
                {
                    // TODO: if the two states have self-loops, they currently are not counted
                    // as equivalent even if they should be.

                    // I think symbol transition order matters here because it may have been created by previous
                    // optimizations, which originated from epsilon transitions where it was important.
                    dead_states[state_idx] = true;
                    changed = true;
                    self.states[state_idx].epsilons = Vec::new();
                    self.states[state_idx].transitions = Vec::new();
                    // divert other states to other
                    for s in &mut self.states {
                        for ep in &mut s.epsilons {
                            if ep.to == state_idx {
                                ep.to = other_idx;
                            }
                        }
                        for tr in &mut s.transitions {
                            if tr.to == state_idx {
                                tr.to = other_idx;
                            }
                        }
                    }
                    continue 'state_loop;
                }
            }

            // dedupe transitions
            changed |= self.dedupe_transitions();

            // skip redundant
            let incoming: Vec<(usize, usize)> = self
                .states
                .iter()
                .enumerate()
                .flat_map(|(s_i, s)| s.transitions.iter().enumerate().map(move |(t, _)| (s_i, t)))
                .filter(|(s, t)| self.states[*s].transitions[*t].to == state_idx)
                .collect();
            let incoming_eps: Vec<(usize, usize)> = self
                .states
                .iter()
                .enumerate()
                .flat_map(|(s_i, s)| s.epsilons.iter().enumerate().map(move |(e, _)| (s_i, e)))
                .filter(|(s, e)| self.states[*s].epsilons[*e].to == state_idx)
                .collect();

            match (
                incoming.as_slice(),
                incoming_eps.as_slice(),
                self.states[state_idx].transitions.len(),
                self.states[state_idx].epsilons.len(),
            ) {
                // `as -xes> b -e> c` can become `as -xes> c` (assuming no other transitions)
                (incoming, incoming_eps, 0, 1)
                    if self.states[state_idx].epsilons[0].special == EpsilonType::None =>
                {
                    let to = self.states[state_idx].epsilons[0].to;
                    for (s, t) in incoming {
                        self.states[*s].transitions[*t].to = to;
                    }
                    for (s, e) in incoming_eps {
                        self.states[*s].epsilons[*e].to = to;
                    }
                    dead_states[state_idx] = true;
                    self.states[state_idx].epsilons = Vec::new();
                    changed = true;
                    continue;
                }
                // `a -e> b -es> cs` can become `a -es> cs` (assuming no other transitions)
                (&[], &[(incoming_state, incoming_eps)], 0, _)
                    if self.states[incoming_state].epsilons[incoming_eps].special
                        == EpsilonType::None =>
                {
                    let outgoing_eps = std::mem::take(&mut self.states[state_idx].epsilons);
                    let after = self.states[incoming_state]
                        .epsilons
                        .split_off(incoming_eps + 1);
                    self.states[incoming_state].epsilons.pop();
                    self.states[incoming_state]
                        .epsilons
                        .extend_from_slice(&outgoing_eps);
                    self.states[incoming_state]
                        .epsilons
                        .extend_from_slice(&after);

                    dead_states[state_idx] = true;
                    changed = true;
                    continue;
                }
                _ => {}
            }

            // TODO:
            // `a -e> b -xes> cs` can become `a -xes> cs` (assuming no other transitions)
            // `a -e> b -e> a` can combine `a` and `b` (including other transitions)
            // TODO: might cause additional overhead in some cases, should we do
            // ??? `a -x> b -es> cs` can become `a -xs> cs`
            // ??? `as -es> b -x> c` can become `as -xs> c`
        }

        if changed {
            self.remove_dead_states(dead_states);
            return true;
        }
        return false;
    }

    /// Finds the states that can be reached from the start via any path
    fn states_reachable_start(&self) -> Vec<bool> {
        let mut reachable = vec![false; self.states.len()];
        reachable[0] = true;
        let mut stack = vec![0];

        while let Some(state) = stack.pop() {
            for src in &self.states[state].epsilons {
                if !reachable[src.to] {
                    stack.push(src.to);
                }
                reachable[src.to] = true;
            }
            for src in &self.states[state].transitions {
                if !reachable[src.to] {
                    stack.push(src.to);
                }
                reachable[src.to] = true;
            }
        }

        return reachable;
    }
    /// Finds the states that can reach the end via any path
    fn states_reachable_end(&self) -> Vec<bool> {
        let mut reverse = vec![Vec::new(); self.states.len()];
        for (i, state) in self.states.iter().enumerate() {
            for e in &state.epsilons {
                reverse[e.to].push(i);
            }
            for t in &state.transitions {
                reverse[t.to].push(i);
            }
        }

        let mut reachable = vec![false; self.states.len()];
        reachable[self.states.len() - 1] = true;
        let mut stack = vec![self.states.len() - 1];

        while let Some(state) = stack.pop() {
            for src in &reverse[state] {
                if !reachable[*src] {
                    stack.push(*src);
                }
                reachable[*src] = true;
            }
        }

        return reachable;
    }

    /// Removes all nodes that cannot be reached or cannot reach the end.
    ///
    /// Ignores special epsilon types (so should be called after they have been resolved)
    fn remove_unreachable(&mut self) {
        let reach_start = self.states_reachable_start();
        let reach_end = self.states_reachable_end();

        // Remove transitions that involve redundant states
        for state in &mut self.states {
            state
                .epsilons
                .retain(|e| reach_start[e.to] && reach_end[e.to]);
            state
                .transitions
                .retain(|t| reach_start[t.to] && reach_end[t.to]);
        }

        // Then remove the states
        self.remove_dead_states(
            std::iter::zip(reach_start.into_iter(), reach_end.into_iter()).map(|(a, b)| !a || !b),
        );
    }

    /// Finds the number of capture groups in this NFA
    pub fn num_capture_groups(&self) -> usize {
        return self
            .states
            .iter()
            .flat_map(|state| &state.epsilons)
            .map(|eps| match eps.special {
                EpsilonType::StartCapture(n) => n,
                _ => 0,
            })
            .max()
            .unwrap_or(0)
            + 1;
    }

    /// Returns whether each there is any matching path where the capture group is unused
    pub fn capture_group_is_optional(&self, group_num: usize) -> bool {
        let mut reached_states = vec![false; self.states.len()];
        reached_states[0] = true;
        let mut stack = vec![0];
        while let Some(idx) = stack.pop() {
            for t in &self.states[idx].transitions {
                if !reached_states[t.to] {
                    reached_states[t.to] = true;
                    stack.push(t.to);
                }
            }
            for e in &self.states[idx].epsilons {
                if !reached_states[e.to] && e.special != EpsilonType::StartCapture(group_num) {
                    // the end capture should not be reachable without a preceding
                    // start capture, so we only need to check the start.
                    debug_assert_ne!(e.special, EpsilonType::EndCapture(group_num));
                    reached_states[e.to] = true;
                    stack.push(e.to);
                }
            }
        }

        return *reached_states.last().unwrap();
    }

    /// Writes a LaTeX TikZ representation to visualize the graph.
    ///
    /// If `include_doc` is `true`, will include the headers.
    /// Otherwise, you should include `\usepackage{tikz}` and `\usetikzlibrary{automata, positioning}`.
    pub fn to_tikz(&self, include_doc: bool) -> String {
        let map_state =
            |(i, state): (usize, &WorkingState)| -> crate::visualization::LatexGraphState {
                let transitions =
                    state
                        .transitions
                        .iter()
                        .map(|t| crate::visualization::LatexGraphTransition {
                            label: crate::visualization::escape_latex(t.symbol.to_string()),
                            to: t.to,
                        });
                let epsilons = state.epsilons.iter().enumerate().map(|(i, e)| {
                    let label = match e.special {
                        EpsilonType::None => format!(r"$\epsilon_{{{i}}}$"),
                        EpsilonType::StartAnchor => format!(r"{{\textasciicircum}}$_{{{i}}}$"),
                        EpsilonType::EndAnchor => format!(r"$\$_{{{i}}}$"),
                        EpsilonType::StartCapture(group) => format!("${group}(_{{{i}}}$"),
                        EpsilonType::EndCapture(group) => format!("$){group}_{{{i}}}$"),
                    };
                    return crate::visualization::LatexGraphTransition { label, to: e.to };
                });
                let transitions = transitions.chain(epsilons).collect();
                return crate::visualization::LatexGraphState {
                    label: format!("q{i}"),
                    transitions,
                    initial: i == 0,
                    accept: i + 1 == self.states.len(),
                };
            };

        let graph = crate::visualization::LatexGraph {
            states: self.states.iter().enumerate().map(map_state).collect(),
        };
        return graph.to_tikz(include_doc);
    }

    /// Using the classical NFA algorithm to do a simple boolean test on a string.
    pub fn test(&self, text: &str) -> bool {
        let mut list = vec![false; self.states.len()];
        let mut new_list = vec![false; self.states.len()];
        list[0] = true;

        // Adds all states reachable by epsilon transitions
        let propogate_epsilon = |list: &mut Vec<bool>, idx: usize| {
            let mut stack: Vec<usize> = list
                .iter()
                .enumerate()
                .filter_map(|(i, set)| set.then_some(i))
                .collect();

            while let Some(from) = stack.pop() {
                for EpsilonTransition { to, special } in &self.states[from].epsilons {
                    if list[from]
                        && !list[*to]
                        && (match special {
                            EpsilonType::StartAnchor => idx == 0,
                            EpsilonType::EndAnchor => idx == text.len(),
                            _ => true,
                        })
                    {
                        stack.push(*to);
                        list[*to] = true;
                    }
                }
            }
        };

        for (i, c) in text.char_indices() {
            propogate_epsilon(&mut list, i);
            for (from, state) in self.states.iter().enumerate() {
                if !list[from] {
                    continue;
                }

                for WorkingTransition { to, symbol } in &state.transitions {
                    if symbol.check(c) {
                        new_list[*to] = true;
                    }
                }
            }
            let tmp = list;
            list = new_list;
            new_list = tmp;
            new_list.fill(false);
        }
        propogate_epsilon(&mut list, text.len());
        return *list.last().unwrap_or(&false);
    }
}
impl std::fmt::Display for WorkingNFA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, state) in self.states.iter().enumerate() {
            writeln!(f, "State {i}:")?;
            for e in &state.epsilons {
                writeln!(f, "  {e}")?;
            }
            for t in &state.transitions {
                writeln!(f, "  {t}")?;
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, parse_tree::ERE};

    #[test]
    fn abbc_raw() {
        let nfa = WorkingNFA {
            states: vec![
                WorkingState::new().with_transition(1, 'a'.into()),
                WorkingState::new().with_transition(2, 'b'.into()),
                WorkingState::new()
                    .with_transition(3, 'c'.into())
                    .with_epsilon(1),
                WorkingState::new(),
            ],
        };
        println!("{}", nfa.to_tikz(true));

        assert!(nfa.test("abc"));
        assert!(nfa.test("abbc"));
        assert!(nfa.test("abbbc"));
        assert!(nfa.test("abbbbc"));

        assert!(!nfa.test("ac"));
        assert!(!nfa.test("abcc"));
        assert!(!nfa.test("bac"));
        assert!(!nfa.test("acb"));
    }

    #[test]
    fn phone_number() {
        let ere = ERE::parse_str(r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$").unwrap();
        let (tree, capture_groups) = SimplifiedTreeNode::from_ere(&ere, &Config::default());
        assert_eq!(capture_groups, 2);
        let nfa = WorkingNFA::new(&tree);
        println!("{}", nfa.to_tikz(true));

        assert!(nfa.test("012-345-6789"));
        assert!(nfa.test("987-654-3210"));
        assert!(nfa.test("+1 555-555-5555"));
        assert!(nfa.test("123-555-9876"));

        assert!(!nfa.test("abcd"));
        assert!(!nfa.test("0123456789"));
        assert!(!nfa.test("012--345-6789"));
        assert!(!nfa.test("(555) 555-5555"));
        assert!(!nfa.test("1 555-555-5555"));
    }

    #[test]
    fn double_loop() {
        let ere = ERE::parse_str(r"^.*(.*)*$").unwrap();
        let (tree, capture_groups) = SimplifiedTreeNode::from_ere(&ere, &Config::default());
        assert_eq!(capture_groups, 2);
        let nfa = WorkingNFA::new(&tree);
        // println!("{}", nfa.to_tikz(true));

        assert!(nfa.test(""));
        assert!(nfa.test("asdf"));
        assert!(nfa.test("1234567"));
        assert!(nfa.test("0"));

        assert!(!nfa.test("\0"));
    }

    #[test]
    fn good_anchored_start() {
        let ere = ERE::parse_str(r"^a|b*^c|d^|n").unwrap();
        let (tree, capture_groups) = SimplifiedTreeNode::from_ere(&ere, &Config::default());
        assert_eq!(capture_groups, 1);
        let nfa = WorkingNFA::new(&tree);
        // println!("{}", nfa.to_tikz(true));

        assert!(nfa.test("a"));
        assert!(nfa.test("c"));
        assert!(nfa.test("cq"));
        assert!(nfa.test("wwwnwww"));

        assert!(!nfa.test(""));
        assert!(!nfa.test("qb"));
        assert!(!nfa.test("qc"));
        assert!(!nfa.test("b"));
        assert!(!nfa.test("bc"));
        assert!(!nfa.test("bbbbbbc"));
        assert!(!nfa.test("d"));
    }

    #[test]
    fn good_anchored_end() {
        let ere = ERE::parse_str(r"a$|b$c*|$d|n").unwrap();
        let (tree, capture_groups) = SimplifiedTreeNode::from_ere(&ere, &Config::default());
        assert_eq!(capture_groups, 1);
        let nfa = WorkingNFA::new(&tree);
        println!("{}", nfa.to_tikz(true));

        assert!(nfa.test("a"));
        assert!(nfa.test("b"));
        assert!(nfa.test("qb"));
        assert!(nfa.test("wwwnwww"));

        assert!(!nfa.test(""));
        assert!(!nfa.test("bq"));
        assert!(!nfa.test("qc"));
        assert!(!nfa.test("c"));
        assert!(!nfa.test("bc"));
        assert!(!nfa.test("bcccccc"));
        assert!(!nfa.test("d"));
    }

    #[test]
    fn range_digit() {
        let ere = ERE::parse_str(r"^[[:digit:].]$").unwrap();
        let (tree, capture_groups) = SimplifiedTreeNode::from_ere(&ere, &Config::default());
        assert_eq!(capture_groups, 1);
        let nfa = WorkingNFA::new(&tree);
        // println!("{}", nfa.to_tikz(true));

        assert!(nfa.test("0"));
        assert!(nfa.test("1"));
        assert!(nfa.test("9"));
        assert!(nfa.test("."));

        assert!(!nfa.test(""));
        assert!(!nfa.test("a"));
        assert!(!nfa.test("11"));
        assert!(!nfa.test("1."));
        assert!(!nfa.test(".2"));
        assert!(!nfa.test("09"));
        assert!(!nfa.test("d"));
    }
}
