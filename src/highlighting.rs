use termion::color;

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
    MultipleComments,
    PrimaryKeywords,
    SecondaryKeywords,
}

impl Type {
   pub fn to_color(self) -> impl color::Color {
        match self {
            Type::Number => color::Rgb(220, 163, 163),
            Type::Match => color::Rgb(255, 0, 0),
            Type::String => color::Rgb(211, 54, 130),
            Type::Character => color::Rgb(108, 113, 196),
            Type::Comment => color::Rgb(0, 205, 0),
            Type::PrimaryKeywords => color::Rgb(181, 137, 0),
            Type::SecondaryKeywords => color::Rgb(42, 161, 152),
            Type::MultipleComments => color::Rgb(154, 255, 154),
            _ => color::Rgb(255, 255, 255),
        }
    }
}