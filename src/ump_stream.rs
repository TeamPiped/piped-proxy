use crate::utils;
use bytes::{Bytes, BytesMut};
use futures_util::Stream;
use std::io;
use std::io::ErrorKind;
use std::pin::Pin;
use std::task::{Context, Poll};

fn read_variable_integer(buf: &[u8], offset: usize) -> io::Result<(i32, usize)> {
    let mut pos = offset;
    let prefix = utils::read_buf(buf, &mut pos);
    let mut size = 0;
    for shift in 1..=5 {
        if prefix & (128 >> (shift - 1)) == 0 {
            size = shift;
            break;
        }
    }
    if !(1..=5).contains(&size) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Invalid integer size {} at position {}", size, offset),
        ));
    }

    match size {
        1 => Ok((prefix as i32, size)),
        2 => {
            let value = ((utils::read_buf(buf, &mut pos) as i32) << 6) | (prefix as i32 & 0b111111);
            Ok((value, size))
        }
        3 => {
            let value = (((utils::read_buf(buf, &mut pos) as i32)
                | ((utils::read_buf(buf, &mut pos) as i32) << 8))
                << 5)
                | (prefix as i32 & 0b11111);
            Ok((value, size))
        }
        4 => {
            let value = (((utils::read_buf(buf, &mut pos) as i32)
                | ((utils::read_buf(buf, &mut pos) as i32) << 8)
                | ((utils::read_buf(buf, &mut pos) as i32) << 16))
                << 4)
                | (prefix as i32 & 0b1111);
            Ok((value, size))
        }
        _ => {
            let value = (utils::read_buf(buf, &mut pos) as i32)
                | ((utils::read_buf(buf, &mut pos) as i32) << 8)
                | ((utils::read_buf(buf, &mut pos) as i32) << 16)
                | ((utils::read_buf(buf, &mut pos) as i32) << 24);
            Ok((value, size))
        }
    }
}

pub struct UmpTransformStream<S>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Unpin,
{
    inner: S,
    buffer: BytesMut,
    found_stream: bool,
    remaining: usize,
}

impl<S> UmpTransformStream<S>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Unpin,
{
    pub fn new(stream: S) -> Self {
        UmpTransformStream {
            inner: stream,
            buffer: BytesMut::new(),
            found_stream: false,
            remaining: 0,
        }
    }
}

impl<S> Stream for UmpTransformStream<S>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Unpin,
{
    type Item = Result<Bytes, io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        while let Poll::Ready(item) = Pin::new(&mut this.inner).poll_next(cx) {
            match item {
                Some(Ok(bytes)) => {
                    if this.found_stream {
                        if this.remaining > 0 {
                            let len = std::cmp::min(this.remaining, bytes.len());
                            this.remaining -= len;
                            if this.remaining == 0 {
                                this.buffer.clear();
                                this.buffer.extend_from_slice(&bytes[len..]);
                                this.found_stream = false;
                            }
                            return Poll::Ready(Some(Ok(bytes.slice(0..len))));
                        } else {
                            this.found_stream = false;
                            this.buffer.clear();
                            this.buffer.extend_from_slice(&bytes);
                        };
                    } else {
                        this.buffer.extend_from_slice(&bytes);
                    }
                }
                Some(Err(e)) => return Poll::Ready(Some(Err(e))),
                None => {
                    return Poll::Ready(None);
                }
            }
        }

        if !this.found_stream && !this.buffer.is_empty() {
            let (segment_type, s1) = match read_variable_integer(&this.buffer, 0) {
                Ok(result) => result,
                Err(e) => return Poll::Ready(Some(Err(e))),
            };
            let (segment_length, s2) = match read_variable_integer(&this.buffer, s1) {
                Ok(result) => result,
                Err(e) => return Poll::Ready(Some(Err(e))),
            };
            if segment_type != 21 {
                // Not the stream
                if this.buffer.len() > s1 + s2 + segment_length as usize {
                    let _ = this.buffer.split_to(s1 + s2 + segment_length as usize);
                }
            } else {
                this.remaining = segment_length as usize - 1;

                let _ = this.buffer.split_to(s1 + s2 + 1);

                if this.buffer.len() > segment_length as usize {
                    let len = std::cmp::min(this.remaining, this.buffer.len());
                    this.remaining -= len;

                    return Poll::Ready(Some(Ok(this.buffer.split_to(len).into())));
                } else {
                    this.remaining -= this.buffer.len();
                    this.found_stream = true;

                    return Poll::Ready(Some(Ok(this.buffer.to_vec().into())));
                }
            }
        }

        Poll::Pending
    }
}
