pub struct WaveWriter;

impl WaveWriter {
  #[must_use]
  pub fn new_wave(samples: &[i16]) -> Vec<u8> {
    let channels: u16 = 1;
    let frequency: u32 = 48_000;
    let bits_per_sample: u16 = 16;
    let byte_per_block: u16 = channels * (bits_per_sample / 8);
    let byte_per_sec: u32 = frequency * u32::from(byte_per_block);
    #[allow(clippy::as_conversions)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::arithmetic_side_effects)]
    let data_size: u32 = samples.len() as u32 * u32::from(bits_per_sample / 8);
    #[allow(clippy::arithmetic_side_effects)]
    let file_size: u32 = data_size + 36;
    #[allow(clippy::arithmetic_side_effects)]
    let total_size: u32 = file_size + 8;

    #[allow(clippy::as_conversions)]
    let mut wave: Vec<u8> = Vec::with_capacity(total_size as usize);
    wave.extend(b"RIFF");
    wave.extend(&file_size.to_le_bytes());
    wave.extend(b"WAVE");
    wave.extend(b"fmt ");
    wave.extend(&16u32.to_le_bytes());
    wave.extend(&1u16.to_le_bytes());
    wave.extend(&channels.to_le_bytes());
    wave.extend(&frequency.to_le_bytes());
    wave.extend(&byte_per_sec.to_le_bytes());
    wave.extend(&byte_per_block.to_le_bytes());
    wave.extend(&bits_per_sample.to_le_bytes());
    wave.extend(b"data");
    wave.extend(&data_size.to_le_bytes());
    wave.extend(samples.iter().flat_map(|x| x.to_le_bytes()));

    wave
  }
}
