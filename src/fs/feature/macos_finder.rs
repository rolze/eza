// SPDX-FileCopyrightText: 2024 eza contributors
// SPDX-License-Identifier: MIT
//! macOS Finder metadata: tag colors and SF Symbol folder overlays.

use nu_ansi_term::{Color, Style};
use phf::phf_map;

use crate::fs::feature::xattr::Attribute;

const TAG_XATTR: &str = "com.apple.metadata:_kMDItemUserTags";
const SYMBOL_XATTR: &str = "com.apple.icon.folder#S";

/// Returns the Finder tag color index (1–7) for the first colored tag found,
/// by parsing the binary plist in `_kMDItemUserTags`.
pub fn finder_tag_color(attrs: &[Attribute]) -> Option<u8> {
    let data = attr_value(attrs, TAG_XATTR)?;
    parse_tag_color(data)
}

/// Returns a Unicode char representing the folder's SF Symbol overlay,
/// or `'◆'` if the symbol name has no known mapping.
/// Returns `None` if no symbol xattr is present.
pub fn finder_symbol_char(attrs: &[Attribute]) -> Option<char> {
    let data = attr_value(attrs, SYMBOL_XATTR)?;
    let text = std::str::from_utf8(data).ok()?;
    let sym = extract_sym_field(text)?;
    Some(SF_SYMBOL_MAP.get(sym).copied().unwrap_or('◆'))
}

/// Maps a Finder tag color index to an ANSI terminal style.
pub fn finder_tag_style(color_idx: u8) -> Style {
    match color_idx {
        1 => Style::new().fg(Color::DarkGray),
        2 => Style::new().fg(Color::Green),
        3 => Style::new().fg(Color::Purple),
        4 => Style::new().fg(Color::Blue),
        5 => Style::new().fg(Color::Yellow),
        6 => Style::new().fg(Color::Red),
        7 => Style::new().fg(Color::Fixed(208)), // Orange
        _ => Style::default(),
    }
}

fn attr_value<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a [u8]> {
    attrs
        .iter()
        .find(|a| a.name == name)
        .and_then(|a| a.value.as_deref())
}

/// Minimal binary plist parser for the specific format of `_kMDItemUserTags`.
/// Format: "bplist00" magic + array of short UTF-8 strings "TagName\nN".
fn parse_tag_color(data: &[u8]) -> Option<u8> {
    // Must start with bplist00
    if data.len() < 9 || &data[..8] != b"bplist00" {
        return None;
    }
    let mut pos = 8;

    // Expect array marker 0xAx (x = element count, for small arrays)
    let marker = *data.get(pos)?;
    if (marker & 0xF0) != 0xA0 {
        return None;
    }
    let count = (marker & 0x0F) as usize;
    pos += 1;

    for _ in 0..count {
        let str_marker = *data.get(pos)?;
        if (str_marker & 0xF0) != 0x50 {
            // Skip non-string objects
            pos += 1;
            continue;
        }
        let str_len = if (str_marker & 0x0F) == 0x0F {
            // Extended length: next byte is 0x10, byte after is length
            if data.get(pos + 1).copied() != Some(0x10) {
                return None;
            }
            let l = *data.get(pos + 2)? as usize;
            pos += 3;
            l
        } else {
            let l = (str_marker & 0x0F) as usize;
            pos += 1;
            l
        };

        let end = pos.checked_add(str_len)?;
        let str_bytes = data.get(pos..end)?;
        pos = end;

        // Tag string is "TagName\nN" — color index is the last byte
        if let Some(&digit) = str_bytes.last() {
            if (b'1'..=b'7').contains(&digit) {
                return Some(digit - b'0');
            }
        }
    }
    None
}

/// Extracts the value of the `sym` field from `{"sym":"<name>"}`.
/// Handles the simple single-field JSON that macOS writes.
fn extract_sym_field(json: &str) -> Option<&str> {
    let key = "\"sym\"";
    let key_pos = json.find(key)?;
    let after_key = &json[key_pos + key.len()..];
    let colon = after_key.find(':')? + 1;
    let after_colon = after_key[colon..].trim_start();
    if after_colon.starts_with('"') {
        let inner = &after_colon[1..];
        let end = inner.find('"')?;
        Some(&inner[..end])
    } else {
        None
    }
}

static SF_SYMBOL_MAP: phf::Map<&'static str, char> = phf_map! {
    "hammer.fill"           => '🔨',
    "star.fill"             => '★',
    "heart.fill"            => '❤',
    "lock.fill"             => '🔒',
    "person.fill"           => '👤',
    "gear"                  => '⚙',
    "house.fill"            => '🏠',
    "trash.fill"            => '🗑',
    "archivebox.fill"       => '📦',
    "briefcase.fill"        => '💼',
    "book.fill"             => '📖',
    "cloud.fill"            => '☁',
    "calendar"              => '📅',
    "film.fill"             => '🎬',
    "flag.fill"             => '🚩',
    "flame.fill"            => '🔥',
    "lightbulb.fill"        => '💡',
    "wrench.fill"           => '🔧',
    "paintbrush.fill"       => '🖌',
    "magnifyingglass"       => '🔍',
    "music.note"            => '🎵',
    "camera.fill"           => '📷',
    "envelope.fill"         => '✉',
    "gamecontroller.fill"   => '🎮',
    "graduationcap.fill"    => '🎓',
    "leaf.fill"             => '🍃',
    "car.fill"              => '🚗',
    "airplane"              => '✈',
    "tag.fill"              => '🏷',
    "link"                  => '🔗',
    "dollarsign.circle.fill"=> '💰',
    "person.2.fill"         => '👥',
    "chart.bar.fill"        => '📊',
    "doc.fill"              => '📄',
    "folder.fill"           => '📁',
};
