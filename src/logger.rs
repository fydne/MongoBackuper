pub fn warn(content: &str) {
    println!("[{}] {} {content}", crate::exts::get_date(), colors::yellow("{Warn}"));
}

pub fn error(content: &str) {
    println!("[{}] {} {content}", crate::exts::get_date(), colors::red("{Error}"));
}

pub fn debug(content: &str) {
    println!("[{}] {} {content}", crate::exts::get_date(), colors::green("{Debug}"));
}



pub fn info_string(content: String) {
    println!("[{}] {} {content}", crate::exts::get_date(), colors::blue("{Info}"));
}

pub fn warn_string(content: String) {
    println!("[{}] {} {content}", crate::exts::get_date(), colors::yellow("{Warn}"));
}

pub fn error_string(content: String) {
    println!("[{}] {} {content}", crate::exts::get_date(), colors::red("{Error}"));
}

pub fn debug_string(content: String) {
    println!("[{}] {} {content}", crate::exts::get_date(), colors::green("{Debug}"));
}




pub mod colors {
    pub fn blue(content: &str) -> String{
        format!("{}{content}{}", parse_code(34), parse_code(39))
    }
    pub fn yellow(content: &str) -> String{
        format!("{}{content}{}", parse_code(33), parse_code(39))
    }
    pub fn red(content: &str) -> String{
        format!("{}{content}{}", parse_code(31), parse_code(39))
    }
    pub fn green(content: &str) -> String{
        format!("{}{content}{}", parse_code(32), parse_code(39))
    }

    fn parse_code(code: u8) -> String {
        return format!("\u{001b}[{}m", code);
    }
}