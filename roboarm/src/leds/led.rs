pub mod led {
    pub enum Color {
        Red,
        Green,
        Blue,
    }

    pub struct Led {
        pub color: Color,
    }
}