#[allow(dead_code)]
pub enum Alignment {
    Left, Right, Center
}

impl Default for Alignment {
    fn default() -> Alignment {
        Alignment::Left
    }
}

macro_rules! box_format {
    ($name:ident, $left:expr, $right:expr, $center:expr) => {
        pub fn $name(&mut self, text: &str, align: Option<Alignment>) -> Result<&mut Self, std::fmt::Error> {
            let mut text = String::from(text);
            if !text.is_empty() {
                text = format!(" {} ", text);
            }
    
            match align.unwrap_or_default() {
                Alignment::Left     =>  writeln!(self.formatter, $left,     text)?,
                Alignment::Right    =>  writeln!(self.formatter, $right,    text)?,
                Alignment::Center   =>  writeln!(self.formatter, $center,   text)?,
            }
            
            Ok(self)
        }
    }
}

pub struct BoxFormat<'a, 'b: 'a> {
    formatter: &'a mut std::fmt::Formatter<'b>
}

impl<'a, 'b: 'a> BoxFormat<'a, 'b> {
    pub fn new(formatter: &'a mut std::fmt::Formatter<'b>) -> Self {
        Self { formatter }
    }

    box_format!(header,     "╭{:─<83}╮", "╭{:─>83}╮", "╭{:─^83}╮");
    box_format!(section,    "├{:┄<83}┤", "├{:┄>83}┤", "├{:┄^83}┤");
    box_format!(content,    "│{: <83}│", "│{: >83}│", "│{: ^83}│");
    box_format!(footer,     "╰{:─<83}╯", "╰{:─>83}╯", "╰{:─^83}╯");

    #[inline]
    pub fn empty_line(&mut self) -> Result<&mut Self, std::fmt::Error> {
        self.content("", None)
    }

    #[inline]
    pub fn line_break(&mut self) -> Result<&mut Self, std::fmt::Error> {
        self.section("", None)
    }
}

