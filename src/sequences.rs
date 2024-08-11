use std::vec::Vec;

#[derive(Clone)]
pub struct Color {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
    pub duration: i32,
}

pub fn get_cycle_seq() -> Vec<Color> {
    vec![
        Color {
            red: true,
            green: false,
            blue: false,
            duration: 3000,
        },
        Color {
            red: true,
            green: true,
            blue: false,
            duration: 1000,
        },
        Color {
            red: false,
            green: true,
            blue: false,
            duration: 3000,
        },
        Color {
            red: true,
            green: true,
            blue: false,
            duration: 1000,
        },
    ]
}

pub fn get_red_seq() -> Vec<Color> {
    vec![Color {
        red: true,
        green: false,
        blue: false,
        duration: 1000,
    }]
}

pub fn get_green_seq() -> Vec<Color> {
    vec![Color {
        red: false,
        green: true,
        blue: false,
        duration: 1000,
    }]
}

pub fn get_yellow_seq() -> Vec<Color> {
    vec![Color {
        red: true,
        green: true,
        blue: false,
        duration: 3000,
    }]
}

pub fn get_blue_seq() -> Vec<Color> {
    vec![Color {
        red: false,
        green: false,
        blue: true,
        duration: 3000,
    }]
}
