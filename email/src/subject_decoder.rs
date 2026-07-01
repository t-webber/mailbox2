/// Parser for a Quoted string.
#[derive(Default)]
struct QEqParser {
    /// Decoded result buffer.
    buf: String,
    /// Bytes being parsed.
    bytes: Vec<u8>,
    /// Parsing state.
    state: QEqState,
}

impl QEqParser {
    /// Parses one more char with the given state.
    fn add_char(&mut self, ch: char) {
        match self.state {
            QEqState::None if ch == '=' => self.state = QEqState::Eq,
            QEqState::None => {
                self.clear_bytes();
                self.buf.push(ch);
            }
            QEqState::Eq => self.state = QEqState::One(ch),
            QEqState::One(prev) if ch == '=' => {
                self.clear_bytes();
                self.buf.extend(['=', prev]);
                self.state = QEqState::Eq;
            }
            QEqState::One(prev) => {
                if let Some(prev_digit) = hex_char_to_int(prev)
                    && let Some(next_digit) = hex_char_to_int(ch)
                {
                    #[expect(
                        clippy::arithmetic_side_effects,
                        reason = "safe here"
                    )]
                    self.bytes.push(prev_digit * 16 + next_digit);
                } else {
                    self.clear_bytes();
                    self.buf.extend(['=', prev, ch]);
                }
                self.state = QEqState::None;
            }
        }
    }

    /// Clears the state to push everything into the buffer.
    fn clear(&mut self) {
        self.clear_bytes();
        match self.state {
            QEqState::None => (),
            QEqState::Eq => self.buf.push('='),
            QEqState::One(ch) => self.buf.extend(['=', ch]),
        }
    }

    /// Clears the bytes, and pushes them onto the buffer.
    fn clear_bytes(&mut self) {
        self.buf.push_str(&String::from_utf8_lossy(&self.bytes));
        self.bytes.clear();
    }

    /// Parses the whole string.
    fn parse(raw: &str) -> String {
        let mut this = Self::default();
        for ch in raw.replace('_', " ").replace("?= =?UTF-8?Q?", "").chars() {
            this.add_char(ch);
        }
        this.clear();
        this.buf
    }
}

/// Parsing state for quoted strings.
#[derive(Default)]
enum QEqState {
    /// Found an '=' char.
    Eq,
    /// No special byte being read.
    #[default]
    None,
    /// Found an '=' char followed by another char.
    One(char),
}

/// Decodes the subject.
///
/// It supports multiple encodings, under RFC2047.
///
/// - No encoding.
/// - Base 64 encoding.
/// - Quote encoding.
pub fn decode_subject(raw: &str) -> String {
    let out = if let Some(no_prefix) = raw.strip_prefix("=?UTF-8?Q?")
        && let Some(current) = no_prefix.strip_suffix("?=")
    {
        QEqParser::parse(current)
    } else if let Some(no_prefix) = raw.strip_prefix("=?UTF-8?B?")
        && let Some(current) = no_prefix.strip_suffix("?=")
        && let Some(decoded) = decode_base64(current)
    {
        decoded
    } else {
        raw.to_owned()
    };
    out.trim().to_owned()
}

/// Decodes a Base64 block.
#[expect(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "safe here"
)]
fn decode_base64(raw: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0u8;

    for byte in raw.bytes() {
        if byte == b'=' {
            break;
        }

        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };

        buffer = (buffer << 6u32) | u32::from(value);
        bits_collected += 6;

        if bits_collected >= 8 {
            bits_collected -= 8;
            bytes.push((buffer >> bits_collected) as u8);
        }
    }

    Some(String::from_utf8_lossy(&bytes).to_string())
}

/// Converts a char to an hexdigit, if it is one.
#[expect(
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "safe here"
)]
const fn hex_char_to_int(ch: char) -> Option<u8> {
    Some(match ch {
        '0'..='9' => (ch as u8) - b'0',
        'a'..='f' => (ch as u8) - b'a' + 10,
        'A'..='F' => (ch as u8) - b'A' + 10,
        _ => return None,
    })
}
