//! Implements a simplified intermediate representation of a regular expression.

use std::num::NonZeroUsize;

use crate::{config::Config, parse_tree::*};

/// For translation between our parsed [`ERE`] and the [`crate::working_nfa::WorkingNFA`]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SimplifiedTreeNode {
    /// Translates to `epsilon`
    Empty,
    Symbol(Atom),
    Union(Vec<SimplifiedTreeNode>),
    /// `Capture(child, group_num)`
    Capture(Box<SimplifiedTreeNode>, usize),
    Concat(Vec<SimplifiedTreeNode>),
    /// `Repeat(child, times)`
    Repeat(Box<SimplifiedTreeNode>, NonZeroUsize),
    /// `UpTo(child, max_times, longest)`
    UpTo(Box<SimplifiedTreeNode>, NonZeroUsize, bool),
    /// `Star(child, longest)`
    Star(Box<SimplifiedTreeNode>, bool),
    Start,
    End,
    Never,
}
impl SimplifiedTreeNode {
    pub fn optional(self, longest: bool) -> SimplifiedTreeNode {
        return self.upto(1, longest);
    }
    pub fn union(self, other: SimplifiedTreeNode) -> SimplifiedTreeNode {
        if let SimplifiedTreeNode::Union(mut u) = self {
            u.push(other);
            return SimplifiedTreeNode::Union(u);
        }
        return SimplifiedTreeNode::Union(vec![self, other]);
    }
    pub fn capture(self, group_num: usize) -> SimplifiedTreeNode {
        return SimplifiedTreeNode::Capture(Box::new(self), group_num);
    }
    pub fn concat(self, other: SimplifiedTreeNode) -> SimplifiedTreeNode {
        if let SimplifiedTreeNode::Concat(mut c) = self {
            c.push(other);
            return SimplifiedTreeNode::Concat(c);
        }
        return SimplifiedTreeNode::Concat(vec![self, other]);
    }
    pub fn repeat(self, count: usize) -> SimplifiedTreeNode {
        return match NonZeroUsize::new(count) {
            Some(count) => SimplifiedTreeNode::Repeat(self.into(), count),
            None => SimplifiedTreeNode::Empty,
        };
    }
    pub fn upto(self, count: usize, longest: bool) -> SimplifiedTreeNode {
        return match NonZeroUsize::new(count) {
            Some(count) => SimplifiedTreeNode::UpTo(self.into(), count, longest),
            None => SimplifiedTreeNode::Empty,
        };
    }
    pub fn star(self, longest: bool) -> SimplifiedTreeNode {
        return SimplifiedTreeNode::Star(self.into(), longest);
    }

    // /// A non-optimized, backtracking implementation
    // /// - `start` is the index of the original `text` it starts at
    // ///
    // /// # Returns
    // /// The number of matched symbols, or `None` if no match is found.
    // fn _check(&self, text: &str, start: usize, variation: usize) -> Option<usize> {
    //     return match self {
    //         SimplifiedTreeNode::Empty => Some(0),
    //         SimplifiedTreeNode::Symbol(atom) => atom.check(text.chars().next()?).then_some(1),
    //         SimplifiedTreeNode::Union(vec) => todo!(),
    //         SimplifiedTreeNode::Capture(node, _) => node._check(text, start),
    //         SimplifiedTreeNode::Concat(vec) => {
    //             let mut size = 0;
    //             for node in vec {
    //                 size += node._check(&text[size..], start + size)?;
    //             }
    //             Some(size)
    //         }
    //         SimplifiedTreeNode::Repeat(node, n) => {
    //             let mut size = 0;
    //             for _ in 0..(*n) {
    //                 size += node._check(&text[size..], start + size)?;
    //             }
    //             Some(size)
    //         }
    //         SimplifiedTreeNode::UpTo(node, n) => todo!(),
    //         SimplifiedTreeNode::Star(node) => todo!(),
    //         SimplifiedTreeNode::Start if start == 0 => Some(0),
    //         SimplifiedTreeNode::Start => None,
    //         SimplifiedTreeNode::End if start == text.len() => Some(0),
    //         SimplifiedTreeNode::End => None,
    //         SimplifiedTreeNode::Never => None,
    //     };
    // }
    // /// A non-optimized, backtracking implementation
    // pub fn check(&self, text: &str) -> bool {
    //     for i in 0..text.len() {
    //         if let Some(_) = self._check(&text[..i], i) {
    //             return true;
    //         }
    //     }
    //     return self._check("", text.len()).is_some();
    // }

    /// An upper bound for matched text length, in bytes.
    /// If possibly infinite, returns `None`.
    pub fn max_bytes(&self) -> Option<usize> {
        return match self {
            SimplifiedTreeNode::Empty => Some(0),
            SimplifiedTreeNode::Symbol(atom) => {
                let range = atom.to_ranges().last()?.clone();
                Some(range.end().len_utf8())
            }
            SimplifiedTreeNode::Union(nodes) if nodes.is_empty() => Some(0),
            SimplifiedTreeNode::Union(nodes) => nodes
                .iter()
                .map(SimplifiedTreeNode::max_bytes)
                .reduce(|a, b| Some(std::cmp::max(a?, b?)))?,
            SimplifiedTreeNode::Capture(node, _) => node.max_bytes(),
            SimplifiedTreeNode::Concat(nodes) if nodes.is_empty() => Some(0),
            SimplifiedTreeNode::Concat(nodes) => {
                nodes.iter().map(SimplifiedTreeNode::max_bytes).sum()
            }
            SimplifiedTreeNode::Repeat(node, times) => Some(node.max_bytes()? * times.get()),
            SimplifiedTreeNode::UpTo(node, times, _) => Some(node.max_bytes()? * times.get()),
            SimplifiedTreeNode::Star(_, _) => None,
            SimplifiedTreeNode::Start => Some(0),
            SimplifiedTreeNode::End => Some(0),
            SimplifiedTreeNode::Never => Some(0),
        };
    }

    /// A lower bound for matched text length, in bytes.
    pub fn min_bytes(&self) -> usize {
        return match self {
            SimplifiedTreeNode::Empty => 0,
            SimplifiedTreeNode::Symbol(atom) => {
                let Some(range) = atom.to_ranges().first().cloned() else {
                    return 0;
                };
                range.start().len_utf8()
            }
            SimplifiedTreeNode::Union(nodes) => nodes
                .iter()
                .map(SimplifiedTreeNode::min_bytes)
                .min()
                .unwrap_or(0),
            SimplifiedTreeNode::Capture(node, _) => node.min_bytes(),
            SimplifiedTreeNode::Concat(nodes) => {
                nodes.iter().map(SimplifiedTreeNode::min_bytes).sum()
            }
            SimplifiedTreeNode::Repeat(node, times) => node.min_bytes() * times.get(),
            SimplifiedTreeNode::UpTo(node, times, _) => node.min_bytes() * times.get(),
            SimplifiedTreeNode::Star(_, _) => 0,
            SimplifiedTreeNode::Start => 0,
            SimplifiedTreeNode::End => 0,
            SimplifiedTreeNode::Never => 0,
        };
    }
}
impl SimplifiedTreeNode {
    fn from_sub_ere(
        value: &ERE,
        mut group_num: usize,
        config: &Config,
    ) -> (SimplifiedTreeNode, usize) {
        let parts = value
            .0
            .iter()
            .map(|part| {
                let (new_node, new_group_num) =
                    SimplifiedTreeNode::from_ere_branch(&part, group_num, config);
                group_num = new_group_num;
                new_node
            })
            .collect();
        return (SimplifiedTreeNode::Union(parts), group_num);
    }
    fn from_ere_branch(
        value: &EREBranch,
        mut group_num: usize,
        config: &Config,
    ) -> (SimplifiedTreeNode, usize) {
        let parts = value
            .0
            .iter()
            .map(|part| {
                let (new_node, new_group_num) =
                    SimplifiedTreeNode::from_ere_part(&part, group_num, config);
                group_num = new_group_num;
                new_node
            })
            .collect();
        return (SimplifiedTreeNode::Concat(parts), group_num);
    }
    fn from_ere_part(
        value: &EREPart,
        group_num: usize,
        config: &Config,
    ) -> (SimplifiedTreeNode, usize) {
        return match value {
            EREPart::Single(expr) => {
                SimplifiedTreeNode::from_ere_expression(expr, group_num, config)
            }
            EREPart::Quantified(expr, quantifier) => {
                let (child, group_num) =
                    SimplifiedTreeNode::from_ere_expression(expr, group_num, config);
                let longest = config.quantifiers_prefer_longest() ^ quantifier.alt;
                let part = match &quantifier.quantifier {
                    QuantifierType::Star => child.star(longest),
                    QuantifierType::Plus => child.clone().concat(child.star(longest)),
                    QuantifierType::QuestionMark => child.optional(longest),
                    QuantifierType::Multiple(n) => child.repeat(*n as usize),
                    QuantifierType::Range(n, None) => child
                        .clone()
                        .repeat(*n as usize)
                        .concat(child.star(longest)),
                    QuantifierType::Range(n, Some(m)) => match m.checked_sub(*n) {
                        None => SimplifiedTreeNode::Never,
                        Some(0) => child.repeat(*n as usize),
                        Some(r) => child
                            .clone()
                            .repeat(*n as usize)
                            .concat(child.upto(r as usize, longest)),
                    },
                };
                (part, group_num)
            }
            EREPart::Start => (SimplifiedTreeNode::Start, group_num),
            EREPart::End => (SimplifiedTreeNode::End, group_num),
        };
    }
    fn from_ere_expression(
        value: &EREExpression,
        group_num: usize,
        config: &Config,
    ) -> (SimplifiedTreeNode, usize) {
        return match value {
            EREExpression::Atom(atom) => (atom.clone().into(), group_num),
            EREExpression::Subexpression(ere) => {
                let (capture, next_group_num) =
                    SimplifiedTreeNode::from_sub_ere(ere, group_num + 1, config);
                (
                    SimplifiedTreeNode::Capture(capture.into(), group_num),
                    next_group_num,
                )
            }
        };
    }
    /// Returns the simplified tree, along with the number of capture groups (full expression is group 0)
    pub fn from_ere(value: &ERE, config: &Config) -> (SimplifiedTreeNode, usize) {
        let (root, groups) = SimplifiedTreeNode::from_sub_ere(value, 1, config);
        return (SimplifiedTreeNode::Capture(Box::new(root), 0), groups);
    }

    /// [`SimplifiedTreeNode::from_ere`] except it doesn't wrap in the capture group 0
    pub(crate) fn from_ere_no_group0(value: &ERE, config: &Config) -> (SimplifiedTreeNode, usize) {
        return SimplifiedTreeNode::from_sub_ere(value, 1, config);
    }
}
impl From<ERE> for SimplifiedTreeNode {
    fn from(value: ERE) -> Self {
        return SimplifiedTreeNode::from_ere(&value, &Config::default()).0;
    }
}
impl From<Atom> for SimplifiedTreeNode {
    fn from(value: Atom) -> Self {
        return SimplifiedTreeNode::Symbol(value);
    }
}
