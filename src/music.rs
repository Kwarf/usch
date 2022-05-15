use std::{sync::{Arc, Mutex}, cmp::min, iter, time::Duration};

use crate::DemoBuilder;

impl DemoBuilder {
    #[cfg(feature = "ogg")]
    pub fn with_ogg_music(mut self, data: &[u8], samples_hint: Option<usize>) -> DemoBuilder {
        let mut cursor = std::io::Cursor::new(data);
        let mut reader = lewton::inside_ogg::OggStreamReader::new(&mut cursor).unwrap();
        assert_eq!(2, reader.ident_hdr.audio_channels);

        let mut data = Vec::with_capacity(samples_hint.unwrap_or_default());
        while let Some(mut pck) = reader.read_dec_packet_itl().unwrap() {
            data.append(&mut pck);
        }

        #[cfg(debug_assertions)]
        println!(
            "Decoded {} audio samples ({} B)",
            data.len(),
            data.len() * std::mem::size_of::<i16>(),
        );

        self.demo.music = Some(Arc::new(Mutex::new(Music {
            sample_rate: reader.ident_hdr.audio_sample_rate,
            data,
            position: 0,
            #[cfg(feature = "ui")]
            paused: false,
            #[cfg(feature = "ui")]
            zero: Vec::new(),
        })));
        self
    }
}

pub struct Music {
    pub(super) sample_rate: u32,
    pub(super) data: Vec<i16>,
    pub(super) position: usize,
    #[cfg(feature = "ui")]
    pub(super) paused: bool,
    #[cfg(feature = "ui")]
    zero: Vec<i16>,
}

impl Music {
    pub fn read<'a>(&mut self, len: usize) -> &[i16] {
        #[cfg(feature = "ui")]
        if self.paused {
            if self.zero.len() < len {
                self.zero = iter::repeat(0).take(len).collect();
            }
            return &self.zero;
        }

        let data = &self.data[self.position..self.position + len];
        self.position += min(self.data.len(), len);
        data
    }

    #[cfg(feature = "ui")]
    pub fn seek(&mut self, position: &Duration) {
        self.position = (position.as_secs_f32() * self.sample_rate as f32) as usize * 2;
    }
}
