//! Implements the [ERE](https://en.wikibooks.org/wiki/Regular_Expressions/POSIX-Extended_Regular_Expressions) parser
//! and primitive types (like [`Atom`]).

use std::{
    fmt::{Display, Write},
    ops::RangeInclusive,
};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// On input string `text: &'a str` with options `{<prefix> => T, ..}`,
/// it returns `Option<(&'a str, T)>` based on which literal prefix is matched
///
/// Example:
/// ```
/// let text = "asdf";
/// let test = match_prefix!(text, {
///     "fdsa" => 0,
///     "qwerty" => 1,
///     "as" => 2,
///     "asd" => 3,
/// });
/// assert_eq!(test, Some("df", 2));
/// ```
macro_rules! match_prefix {
    ($text:ident, { }) => (::core::option::Option::None);
    ($text:ident, {
        $x:literal => $y:expr,
        $($xs:literal => $ys:expr,)*
    }) => {
        if let ::core::option::Option::Some(rest) = str::strip_prefix($text, $x) {
            ::core::option::Option::Some((rest, $y))
        } $(else if let ::core::option::Option::Some(rest) = str::strip_prefix($text, $xs) {
            ::core::option::Option::Some((rest, $ys))
        })* else {
            ::core::option::Option::None
        }
    };
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RegexParseError {
    /// A capture group (e.g. `(abc)`) was unterminated
    #[error("A capture group is not properly terminated.")]
    UnterminatedCaptureGroup,
    /// A character class set (e.g. `[a-z]`) was unterminated
    #[error("A character class set is not properly terminated.")]
    UnterminatedCharSet,
    /// An escaped symbol was invalid.
    #[error("An unknown regex escape (not rust string escape) was found: `\\{0}`.")]
    InvalidEscapedChar(char),
    #[error("An atom was expected but was invalid due to being a special character: `{0}`.")]
    InvalidAtom(char),
    /// Should only happen in debug settings, since it happens only when we call `Atom::take("")`
    #[error("An atom was expected but was not found due to the end of the string.")]
    UnexpectedEOF,
}

/// A represents a [POSIX-compliant ERE](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap09.html).
/// Primarily intended for use as a parser.
#[derive(Clone)]
pub struct ERE(
    /// Should always have at least one branch, even if it is empty.
    pub(crate) Vec<EREBranch>,
);
impl ERE {
    /// Goes until a non-nested `)` or the end of the string (does not consume).
    fn take<'a>(rest: &'a str) -> Result<(&'a str, ERE), RegexParseError> {
        let mut branches = Vec::new();
        let (mut rest, branch) = EREBranch::take(rest)?;
        branches.push(branch);
        loop {
            if rest.is_empty() || rest.starts_with(')') {
                break;
            }
            let Some(branch_start) = rest.strip_prefix('|') else {
                break;
            };
            let (new_rest, branch) = EREBranch::take(branch_start)?;
            rest = new_rest;
            branches.push(branch);
        }
        return Ok((rest, ERE(branches)));
    }

    pub fn parse_str(input: &str) -> Result<Self, RegexParseError> {
        let Ok(("", ere)) = ERE::take(&input) else {
            // The only reason we would have `rest` left over is if
            // we hit a ')'. Since there is no opening '(', this is treated as an invalid atom.
            return Err(RegexParseError::InvalidAtom(')'));
        };
        return Ok(ere);
    }
    pub(crate) fn parse_str_syn(string: &str, span: proc_macro2::Span) -> syn::Result<Self> {
        return ERE::parse_str(&string).map_err(|err| {
            syn::Error::new(
                span,
                format_args!("While parsing regular expression:\n{err}"),
            )
        });
    }
}
impl syn::parse::Parse for ERE {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let literal: syn::LitStr = input.parse()?;
        let string = literal.value();
        return ERE::parse_str_syn(&string, literal.span());
    }
}
impl Display for ERE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut it = self.0.iter();
        let Some(first) = it.next() else {
            return Ok(());
        };
        write!(f, "{first}")?;
        for part in it {
            write!(f, "|{part}")?;
        }
        return Ok(());
    }
}

#[derive(Clone)]
pub(crate) struct EREBranch(
    /// May be empty.
    pub(crate) Vec<EREPart>,
);
impl EREBranch {
    /// May be empty.
    /// If it is passed `")"` it will return `Ok((")", EREBranch(vec![]))`,
    /// so end of group should be checked before taking a branch.
    ///
    /// ## Returns
    /// - `Ok((rest, branch))` if we successfully found a branch
    /// - `Err(err)` if parsing failed (probably an invalid regex)
    fn take<'a>(mut rest: &'a str) -> Result<(&'a str, EREBranch), RegexParseError> {
        let mut parts = Vec::new();
        while !rest.is_empty() && !rest.starts_with(')') && !rest.starts_with('|') {
            let (new_rest, part) = EREPart::take(rest)?;
            rest = new_rest;
            parts.push(part);
        }
        return Ok((rest, EREBranch(parts)));
    }
}
impl Display for EREBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for part in &self.0 {
            write!(f, "{part}")?;
        }
        return Ok(());
    }
}

#[derive(Clone)]
pub(crate) enum EREPart {
    Single(EREExpression),
    Quantified(EREExpression, Quantifier),
    Start,
    End,
}
impl EREPart {
    fn take<'a>(rest: &'a str) -> Result<(&'a str, EREPart), RegexParseError> {
        if let Some(rest) = rest.strip_prefix('^') {
            return Ok((rest, EREPart::Start));
        } else if let Some(rest) = rest.strip_prefix('$') {
            return Ok((rest, EREPart::End));
        }

        let (rest, expr) = if let Some(rest) = rest.strip_prefix('(') {
            let (rest, ere) = ERE::take(rest)?;
            let Some(rest) = rest.strip_prefix(')') else {
                return Err(RegexParseError::UnterminatedCaptureGroup);
            };
            (rest, EREExpression::Subexpression(ere))
        } else {
            let (rest, atom) = Atom::take(rest)?;
            (rest, EREExpression::Atom(atom))
        };

        let part = match Quantifier::take(rest) {
            Some((rest, quantifier)) => (rest, EREPart::Quantified(expr, quantifier)),
            None => (rest, EREPart::Single(expr)),
        };
        return Ok(part);
    }
}
impl Display for EREPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            EREPart::Single(expr) => write!(f, "{expr}"),
            EREPart::Quantified(expr, quantifier) => write!(f, "{expr}{quantifier}"),
            EREPart::Start => f.write_char('^'),
            EREPart::End => f.write_char('$'),
        };
    }
}

#[derive(Clone)]
pub(crate) enum EREExpression {
    Atom(Atom),
    Subexpression(ERE),
}
impl Display for EREExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            EREExpression::Atom(atom) => write!(f, "{atom}"),
            EREExpression::Subexpression(ere) => write!(f, "({ere})"),
        };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum QuantifierType {
    Star,
    Plus,
    QuestionMark,
    /// The equivalent to a range specifier with fixed size
    Multiple(u32),
    /// `(u32, None)` is unbounded, `(u32, Some(u32))` is bounded
    Range(u32, Option<u32>),
}

impl QuantifierType {
    /// The minimum this quantifier matches, inclusive
    #[inline]
    const fn min(&self) -> u32 {
        return match self {
            QuantifierType::Star => 0,
            QuantifierType::Plus => 1,
            QuantifierType::QuestionMark => 0,
            QuantifierType::Multiple(n) => *n,
            QuantifierType::Range(n, _) => *n,
        };
    }
    /// The maximum this quantifier matches, inclusive. If `None`, it is unbounded
    #[inline]
    const fn max(&self) -> Option<u32> {
        return match self {
            QuantifierType::Star => None,
            QuantifierType::Plus => None,
            QuantifierType::QuestionMark => Some(1),
            QuantifierType::Multiple(n) => Some(*n),
            QuantifierType::Range(_, m) => *m,
        };
    }
}
impl Display for QuantifierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            QuantifierType::Star => f.write_char('*'),
            QuantifierType::Plus => f.write_char('+'),
            QuantifierType::QuestionMark => f.write_char('?'),
            QuantifierType::Multiple(n) => write!(f, "{{{n}}}"),
            QuantifierType::Range(n, None) => write!(f, "{{{n},}}"),
            QuantifierType::Range(n, Some(m)) => write!(f, "{{{n},{m}}}"),
        };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct Quantifier {
    pub quantifier: QuantifierType,
    /// By default, (unless the `REG_MINIMAL` flag is set), quantifiers should prefer the longest valid
    /// match for quantifiers. However, if an additional `?` occurs after the quantifier, it should prefer the
    /// shortest (or longest if `REG_MINIMAL` makes default shortest).
    pub alt: bool,
}
impl Quantifier {
    #[inline]
    fn take<'a>(rest: &'a str) -> Option<(&'a str, Quantifier)> {
        let mut it = rest.chars();
        let (rest, quantifier) = match it.next() {
            Some('*') => (it.as_str(), QuantifierType::Star),
            Some('+') => (it.as_str(), QuantifierType::Plus),
            Some('?') => (it.as_str(), QuantifierType::QuestionMark),
            Some('{') => {
                let (inside, rest) = it.as_str().split_once('}')?;
                match inside.split_once(',') {
                    None => (rest, QuantifierType::Multiple(inside.parse().ok()?)),
                    Some((min, "")) => (rest, QuantifierType::Range(min.parse().ok()?, None)),
                    Some((min, max)) => (
                        rest,
                        QuantifierType::Range(min.parse().ok()?, Some(max.parse().ok()?)),
                    ),
                }
            }
            _ => return None,
        };
        if let Some(rest) = rest.strip_prefix('?') {
            return Some((
                rest,
                Quantifier {
                    quantifier,
                    alt: true,
                },
            ));
        } else {
            return Some((
                rest,
                Quantifier {
                    quantifier,
                    alt: false,
                },
            ));
        }
    }
}
impl Display for Quantifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.quantifier.fmt(f)?;
        if self.alt {
            return f.write_char('?');
        } else {
            return Ok(());
        }
    }
}
impl From<QuantifierType> for Quantifier {
    fn from(quantifier: QuantifierType) -> Self {
        return Quantifier {
            quantifier,
            alt: false,
        };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CharClass {
    /// Matches anything but `NUL` (`'\0'`)
    Dot,
}
impl CharClass {
    pub const fn check(&self, c: char) -> bool {
        return match self {
            CharClass::Dot => c != '\0',
        };
    }
}
impl Display for CharClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            CharClass::Dot => f.write_char('.'),
        };
    }
}
impl ToTokens for CharClass {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            CharClass::Dot => tokens.extend(quote! {::ere::parse_tree::CharClass::Dot}),
        }
    }
}

/// Represents a part of an [`ERE`] that matches a single character.
/// For example, a single char `a`, a char class `.`, or a bracket expression `[a-z]`.
///
/// Equality checks are semantic:
/// ```
/// use ere_core::parse_tree::Atom;
/// assert_eq!(
///     "[abcd]".parse::<Atom>(),
///     "[a-d]".parse::<Atom>(),
/// );
/// assert_eq!(
///     "[a-z]".parse::<Atom>(),
///     "[[:lower:]]".parse::<Atom>(),
/// );
/// ```
#[derive(Debug, Clone)]
pub enum Atom {
    /// Includes normal char and escaped chars
    NormalChar(char),
    CharClass(CharClass),
    /// A matching bracket expression
    MatchingList(Vec<BracketExpressionTerm>),
    /// A nonmatching bracket expression
    NonmatchingList(Vec<BracketExpressionTerm>),
}
impl From<char> for Atom {
    fn from(value: char) -> Self {
        return Atom::NormalChar(value);
    }
}
impl From<CharClass> for Atom {
    fn from(value: CharClass) -> Self {
        return Atom::CharClass(value);
    }
}
impl std::str::FromStr for Atom {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(("", atom)) = Atom::take(s) else {
            return Err(());
        };
        return Ok(atom);
    }
}

impl Atom {
    /// ## Returns
    /// - `Ok((rest, atom))` if an atom was successfully found at the start
    /// - `Err(err)` if the input does not start with a valid atom.
    fn take<'a>(rest: &'a str) -> Result<(&'a str, Atom), RegexParseError> {
        let mut it = rest.chars();
        return match it.next() {
            Some('\\') => match it.next() {
                Some(c) if is_escapable_character(c) => Ok((it.as_str(), Atom::NormalChar(c))),
                Some(c) => Err(RegexParseError::InvalidEscapedChar(c)),
                None => Err(RegexParseError::InvalidAtom('\\')),
            },
            Some('.') => Ok((it.as_str(), CharClass::Dot.into())),
            Some('[') => {
                let mut rest = it.as_str();
                let mut items = Vec::new();
                let none_of = if let Some(new_rest) = rest.strip_prefix('^') {
                    rest = new_rest;
                    true
                } else {
                    false
                };
                if let Some(new_rest) = rest.strip_prefix(']') {
                    rest = new_rest;
                    items.push(BracketExpressionTerm::Single(']'));
                }
                loop {
                    if let Some(new_rest) = rest.strip_prefix(']') {
                        // End of the bracket expression
                        rest = new_rest;
                        break;
                    } else if let Some((new_rest, class)) = BracketCharClass::take(rest) {
                        // A bracket char class
                        rest = new_rest;
                        items.push(BracketExpressionTerm::CharClass(class));
                    } else {
                        // Normal
                        let mut it = rest.chars();
                        let first = it.next().ok_or(RegexParseError::UnterminatedCharSet)?;
                        rest = it.as_str();
                        if let '-' = it.next().ok_or(RegexParseError::UnterminatedCharSet)? {
                            let second = it.next().ok_or(RegexParseError::UnterminatedCharSet)?;
                            rest = it.as_str();
                            if second == ']' {
                                // it's just two characters at the end
                                items.push(BracketExpressionTerm::Single(first));
                                items.push(BracketExpressionTerm::Single('-'));
                                break;
                            } else {
                                // it's a range
                                items.push(BracketExpressionTerm::Range(first, second));
                            }
                        } else {
                            items.push(BracketExpressionTerm::Single(first));
                        }
                    }
                }

                if none_of {
                    return Ok((rest, Atom::NonmatchingList(items)));
                } else {
                    return Ok((rest, Atom::MatchingList(items)));
                }
            }
            Some(c) if !is_special_character(c) => Ok((it.as_str(), Atom::NormalChar(c))),
            Some(c) => Err(RegexParseError::InvalidAtom(c)),
            None => Err(RegexParseError::UnexpectedEOF),
        };
    }
    pub fn check(&self, c: char) -> bool {
        return match self {
            Atom::NormalChar(a) => *a == c,
            Atom::CharClass(char_class) => char_class.check(c),
            Atom::MatchingList(vec) => vec.into_iter().any(|b| b.check(c)),
            Atom::NonmatchingList(vec) => !vec.into_iter().any(|b| b.check(c)),
        };
    }
    pub(crate) fn serialize_check(&self) -> TokenStream {
        let ranges = self.to_ranges();
        let mut stream = TokenStream::new();
        for range in ranges {
            let start = range.start();
            let end = range.end();
            stream.extend(quote! { (#start <= c && c <= #end) || });
        }
        return quote! {(#stream false)};
    }
    /// Produces the sorted, minimal set of ranges to represent the Atom.
    ///
    /// Example:
    /// ```
    /// use ere_core::parse_tree::Atom;
    /// assert_eq!(
    ///     "[a-z2-9A-X0-1YZ[:xdigit:]]".parse::<Atom>().unwrap().to_ranges(),
    ///     vec!['0'..='9', 'A'..='Z', 'a'..='z'],
    /// );
    /// ```
    pub fn to_ranges(&self) -> Vec<RangeInclusive<char>> {
        /// Sorts and combines ranges
        fn combine_ranges_chars(
            mut ranges: Vec<RangeInclusive<char>>,
        ) -> Vec<RangeInclusive<char>> {
            ranges.sort_by_key(|range| *range.start());
            let Some((first_range, terms)) = ranges.split_first_chunk::<1>() else {
                return Vec::new();
            };
            let mut reduced_terms = Vec::new();

            let mut current_start = *first_range[0].start();
            let mut current_end = *first_range[0].end();
            for term in terms {
                if term.is_empty() {
                    continue;
                } else if *term.start() <= current_end || (current_end..=*term.start()).count() == 2
                {
                    // the next term is overlapping (or starts immediately after) so combine them.
                    current_end = std::cmp::max(current_end, *term.end());
                } else {
                    reduced_terms.push(current_start.clone()..=current_end.clone());
                    current_start = *term.start();
                    current_end = *term.end();
                }
            }
            reduced_terms.push(current_start.clone()..=current_end.clone());
            return reduced_terms;
        }

        return match self {
            Atom::NormalChar(c) => vec![*c..=*c],
            Atom::CharClass(CharClass::Dot) => vec!['\x01'..=char::MAX],
            Atom::MatchingList(_terms) => {
                let mut terms = Vec::new();
                for term in _terms {
                    match term {
                        crate::parse_tree::BracketExpressionTerm::Single(c) => {
                            terms.push(*c..=*c);
                        }
                        crate::parse_tree::BracketExpressionTerm::Range(a, b) => {
                            terms.push(*a..=*b);
                        }
                        crate::parse_tree::BracketExpressionTerm::CharClass(class) => {
                            terms.extend_from_slice(class.to_ranges());
                        }
                    }
                }
                combine_ranges_chars(terms)
            }
            Atom::NonmatchingList(terms) => {
                let mut ranges = Vec::new();
                ranges.push('\0'..=char::MAX);
                for term in terms.iter().flat_map(|term| term.to_ranges()) {
                    ranges = ranges
                        .iter()
                        .flat_map(|range| {
                            if term.end() < range.start() || range.end() < term.start() {
                                return vec![range.clone()]; // unchanged
                            }
                            let mut out = Vec::new();
                            if range.start() < term.start() {
                                let mut new_range = *range.start()..=*term.start();
                                new_range.next_back();
                                out.push(new_range); // part at start
                            }
                            if term.end() < range.end() {
                                let mut new_range = *term.end()..=*range.end();
                                new_range.next();
                                out.push(new_range); // part at end
                            }
                            return out;
                        })
                        .collect();
                }

                ranges.sort_by_key(|range: &RangeInclusive<char>| *range.start());
                ranges
            }
        };
    }
    pub fn max_bytes(&self) -> usize {
        match self.to_ranges().last() {
            None => 0,
            Some(range) => range.end().len_utf8(),
        }
    }
    pub fn min_bytes(&self) -> usize {
        match self.to_ranges().first() {
            None => 0,
            Some(range) => range.start().len_utf8(),
        }
    }
}
impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Atom::NormalChar(c) if is_escapable_character(*c) => write!(f, "\\{c}"),
            Atom::NormalChar(c) => f.write_char(*c),
            Atom::CharClass(c) => c.fmt(f),
            Atom::MatchingList(vec) => {
                f.write_char('[')?;
                for term in vec {
                    write!(f, "{term}")?;
                }
                f.write_char(']')
            }
            Atom::NonmatchingList(vec) => {
                f.write_str("[^")?;
                for term in vec {
                    write!(f, "{term}")?;
                }
                f.write_char(']')
            }
        };
    }
}
impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        return self.to_ranges() == other.to_ranges();
    }
}
impl Eq for Atom {}

/// From <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap09.html#tag_09_03_05>
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BracketCharClass {
    /// `[:alnum:]`
    Alphanumeric,
    /// `[:cntrl:]`
    Control,
    /// `[:lower:]`
    Lower,
    /// `[:space:]`
    Space,
    /// `[:alpha:]`
    Alphabet,
    /// `[:digit:]`
    Digit,
    /// `[:print:]`
    Print,
    /// `[:upper:]`
    Upper,
    /// `[:blank:]`
    Blank,
    /// `[:graph:]`
    Graphic,
    /// `[:punct:]`
    Punctuation,
    /// `[:xdigit:]`
    HexDigit,
}
impl BracketCharClass {
    pub const fn to_ranges(&self) -> &'static [RangeInclusive<char>] {
        return match self {
            BracketCharClass::Alphanumeric => &['0'..='9', 'A'..='Z', 'a'..='z'],
            BracketCharClass::Control => &['\0'..='\x1f', '\x7f'..='\x7f'],
            BracketCharClass::Lower => &['a'..='z'],
            BracketCharClass::Space => &['\t'..='\t', '\x0a'..='\x0d', ' '..=' '],
            BracketCharClass::Alphabet => &['A'..='Z', 'a'..='z'],
            BracketCharClass::Digit => &['0'..='9'],
            BracketCharClass::Print => &['\x20'..='\x7E'],
            BracketCharClass::Upper => &['A'..='Z'],
            BracketCharClass::Blank => &['\t'..='\t', ' '..=' '],
            BracketCharClass::Graphic => &['\x21'..='\x7e'],
            BracketCharClass::Punctuation => &[
                '\x21'..='\x2f',
                '\x3a'..='\x40',
                '\x5b'..='\x60',
                '\x7b'..='\x7e',
            ],
            BracketCharClass::HexDigit => &['0'..='9', 'A'..='F', 'a'..='f'],
        };
    }
    /// Checks matches to the char classes.
    pub const fn check_ascii(&self, c: char) -> bool {
        return match self {
            BracketCharClass::Alphanumeric => c.is_ascii_alphanumeric(),
            BracketCharClass::Control => c.is_ascii_control(),
            BracketCharClass::Lower => c.is_ascii_lowercase(),
            BracketCharClass::Space => c.is_ascii_whitespace() || c == '\x0b', // POSIX includes vertical tab
            BracketCharClass::Alphabet => c.is_ascii_alphabetic(),
            BracketCharClass::Digit => c.is_ascii_digit(),
            BracketCharClass::Print => matches!(c, '\x20'..='\x7E'),
            BracketCharClass::Upper => c.is_ascii_uppercase(),
            BracketCharClass::Blank => c == ' ' || c == '\t',
            BracketCharClass::Graphic => c.is_ascii_graphic(),
            BracketCharClass::Punctuation => c.is_ascii_punctuation(),
            BracketCharClass::HexDigit => c.is_ascii_hexdigit(),
        };
    }
    fn take<'a>(rest: &'a str) -> Option<(&'a str, BracketCharClass)> {
        let rest = rest.strip_prefix("[:")?;
        return match_prefix!(rest, {
            "alnum:]" => BracketCharClass::Alphanumeric,
            "cntrl:]" => BracketCharClass::Control,
            "lower:]" => BracketCharClass::Lower,
            "space:]" => BracketCharClass::Space,
            "alpha:]" => BracketCharClass::Alphabet,
            "digit:]" => BracketCharClass::Digit,
            "print:]" => BracketCharClass::Print,
            "upper:]" => BracketCharClass::Upper,
            "blank:]" => BracketCharClass::Blank,
            "graph:]" => BracketCharClass::Graphic,
            "punct:]" => BracketCharClass::Punctuation,
            "xdigit:]" => BracketCharClass::HexDigit,
        });
    }
}
impl Display for BracketCharClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            BracketCharClass::Alphanumeric => f.write_str("[:alnum:]"),
            BracketCharClass::Control => f.write_str("[:cntrl:]"),
            BracketCharClass::Lower => f.write_str("[:lower:]"),
            BracketCharClass::Space => f.write_str("[:space:]"),
            BracketCharClass::Alphabet => f.write_str("[:alpha:]"),
            BracketCharClass::Digit => f.write_str("[:digit:]"),
            BracketCharClass::Print => f.write_str("[:print:]"),
            BracketCharClass::Upper => f.write_str("[:upper:]"),
            BracketCharClass::Blank => f.write_str("[:blank:]"),
            BracketCharClass::Graphic => f.write_str("[:graph:]"),
            BracketCharClass::Punctuation => f.write_str("[:punct:]"),
            BracketCharClass::HexDigit => f.write_str("[:xdigit:]"),
        };
    }
}
impl ToTokens for BracketCharClass {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BracketCharClass::Alphanumeric => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Alphanumeric})
            }
            BracketCharClass::Control => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Control})
            }
            BracketCharClass::Lower => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Lower})
            }
            BracketCharClass::Space => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Space})
            }
            BracketCharClass::Alphabet => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Alphabet})
            }
            BracketCharClass::Digit => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Digit})
            }
            BracketCharClass::Print => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Print})
            }
            BracketCharClass::Upper => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Upper})
            }
            BracketCharClass::Blank => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Blank})
            }
            BracketCharClass::Graphic => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Graph})
            }
            BracketCharClass::Punctuation => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::Punctuation})
            }
            BracketCharClass::HexDigit => {
                tokens.extend(quote! {::ere::parse_tree::BracketCharClass::XDigit})
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BracketExpressionTerm {
    Single(char),
    Range(char, char),
    CharClass(BracketCharClass),
}
impl BracketExpressionTerm {
    pub const fn check(&self, c: char) -> bool {
        return match self {
            BracketExpressionTerm::Single(a) => *a == c,
            BracketExpressionTerm::Range(a, b) => *a <= c && c <= *b,
            BracketExpressionTerm::CharClass(class) => class.check_ascii(c),
        };
    }
    pub(crate) fn to_ranges(&self) -> Vec<RangeInclusive<char>> {
        return match self {
            BracketExpressionTerm::Single(c) => vec![*c..=*c],
            BracketExpressionTerm::Range(a, b) => vec![*a..=*b],
            BracketExpressionTerm::CharClass(bracket_char_class) => {
                bracket_char_class.to_ranges().to_vec()
            }
        };
    }
}
impl Display for BracketExpressionTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            BracketExpressionTerm::Single(c) => f.write_char(*c),
            BracketExpressionTerm::Range(first, second) => write!(f, "{first}-{second}"),
            BracketExpressionTerm::CharClass(class) => class.fmt(f),
        };
    }
}
impl From<char> for BracketExpressionTerm {
    fn from(value: char) -> Self {
        return BracketExpressionTerm::Single(value);
    }
}
impl From<BracketCharClass> for BracketExpressionTerm {
    fn from(value: BracketCharClass) -> Self {
        return BracketExpressionTerm::CharClass(value);
    }
}
impl ToTokens for BracketExpressionTerm {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            BracketExpressionTerm::Single(c) => quote! {
                ::ere::parse_tree::BracketExpressionTerm::Single(#c)
            },
            BracketExpressionTerm::Range(a, z) => quote! {
                ::ere::parse_tree::BracketExpressionTerm::Range(#a, #z)
            },
            BracketExpressionTerm::CharClass(char_class) => quote! {
                ::ere::parse_tree::BracketExpressionTerm::CharClass(#char_class)
            },
        });
    }
}

/// The characters that can only occur if quoted
#[inline]
const fn is_special_character(c: char) -> bool {
    return c == '^'
        || c == '.'
        || c == '['
        || c == '$'
        || c == '('
        || c == ')'
        || c == '|'
        || c == '*'
        || c == '+'
        || c == '?'
        || c == '{'
        || c == '\\';
}

/// The characters that can only occur if quoted
#[inline]
pub(crate) const fn is_escapable_character(c: char) -> bool {
    return is_special_character(c) || c == ']' || c == '}';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconstruction() {
        fn test_reconstruction(text: &str) {
            let (rest, ere) = match ERE::take(text) {
                Ok(ok) => ok,
                Err(err) => panic!("Failed to parse `{text}`, got error: {err}"),
            };
            assert!(rest.is_empty(), "{text} did not get used (left {rest})");
            let reconstructed = ere.to_string();
            assert_eq!(text, &reconstructed);
        }
        test_reconstruction("asdf");
        test_reconstruction("as+df123$");
        test_reconstruction("asd?f*123");
        test_reconstruction("^asdf123.*");

        test_reconstruction("a[|]");
        test_reconstruction("[^]a-z1-4A-X-]asdf");
        test_reconstruction("cd[X-]er");

        test_reconstruction("my word is [[:alnum:]_]");
        test_reconstruction("my word is [[:lower:][:digit:]_]");

        test_reconstruction("a|b");
        test_reconstruction("a|b|c");
        test_reconstruction("a(a(b|c)|d)|(g|f)b");
        test_reconstruction("(a|b)|(c|d){3}");

        test_reconstruction("a[y-z]{1,3}");
        test_reconstruction("a{3,}");
        test_reconstruction("a(efg){1,}");
    }

    #[test]
    fn parse_quantifiers() {
        assert_eq!(Quantifier::take(""), None);
        assert_eq!(
            Quantifier::take("+asdf"),
            Some(("asdf", QuantifierType::Plus.into()))
        );
        assert_eq!(
            Quantifier::take("*"),
            Some(("", QuantifierType::Star.into()))
        );
        assert_eq!(Quantifier::take("e?"), None);
        assert_eq!(Quantifier::take("{"), None);
        assert_eq!(Quantifier::take("{}"), None);
        assert_eq!(
            Quantifier::take("{1}"),
            Some(("", QuantifierType::Multiple(1).into()))
        );
        assert_eq!(
            Quantifier::take("{9}ee"),
            Some(("ee", QuantifierType::Multiple(9).into()))
        );
        assert_eq!(
            Quantifier::take("{10,}ee"),
            Some(("ee", QuantifierType::Range(10, None).into()))
        );
        assert_eq!(
            Quantifier::take("{0,11}ef"),
            Some(("ef", QuantifierType::Range(0, Some(11)).into()))
        );
        assert_eq!(Quantifier::take("{0,e11}ef"), None);
        assert_eq!(Quantifier::take("{0;11}ef"), None);
    }

    #[test]
    fn parse_atom_simple() {
        assert_eq!(Atom::take("a"), Ok(("", Atom::NormalChar('a'))));
        assert_eq!(Atom::take(r"abcd"), Ok(("bcd", Atom::NormalChar('a'))));
        assert_eq!(Atom::take(r"\\"), Ok(("", Atom::NormalChar('\\'))));
        assert_eq!(
            Atom::take(r"\[asdf\]"),
            Ok((r"asdf\]", Atom::NormalChar('[')))
        );
        assert_eq!(Atom::take(r"\."), Ok(("", Atom::NormalChar('.'))));
        assert_eq!(Atom::take(r" "), Ok(("", Atom::NormalChar(' '))));
        assert_eq!(Atom::take(r"\"), Err(RegexParseError::InvalidAtom('\\')));

        assert_eq!(Atom::take("."), Ok(("", Atom::CharClass(CharClass::Dot))));
        assert_eq!(Atom::take(".."), Ok((".", Atom::CharClass(CharClass::Dot))));
    }

    #[test]
    fn parse_atom_brackets() {
        assert_eq!(
            Atom::take("[ab]"),
            Ok((
                "",
                Atom::MatchingList(vec![
                    BracketExpressionTerm::Single('a'),
                    BracketExpressionTerm::Single('b'),
                ])
            ))
        );
        assert_eq!(
            Atom::take("[]ab]"),
            Ok((
                "",
                Atom::MatchingList(vec![
                    BracketExpressionTerm::Single(']'),
                    BracketExpressionTerm::Single('a'),
                    BracketExpressionTerm::Single('b'),
                ])
            ))
        );
        assert_eq!(
            Atom::take("[]ab-]"),
            Ok((
                "",
                Atom::MatchingList(vec![
                    BracketExpressionTerm::Single(']'),
                    BracketExpressionTerm::Single('a'),
                    BracketExpressionTerm::Single('b'),
                    BracketExpressionTerm::Single('-'),
                ])
            ))
        );

        assert_eq!(
            Atom::take("[]a-y]"),
            Ok((
                "",
                Atom::MatchingList(vec![
                    BracketExpressionTerm::Single(']'),
                    BracketExpressionTerm::Range('a', 'y'),
                ])
            ))
        );
        assert_eq!(
            Atom::take("[]+--]"),
            Ok((
                "",
                Atom::MatchingList(vec![
                    BracketExpressionTerm::Single(']'),
                    BracketExpressionTerm::Range('+', '-'),
                ])
            ))
        );

        assert_eq!(
            Atom::take("[^]a-y]"),
            Ok((
                "",
                Atom::NonmatchingList(vec![
                    BracketExpressionTerm::Single(']'),
                    BracketExpressionTerm::Range('a', 'y'),
                ])
            ))
        );
        assert_eq!(
            Atom::take("[^]+--]"),
            Ok((
                "",
                Atom::NonmatchingList(vec![
                    BracketExpressionTerm::Single(']'),
                    BracketExpressionTerm::Range('+', '-'),
                ])
            ))
        );
    }

    #[test]
    fn parse_atom_err() {
        assert_eq!(Atom::take(")"), Err(RegexParseError::InvalidAtom(')')));
        assert_eq!(Atom::take(r"\"), Err(RegexParseError::InvalidAtom('\\')));
        assert_eq!(
            Atom::take(r"\a"),
            Err(RegexParseError::InvalidEscapedChar('a'))
        );
    }

    #[test]
    fn atom_to_ranges_normal_char() {
        let (_, atom) = Atom::take("a").unwrap();
        assert_eq!(atom.to_ranges(), vec!['a'..='a']);
    }

    #[test]
    fn atom_to_ranges_matching_list() {
        let (_, atom) = Atom::take("[a-z]").unwrap();
        assert_eq!(atom.to_ranges(), vec!['a'..='z']);

        let (_, atom) = Atom::take("[0-45-9]").unwrap();
        assert_eq!(atom.to_ranges(), vec!['0'..='9']);

        let (_, atom) = Atom::take("[5-90-4]").unwrap();
        assert_eq!(atom.to_ranges(), vec!['0'..='9']);

        let (_, atom) = Atom::take("[0-46-89]").unwrap();
        assert_eq!(atom.to_ranges(), vec!['0'..='4', '6'..='9']);

        let (_, atom) = Atom::take("[96-80-4]").unwrap();
        assert_eq!(atom.to_ranges(), vec!['0'..='4', '6'..='9']);
    }

    #[test]
    fn atom_to_ranges_nonmatching_list() {
        let (_, atom) = Atom::take("[^b-y]").unwrap();
        assert_eq!(atom.to_ranges(), vec!['\0'..='a', 'z'..=char::MAX]);

        let (_, atom) = Atom::take("[^1-45-8]").unwrap();
        assert_eq!(atom.to_ranges(), vec!['\0'..='0', '9'..=char::MAX]);

        let (_, atom) = Atom::take("[^5-81-4]").unwrap();
        assert_eq!(atom.to_ranges(), vec!['\0'..='0', '9'..=char::MAX]);

        let (_, atom) = Atom::take("[^1-486-7]").unwrap();
        assert_eq!(
            atom.to_ranges(),
            vec!['\0'..='0', '5'..='5', '9'..=char::MAX]
        );

        let (_, atom) = Atom::take("[^86-71-4]").unwrap();
        assert_eq!(
            atom.to_ranges(),
            vec!['\0'..='0', '5'..='5', '9'..=char::MAX]
        );
    }

    #[test]
    fn atom_eq() {
        assert_eq!(
            Atom::take("[a-z]").unwrap().1,
            Atom::take("[a-kl-z]").unwrap().1
        );

        assert_eq!(
            Atom::take("[a-z]").unwrap().1,
            Atom::take("[l-za-k]").unwrap().1
        );

        assert_eq!(Atom::take("a").unwrap().1, Atom::take("[a]").unwrap().1);

        assert_eq!(
            Atom::take("[1-8]").unwrap().1,
            Atom::take("[^\0-09-\u{10ffff}]").unwrap().1
        );

        assert_eq!(
            Atom::take("[[:upper:]]").unwrap().1,
            Atom::take("[A-Z]").unwrap().1
        );

        assert_ne!(
            Atom::take("[a-z]").unwrap().1,
            Atom::take("[A-Z]").unwrap().1
        )
    }
}
