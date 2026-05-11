use ere::regex;

fn main() {
    #[regex(r"^(?<middle>.)\. Simpson$")]
    struct HomerSimpson<'a> {
        #[group(0)]
        matched: &'a str,
        middle: &'a str,
        // "foo" does not exist as a named group in the regex
        foo: &'a str,
    }

    let _ = HomerSimpson::exec("J. Simpson");
}
