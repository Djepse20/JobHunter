use core::{num, slice, str};

use axum::body::Bytes;
use bytes::{BufMut, BytesMut};
use chrono::offset;
use futures::{Stream, StreamExt};
use sqlx::encode::IsNull;

pub(crate) struct Parser;

impl Parser {
    pub async fn from_stream<S, const N1: usize, const N2: usize>(
        mut stream: S,
        start_seq: &[u8; N1],
        end_seq: &[u8; N2],
        _len: Option<u64>,
    ) -> Option<String>
    where
        S: futures::Stream<Item = reqwest::Result<bytes::Bytes>> + Unpin,
    {
        if start_seq.is_empty() || end_seq.is_empty() {
            return None;
        }

        let mut start_j = 0;
        let mut end_j = 0;
        let mut started = false;

        let mut output: Vec<u8> = Vec::new();

        while let Some(chunk_res) = stream.next().await {
            let chunk = match chunk_res {
                Ok(c) => c,
                Err(_) => return None,
            };
            let bytes = chunk.as_ref();

            for &b in bytes {
                if !started {
                    if start_seq[start_j] != b {
                        start_j = 0;
                        continue;
                    }
                    if start_seq[start_j] == b {
                        start_j += 1;
                    }
                    if start_j == start_seq.len() {
                        started = true;
                        start_j = 0;
                        end_j = 0;
                    }
                } else {
                    output.push(b);
                    if end_seq[end_j] != b {
                        end_j = 0;
                        continue;
                    }
                    if end_seq[end_j] == b {
                        end_j += 1;
                    }

                    if end_j == N2 {
                        let remove = N2;
                        if remove > 0 {
                            if output.len() >= remove {
                                output.truncate(output.len() - remove);
                            } else {
                                output.clear();
                            }
                        }
                        output.shrink_to_fit();
                        return String::from_utf8(output).ok();
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod parsertests {

    use super::Parser;
    use axum::body::Bytes;
    use futures::stream;

    #[tokio::test]
    async fn works_on_not_split_stream() {
        let file_str = tokio::fs::read_to_string("job_test.txt")
            .await
            .unwrap();
        let stream = stream::iter(vec![Ok::<Bytes, reqwest::Error>(Bytes::from_owner(
            file_str,
        ))]);
        let start_seq = br#""results":"#;
        let end_seq = br#","skyscraper":"#;

        let result = Parser::from_stream(stream, start_seq, end_seq, None)
            .await
            .unwrap();
        let expected = r#"[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}]"#;

        assert_eq!(&result, &expected)
    }

    #[tokio::test]
    async fn works_on_split_stream() {
        let mut file_str = tokio::fs::read("job_test.txt")
            .await
            .unwrap();
        let part = file_str.split_off(40);
        let stream = stream::iter(vec![
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(file_str)),
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(part)),
        ]);
        let start_seq = br#""results":"#;
        let end_seq = br#","skyscraper":"#;

        let result = Parser::from_stream(stream, start_seq, end_seq, None)
            .await
            .unwrap();
        let expected = r#"[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}]"#;

        assert_eq!(&result, &expected)
    }

    #[tokio::test]
    async fn works_on_split_on_startseq_one_stream() {
        let mut file_str = tokio::fs::read("job_test.txt")
            .await
            .unwrap();
        let part1 = file_str.split_off(74);
        let stream = stream::iter(vec![
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(file_str)),
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(part1)),
        ]);
        let start_seq = br#""results":"#;
        let end_seq = br#","skyscraper":"#;

        let result = Parser::from_stream(stream, start_seq, end_seq, None)
            .await
            .unwrap();
        let expected = r#"[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}]"#;

        assert_eq!(&result, &expected);
    }

    #[tokio::test]
    async fn works_on_split_on_startseq_two_stream() {
        let mut file_str = tokio::fs::read("job_test.txt")
            .await
            .unwrap();
        let mut part1 = file_str.split_off(76);
        let between = file_str.split_off(72);

        let stream = stream::iter(vec![
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(file_str)),
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(between)),
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(part1)),
        ]);
        let start_seq = br#""results":"#;
        let end_seq = br#","skyscraper":"#;

        let result = Parser::from_stream(stream, start_seq, end_seq, None)
            .await
            .unwrap();
        let expected = r#"[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}]"#;

        assert_eq!(&result, &expected);
    }

    #[tokio::test]
    async fn works_on_split_on_endseq_multiple_matches_stream() {
        let mut file_str = r#"{"sus":{},"haha":{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah","results":[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"},"skyscrape": {}],"skyscraper":{"default_height":600,"default_width":160,"fallbackUrl":"\/iframe\/skyscraper\/3984","url":"\/iframe\/skyscraper\/3984?catid=-2&cattype=p"},"suggestedChanges":{"category":{"suggestions":null,"suggestionsAllParams":null,"suggestionsAllUrl":null},"company":"","jobsForUkraine":0,"query":{"suggestion":null,"suggestionParams":null,"suggestionUrl":null}},"title":"Ledigejob-Software"}}"#.as_bytes().to_vec();
        let mut part1 = file_str.split_off(163);

        let stream = stream::iter(vec![
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(file_str)),
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(part1)),
        ]);
        let start_seq = br#""results":"#;
        let end_seq = br#","skyscraper":"#;

        let result = Parser::from_stream(stream, start_seq, end_seq, None)
            .await
            .unwrap();
        let expected = r#"[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"},"skyscrape": {}]"#;

        assert_eq!(&result, &expected);
    }

    #[tokio::test]
    async fn works_on_split_on_endseq_one_stream() {
        let mut file_str = r#"{"sus":{},"haha":{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah","results":[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}],"skyscraper":{"default_height":600,"default_width":160,"fallbackUrl":"\/iframe\/skyscraper\/3984","url":"\/iframe\/skyscraper\/3984?catid=-2&cattype=p"},"suggestedChanges":{"category":{"suggestions":null,"suggestionsAllParams":null,"suggestionsAllUrl":null},"company":"","jobsForUkraine":0,"query":{"suggestion":null,"suggestionParams":null,"suggestionUrl":null}},"title":"Ledigejob-Software"}}"#.as_bytes().to_vec();
        let mut part1 = file_str.split_off(163);

        let stream = stream::iter(vec![
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(file_str)),
            Ok::<Bytes, reqwest::Error>(Bytes::from_owner(part1)),
        ]);
        let start_seq = br#""results":"#;
        let end_seq = br#","skyscraper":"#;

        let result = Parser::from_stream(stream, start_seq, end_seq, None)
            .await
            .unwrap();
        let expected = r#"[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}]"#;

        assert_eq!(&result, &expected);
    }

    #[tokio::test]
    async fn works_on_split_at_colon() {
        let  file_str = r#"{"sus":{},"haha":{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah","results":[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}],"skyscraper":{"default_height":600,"default_width":160,"fallbackUrl":"\/iframe\/skyscraper\/3984","url":"\/iframe\/skyscraper\/3984?catid=-2&cattype=p"},"suggestedChanges":{"category":{"suggestions":null,"suggestionsAllParams":null,"suggestionsAllUrl":null},"company":"","jobsForUkraine":0,"query":{"suggestion":null,"suggestionParams":null,"suggestionUrl":null}},"title":"Ledigejob-Software"}}"#
        .as_bytes()
        .to_vec();
        let file_str = file_str
            .split_inclusive(|b| *b == b':')
            .map(|string| Ok::<Bytes, reqwest::Error>(Bytes::from_owner(string.to_vec())));

        let stream = stream::iter(file_str);
        let start_seq = br#""results":"#;
        let end_seq = br#","skyscraper":"#;

        let result = Parser::from_stream(stream, start_seq, end_seq, None)
            .await
            .unwrap();
        let expected = r#"[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}]"#;
        assert_eq!(&result, &expected);
    }

    #[tokio::test]
    async fn works_on_split_at_s() {
        let  file_str = r#"{"sus":{},"haha":{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah","results":[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}],"skyscraper":{"default_height":600,"default_width":160,"fallbackUrl":"\/iframe\/skyscraper\/3984","url":"\/iframe\/skyscraper\/3984?catid=-2&cattype=p"},"suggestedChanges":{"category":{"suggestions":null,"suggestionsAllParams":null,"suggestionsAllUrl":null},"company":"","jobsForUkraine":0,"query":{"suggestion":null,"suggestionParams":null,"suggestionUrl":null}},"title":"Ledigejob-Software"}}"#
        .as_bytes()
        .to_vec();
        let file_str = file_str
            .split_inclusive(|b| *b == b's')
            .map(|string| Ok::<Bytes, reqwest::Error>(Bytes::from_owner(string.to_vec())));

        let stream = stream::iter(file_str);
        let start_seq = br#""results":"#;
        let end_seq = br#","skyscraper":"#;

        let result = Parser::from_stream(stream, start_seq, end_seq, None)
            .await
            .unwrap();
        let expected = r#"[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"}]"#;

        assert_eq!(&result, &expected);
    }
}
