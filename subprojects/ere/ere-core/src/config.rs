#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbiguousSubmatchMode {
    /// While not POSIX standard, the greedy method for determining ambiguous submatches is
    /// able to be implemented the most efficiently in the general case.
    /// This is based on the popular Perl regex rules for ambiguous submatching.
    ///
    /// - Quantifiers match as much as possible while allowing the rest to match
    /// - Alternations prioritize the earlier options
    /// - Since it is greedy, the leftmost quantifier/alternations will match first.
    Greedy,
    // /// The mode based on `REG_MINIMAL=false` in C header [`<regex.h>`](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/regex.h.html).
    // /// While the crate attempts to provide optimizations, there are no known ways to implement this as efficiently as the greedy mode.
    // ERELongest,
    // /// The mode based on `REG_MINIMAL=true` in C header [`<regex.h>`](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/regex.h.html)
    // /// While the crate attempts to provide optimizations, there are no known ways to implement this as efficiently as the greedy mode.
    // EREShortest,
}

#[derive(Debug, Clone)]
pub struct Config {
    // case_insensitive: bool,
    /// When finding submatches during [`crate::Regex::exec`], determines the rules for what should happen if the
    /// capture group submatches are ambiguous (e.g. `^(a*)(a*)$` on `aaaa` is ambiguous as to which capture group gets how many `a`'s).
    ambiguous_submatches: AmbiguousSubmatchMode,
}
impl Config {
    pub const fn quantifiers_prefer_longest(&self) -> bool {
        return match self.ambiguous_submatches {
            AmbiguousSubmatchMode::Greedy => true,
        };
    }
    pub const fn builder() -> ConfigBuilder {
        return ConfigBuilder::new();
    }
    pub const fn const_default() -> Self {
        return Config {
            ambiguous_submatches: AmbiguousSubmatchMode::Greedy,
        };
    }
}
impl Default for Config {
    fn default() -> Self {
        return Config::const_default();
    }
}

#[derive(Debug)]
pub struct ConfigBuilder(Config);
impl ConfigBuilder {
    pub const fn new() -> ConfigBuilder {
        return ConfigBuilder(Config::const_default());
    }
    pub const fn build(self) -> Config {
        return self.0;
    }
    pub const fn greedy(mut self) -> ConfigBuilder {
        self.0.ambiguous_submatches = AmbiguousSubmatchMode::Greedy;
        return self;
    }
}
