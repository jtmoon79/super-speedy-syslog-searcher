// bench_regex.rs

//! Compare different regular expression engines and their performance on the
//! same regex pattern and haystack.

#![allow(non_snake_case)]

use ::criterion::{
    criterion_group,
    criterion_main,
    Criterion,
};
use ::regex_automata::{
    nfa::thompson::pikevm::PikeVM,
    nfa::thompson::Config as ThompsonConfig,
    dfa::onepass::DFA,
    util::iter::Searcher,
};
#[allow(unused_imports)]
use ::si_trace_print::{
    defn,
    defo,
    defx,
};

pub use ::ere::regex;
pub use ::ere_automator_procmacro::new_ere_regex;
pub use ::ere_datetimes_impl::{
    CaptureGroupName,
    CaptureGroupPattern,
    CGI_YEAR,
    CGI_MONTH,
    CGI_DAY,
    CGI_DAY_IGNORE,
    CGI_HOUR,
    CGI_MINUTE,
    CGI_SECOND,
    CGI_FRACTIONAL,
    CGI_TZ,
    CGI_UPTIME,
    CGI_EPOCH,
    CGN_ALL,
    DateTimeParseInstr,
    DateTimePattern_str,
    DateTimePattern_string,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_LEN,
    GROUP_NAMES,
    GROUP_NAMES_MAP_STR,
    MatchType,
    MatchesType,
    MatchesTypeOpt,
    fos,
    RegexFn,
    RegexPattern,
    DTFSSet,
    DTFS_Year,
    DTFS_Month,
    DTFS_Day,
    DTFS_Hour,
    DTFS_Minute,
    DTFS_Second,
    DTFS_Fractional,
    DTFS_Tz,
    DTFS_Epoch,
    DTFS_Uptime,
    MINUS_SIGN,
    HYPHEN_MINUS,
    regex_id_compiled,
    O_L,
    SpanS4,
    SpanS4_from_ptrs,
    YEAR_FALLBACKDUMMY_VAL,
    YEAR_FALLBACKDUMMY,
    Uptime,
};

use std::hint::black_box;

pub const CHARSZ: usize = 1;

pub const PATTERN1: &str = 
    r"^(\[|\()(?<year>[[:digit:]]{4})(/|-)(?<month>[[:digit:]]{2})(/|-)(?<day>[[:digit:]]{2}) (?<hour>[[:digit:]]{2}):(?<minute>[[:digit:]]{2}):(?<second>[[:digit:]]{2})(?<fractional>(\.|,)[[:digit:]]{6})(\]|\))";
pub const PATTERN1_BYTES: &[u8] = PATTERN1.as_bytes();

/// matches
const HAYSTACK1: &[u8] =
    b"[2001/01/01 11:21:12.111222] ../source3/smbd/oplock.c:1340(init_oplocks)";
/// matches
const HAYSTACK2: &[u8] =
    b"(2003-03-04 23:34:44,333444) ../source3/smbd/oplock.c:1340(init_oplocks)";
/// fails to match
const HAYSTACK3: &[u8] =
    b"2005-05-06 05:06:56.555666 ../source3/smbd/oplock.c:1340(init_oplocks)";
/// fails to match
const HAYSTACK4: &[u8] =
    b"[2007/07/08 17:18:58,777888 ../source3/smbd/oplock.c:1340(init_oplocks)";

pub const HAYSTACKS: &[(usize, &[u8])] = &[
    (7, HAYSTACK1),
    (7, HAYSTACK2),
    (0, HAYSTACK3),
    (0, HAYSTACK4),
];

pub const LINE: &str = "🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺🭺";

pub static mut EPRINTLN_ENABLE: bool = true;

#[macro_export]
macro_rules! eprintln_onoff {
    ($($arg:tt)*) => {{
        unsafe {
            if EPRINTLN_ENABLE {
                eprintln!($($arg)*);
            }
        }
    }};
}

pub fn match_using_regex_bytes(
    haystack: &[u8],
    re: &regex::bytes::Regex
) -> MatchesType {
    let mut matches: MatchesType = MatchesType::with_capacity(GROUP_NAMES.len() + 1);
    if let Some(caps) = re.captures(haystack) {
        eprintln_onoff!("caps: {caps:?}");
        for group_name in GROUP_NAMES {
            if let Some(m) = caps.name(group_name) {
                let (a, b) = (m.start(), m.end());
                eprintln_onoff!("  caps.name({group_name:<20?}) = haystack[{:02}..{:02}] = {:?}",
                    a, b, std::str::from_utf8(&haystack[a..b]).unwrap());
                let index = GROUP_NAMES.iter().position(|&name| name == group_name).unwrap();
                matches.push(MatchType::new(SpanS4{start: a, end: b}, index));
            } else {
                eprintln_onoff!("  caps.name({group_name:<20?}) = None");
            }
        }
    } else {
        eprintln_onoff!("No match found");
    }

    matches
}

pub fn benchmark__regex__bytes(c: &mut Criterion) {
    unsafe {
        EPRINTLN_ENABLE = false;
    }
    let re = regex::bytes::Regex::new(PATTERN1).unwrap();
    c.bench_function("benchmark__regex__bytes", |b| {
        b.iter(|| {
            for (expected, haystack) in HAYSTACKS {
                let v = match_using_regex_bytes(haystack, &re);
                assert_eq!(*expected, v.len(), "Expected {} matches, but got {} matches for haystack", expected, v.len());
                black_box(v);
            }
        })
    });
}

pub fn match_using_regex_automata_regex_captures_iter(
    regex: &regex_automata::meta::Regex,
    haystack: &[u8],
) -> MatchesType {
    let mut matches: MatchesType = MatchesType::with_capacity(GROUP_NAMES.len() + 1);
    for ra_caps in regex.captures_iter(haystack) {
        eprintln_onoff!("ra_caps: {ra_caps:?}");
        for group_name in GROUP_NAMES {
            match ra_caps.get_group_by_name(group_name) {
                Some(span) => {
                    eprintln_onoff!(
                        "    ra_caps.get_group_by_name({group_name:?}) = {span:?} = haystack[{}..{}] = {:?}",
                        span.start, span.end, std::str::from_utf8(&haystack[span.start..span.end]).unwrap());
                    // find the index of the same `group_name` value in `GROUP_NAMES`
                    let group_index = GROUP_NAMES.iter().position(|name| name == &group_name).unwrap();
                    eprintln_onoff!("    GROUP_NAMES index = {group_index:?}");
                    eprintln_onoff!("    match: {span:?} = haystack[{}..{}] = {:?} (group_name: {group_name:?})",
                        span.start, span.end, std::str::from_utf8(&haystack[span.start..span.end]).unwrap());
                    matches.push(MatchType::new(SpanS4::new(span.start, span.end), group_index));
                }
                None => {
                    eprintln_onoff!("    ra_caps.get_group_by_name({group_name:?}) = None");
                }
            }
        }
    }

    matches
}

pub fn benchmark__regex_automata__regex_captures_iter(c: &mut Criterion) {
    unsafe {
        EPRINTLN_ENABLE = false;
    }
    use regex_automata::meta::Regex;
    let re = Regex::new(PATTERN1).unwrap();
    c.bench_function("benchmark__regex_automata__regex_captures_iter", |b| {
        b.iter(|| {
            for (expected, haystack) in HAYSTACKS {
                let v = match_using_regex_automata_regex_captures_iter(&re, haystack);
                assert_eq!(*expected, v.len(), "Expected {} matches, but got {} matches for haystack", expected, v.len());
                black_box(v);
            }
        })
    });
}

pub fn match_using_regex_automata_regex_search_captures_with(
    regex: &regex_automata::meta::Regex,
    cache: &mut regex_automata::meta::Cache,
    ra_caps: &mut regex_automata::util::captures::Captures,
    haystack: &[u8],
) -> MatchesType {
    let mut matches: MatchesType = MatchesType::with_capacity(GROUP_NAMES.len() + 1);
    let input_ = regex_automata::Input::new(haystack).span(0..haystack.len());
    regex.search_captures_with(
        cache,
        &input_,
        ra_caps,
    );
    eprintln_onoff!("ra_caps: {ra_caps:?}");
    for group_name in GROUP_NAMES {
        match ra_caps.get_group_by_name(group_name) {
            Some(span) => {
                eprintln_onoff!(
                    "    ra_caps.get_group_by_name({group_name:?}) = {span:?} = haystack[{}..{}] = {:?}",
                    span.start, span.end, std::str::from_utf8(&haystack[span.start..span.end]).unwrap());
                // find the index of the same `group_name` value in `GROUP_NAMES`
                let group_index = GROUP_NAMES.iter().position(|name| name == &group_name).unwrap();
                eprintln_onoff!("    GROUP_NAMES index = {group_index:?}");
                eprintln_onoff!("    match: {span:?} = haystack[{}..{}] = {:?} (group_name: {group_name:?})",
                    span.start, span.end, std::str::from_utf8(&haystack[span.start..span.end]).unwrap());
                matches.push(MatchType::new(SpanS4::new(span.start, span.end), group_index));
            }
            None => {
                eprintln_onoff!("    ra_caps.get_group_by_name({group_name:?}) = None");
            }
        }
    }

    matches
}

pub fn benchmark__regex_automata__regex_search_captures_with(c: &mut Criterion) {
    use regex_automata::meta::Regex;
    let re = Regex::new(PATTERN1).unwrap();
    let (mut ra_cache, mut ra_caps) = (re.create_cache(), re.create_captures());
    c.bench_function("benchmark__regex_automata__regex_search_captures_with", |b| {
        b.iter(|| {
            for (expected, haystack) in HAYSTACKS {
                let v = match_using_regex_automata_regex_search_captures_with(
                    &re,
                    &mut ra_cache,
                    &mut ra_caps,
                    haystack,
                );
                assert_eq!(*expected, v.len(), "Expected {} matches, but got {} matches for haystack", expected, v.len());
                black_box(v);
            }
        })
    });
}

pub fn match_using_regex_automata_pikevm_searcher(
    ra_vm: &PikeVM,
    haystack: &[u8],
    mut ra_cache: &mut regex_automata::nfa::thompson::pikevm::Cache,
    mut ra_caps: &mut regex_automata::util::captures::Captures,
) -> MatchesType {
    eprintln_onoff!("haystack: {:?}", std::str::from_utf8(haystack).unwrap());
    let ra_input = regex_automata::Input::new(haystack).span(0..haystack.len());
    let mut matches: MatchesType = Vec::with_capacity(GROUP_NAMES.len() + 1);
    let mut ra_searcher = Searcher::new(ra_input);
    // from https://docs.rs/regex-automata/latest/regex_automata/util/iter/struct.Searcher.html#searcher-vs-iterator
    eprintln_onoff!("ra_searcher: {ra_searcher:?}");

    while let Some(m) = ra_searcher.advance(|input_| {
        eprintln_onoff!("ra_searcher.advance(...) = {input_:?}");
        ra_vm.search(&mut ra_cache, input_, &mut ra_caps);

        let m = match  ra_caps.get_match() {
            Some(m) => m,
            None => {
                eprintln_onoff!("ra_caps.get_match() = None");
                return Ok(None);
            }
        };
        eprintln_onoff!("ra_vm.search(cache, caps) = {m:?} = haystack[{}..{}] = {:?}",
            m.span().start, m.span().end, std::str::from_utf8(&haystack[m.span().start..m.span().end]).unwrap());

        let gi = ra_caps.group_info();
        eprintln_onoff!("ra_caps.group_info() = {gi:?}, gi.memory_usage() = {}", gi.memory_usage());
        for (i, gi_all_names) in gi.all_names().enumerate() {
            eprintln_onoff!("  group_info.all_names[{i}]: {gi_all_names:?}");
            if let Some(group_name) = gi_all_names.2 {
                eprintln_onoff!("    group_info.all_names[{i}].2 = {group_name:?}");
                match ra_caps.get_group_by_name(group_name) {
                    Some(span) => {
                        eprintln_onoff!(
                            "    ra_caps.get_group_by_name({group_name:?}) = {span:?} = haystack[{}..{}] = {:?}",
                            span.start, span.end, std::str::from_utf8(&haystack[span.start..span.end]).unwrap());
                        // find the index of the same `group_name` value in `GROUP_NAMES`
                        let group_index = GROUP_NAMES.iter().position(|name| name == &group_name).unwrap();
                        eprintln_onoff!("    GROUP_NAMES index = {group_index:?}");
                        eprintln_onoff!("    match {i}: {span:?} = haystack[{}..{}] = {:?} (group_name: {group_name:?})",
                            span.start, span.end, std::str::from_utf8(&haystack[span.start..span.end]).unwrap());
                        matches.push(MatchType::new(SpanS4::new(span.start, span.end), group_index));
                    }
                    None => {
                        eprintln_onoff!("    ra_caps.get_group_by_name({group_name:?}) = None");
                    }
                }
            }
        }

        Ok(Some(m))
    }) {
        eprintln_onoff!("ra_searcher.advance(...) = Some({m:?})");
    }
    eprintln_onoff!();

    eprintln_onoff!("{}", LINE);

    matches
}

pub fn benchmark__regex_automata__pikevm_searcher(c: &mut Criterion) {
    let ra_vm = PikeVM::builder()
        .thompson(ThompsonConfig::new().utf8(false))
        .syntax(regex_automata::util::syntax::Config::new().utf8(false))
        .build(PATTERN1).unwrap();
    let (mut ra_cache, mut ra_caps) = (ra_vm.create_cache(), ra_vm.create_captures());
    c.bench_function("benchmark__regex_automata__pikevm_searcher", |b| {
        b.iter(|| {
            for (expected, haystack) in HAYSTACKS {
                let v = match_using_regex_automata_pikevm_searcher(&ra_vm, haystack, &mut ra_cache, &mut ra_caps);
                assert_eq!(*expected, v.len(), "Expected {} matches, but got {} matches for haystack", expected, v.len());
                black_box(v);
            };
            
        })
    });
}

#[allow(dead_code)]
pub fn match_using_regex_automata_dfa(
    haystack: &[u8],
    dfa: &regex_automata::hybrid::dfa::DFA,
    mut cache: &mut regex_automata::hybrid::dfa::Cache,
) -> MatchesType {
    let input_ = regex_automata::Input::new(haystack).span(0..haystack.len());
    // The start state is determined by inspecting the position and the
    // initial bytes of the haystack.
    let mut state = match dfa.start_state_forward(
        &mut cache,
        &input_,
    ) {
        Ok(state) => state,
        Err(_err) => {
            // If there is no start state, then the search fails.
            return vec![];
        }
    };
    // Walk all the bytes in the haystack.
    for &b in haystack.iter() {
        state = match dfa.next_state(&mut cache, state, b) {
            Ok(state) => state,
            Err(_err) => {
                // If there is no next state, then the search fails.
                return vec![];
            }
        }
    }
    // DFAs in this crate require an explicit
    // end-of-input transition if a search reaches
    // the end of a haystack.
    state = match dfa.next_eoi_state(&mut cache, state) {
        Ok(state) => state,
        Err(_err) => {
            // If there is no next state, then the search fails.
            return vec![];
        }
    };
    black_box(state);

    vec![]
}

pub fn match_using_regex_automata_dfa_onepass(
    haystack: &[u8],
    dfa: &regex_automata::dfa::onepass::DFA,
    mut cache: &mut regex_automata::dfa::onepass::Cache,
    mut captures: &mut regex_automata::util::captures::Captures,
) -> MatchesType {
    let mut matches: MatchesType = Vec::with_capacity(GROUP_NAMES.len() + 1);
    let input_ = regex_automata::Input::new(haystack).span(0..haystack.len());
    dfa.captures(&mut cache, input_, &mut captures);

    for group_name in GROUP_NAMES {
        match captures.get_group_by_name(group_name) {
            Some(span) => {
                eprintln_onoff!(
                    "    captures.get_group_by_name({group_name:?}) = {span:?} = haystack[{}..{}] = {:?}",
                    span.start, span.end, std::str::from_utf8(&haystack[span.start..span.end]).unwrap());
                // find the index of the same `group_name` value in `GROUP_NAMES`
                let group_index = GROUP_NAMES.iter().position(|name| name == &group_name).unwrap();
                eprintln_onoff!("    GROUP_NAMES index = {group_index:?}");
                eprintln_onoff!("    match: {span:?} = haystack[{}..{}] = {:?} (group_name: {group_name:?})",
                    span.start, span.end, std::str::from_utf8(&haystack[span.start..span.end]).unwrap());
                let span_s4 = SpanS4 {
                    start: span.start,
                    end: span.end,
                };
                matches.push(MatchType::new(span_s4, group_index));
            }
            None => {
                eprintln_onoff!("    captures.get_group_by_name({group_name:?}) = None");
            }
        }
    }

    matches
}

pub fn benchmark__regex_automata__dfa_onepass_custom_config(c: &mut Criterion) {
    let dfa = DFA::builder()
        .syntax(
            regex_automata::util::syntax::Config::new()
                .utf8(false)
                .unicode(false)
                .multi_line(false)
                .dot_matches_new_line(false)
        )
        .build(PATTERN1)
        .unwrap();
    let (mut cache, mut caps) = (dfa.create_cache(), dfa.create_captures());
    c.bench_function("benchmark__regex_automata__dfa_onepass_custom_config", |b| {
        b.iter(|| {
            for (expected, haystack) in HAYSTACKS {
                let v = match_using_regex_automata_dfa_onepass(
                    haystack,
                    &dfa,
                    &mut cache,
                    &mut caps,
                );
                assert_eq!(*expected, v.len(), "Expected {} matches, but got {} matches for haystack", expected, v.len());
                black_box(v);
            }
        })
    });
}

pub fn benchmark__regex_automata__dfa_onepass_default_config(c: &mut Criterion) {
    let dfa = DFA::builder()
        .syntax(
            regex_automata::util::syntax::Config::new()
        )
        .build(PATTERN1)
        .unwrap();
    let (mut cache, mut caps) = (dfa.create_cache(), dfa.create_captures());
    c.bench_function("benchmark__regex_automata__dfa_onepass_default_config", |b| {
        b.iter(|| {
            for (expected, haystack) in HAYSTACKS {
                let v = match_using_regex_automata_dfa_onepass(
                    haystack,
                    &dfa,
                    &mut cache,
                    &mut caps,
                );
                assert_eq!(*expected, v.len(), "Expected {} matches, but got {} matches for haystack", expected, v.len());
                black_box(v);
            }
        })
    });
}

#[allow(non_upper_case_globals)]
static ere_fn_dfau8: RegexFn = new_ere_regex!(
    1001,
    // XXX: must match PATTERN1
    br"^(\[|\()(?<year>[[:digit:]]{4})(/|-)(?<month>[[:digit:]]{2})(/|-)(?<day>[[:digit:]]{2}) (?<hour>[[:digit:]]{2}):(?<minute>[[:digit:]]{2}):(?<second>[[:digit:]]{2})(?<fractional>(\.|,)[[:digit:]]{6})(\]|\))",
    DfaU8,
    file!(),
    line!(),
    false
);

pub fn match_using_ere_dfau8(
    haystack: &[u8],
) -> MatchesType {
    let matches: MatchesType = match ere_fn_dfau8(haystack) {
        Some(matches_) => matches_,
        None => {
            eprintln_onoff!("ERERegex failed to match");
            return MatchesType::with_capacity(0);
        }
    };
    eprintln_onoff!("ERERegex: {matches:?}");

    matches
}

pub fn benchmark__ere__regex_dfau8(c: &mut Criterion) {
    unsafe {
        EPRINTLN_ENABLE = false;
    }
    c.bench_function("benchmark__ere__regex_dfau8", |b| {
        b.iter(|| {
            for (expected, haystack) in HAYSTACKS {
                let v = match_using_ere_dfau8(haystack);
                assert_eq!(*expected, v.len(),
                    "Expected {} matches, but got {} matches for haystack", expected, v.len());
                black_box(v);
            }
        })
    });
}

#[allow(non_upper_case_globals)]
static ere_fn_FlatLockstepNfaU8: RegexFn = new_ere_regex!(
    1002,
    // XXX: must match PATTERN1
    br"^(\[|\()(?<year>[[:digit:]]{4})(/|-)(?<month>[[:digit:]]{2})(/|-)(?<day>[[:digit:]]{2}) (?<hour>[[:digit:]]{2}):(?<minute>[[:digit:]]{2}):(?<second>[[:digit:]]{2})(?<fractional>(\.|,)[[:digit:]]{6})(\]|\))",
    FlatLockstepNfaU8,
    file!(),
    line!(),
    false
);

pub fn match_using_ere_FlatLockstepNfaU8(
    haystack: &[u8],
) -> MatchesType {
    let matches: MatchesType = match ere_fn_FlatLockstepNfaU8(haystack) {
        Some(matches_) => matches_,
        None => {
            eprintln_onoff!("ERERegex failed to match");
            return MatchesType::with_capacity(0);
        }
    };
    eprintln_onoff!("ERERegex: {matches:?}");

    matches
}

pub fn benchmark__ere__regex_FlatLockstepNfaU8(c: &mut Criterion) {
    unsafe {
        EPRINTLN_ENABLE = false;
    }
    c.bench_function("benchmark__ere__regex_FlatLockstepNfaU8", |b| {
        b.iter(|| {
            for (expected, haystack) in HAYSTACKS {
                let v = match_using_ere_FlatLockstepNfaU8(haystack);
                assert_eq!(*expected, v.len(),
                    "Expected {} matches, but got {} matches for haystack", expected, v.len());
                black_box(v);
            }
        })
    });
}

#[allow(non_upper_case_globals)]
static ere_fn_onepassu8: RegexFn = new_ere_regex!(
    1003,
    // XXX: must match PATTERN1
    br"^(\[|\()(?<year>[[:digit:]]{4})(/|-)(?<month>[[:digit:]]{2})(/|-)(?<day>[[:digit:]]{2}) (?<hour>[[:digit:]]{2}):(?<minute>[[:digit:]]{2}):(?<second>[[:digit:]]{2})(?<fractional>(\.|,)[[:digit:]]{6})(\]|\))",
    OnePassU8,
    file!(),
    line!(),
    false
);

pub fn match_using_ere_onepassu8(
    haystack: &[u8],
) -> MatchesType {
    let matches: MatchesType = match ere_fn_onepassu8(haystack) {
        Some(matches_) => matches_,
        None => {
            eprintln_onoff!("ERERegex failed to match");
            return MatchesType::with_capacity(0);
        }
    };
    eprintln_onoff!("ERERegex: {matches:?}");

    matches
}

pub fn benchmark__ere__regex_onepassu8(c: &mut Criterion) {
    unsafe {
        EPRINTLN_ENABLE = false;
    }
    c.bench_function("benchmark__ere__regex_onepassu8", |b| {
        b.iter(|| {
            for (expected, haystack) in HAYSTACKS {
                let v = match_using_ere_onepassu8(haystack);
                assert_eq!(*expected, v.len(),
                    "Expected {} matches, but got {} matches for haystack", expected, v.len());
                black_box(v);
            }
        })
    });
}

criterion_group!(benches,
    benchmark__regex__bytes,
    benchmark__regex_automata__regex_captures_iter,
    benchmark__regex_automata__regex_search_captures_with,
    benchmark__regex_automata__pikevm_searcher,
    benchmark__regex_automata__dfa_onepass_custom_config,
    benchmark__regex_automata__dfa_onepass_default_config,
    benchmark__ere__regex_dfau8,
    benchmark__ere__regex_FlatLockstepNfaU8,
    benchmark__ere__regex_onepassu8,
);

criterion_main!(benches);
