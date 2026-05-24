extern crate super_speedy_syslog_searcher_ere as ere;
use ere::regex;

fn main() {
    #[regex(r"^(?<first>.)\. (?<last>.+)$", bind = Named)]
    struct Partial<'a> {
        #[group(0)]
        matched: &'a str,
        first: &'a str,
        // missing `last` — should error under bind = Named
    }

    let _ = Partial::exec("J. Simpson");
}
