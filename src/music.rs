use std::io::Cursor;

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
                assert_eq!(48000, reader.ident_hdr.audio_sample_rate);

                // Let's do one big allocation up front for 5 minutes of music, to avoid incremental mallocs
                let mut data = Vec::with_capacity(2 * 48000 * 60 * 5);
                while let Some(mut pck) = reader.read_dec_packet_itl().unwrap() {
                    data.append(&mut pck);
                }
                data.shrink_to_fit();
                data
            }
        }
    }
}
