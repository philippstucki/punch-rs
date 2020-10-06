use colored::ColoredString;
use colored::Colorize;

pub trait Colors {
    fn color_heading(self) -> ColoredString;
    fn color_project(self) -> ColoredString;
    fn color_time(self) -> ColoredString;
    fn color_duration(self) -> ColoredString;
    fn color_tag(self) -> ColoredString;
}

impl<'a> Colors for &'a str {
    fn color_heading(self) -> ColoredString {
        self.to_string().bold()
    }
    fn color_project(self) -> ColoredString {
        self.to_string().purple()
    }
    fn color_time(self) -> ColoredString {
        self.to_string().to_string().green()
    }
    fn color_duration(self) -> ColoredString {
        self.to_string().to_string().white()
    }
    fn color_tag(self) -> ColoredString {
        self.to_string().to_string().bright_magenta()
    }
}
