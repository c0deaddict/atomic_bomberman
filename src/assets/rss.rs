use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    utils::BoxedFuture,
};
use byteorder::{WriteBytesExt, LE};

const SAMPLE_RATE: u32 = 44100;
const NUM_CHANNELS: u16 = 1;
const BITS_PER_SAMPLE: u16 = 16;

/// RSS supposedly is an acronym for "Raw Sound Stream". It has no header, it
/// only contains audio samples. The samples are PCM, 16bit, 44100Hz, little
/// endian, mono.
///
/// Bevy is using the `rodio` library for audio decoding. An audio asset
/// contains just the raw bytes of the loaded file. By prepending a WAV header
/// to the data with the fixed parameters, rodio can play the asset.
#[derive(Default)]
pub struct RssAssetLoader;

impl AssetLoader for RssAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(AudioSource {
                bytes: prepend_wav_header(bytes).unwrap().into(),
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rss", "RSS"]
    }
}

// http://soundfile.sapp.org/doc/WaveFormat/
fn prepend_wav_header(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut res = Vec::with_capacity(bytes.len() + 44);

    let block_align: u16 = NUM_CHANNELS * (BITS_PER_SAMPLE / 8);
    let byte_rate: u32 = SAMPLE_RATE * block_align as u32;

    // ChunkID
    res.extend_from_slice(b"RIFF");
    // ChunkSize: file size - 8 bytes
    res.write_u32::<LE>(36 + bytes.len() as u32)?;
    // Format
    res.extend_from_slice(b"WAVE");
    // Subchunk1ID
    res.extend_from_slice(b"fmt ");
    // Subchunk1Size: 16 = PCM)
    res.write_u32::<LE>(16)?;
    // AudioFormat: 1 = PCM
    res.write_u16::<LE>(1)?;
    // NumChannels: 1
    res.write_u16::<LE>(NUM_CHANNELS)?;
    // SampleRate: 44100
    res.write_u32::<LE>(SAMPLE_RATE)?;
    // ByteRate: SampleRate * NumChannels * BitsPerSample / 8
    res.write_u32::<LE>(byte_rate)?;
    // BlockAlign: NumChannels * BitsPerSample / 8
    res.write_u16::<LE>(block_align)?;
    // BitsPerSample
    res.write_u16::<LE>(BITS_PER_SAMPLE)?;

    // SubChunk2ID
    res.extend_from_slice(b"data");
    // Subchunk2Size
    res.write_u32::<LE>(bytes.len() as u32)?;
    // Samples
    res.extend_from_slice(bytes);

    Ok(res)
}
