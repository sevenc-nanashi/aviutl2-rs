pub fn straight_alpha_to_premultiplied_alpha(buffer: &mut [u8]) {
    for i in (0..buffer.len()).step_by(4) {
        let r = buffer[i] as u16;
        let g = buffer[i + 1] as u16;
        let b = buffer[i + 2] as u16;
        let a = buffer[i + 3] as u16;

        buffer[i] = premultiply_u8(r, a);
        buffer[i + 1] = premultiply_u8(g, a);
        buffer[i + 2] = premultiply_u8(b, a);
    }
}

pub fn straight_alpha_to_premultiplied_alpha_u16(buffer: &mut [u16]) {
    for i in (0..buffer.len()).step_by(4) {
        let r = buffer[i] as u64;
        let g = buffer[i + 1] as u64;
        let b = buffer[i + 2] as u64;
        let a = buffer[i + 3] as u64;

        buffer[i] = premultiply_u16(r, a);
        buffer[i + 1] = premultiply_u16(g, a);
        buffer[i + 2] = premultiply_u16(b, a);
    }
}

fn premultiply_u8(color: u16, alpha: u16) -> u8 {
    ((color * alpha + 127) / 255) as u8
}

fn premultiply_u16(color: u64, alpha: u64) -> u16 {
    ((color * alpha + 32767) / 65535) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premultiplies_transparent_u16_to_zero() {
        let mut buffer = [65535, 32768, 12345, 0];

        straight_alpha_to_premultiplied_alpha_u16(&mut buffer);

        assert_eq!(buffer, [0, 0, 0, 0]);
    }

    #[test]
    fn keeps_opaque_u16_color() {
        let mut buffer = [65535, 32768, 12345, 65535];

        straight_alpha_to_premultiplied_alpha_u16(&mut buffer);

        assert_eq!(buffer, [65535, 32768, 12345, 65535]);
    }

    #[test]
    fn rounds_premultiplied_u16_color() {
        let mut buffer = [65535, 32768, 1, 32768];

        straight_alpha_to_premultiplied_alpha_u16(&mut buffer);

        assert_eq!(buffer, [32768, 16384, 1, 32768]);
    }
}
