use ere::regex;

fn main() {
    #[regex(r"^(.)\. Simpson$", bind = Strict)]
    struct HomerSimpson<'a> {
        #[group(0)]
        matched: &'a str,
    }

    let _ = HomerSimpson::exec("J. Simpson");
}
