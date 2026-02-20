pub fn straight_alpha_to_premultiplied_alpha(buffer: &mut [u8]) {
    for i in (0..buffer.len()).step_by(4) {
        let r = buffer[i] as u16;
        let g = buffer[i + 1] as u16;
        let b = buffer[i + 2] as u16;
        let a = buffer[i + 3] as u16;

        buffer[i] = ((r * a) / 255) as u8;
        buffer[i + 1] = ((g * a) / 255) as u8;
        buffer[i + 2] = ((b * a) / 255) as u8;
    }
}

pub fn straight_alpha_to_premultiplied_alpha_u16(buffer: &mut [u16]) {
    for i in (0..buffer.len()).step_by(4) {
        let r = buffer[i] as u32;
        let g = buffer[i + 1] as u32;
        let b = buffer[i + 2] as u32;
        let a = buffer[i + 3] as u32;

        buffer[i] = ((r * a) / 65535) as u16;
        buffer[i + 1] = ((g * a) / 65535) as u16;
        buffer[i + 2] = ((b * a) / 65535) as u16;
    }
}
