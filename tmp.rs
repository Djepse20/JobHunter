pub struct SeqDeserializer<'de, R, T> {
    seq: Option<SeqAccessOwned<R>>,
    life_time: PhantomData<&'de ()>,
    output: PhantomData<T>,
}

impl<'de, R: Read<'de>> Deserializer<R> {
    pub fn into_iter_seq<T>(self) -> SeqDeserializer<'de, R, T>
    where
        T: de::Deserialize<'de>,
    {
        // This cannot be an implementation of std::iter::IntoIterator because
        // we need the caller to choose what T is.
        SeqDeserializer::new(self)
    }
}

impl<'de, R: Read<'de>> de::SeqAccess<'de> for SeqAccessOwned<R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        fn has_next_element<'de, R: Read<'de>>(
            seq: &mut SeqAccessOwned<R>,
        ) -> Result<bool> {
            let peek = match tri!(seq.de.parse_whitespace()) {
                Some(b) => b,
                None => {
                    return Err(seq
                        .de
                        .peek_error(ErrorCode::EofWhileParsingList));
                }
            };

            if peek == b']' {
                Ok(false)
            } else if seq.first {
                seq.first = false;
                Ok(true)
            } else if peek == b',' {
                seq.de.eat_char();
                match tri!(seq.de.parse_whitespace()) {
                    Some(b']') => {
                        Err(seq.de.peek_error(ErrorCode::TrailingComma))
                    }
                    Some(_) => Ok(true),
                    None => {
                        Err(seq.de.peek_error(ErrorCode::EofWhileParsingValue))
                    }
                }
            } else {
                Err(seq.de.peek_error(ErrorCode::ExpectedListCommaOrEnd))
            }
        }

        if tri!(has_next_element(self)) {
            Ok(Some(tri!(seed.deserialize(&mut self.de))))
        } else {
            Ok(None)
        }
    }
}

impl<'de, R: Read<'de>, T> SeqDeserializer<'de, R, T> {
    fn new(mut de: Deserializer<R>) -> SeqDeserializer<'de, R, T> {
        let seq = match de.parse_whitespace() {
            Ok(Some(b'[')) => {
                de.eat_char();
                Some(SeqAccessOwned::new(de))
            }
            _ => None,
        };

        SeqDeserializer {
            seq: seq,
            output: PhantomData,
            life_time: PhantomData,
        }
    }
}
impl<'de, R: Read<'de>, T: serde::Deserialize<'de> + 'de> Iterator
    for SeqDeserializer<'de, R, T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.seq.as_mut().and_then(|seq| {
            de::SeqAccess::next_element::<T>(seq).ok().flatten()
        })
    }
}
