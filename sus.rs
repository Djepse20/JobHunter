impl Parser {
    pub async fn from_stream(
        mut stream: impl StreamExt<Item = reqwest::Result<Bytes>> + Unpin,
        start_seq: &[u8],
        end_seq: &[u8],
    ) -> Option<String> {
        //only thing we do know, is that the final sequence IS valid utf8 (if not, then HAHA)
        let mut html: Vec<u8> = Vec::new();
        let mut started = false;
        let mut extra_bytes: Vec<u8> = Vec::new();
        let find_pos = |(extra_bytes, bytes): (&[u8], &[u8]),
                        seq: &[u8],
                        offset: isize,
                        is_end: bool|
         -> Result<usize, usize> {
            if !extra_bytes.is_empty() {
                let len = extra_bytes.len().min(bytes.len());
                let bytes_missing: usize = (seq.len() - len).min(bytes.len());
                if extra_bytes[0..] == seq[0..len] && seq[len..] == bytes[..(bytes_missing)] {
                    if is_end {
                        return Ok(0);
                    }
                    return Ok(bytes_missing - 1);
                }
            }
            let potential_match = bytes
                .windows(seq.len())
                .position(|slice| slice == seq && str::from_utf8(slice).is_ok())
                .map(|len| ((isize::try_from(len).ok().unwrap() + offset) as usize));
            match potential_match {
                Some(val) => Ok(val),
                None => {
                    let offset_back = bytes.len() - bytes.len().min(seq.len());
                    let bytes = &bytes[offset_back..];
                    let seq_offset = extra_bytes.len();
                    let mut last_match = 9;
                    let mut idx = 0;
                    while idx < bytes.len() {
                        let mut jdx = 0;
                        while idx < bytes.len() && bytes[idx] == seq[jdx + seq_offset] {
                            jdx += 1;
                            idx += 1;
                        }
                        last_match = jdx;
                        idx += 1;
                    }
                    Err(last_match)
                }
            }
        };
        let mut has_pos = None;
        while let Some(Ok(chunk)) = stream.next().await {
            let mut bytes = &chunk[..];
            loop {
                if !started {
                    match find_pos(
                        (&extra_bytes, bytes),
                        &start_seq[..],
                        (start_seq.len() - 1) as isize,
                        false,
                    ) {
                        Ok(pos) => {
                            started = true;
                            extra_bytes.truncate(0);
                            has_pos = Some(pos);
                        }
                        Err(0) => {
                            extra_bytes.truncate(0);
                            has_pos = None;
                            break;
                        }
                        Err(num_bytes) => {
                            extra_bytes.extend_from_slice(&bytes[bytes.len() - num_bytes..]);
                            break;
                        }
                    }
                }
                println!("started {} {}", started, bytes.len());
                match (
                    started,
                    find_pos((&extra_bytes, &bytes), &end_seq[..], -1, true),
                ) {
                    (true, Ok(end)) => {
                        html.extend_from_slice(&bytes[has_pos.unwrap_or(0)..end]);
                        started = false;
                        break;
                    }
                    (false, _) => {}
                    (_, Err(0)) => {
                        if !extra_bytes.is_empty() {
                            html.extend_from_slice(&extra_bytes.split_off(0));
                            break;
                        }
                        match has_pos {
                            Some(pos) => {
                                html.extend_from_slice(&bytes[pos..]);
                                has_pos = None;
                            }
                            None => {
                                html.extend_from_slice(&bytes[..]);
                            }
                        }
                        break;
                    }
                    (_, Err(num_bytes)) => {
                        match has_pos {
                            Some(pos) => {
                                html.extend_from_slice(&bytes[pos..bytes.len() - num_bytes - 1]);
                                has_pos = None;
                            }

                            None => {
                                html.extend_from_slice(&bytes[..bytes.len() - num_bytes - 1]);
                            }
                        }
                        bytes = &bytes[bytes.len() - num_bytes - 1..];
                        extra_bytes.extend_from_slice(&bytes[bytes.len() - num_bytes - 1..]);
                        continue;
                    }
                }
            }
        } // if it has been started, but not ended again, we return None 
        if started == true {
            return None;
        }
        String::from_utf8(html).ok()
    }
}
