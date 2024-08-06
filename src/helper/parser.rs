//! This module groups various parsing utilities.

/// Parses a Discord channel ID from a URL.
/// Can also parse a simple channel ID.
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

/// Parses a Discord mention from a string (form <@ID> or <@!ID>)
/// Can also parse a simple ID.
///
/// # Examples
///
/// ```
/// let mention = "<@123456789>";
/// let id = helper::parser::parse_discord_mention("987654321");
/// assert_eq!(id, Some(123456789));
///
/// let id = "123456789";
/// let id = helper::parser::parse_discord_mention(id);
/// assert_eq!(id, Some(123456789));
/// ```
pub fn parse_discord_mention(mention: &str) -> Option<u64> {
    info!("Parsing mention: {}", mention);
    if mention.starts_with("<@") && mention.ends_with('>') {
        // Handle <@ID>, <@!ID> and <@&ID>
        mention
            .trim_start_matches("<@")
            .trim_start_matches('!')
            .trim_start_matches('&')
            .trim_end_matches('>')
            .parse()
            .ok()
    } else {
        mention.parse().ok()
    }
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

    #[test]
    fn test_parse_discord_channel_id_url_no_id() {
        let url = "https://discord.com/channels/123456789/";
        let channel_id = parse_discord_channel_id_url(url);
        assert_eq!(channel_id, None);
    }

    #[test]
    fn test_parse_discord_channel_simple_code() {
        let url = "987654321";
        let channel_id = parse_discord_channel_id_url(url);
        assert_eq!(channel_id, Some(987654321));
    }

    #[test]
    fn test_parse_discord_mention() {
        let mention = "<@123456789>";
        let id = parse_discord_mention(mention);
        assert_eq!(id, Some(123456789));
    }
}
