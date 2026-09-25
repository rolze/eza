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
    // People & Body
    "person.fill"                   => '👤',
    "person.2.fill"                 => '👥',
    "person.circle.fill"            => '👤',
    "person.crop.circle.fill"       => '👤',
    "figure.stand"                  => '🧍',
    "figure.walk"                   => '🚶',
    "figure.run"                    => '🏃',
    "figure.pregnant"               => '🤰',
    "figure.2.arms.open"            => '🤸',
    "brain"                         => '🧠',
    "brain.head.profile"            => '🧠',
    "eye"                           => '👁',
    "eye.fill"                      => '👁',
    "eye.slash"                     => '🙈',
    "mouth.fill"                    => '👄',
    "ear"                           => '👂',
    "ear.fill"                      => '👂',
    "nose"                          => '👃',
    "nose.fill"                     => '👃',
    "lungs"                         => '🫁',
    "lungs.fill"                    => '🫁',
    "hand.raised"                   => '✋',
    "hand.raised.fill"              => '✋',
    "hand.thumbsup"                 => '👍',
    "hand.thumbsup.fill"            => '👍',
    "hand.thumbsdown"               => '👎',
    "hand.thumbsdown.fill"          => '👎',
    "hand.point.up"                 => '👆',
    "hand.point.up.fill"            => '👆',
    "face.smiling"                  => '😊',
    "face.smiling.fill"             => '😊',
    "mustache"                      => '🥸',
    "mustache.fill"                 => '🥸',

    // Nature
    "leaf.fill"                     => '🍃',
    "leaf"                          => '🍃',
    "tree.fill"                     => '🌲',
    "tree"                          => '🌲',
    "sun.max.fill"                  => '☀',
    "sun.max"                       => '☀',
    "moon.fill"                     => '🌙',
    "moon"                          => '🌙',
    "cloud.fill"                    => '☁',
    "cloud.rain.fill"               => '🌧',
    "cloud.bolt.fill"               => '⛈',
    "cloud.snow.fill"               => '🌨',
    "snow"                          => '❄',
    "wind"                          => '💨',
    "tornado"                       => '🌪',
    "flame.fill"                    => '🔥',
    "flame"                         => '🔥',
    "drop.fill"                     => '💧',
    "drop"                          => '💧',
    "bolt.fill"                     => '⚡',
    "bolt"                          => '⚡',
    "ant.fill"                      => '🐜',
    "ant"                           => '🐜',
    "ladybug.fill"                  => '🐞',
    "ladybug"                       => '🐞',
    "fish.fill"                     => '🐟',
    "fish"                          => '🐟',
    "bird.fill"                     => '🐦',
    "bird"                          => '🐦',
    "pawprint.fill"                 => '🐾',
    "pawprint"                      => '🐾',
    "tortoise.fill"                 => '🐢',
    "tortoise"                      => '🐢',
    "hare.fill"                     => '🐇',
    "hare"                          => '🐇',
    "cat.fill"                      => '🐱',
    "cat"                           => '🐱',
    "dog.fill"                      => '🐶',
    "dog"                           => '🐶',
    "lizard.fill"                   => '🦎',
    "lizard"                        => '🦎',
    "snail.fill"                    => '🐌',
    "snail"                         => '🐌',
    "worm.fill"                     => '🪱',
    "worm"                          => '🪱',
    "teddybear.fill"                => '🧸',
    "teddybear"                     => '🧸',
    "globe"                         => '🌐',
    "globe.americas.fill"           => '🌎',
    "globe.europe.africa.fill"      => '🌍',
    "globe.asia.australia.fill"     => '🌏',
    "mountain.2.fill"               => '🏔',
    "mountain.2"                    => '🏔',

    // Objects & Tools
    "hammer.fill"                   => '🔨',
    "hammer"                        => '🔨',
    "wrench.fill"                   => '🔧',
    "wrench"                        => '🔧',
    "screwdriver.fill"              => '🪛',
    "screwdriver"                   => '🪛',
    "paintbrush.fill"               => '🖌',
    "paintbrush"                    => '🖌',
    "paintpalette.fill"             => '🎨',
    "paintpalette"                  => '🎨',
    "pencil"                        => '✏',
    "scissors"                      => '✂',
    "ruler.fill"                    => '📏',
    "ruler"                         => '📏',
    "magnifyingglass"               => '🔍',
    "camera.fill"                   => '📷',
    "camera"                        => '📷',
    "photo.fill"                    => '🖼',
    "photo"                         => '🖼',
    "flashlight.on.fill"            => '🔦',
    "flashlight.off.fill"           => '🔦',
    "phone.fill"                    => '📞',
    "phone"                         => '📞',
    "envelope.fill"                 => '✉',
    "envelope"                      => '✉',
    "message.fill"                  => '💬',
    "message"                       => '💬',
    "house.fill"                    => '🏠',
    "house"                         => '🏠',
    "building.fill"                 => '🏢',
    "building.2.fill"               => '🏘',
    "bicycle"                       => '🚲',
    "car.fill"                      => '🚗',
    "car"                           => '🚗',
    "bus.fill"                      => '🚌',
    "bus"                           => '🚌',
    "tram.fill"                     => '🚋',
    "tram"                          => '🚋',
    "airplane"                      => '✈',
    "boat.fill"                     => '⛵',
    "cart.fill"                     => '🛒',
    "cart"                          => '🛒',
    "bag.fill"                      => '👜',
    "bag"                           => '👜',
    "fork.knife"                    => '🍴',
    "cup.and.saucer.fill"           => '☕',
    "cup.and.saucer"                => '☕',
    "birthday.cake.fill"            => '🎂',
    "birthday.cake"                 => '🎂',
    "book.fill"                     => '📖',
    "book"                          => '📖',
    "books.vertical.fill"           => '📚',
    "books.vertical"                => '📚',
    "doc.fill"                      => '📄',
    "doc"                           => '📄',
    "folder.fill"                   => '📁',
    "folder"                        => '📁',
    "archivebox.fill"               => '📦',
    "archivebox"                    => '📦',
    "briefcase.fill"                => '💼',
    "briefcase"                     => '💼',
    "tv.fill"                       => '📺',
    "tv"                            => '📺',
    "desktopcomputer"               => '🖥',
    "laptopcomputer"                => '💻',
    "keyboard"                      => '⌨',
    "printer.fill"                  => '🖨',
    "printer"                       => '🖨',
    "externaldrive.fill"            => '💾',
    "externaldrive"                 => '💾',
    "internaldrive.fill"            => '💾',
    "music.note"                    => '🎵',
    "guitar.fill"                   => '🎸',
    "guitar"                        => '🎸',
    "piano.keys"                    => '🎹',
    "headphones"                    => '🎧',
    "speaker.wave.3.fill"           => '🔊',
    "speaker.wave.2.fill"           => '🔉',
    "speaker.wave.1.fill"           => '🔈',
    "speaker.slash.fill"            => '🔇',
    "gamecontroller.fill"           => '🎮',
    "gamecontroller"                => '🎮',
    "puzzlepiece.fill"              => '🧩',
    "puzzlepiece"                   => '🧩',
    "sportscourt.fill"              => '🏟',
    "soccerball"                    => '⚽',
    "football.fill"                 => '🏈',
    "football"                      => '🏈',
    "basketball.fill"               => '🏀',
    "basketball"                    => '🏀',
    "tennisball.fill"               => '🎾',
    "tennisball"                    => '🎾',
    "baseball.fill"                 => '⚾',
    "baseball"                      => '⚾',
    "trophy.fill"                   => '🏆',
    "trophy"                        => '🏆',
    "medal.fill"                    => '🏅',
    "medal"                         => '🏅',
    "bed.double.fill"               => '🛏',
    "sofa.fill"                     => '🛋',
    "sofa"                          => '🛋',
    "trash.fill"                    => '🗑',
    "trash"                         => '🗑',
    "lock.fill"                     => '🔒',
    "lock"                          => '🔒',
    "lock.open.fill"                => '🔓',
    "lock.open"                     => '🔓',
    "key.fill"                      => '🔑',
    "key"                           => '🔑',
    "shield.fill"                   => '🛡',
    "shield"                        => '🛡',
    "lightbulb.fill"                => '💡',
    "lightbulb"                     => '💡',
    "stethoscope"                   => '🩺',
    "cross.fill"                    => '✚',
    "cross"                         => '✚',
    "pills.fill"                    => '💊',
    "pills"                         => '💊',
    "syringe.fill"                  => '💉',
    "syringe"                       => '💉',
    "bandage.fill"                  => '🩹',
    "bandage"                       => '🩹',
    "gift.fill"                     => '🎁',
    "gift"                          => '🎁',
    "ticket.fill"                   => '🎟',
    "ticket"                        => '🎟',
    "creditcard.fill"               => '💳',
    "creditcard"                    => '💳',
    "clock.fill"                    => '🕐',
    "clock"                         => '🕐',
    "alarm.fill"                    => '⏰',
    "alarm"                         => '⏰',
    "stopwatch.fill"                => '⏱',
    "stopwatch"                     => '⏱',
    "timer"                         => '⏱',
    "map.fill"                      => '🗺',
    "map"                           => '🗺',
    "location.fill"                 => '📍',
    "location"                      => '📍',

    // Symbols & Misc
    "star.fill"                     => '★',
    "star"                          => '★',
    "heart.fill"                    => '❤',
    "heart"                         => '❤',
    "tag.fill"                      => '🏷',
    "tag"                           => '🏷',
    "flag.fill"                     => '🚩',
    "flag"                          => '🚩',
    "bookmark.fill"                 => '🔖',
    "bookmark"                      => '🔖',
    "ribbon.fill"                   => '🎀',
    "ribbon"                        => '🎀',
    "crown.fill"                    => '👑',
    "crown"                         => '👑',
    "graduationcap.fill"            => '🎓',
    "graduationcap"                 => '🎓',
    "film.fill"                     => '🎬',
    "film"                          => '🎬',
    "calendar"                      => '📅',
    "link"                          => '🔗',
    "gear"                          => '⚙',
    "gearshape.fill"                => '⚙',
    "gearshape"                     => '⚙',
    "chart.bar.fill"                => '📊',
    "chart.bar"                     => '📊',
    "chart.pie.fill"                => '📊',
    "chart.pie"                     => '📊',
    "chart.line.uptrend.xyaxis"     => '📈',
    "dollarsign.circle.fill"        => '💰',
    "eurosign.circle.fill"          => '💶',
    "sterlingsign.circle.fill"      => '💷',
    "yensign.circle.fill"           => '💴',
    "arrow.up"                      => '⬆',
    "arrow.down"                    => '⬇',
    "arrow.right"                   => '➡',
    "arrow.left"                    => '⬅',
    "arrow.up.arrow.down"           => '↕',
    "arrow.left.arrow.right"        => '↔',
    "arrow.clockwise"               => '🔃',
    "arrow.counterclockwise"        => '🔄',
    "checkmark.circle.fill"         => '✅',
    "checkmark.circle"              => '✅',
    "xmark.circle.fill"             => '❌',
    "xmark.circle"                  => '❌',
    "exclamationmark.circle.fill"   => '❗',
    "questionmark.circle.fill"      => '❓',
    "infinity"                      => '∞',
    "at"                            => '@',
    "sparkles"                      => '✨',
    "wand.and.stars.inverse"        => '✨',
    "wand.and.stars"                => '✨',
};
