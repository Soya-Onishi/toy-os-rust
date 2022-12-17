use psf;

pub fn generate_font_array(data: &[u8]) -> Vec<[u8; 16]> {
    let font = psf::Font::parse_font_data(data.to_vec()).unwrap();

    fn get_font(font: &psf::Font, c: char) -> [u8; 16] {
        let glyph = font.get_char(c).unwrap();
        let mut buf = Vec::new();

        for y in 0..glyph.height() {
            let mut line = Vec::new();
            for x in 0..glyph.width() {
                line.push(glyph.get(x, y).unwrap()); 
            }
            buf.push(line);
        }
     
        let mut bitmap = [0; 16];
        bitmap.iter_mut().enumerate().for_each(|(y, row)| {
            for x in 0..8 {
                *row |= (buf[y][x] as u8) << (7 - x);
            }
        });

        bitmap
    }

    (33..=126).map(|u: u8| get_font(&font, u as char)).collect()
}