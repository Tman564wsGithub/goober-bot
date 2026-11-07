use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct AudioBuffer {
  len: usize,
  buf: VecDeque<i16>,
}

impl AudioBuffer {
  #[must_use]
  pub fn new(len: usize) -> Self {
    if len == 0 {
      return Self {
        len,
        buf: VecDeque::new(),
      };
    }

    let mut vec: Vec<i16> = Vec::with_capacity(len);
    let audio_buf_filler = vec![0i16; len].into_boxed_slice();
    vec.extend(audio_buf_filler);

    let buf = VecDeque::from(vec);

    Self { len, buf }
  }

  pub fn push(&mut self, value: i16) {
    if self.len == 0 {
      return;
    }

    self.buf.pop_front();
    self.buf.push_back(value);
  }

  pub fn push_slice(&mut self, slice: &[i16]) {
    if self.len == 0 {
      return;
    }

    self.buf.drain(0..slice.len());
    self.buf.extend(slice);
  }

  #[must_use]
  pub fn as_vec(&self) -> Vec<i16> {
    let (first, last) = self.buf.as_slices();

    #[allow(clippy::tuple_array_conversions)]
    [first, last].concat()
  }

  #[must_use]
  pub const fn len(&self) -> usize {
    self.len
  }

  #[must_use]
  pub const fn is_empty(&self) -> bool {
    self.buf.is_empty()
  }
}

impl Default for AudioBuffer {
  fn default() -> Self {
    Self::new(48_000 * 30)
  }
}
