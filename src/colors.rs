use raylib::prelude::*;

pub static COLOR_RED: Color = Color::new(232, 57, 53, 255);
pub static COLOR_GREEN: Color = Color::new(54, 184, 62, 255);
pub static COLOR_YELLOW: Color = Color::new(227, 210, 70, 255);
pub static COLOR_BLUE: Color = Color::new(67, 130, 232, 255);
pub static COLOR_DARK: Color = Color::new(89, 89, 89, 255);
pub static COLOR_BLACK: Color = Color::new(0, 0, 0, 255);
pub static COLOR_WHITE: Color = Color::new(255, 255, 255, 255);
pub static COLOR_LIGHT: Color = Color::new(247, 251, 252, 255);

pub fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> Color {
    let mut r = lightness;
    let mut g = lightness;
    let mut b = lightness;

    if saturation != 0.0 {
        let q = if lightness > 0.5 {
            lightness * (1.0 + saturation)
        } else {
            1.0 + saturation - lightness * saturation
        };
        let p = 2.0 * lightness - q;
        r = hue_to_rgb(p, q, hue + 1.0/3.0);
        g = hue_to_rgb(p, q, hue);
        b = hue_to_rgb(p, q, hue - 1.0/3.0);
    }

    Color::new((r*255.0).round() as u8, (g*255.0).round() as u8, (b*255.0).round() as u8, 255)
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0/6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0/2.0 { return q; }
    if t < 2.0/3.0 { return p + (q - p) * (2.0/3.0 - t) * 6.0; }
    p
}
