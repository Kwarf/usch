use std::io::Cursor;

use cpal::{
    traits::{DeviceTrait, HostTrait},
    SampleFormat, Stream,
};

const SAMPLE_RATE: u32 = 48000;

pub enum Music {
    Pcm(Vec<i16>),
    #[cfg(feature = "ogg")]
    Ogg(&'static [u8]),
}

impl Music {
    pub(crate) fn decode(&self) -> Vec<i16> {
        match self {
            Self::Pcm(data) => data.clone(),
            #[cfg(feature = "ogg")]
            Self::Ogg(data) => {
                use lewton::inside_ogg::OggStreamReader;

                let mut cursor = Cursor::new(data);
                let mut reader = OggStreamReader::new(&mut cursor).unwrap();
                assert_eq!(2, reader.ident_hdr.audio_channels);
                assert_eq!(SAMPLE_RATE, reader.ident_hdr.audio_sample_rate);

                // Let's do one big allocation up front for 5 minutes of music, to avoid incremental mallocs
                let mut data = Vec::with_capacity(2 * SAMPLE_RATE as usize * 60 * 5);
                while let Some(mut pck) = reader.read_dec_packet_itl().unwrap() {
                    data.append(&mut pck);
                }
                data.shrink_to_fit();
                data
            }
        }
    }
}

pub(crate) fn play(data: Vec<i16>) -> Stream {
    let device = cpal::default_host().default_output_device().unwrap();
    let config = device
        .supported_output_configs()
        .unwrap()
        .find(|x| {
            x.channels() == 2
                && x.min_sample_rate().0 <= SAMPLE_RATE
                && x.max_sample_rate().0 >= SAMPLE_RATE
                && x.sample_format() == SampleFormat::F32
        })
        .expect(&format!(
            "No audio output device supporting {} sample rate found",
            SAMPLE_RATE
        ))
        .with_sample_rate(cpal::SampleRate(SAMPLE_RATE))
        .config();

    let mut reader = data.into_iter();
    device
        .build_output_stream(
            &config,
            {
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for i in data.iter_mut() {
                        *i = match reader.next() {
                            Some(sample) => cpal::Sample::from(&sample),
                            None => cpal::Sample::from(&0f32),
                        };
                    }
                }
            },
            move |_err| panic!(),
        )
        .unwrap()
}
