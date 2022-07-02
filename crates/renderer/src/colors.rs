#![allow(unused)]

#[macro_export]
macro_rules! make_colour {
    ($NAME:ident [$R:literal, $G:literal, $B:literal]) => {
        pub const $NAME: self::Rgb = self::Rgb {
            r: $R,
            g: $G,
            b: $B,
        };
    };
    ($NAME:ident [$R:literal, $G:literal, $B:literal, $A:literal]) => {
        pub const $NAME: self::Rgba = self::Rgba {
            r: $R,
            g: $G,
            b: $B,
            a: $A,
        };
    };
}

#[derive(Debug, Copy, Clone)]
pub enum Colour {
    Rgb(Rgb),
    Rgba(Rgba),
}

#[derive(Debug, Copy, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Rgb> for Colour {
    fn from(rgb: Rgb) -> Colour {
        Colour::Rgb(rgb)
    }
}

impl From<(u8, u8, u8)> for Colour {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Colour::Rgb(Rgb { r, g, b })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<Rgba> for Colour {
    fn from(rgba: Rgba) -> Colour {
        Colour::Rgba(rgba)
    }
}

impl From<(u8, u8, u8, u8)> for Colour {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        Colour::Rgba(Rgba { r, g, b, a })
    }
}

impl Colour {
    // White Colours
    make_colour! { WHITE            [255, 255, 255] }
    make_colour! { SNOW             [255, 250, 250] }
    make_colour! { HONEY_DEW        [240, 255, 240] }
    make_colour! { MINT_CREAM       [245, 255, 250] }
    make_colour! { AZURE            [240, 255, 255] }
    make_colour! { ALICE_BLUE       [240, 248, 255] }
    make_colour! { GHOST_WHITE      [248, 248, 255] }
    make_colour! { WHITE_SMOKE      [245, 245, 245] }
    make_colour! { SEA_SHELL        [255, 245, 238] }
    make_colour! { BEIGE            [245, 245, 220] }
    make_colour! { OLD_LACE         [253, 245, 230] }
    make_colour! { FLORAL_WHITE     [255, 250, 240] }
    make_colour! { IVORY            [255, 255, 240] }
    make_colour! { ANTIQUE_WHITE    [250, 235, 215] }
    make_colour! { LINEN            [250, 240, 230] }
    make_colour! { LAVENDER_BLUSH   [255, 240, 245] }
    make_colour! { MISTY_ROSE       [255, 228, 225] }

    // Grey Colours
    make_colour! { BLACK            [0,   0,   0  ] }
    make_colour! { GAINS_BORROW     [220, 220, 220] }
    make_colour! { LIGHT_GREY       [211, 211, 211] }
    make_colour! { SILVER           [192, 192, 192] }
    make_colour! { DARK_GRAY        [169, 169, 169] }
    make_colour! { GRAY             [128, 128, 128] }
    make_colour! { DIM_GRAY         [105, 105, 105] }
    make_colour! { LIGHT_SLATE_GRAY [119, 136, 153] }
    make_colour! { SLATE_GREY       [112, 128, 144] }
    make_colour! { DARK_SLATE_GRAY  [47 , 79 , 79 ] }

    // Red Colours
    make_colour! { RED              [255, 0  , 0  ] }
    make_colour! { DARK_RED         [139, 0  , 0  ] }
    make_colour! { FIREBRICK        [178, 34 , 34 ] }
    make_colour! { INDIAN_RED       [205, 92 , 92 ] }
    make_colour! { CRIMSON          [220, 20 , 60 ] }
    make_colour! { SALMON           [250, 128, 114] }
    make_colour! { LIGHT_SALMON     [255, 160, 122] }
    make_colour! { DARK_SALMON      [233, 150, 122] }
    make_colour! { LIGHT_CORAL      [240, 128, 128] }

    // Orange Colours
    make_colour! { ORANGE          [255, 127, 80 ] }
    make_colour! { DARK_ORANGE     [255, 140, 0  ] }
    make_colour! { ORANGE_RED      [255, 69 , 0  ] }
    make_colour! { GOLD            [255, 215, 0  ] }
    make_colour! { TOMATO          [255, 99 , 0  ] }
    make_colour! { CORAL           [255, 127, 80 ] }

    // Yellow Colours
    make_colour! { YELLOW          [255, 255, 0  ] }

    // Green Colours
    make_colour! { GREEN           [0  , 128, 0  ] }
    make_colour! { DARK_GREEN      [0  , 100, 0  ] }
    make_colour! { LIGHT_GREEN     [144, 238, 144] }
    make_colour! { LIME            [0  , 255, 0  ] }
    make_colour! { GREEN_YELLOW    [173, 255, 47 ] }

    // Cyan Colours
    make_colour! { CYAN            [0  , 255, 255] }

    // Blue Colours
    make_colour! { BLUE            [0  , 0  , 255] }
    make_colour! { SKY_BLUE        [135,206,235] }
    make_colour! { POWDER_BLUE     [176,224,230] }
    make_colour! { LIGHT_BLUE      [173,216,230] }
    make_colour! { LIGHT_SKY_BLUE  [135,206,250] }
    make_colour! { DEEP_SKY_BLUE   [0,191,255] }
    make_colour! { DODGER_BLUE     [30,144,255] }
    make_colour! { ROYAL_BLUE      [30,144,255] }

    // Purple Colours
    make_colour! { PURPLE          [128, 0  , 128] }
    make_colour! { LAVENDER        [230, 230, 250] }
    make_colour! { THISTLE         [216, 191, 216] }
    make_colour! { PLUM            [221, 160, 221] }
    make_colour! { ORCHID          [218, 112, 214] }
    make_colour! { FUSHIA          [218, 112, 214] }
    make_colour! { MAGENTA         [218, 112, 214] }

    // Pink Colours
    make_colour! { PINK            [255, 192, 203] }
    make_colour! { LIGHT_PINK      [255, 182, 193] }
    make_colour! { HOT_PINK        [255, 105, 180] }
    make_colour! { DEEP_PINK       [255, 20 , 147] }
    make_colour! { PALE_VIOLET_RED [219, 112, 147] }
    make_colour! { MED_VIOLET_RED  [199, 21 , 133] }

    // Brown Colours
    make_colour! { BROWN           [165, 42 , 42 ] }
    make_colour! { CORNSILK        [255, 248, 220] }
    make_colour! { BLANCHED_ALMOND [255, 235, 205] }
    make_colour! { BISQUE          [255, 228, 196] }
    make_colour! { NAVAJO_WHITE    [255, 222, 173] }
    make_colour! { WHEAT           [245, 222, 179] }
    make_colour! { BURLY_WOOD      [222, 184, 135] }
    make_colour! { TAN             [210, 180, 140] }
    make_colour! { ROSY_BROWN      [188, 143, 143] }
    make_colour! { SANDY_BROWN     [244, 164, 96 ] }
    make_colour! { GOLDEN_ROD      [218, 165, 32 ] }
    make_colour! { PERU            [205, 133, 63 ] }
    make_colour! { CHOCOLATE       [210, 105, 30 ] }
    make_colour! { SADLE_BROWN     [139, 69 , 19 ] }
    make_colour! { SIENNA          [160, 82 , 45 ] }
    make_colour! { MAROON          [128, 0  , 0  ] }
}
