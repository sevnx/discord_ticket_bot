//! This module groups various parsing utilities.

/// Parses a Discord channel ID from a URL.
///
/// # Examples
///
/// ```
/// let url = "https://discord.com/channels/123456789/987654321";
/// let channel_id = helper::parser::parse_discord_channel_id_url(url);
/// assert_eq!(channel_id, Some(987654321));
/// ```
pub fn parse_discord_channel_id_url(url: &str) -> Option<u64> {
    url.split('/')
        .collect::<Vec<&str>>()
        .last()
        .and_then(|last_part| last_part.parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_discord_channel_id_url() {
        let url = "https://discord.com/channels/123456789/987654321";
        let channel_id = parse_discord_channel_id_url(url);
        assert_eq!(channel_id, Some(987654321));
    }
}
