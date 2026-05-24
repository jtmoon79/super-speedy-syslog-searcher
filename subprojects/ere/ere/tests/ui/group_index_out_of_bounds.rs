extern crate super_speedy_syslog_searcher_ere as ere;
use ere::regex;

fn main() {
    #[derive(Debug)]
    #[regex(r"^([0-9]{4})-([0-9]{2})$")]
    struct Date<'a> {
        #[group(0)]
        matched: &'a str,
        #[group(1)]
        year: &'a str,
        #[group(2)]
        month: &'a str,
        // group 99 does not exist — only groups 0, 1, 2 exist
        #[group(99)]
        invalid: &'a str,
    }

    let _ = Date::exec("2024-03");
}
