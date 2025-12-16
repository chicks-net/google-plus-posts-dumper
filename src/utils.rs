//! String formatting and conversion utilities

use chrono::{DateTime, Utc};
use html_escape::decode_html_entities;

/// Escape double quotes and backslashes for TOML basic string values
/// Also replaces newlines with spaces to ensure single-line TOML strings
pub fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

/// Clean up title text by decoding HTML entities and stripping HTML tags
/// This handles double-encoded entities from Google+ Takeout HTML
pub fn clean_title(title: &str) -> String {
    // Decode HTML entities (this handles &#39;, &quot;, &amp;, etc.)
    let decoded = decode_html_entities(title).to_string();

    // Strip HTML tags using a simple regex-like approach
    // This handles cases like <br>, <br/>, <b>, etc.
    let mut result = String::new();
    let mut in_tag = false;

    for c in decoded.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                // Add a space where tags were to avoid word concatenation
                result.push(' ');
            }
            _ if !in_tag => result.push(c),
            _ => {} // Skip characters inside tags
        }
    }

    // Clean up multiple spaces and trim
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Clean up location text to ensure proper spacing before "Address"
/// Google+ sometimes concatenates location data without proper spacing
pub fn clean_location(location: &str) -> String {
    // Ensure there's a space before "Address" if it's not already there
    let mut result = location.to_string();

    // Check for "Address" without a preceding space
    if let Some(pos) = result.find("Address") {
        if pos > 0 {
            let before = &result[..pos];
            if !before.ends_with(|c: char| c.is_whitespace()) {
                result.insert(pos, ' ');
            }
        }
    }

    result
}

/// Convert Google+ datetime string to UTC
/// Input format: "YYYY-MM-DD HH:MM:SS±HHMM" (e.g., "2011-08-14 20:39:28-0700")
/// Output format: ISO 8601 UTC (e.g., "2011-08-15T03:39:28Z")
pub fn convert_to_utc(datetime_str: &str) -> String {
    // Parse the datetime with timezone offset
    match DateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S%z") {
        Ok(dt) => {
            // Convert to UTC
            let utc_dt: DateTime<Utc> = dt.with_timezone(&Utc);
            // Format as ISO 8601 with Z suffix
            utc_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
        }
        Err(_) => {
            // If parsing fails, return original string
            datetime_str.to_string()
        }
    }
}

/// Format filename date from YYYYMMDD to YYYY-MM-DD and clean up spacing
/// Input: "20110814 - Today is my first day..." or any other filename
/// Output: "2011-08-14-Today_is_my_first_day..."
/// - Converts YYYYMMDD to YYYY-MM-DD
/// - Replaces " - " with "-"
/// - Replaces remaining spaces with underscores
/// - Removes @, !, and # symbols
pub fn format_filename_date(filename: &str) -> String {
    // Check if filename starts with 8 digits
    if filename.len() >= 8 && filename.chars().take(8).all(|c| c.is_ascii_digit()) {
        let year = &filename[0..4];
        let month = &filename[4..6];
        let day = &filename[6..8];
        let rest = &filename[8..];

        // Replace " - " with "-", replace spaces with underscores, and remove @, !, #, &, (, )
        let rest_formatted = rest
            .trim_start_matches(" - ")
            .replace(' ', "_")
            .replace(['@', '!', '#', '&', '(', ')'], "");

        format!("{}-{}-{}-{}", year, month, day, rest_formatted)
    } else {
        // For non-date filenames, replace spaces with underscores and remove @, !, #, &, (, )
        filename
            .replace(' ', "_")
            .replace(['@', '!', '#', '&', '(', ')'], "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for escape_toml_string()
    #[test]
    fn test_escape_toml_string_simple() {
        assert_eq!(escape_toml_string("hello world"), "hello world");
    }

    #[test]
    fn test_escape_toml_string_empty() {
        assert_eq!(escape_toml_string(""), "");
    }

    #[test]
    fn test_escape_toml_string_quotes() {
        assert_eq!(
            escape_toml_string("He said \"hello\""),
            "He said \\\"hello\\\""
        );
    }

    #[test]
    fn test_escape_toml_string_backslashes() {
        assert_eq!(escape_toml_string("C:\\Users\\path"), "C:\\\\Users\\\\path");
    }

    #[test]
    fn test_escape_toml_string_mixed() {
        assert_eq!(
            escape_toml_string("\"quote\\\" and \\backslash"),
            "\\\"quote\\\\\\\" and \\\\backslash"
        );
    }

    #[test]
    fn test_escape_toml_string_unicode() {
        assert_eq!(escape_toml_string("Hello 👋 世界"), "Hello 👋 世界");
    }

    #[test]
    fn test_escape_toml_string_only_quotes() {
        assert_eq!(escape_toml_string("\"\"\""), "\\\"\\\"\\\"");
    }

    #[test]
    fn test_escape_toml_string_only_backslashes() {
        assert_eq!(escape_toml_string("\\\\\\"), "\\\\\\\\\\\\");
    }

    #[test]
    fn test_escape_toml_string_newlines() {
        assert_eq!(
            escape_toml_string("Line one\nLine two\nLine three"),
            "Line one Line two Line three"
        );
    }

    #[test]
    fn test_escape_toml_string_mixed_newlines() {
        assert_eq!(
            escape_toml_string("He said \"hello\"\nAnd then left"),
            "He said \\\"hello\\\" And then left"
        );
    }

    #[test]
    fn test_escape_toml_string_carriage_return() {
        assert_eq!(
            escape_toml_string("Windows\r\nStyle\r\nNewlines"),
            "Windows  Style  Newlines"
        );
    }

    // Tests for convert_to_utc()
    #[test]
    fn test_convert_to_utc_negative_offset() {
        assert_eq!(
            convert_to_utc("2011-08-14 20:39:28-0700"),
            "2011-08-15T03:39:28Z"
        );
    }

    #[test]
    fn test_convert_to_utc_positive_offset() {
        assert_eq!(
            convert_to_utc("2024-01-15 14:30:00+0530"),
            "2024-01-15T09:00:00Z"
        );
    }

    #[test]
    fn test_convert_to_utc_zero_offset() {
        assert_eq!(
            convert_to_utc("2024-06-15 12:00:00+0000"),
            "2024-06-15T12:00:00Z"
        );
    }

    #[test]
    fn test_convert_to_utc_midnight_boundary() {
        // 11:30 PM PST becomes 7:30 AM UTC next day
        assert_eq!(
            convert_to_utc("2024-01-15 23:30:00-0800"),
            "2024-01-16T07:30:00Z"
        );
    }

    #[test]
    fn test_convert_to_utc_date_boundary_backward() {
        // 2 AM IST becomes previous day in UTC
        assert_eq!(
            convert_to_utc("2024-01-16 02:00:00+0530"),
            "2024-01-15T20:30:00Z"
        );
    }

    #[test]
    fn test_convert_to_utc_invalid_format() {
        // Should return original string on parse error
        assert_eq!(convert_to_utc("not a date"), "not a date");
    }

    #[test]
    fn test_convert_to_utc_empty_string() {
        assert_eq!(convert_to_utc(""), "");
    }

    #[test]
    fn test_convert_to_utc_wrong_format() {
        // ISO format instead of expected format
        assert_eq!(
            convert_to_utc("2024-01-15T14:30:00Z"),
            "2024-01-15T14:30:00Z"
        );
    }

    // Tests for format_filename_date()
    #[test]
    fn test_format_filename_date_standard() {
        assert_eq!(
            format_filename_date("20110814 - Today is my first day"),
            "2011-08-14-Today_is_my_first_day"
        );
    }

    #[test]
    fn test_format_filename_date_no_separator() {
        // Function always adds dash after date
        assert_eq!(format_filename_date("20110814Today"), "2011-08-14-Today");
    }

    #[test]
    fn test_format_filename_date_multiple_spaces() {
        assert_eq!(
            format_filename_date("20110814 - Multiple Word Title Here"),
            "2011-08-14-Multiple_Word_Title_Here"
        );
    }

    #[test]
    fn test_format_filename_date_just_date() {
        assert_eq!(format_filename_date("20110814"), "2011-08-14-");
    }

    #[test]
    fn test_format_filename_date_non_date() {
        assert_eq!(format_filename_date("random file name"), "random_file_name");
    }

    #[test]
    fn test_format_filename_date_short_filename() {
        assert_eq!(format_filename_date("short"), "short");
    }

    #[test]
    fn test_format_filename_date_partial_date() {
        // 7 digits, not 8 - should not be treated as date
        assert_eq!(format_filename_date("2011081 - test"), "2011081_-_test");
    }

    #[test]
    fn test_format_filename_date_with_extension() {
        // This tests just the stem, but good to verify
        assert_eq!(
            format_filename_date("20110814 - Post Title"),
            "2011-08-14-Post_Title"
        );
    }

    #[test]
    fn test_format_filename_date_no_dash_separator() {
        // Has date but no " - " separator (just space), so space becomes underscore
        assert_eq!(
            format_filename_date("20110814 Post Title"),
            "2011-08-14-_Post_Title"
        );
    }

    #[test]
    fn test_format_filename_date_empty() {
        assert_eq!(format_filename_date(""), "");
    }

    #[test]
    fn test_format_filename_date_special_chars() {
        assert_eq!(
            format_filename_date("20110814 - Post with (parentheses) & stuff"),
            "2011-08-14-Post_with_parentheses__stuff"
        );
    }

    #[test]
    fn test_format_filename_date_removes_at_symbol() {
        assert_eq!(
            format_filename_date("20110814 - Email @someone about this"),
            "2011-08-14-Email_someone_about_this"
        );
    }

    #[test]
    fn test_format_filename_date_removes_exclamation() {
        assert_eq!(
            format_filename_date("20110814 - Wow! This is cool!"),
            "2011-08-14-Wow_This_is_cool"
        );
    }

    #[test]
    fn test_format_filename_date_removes_hash() {
        assert_eq!(
            format_filename_date("20110814 - Post about #hashtags and #coding"),
            "2011-08-14-Post_about_hashtags_and_coding"
        );
    }

    #[test]
    fn test_format_filename_date_removes_all_symbols() {
        assert_eq!(
            format_filename_date("20110814 - Wow! Email @user about #topic"),
            "2011-08-14-Wow_Email_user_about_topic"
        );
    }

    #[test]
    fn test_format_filename_date_removes_symbols_non_date() {
        assert_eq!(
            format_filename_date("My post @home about #things!"),
            "My_post_home_about_things"
        );
    }

    // Tests for clean_title()
    #[test]
    fn test_clean_title_simple() {
        assert_eq!(clean_title("Hello World"), "Hello World");
    }

    #[test]
    fn test_clean_title_empty() {
        assert_eq!(clean_title(""), "");
    }

    #[test]
    fn test_clean_title_apostrophe_entity() {
        assert_eq!(clean_title("What&#39;d you say"), "What'd you say");
    }

    #[test]
    fn test_clean_title_quote_entity() {
        assert_eq!(clean_title("&quot;Hello World&quot;"), "\"Hello World\"");
    }

    #[test]
    fn test_clean_title_ampersand_entity() {
        assert_eq!(clean_title("Penn &amp; Teller"), "Penn & Teller");
    }

    #[test]
    fn test_clean_title_less_than_greater_than() {
        // When &lt;tag&gt; is decoded, it becomes <tag> which is then stripped
        // This is correct behavior - encoded HTML tags should be removed
        assert_eq!(clean_title("&lt;tag&gt;"), "");
    }

    #[test]
    fn test_clean_title_encoded_text_with_angle_brackets() {
        // To preserve <tag> as text, it needs to be double-encoded
        // But in practice, Google+ doesn't do this, so we strip tags
        assert_eq!(clean_title("Code example: &lt;div&gt;"), "Code example:");
    }

    #[test]
    fn test_clean_title_br_tag() {
        assert_eq!(clean_title("Line one<br>Line two"), "Line one Line two");
    }

    #[test]
    fn test_clean_title_br_self_closing() {
        assert_eq!(clean_title("Line one<br/>Line two"), "Line one Line two");
    }

    #[test]
    fn test_clean_title_multiple_br_tags() {
        assert_eq!(clean_title("Line one<br><br>Line two"), "Line one Line two");
    }

    #[test]
    fn test_clean_title_bold_tag() {
        assert_eq!(clean_title("This is <b>bold</b> text"), "This is bold text");
    }

    #[test]
    fn test_clean_title_mixed_entities_and_tags() {
        assert_eq!(
            clean_title("What&#39;d you say again?<br><br>This is fabulous"),
            "What'd you say again? This is fabulous"
        );
    }

    #[test]
    fn test_clean_title_real_world_example_1() {
        // From the actual Google+ export
        assert_eq!(
            clean_title("Scott&#39;s brother Greg and his girl friend"),
            "Scott's brother Greg and his girl friend"
        );
    }

    #[test]
    fn test_clean_title_real_world_example_2() {
        // From the actual Google+ export
        assert_eq!(
            clean_title("Penn &amp; Teller rock!"),
            "Penn & Teller rock!"
        );
    }

    #[test]
    fn test_clean_title_real_world_example_3() {
        // From the actual Google+ export
        assert_eq!(
            clean_title("&quot;Lessons Learned Developing Software for Space Vehicles&quot;"),
            "\"Lessons Learned Developing Software for Space Vehicles\""
        );
    }

    #[test]
    fn test_clean_title_multiple_spaces() {
        // After tag removal, multiple spaces should be normalized
        assert_eq!(clean_title("Hello<br>  <br>  World"), "Hello World");
    }

    #[test]
    fn test_clean_title_nested_tags() {
        assert_eq!(
            clean_title("This is <b><i>nested</i></b> text"),
            "This is nested text"
        );
    }

    #[test]
    fn test_clean_title_tag_with_attributes() {
        assert_eq!(
            clean_title("Click <a href=\"http://example.com\">here</a>"),
            "Click here"
        );
    }

    #[test]
    fn test_clean_title_unicode() {
        assert_eq!(clean_title("Hello 👋 世界"), "Hello 👋 世界");
    }

    #[test]
    fn test_clean_title_unicode_with_entities() {
        assert_eq!(
            clean_title("Hello 👋 世界&#39;s best"),
            "Hello 👋 世界's best"
        );
    }

    #[test]
    fn test_clean_title_only_tags() {
        assert_eq!(clean_title("<br><br><br>"), "");
    }

    #[test]
    fn test_clean_title_truncated_entity() {
        // Handle case where HTML title was truncated mid-entity
        // The library should handle this gracefully
        assert_eq!(clean_title("Test&#3"), "Test&#3");
    }

    // Tests for clean_location()
    #[test]
    fn test_clean_location_with_missing_space() {
        assert_eq!(
            clean_location("123 Main StreetAddress: City, State"),
            "123 Main Street Address: City, State"
        );
    }

    #[test]
    fn test_clean_location_with_proper_space() {
        assert_eq!(
            clean_location("123 Main Street Address: City, State"),
            "123 Main Street Address: City, State"
        );
    }

    #[test]
    fn test_clean_location_no_address() {
        assert_eq!(clean_location("New York, NY"), "New York, NY");
    }

    #[test]
    fn test_clean_location_empty() {
        assert_eq!(clean_location(""), "");
    }

    #[test]
    fn test_clean_location_address_at_start() {
        assert_eq!(
            clean_location("Address: 123 Main Street"),
            "Address: 123 Main Street"
        );
    }

    #[test]
    fn test_clean_location_multiple_occurrences() {
        // Only fixes the first occurrence
        assert_eq!(
            clean_location("HomeAddress: 123, WorkAddress: 456"),
            "Home Address: 123, WorkAddress: 456"
        );
    }
}
