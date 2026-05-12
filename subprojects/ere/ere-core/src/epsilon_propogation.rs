use crate::{
    working_nfa::{EpsilonType, WorkingNFA},
    working_u8_nfa::U8NFA,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tag {
    /// Marks that the start of capture group `{0}` should be updated to the current offset.
    StartCapture(usize),
    /// Marks that the end of capture group `{0}` should be updated to the current offset.
    EndCapture(usize),
}
impl Tag {
    /// Returns the capture group number this tag is associated with.
    pub fn capture_group(&self) -> usize {
        return match self {
            Tag::StartCapture(group_num) | Tag::EndCapture(group_num) => *group_num,
        };
    }
}

/// Represents a squashed sequence of epsilon transitions
#[derive(Clone, PartialEq, Eq)]
pub struct EpsilonPropogation<T = Tag>
where
    T: Clone + PartialEq + Eq,
{
    /// The state it ends up at
    pub state: usize,
    /// Changes to tags
    pub update_tags: Vec<T>,
    /// If it contained a start anchor
    pub start_only: bool,
    /// If it contained an end anchor
    pub end_only: bool,
}

impl EpsilonPropogation {
    /// ## Params
    /// - `nfa` is the original nfa
    /// - `state` is the index of the state in `nfa.states`
    pub fn calculate_epsilon_propogations_char(
        nfa: &WorkingNFA,
        state: usize,
    ) -> Vec<EpsilonPropogation<Tag>> {
        let WorkingNFA { states } = nfa;
        // reduce epsilons to occur in a single step
        let mut new_threads = vec![];
        fn traverse(
            thread: EpsilonPropogation,
            states: &Vec<crate::working_nfa::WorkingState>,
            out: &mut Vec<EpsilonPropogation>,
        ) {
            out.push(thread.clone());
            for e in &states[thread.state].epsilons {
                let mut new_thread = thread.clone();
                new_thread.state = e.to;
                match e.special {
                    EpsilonType::None => {}
                    EpsilonType::StartAnchor => new_thread.start_only = true,
                    EpsilonType::EndAnchor => new_thread.end_only = true,
                    EpsilonType::StartCapture(capture_group) => {
                        if !new_thread
                            .update_tags
                            .contains(&Tag::StartCapture(capture_group))
                        {
                            new_thread
                                .update_tags
                                .push(Tag::StartCapture(capture_group));
                        }
                    }
                    EpsilonType::EndCapture(capture_group) => {
                        if !new_thread
                            .update_tags
                            .contains(&Tag::EndCapture(capture_group))
                        {
                            new_thread.update_tags.push(Tag::EndCapture(capture_group));
                        }
                    }
                }

                if !out.contains(&new_thread) {
                    traverse(new_thread, states, out);
                }
            }
        }
        traverse(
            EpsilonPropogation {
                state,
                update_tags: Vec::new(),
                start_only: false,
                end_only: false,
            },
            states,
            &mut new_threads,
        );
        return new_threads;
    }

    /// Will maintain priority order
    ///
    /// ## Params
    /// - `nfa` is the original nfa
    /// - `state` is the index of the state in `nfa.states`
    pub fn calculate_epsilon_propogations_u8(
        nfa: &U8NFA,
        state: usize,
    ) -> Vec<EpsilonPropogation<Tag>> {
        let U8NFA { states } = nfa;
        // reduce epsilons to occur in a single step
        let mut new_threads = vec![];
        fn traverse(
            thread: EpsilonPropogation,
            states: &Vec<crate::working_u8_nfa::U8State>,
            out: &mut Vec<EpsilonPropogation>,
        ) {
            out.push(thread.clone());
            for e in &states[thread.state].epsilons {
                let mut new_thread = thread.clone();
                new_thread.state = e.to;
                match e.special {
                    EpsilonType::None => {}
                    EpsilonType::StartAnchor => new_thread.start_only = true,
                    EpsilonType::EndAnchor => new_thread.end_only = true,
                    EpsilonType::StartCapture(capture_group) => {
                        if !new_thread
                            .update_tags
                            .contains(&Tag::StartCapture(capture_group))
                        {
                            new_thread
                                .update_tags
                                .push(Tag::StartCapture(capture_group));
                        }
                    }
                    EpsilonType::EndCapture(capture_group) => {
                        if !new_thread
                            .update_tags
                            .contains(&Tag::EndCapture(capture_group))
                        {
                            new_thread.update_tags.push(Tag::EndCapture(capture_group));
                        }
                    }
                }

                if !out.contains(&new_thread) {
                    traverse(new_thread, states, out);
                }
            }
        }
        traverse(
            EpsilonPropogation {
                state,
                update_tags: Vec::new(),
                start_only: false,
                end_only: false,
            },
            states,
            &mut new_threads,
        );
        return new_threads;
    }
}
