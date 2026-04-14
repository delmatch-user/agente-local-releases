//! ESC/POS command constants and builders

/// ESC character
pub const ESC: u8 = 0x1B;
/// GS character (Group Separator)
pub const GS: u8 = 0x1D;
/// LF (Line Feed)
pub const LF: u8 = 0x0A;
/// CR (Carriage Return)
pub const CR: u8 = 0x0D;
/// HT (Horizontal Tab)
pub const HT: u8 = 0x09;
/// FS (Field Separator)
pub const FS: u8 = 0x1C;

/// Initialize printer (reset to default settings)
pub fn initialize() -> Vec<u8> {
    vec![ESC, b'@']
}

/// Cut paper (partial cut with feed)
pub fn cut_paper() -> Vec<u8> {
    vec![GS, b'V', 0x42, 0x03]
}

/// Full cut paper
pub fn full_cut() -> Vec<u8> {
    vec![GS, b'V', 0x00]
}

/// Open cash drawer (pulse to pin 2)
pub fn open_drawer() -> Vec<u8> {
    vec![ESC, b'p', 0x00, 0x19, 0xFA]
}

/// Open cash drawer (pulse to pin 5)
pub fn open_drawer_pin5() -> Vec<u8> {
    vec![ESC, b'p', 0x01, 0x19, 0xFA]
}

/// Set bold mode
pub fn set_bold(enabled: bool) -> Vec<u8> {
    vec![ESC, b'E', if enabled { 1 } else { 0 }]
}

/// Set underline mode
pub fn set_underline(mode: u8) -> Vec<u8> {
    // 0 = off, 1 = 1-dot, 2 = 2-dot
    vec![ESC, b'-', mode.min(2)]
}

/// Set font size (width and height multiplier 1-8)
pub fn set_font_size(width: u8, height: u8) -> Vec<u8> {
    let w = (width.saturating_sub(1)).min(7);
    let h = (height.saturating_sub(1)).min(7);
    vec![GS, b'!', (w << 4) | h]
}

/// Set text alignment
pub fn align_left() -> Vec<u8> {
    vec![ESC, b'a', 0x00]
}

pub fn align_center() -> Vec<u8> {
    vec![ESC, b'a', 0x01]
}

pub fn align_right() -> Vec<u8> {
    vec![ESC, b'a', 0x02]
}

/// Line feed
pub fn line_feed() -> Vec<u8> {
    vec![LF]
}

/// Feed n lines
pub fn feed_lines(n: u8) -> Vec<u8> {
    vec![ESC, b'd', n]
}

/// Print text (converts to CP850 for Brazilian Portuguese)
pub fn print_text(text: &str) -> Vec<u8> {
    // Convert UTF-8 to CP850 (common for Brazilian thermal printers)
    text.chars()
        .map(|c| match c {
            'á' => 0xA0,
            'à' => 0x85,
            'ã' => 0xC6,
            'â' => 0x83,
            'é' => 0x82,
            'ê' => 0x88,
            'í' => 0xA1,
            'ó' => 0xA2,
            'ô' => 0x93,
            'õ' => 0xE4,
            'ú' => 0xA3,
            'ü' => 0x81,
            'ç' => 0x87,
            'Á' => 0xB5,
            'É' => 0x90,
            'Í' => 0xD6,
            'Ó' => 0xE0,
            'Ú' => 0xE9,
            'Ç' => 0x80,
            'Ã' => 0xC7,
            'Õ' => 0xE5,
            'ñ' => 0xA4,
            'Ñ' => 0xA5,
            '°' => 0xF8,
            '²' => 0xFD,
            '³' => 0xFC,
            '€' => 0xD5,
            c if c.is_ascii() => c as u8,
            _ => b'?',
        })
        .collect()
}

/// Print line separator
pub fn separator(width: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(width + 1);
    result.extend(std::iter::repeat(b'-').take(width));
    result.push(LF);
    result
}

/// Print double line separator
pub fn double_separator(width: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(width + 1);
    result.extend(std::iter::repeat(b'=').take(width));
    result.push(LF);
    result
}

/// Set character code table
pub fn set_code_table(table: u8) -> Vec<u8> {
    // 0 = PC437, 2 = PC850 (Multilingual Latin), 32 = WPC1252
    vec![ESC, b't', table]
}

/// Select Brazilian code page (CP850)
pub fn select_brazilian() -> Vec<u8> {
    set_code_table(2)
}

/// Print QR Code
pub fn print_qr_code(data: &str, size: u8) -> Vec<u8> {
    let mut bytes = Vec::new();
    let data_bytes = data.as_bytes();
    let len = data_bytes.len() as u16;
    
    // Set QR model (Model 2)
    bytes.extend(&[GS, b'(', b'k', 0x04, 0x00, 0x31, 0x41, 0x32, 0x00]);
    
    // Set QR size (1-16)
    bytes.extend(&[GS, b'(', b'k', 0x03, 0x00, 0x31, 0x43, size.min(16).max(1)]);
    
    // Set error correction level (L=48, M=49, Q=50, H=51)
    bytes.extend(&[GS, b'(', b'k', 0x03, 0x00, 0x31, 0x45, 0x31]);
    
    // Store QR data
    let store_len = (len + 3) as u16;
    bytes.extend(&[
        GS, b'(', b'k',
        (store_len & 0xFF) as u8,
        ((store_len >> 8) & 0xFF) as u8,
        0x31, 0x50, 0x30
    ]);
    bytes.extend(data_bytes);
    
    // Print QR
    bytes.extend(&[GS, b'(', b'k', 0x03, 0x00, 0x31, 0x51, 0x30]);
    
    bytes
}

/// Print barcode (Code128)
pub fn print_barcode_code128(data: &str, height: u8) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    // Set barcode height
    bytes.extend(&[GS, b'h', height]);
    
    // Set barcode width (2-6)
    bytes.extend(&[GS, b'w', 2]);
    
    // Set HRI position (below barcode)
    bytes.extend(&[GS, b'H', 2]);
    
    // Set HRI font
    bytes.extend(&[GS, b'f', 0]);
    
    // Print Code128
    bytes.push(GS);
    bytes.push(b'k');
    bytes.push(73); // Code128
    bytes.push(data.len() as u8);
    bytes.extend(data.as_bytes());
    
    bytes
}

/// Beep (if supported)
pub fn beep(times: u8, duration: u8) -> Vec<u8> {
    vec![ESC, b'B', times, duration]
}

/// Set inverted mode (white on black)
pub fn set_inverted(enabled: bool) -> Vec<u8> {
    vec![GS, b'B', if enabled { 1 } else { 0 }]
}

/// Set double width
pub fn set_double_width(enabled: bool) -> Vec<u8> {
    vec![ESC, if enabled { 0x0E } else { 0x14 }]
}

/// Set double height
pub fn set_double_height(enabled: bool) -> Vec<u8> {
    if enabled {
        vec![ESC, b'd', 1]
    } else {
        vec![ESC, b'd', 0]
    }
}
