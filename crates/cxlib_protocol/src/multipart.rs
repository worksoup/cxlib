//! fork from [multipart](https://crates.io/crates/multipart)
//! 此文件存在的意义是为了避免 rust 的警告。
//! TODO: 需要测试。
use log::debug;
use mime_guess::{Mime, mime};
use rand::Rng;
use std::{
    borrow::Cow,
    fmt::Display,
    io::{Cursor, Read, Write},
};

struct PreparedField<'d> {
    header: Cursor<Vec<u8>>,
    stream: Box<dyn Read + 'd>,
}
pub struct PreparedFields<'d> {
    text_data: Cursor<Vec<u8>>,
    streams: Vec<PreparedField<'d>>,
    end_boundary: Cursor<String>,
}
impl Read for PreparedField<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        debug!("PreparedField::read()");

        if !cursor_at_end(&self.header) {
            self.header.read(buf)
        } else {
            self.stream.read(buf)
        }
    }
}
impl Read for PreparedFields<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            debug!("PreparedFields::read() was passed a zero-sized buffer.");
            return Ok(0);
        }

        let mut total_read = 0;

        while total_read < buf.len() && !cursor_at_end(&self.end_boundary) {
            let buf = &mut buf[total_read..];

            total_read += if !cursor_at_end(&self.text_data) {
                self.text_data.read(buf)?
            } else if let Some(mut field) = self.streams.pop() {
                match field.read(buf) {
                    Ok(0) => continue,
                    res => {
                        self.streams.push(field);
                        res
                    }
                }?
            } else {
                self.end_boundary.read(buf)?
            };
        }

        Ok(total_read)
    }
}
impl<'d> PreparedFields<'d> {
    #[inline]
    pub fn get_boundary(&self) -> &str {
        let boundary = self.end_boundary.get_ref();
        &boundary[4..boundary.len() - 2]
    }
    #[inline]
    pub fn from_fields<'n, FileName: ToOwned + Display + ?Sized>(
        fields: &mut Vec<Field<'n, 'd, FileName>>,
    ) -> Result<Self, std::io::Error> {
        fn from_stream<'d>(
            name: &str,
            boundary: &str,
            content_type: &Mime,
            filename: Option<impl Display>,
            stream: Box<dyn Read + 'd>,
        ) -> PreparedField<'d> {
            let mut header = Vec::new();

            write!(
                header,
                "{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"",
            )
            .unwrap();

            if let Some(filename) = filename {
                write!(header, "; filename=\"{filename}\"").unwrap();
            }

            write!(header, "\r\nContent-Type: {content_type}\r\n\r\n",).unwrap();

            PreparedField {
                header: Cursor::new(header),
                stream,
            }
        }
        debug!("Field count: {}", fields.len());
        let mut boundary = format!(
            "\r\n--{}",
            rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(16)
                .map(|c| c as char)
                .collect::<String>()
        );

        let mut text_data = Vec::new();
        let mut streams = Vec::new();

        for field in fields.drain(..) {
            match field.data {
                Data::Text(text) => write!(
                    text_data,
                    "{}\r\nContent-Disposition: form-data; \
                     name=\"{}\"\r\n\r\n{}",
                    boundary, field.name, text
                )
                .expect("[multipart]: Unable to write boundary."),
                Data::Stream(stream) => {
                    streams.push(from_stream(
                        &field.name,
                        &boundary,
                        &stream.content_type,
                        stream.filename.as_deref(),
                        stream.stream,
                    ));
                }
            }
        }

        // So we don't write a spurious end boundary
        if text_data.is_empty() && streams.is_empty() {
            boundary = String::new();
        } else {
            boundary.push_str("--");
        }

        Ok(PreparedFields {
            text_data: Cursor::new(text_data),
            streams,
            end_boundary: Cursor::new(boundary),
        })
    }
}
struct Stream<'n, 'd, FileName: ToOwned + ?Sized> {
    filename: Option<Cow<'n, FileName>>,
    content_type: Mime,
    stream: Box<dyn Read + 'd>,
}
enum Data<'n, 'd, FileName: ToOwned + ?Sized> {
    Text(Cow<'d, str>),
    Stream(Stream<'n, 'd, FileName>),
}
pub struct Field<'n, 'd, FileName: ToOwned + ?Sized> {
    name: Cow<'n, str>,
    data: Data<'n, 'd, FileName>,
}
#[inline]
fn cursor_at_end<T: AsRef<[u8]>>(cursor: &Cursor<T>) -> bool {
    cursor.position() == (cursor.get_ref().as_ref().len() as u64)
}
impl<'n, 'd, FileName: ToOwned + ?Sized> Field<'n, 'd, FileName> {
    pub fn add_stream<N, R, F>(
        fields: &mut Vec<Field<'n, 'd, FileName>>,
        name: N,
        stream: R,
        filename: Option<F>,
        mime: Option<Mime>,
    ) where
        N: Into<Cow<'n, str>>,
        R: Read + 'd,
        F: Into<Cow<'n, FileName>>,
    {
        fields.push(Field {
            name: name.into(),
            data: Data::Stream(Stream {
                content_type: mime.unwrap_or(mime::APPLICATION_OCTET_STREAM),
                filename: filename.map(|f| f.into()),
                stream: Box::new(stream),
            }),
        });
    }
    #[inline]
    pub fn add_text<N, T>(fields: &mut Vec<Field<'n, 'd, FileName>>, name: N, text: T)
    where
        N: Into<Cow<'n, str>>,
        T: Into<Cow<'d, str>>,
    {
        fields.push(Field {
            name: name.into(),
            data: Data::Text(text.into()),
        });
    }
}
