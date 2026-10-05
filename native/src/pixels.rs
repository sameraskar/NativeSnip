pub fn bgra_to_rgb(bytes: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(bytes.len() / 4 * 3);
    for p in bytes.chunks_exact(4) {
        rgb.extend_from_slice(&[p[2], p[1], p[0]]);
    }
    rgb
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channel_order_and_unused_alpha() {
        assert_eq!(
            bgra_to_rgb(&[0, 0, 255, 0, 255, 0, 0, 255]),
            vec![255, 0, 0, 0, 0, 255]
        );
    }
    #[test]
    fn png_roundtrip() {
        let rgb = bgra_to_rgb(&[1, 2, 3, 0, 4, 5, 6, 0]);
        let mut out = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut out, 2, 1);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            let mut writer = enc.write_header().unwrap();
            writer.write_image_data(&rgb).unwrap();
            writer.finish().unwrap();
        }
        let mut reader = png::Decoder::new(std::io::Cursor::new(out))
            .read_info()
            .unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let frame = reader.next_frame(&mut buf).unwrap();
        assert_eq!(&buf[..frame.buffer_size()], rgb);
        assert_eq!((frame.width, frame.height), (2, 1));
    }
}
