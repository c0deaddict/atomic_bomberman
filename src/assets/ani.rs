use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    render::texture::{Extent3d, Texture, TextureDimension, TextureFormat},
    utils::BoxedFuture,
};
use byteorder::{ReadBytesExt, LE};
use std::fmt::Debug;
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Convert Bomberman specific ANI format to a sprite sheet.
///
/// Inspired by:
/// https://github.com/mmatyas/ab_aniex/blob/master/src/AniFile.cpp
/// https://github.com/image-rs/image/blob/master/src/codecs/bmp/decoder.rs
#[derive(Default)]
pub struct AniAssetLoader;

impl AssetLoader for AniAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut decoder = AniDecoder::new(bytes)?;
            let texture = decoder.read_image_data()?;
            println!("image loaded");
            load_context.set_default_asset(LoadedAsset::new(texture));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ani", "ANI"]
    }
}

#[derive(Debug, Default)]
struct AniItem {
    signature: [u8; 4],
    id: u16,
    length: u32,
    start: u64,
}

impl AniItem {
    fn skip(&self, cursor: &mut Cursor<&[u8]>) -> Result<()> {
        cursor.seek(SeekFrom::Current(self.length as i64))?;
        Ok(())
    }
}

struct Image {
    width: usize,
    height: usize,
    /// 32bit RGBA encoded pixels.
    data: Vec<u8>,
}

#[derive(Default, Debug)]
struct AniDecoder<'a> {
    cursor: Cursor<&'a [u8]>,
    has_loaded_metadata: bool,
    id: u16,
    file_end: u64,
}

struct TgaRleIterator<'a, 'b> {
    cursor: &'a mut Cursor<&'b [u8]>,
    rle_data: Option<u16>,
    rle_len: u8,
    raw_len: u8,
}

impl<'a, 'b> TgaRleIterator<'a, 'b> {
    fn new(cursor: &'a mut Cursor<&'b [u8]>) -> Self {
        TgaRleIterator {
            cursor,
            rle_data: None,
            rle_len: 0,
            raw_len: 0,
        }
    }
}

// http://www.ludorg.net/amnesia/TGA_File_Format_Spec.html
impl<'a, 'b> Iterator for TgaRleIterator<'a, 'b> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<u16> {
        if self.rle_len > 0 {
            self.rle_len -= 1;
            return self.rle_data;
        } else if self.raw_len > 0 {
            self.raw_len -= 1;
            return self.cursor.read_u16::<LE>().ok();
        }

        let control = match self.cursor.read_u8() {
            Ok(b) => b,
            Err(_) => return None,
        };

        // For both RLE and Raw packets we need to the next pixel value.
        let len = control & 0b01111111;
        let data = match self.cursor.read_u16::<LE>() {
            Ok(b) => b,
            Err(_) => return None,
        };

        // RLE sequence if highest bit is set.
        if control & 0b10000000 != 0 {
            self.rle_len = len;
            self.rle_data = Some(data);
            self.rle_data
        } else {
            self.raw_len = len;
            Some(data)
        }
    }
}

impl<'a> AniDecoder<'a> {
    fn new(bytes: &'a [u8]) -> Result<Self> {
        let mut decoder = AniDecoder {
            cursor: Cursor::new(bytes),
            ..Default::default()
        };

        decoder.read_metadata()?;
        Ok(decoder)
    }

    fn read_metadata(&mut self) -> Result<()> {
        if !self.has_loaded_metadata {
            self.read_file_header()?;
            self.has_loaded_metadata = true;
        }

        Ok(())
    }

    fn read_file_header(&mut self) -> Result<()> {
        let mut signature = [0; 10];
        self.cursor.read_exact(&mut signature)?;
        if signature != *b"CHFILEANI " {
            bail!("ANI signature is invalid");
        }

        let file_length = self.cursor.read_u32::<LE>()?;
        let _file_id = self.cursor.read_u16::<LE>()?;
        self.file_end = self.cursor.position() + file_length as u64;

        Ok(())
    }

    fn read_item(&mut self) -> Result<AniItem> {
        let mut item = AniItem::default();
        self.cursor.read_exact(&mut item.signature)?;
        item.length = self.cursor.read_u32::<LE>()?;
        item.id = self.cursor.read_u16::<LE>()?;
        item.start = self.cursor.position();
        Ok(item)
    }

    fn parse_frame(&mut self, frame: AniItem) -> Result<Image> {
        let mut _name = None;
        let mut image = None;

        while self.cursor.position() < frame.start + frame.length as u64 {
            let item = self.read_item()?;
            match &item.signature {
                b"FNAM" => {
                    let mut buf = vec![0; item.length as usize];
                    self.cursor.read_exact(&mut buf)?;
                    buf.pop(); // remove \0
                    _name = Some(String::from_utf8(buf)?);
                }
                b"CIMG" => image = Some(self.parse_image(item)?),
                _ => item.skip(&mut self.cursor)?,
            }
        }

        Ok(image.unwrap())
    }

    fn parse_image(&mut self, item: AniItem) -> Result<Image> {
        if item.length < 32 {
            bail!("CIMG is too small: {} < 32", item.length);
        }

        if self.cursor.read_u16::<LE>()? != 0x0004 {
            bail!("CIMG type must be 0x0004 (16 bits per pixel)");
        }

        // Unknown field.
        println!("{:#018b}", self.cursor.read_u16::<LE>()?);

        let additional_size = self.cursor.read_u32::<LE>()?;
        if additional_size >= 32 {
            bail!("CIMG palette header not supported");
        }

        // Unknown field.
        self.cursor.read_u32::<LE>()?;

        let width = self.cursor.read_u16::<LE>()? as usize;
        let height = self.cursor.read_u16::<LE>()? as usize;
        let _hotspot_x = self.cursor.read_u16::<LE>()?;
        let _hotspot_y = self.cursor.read_u16::<LE>()?;
        let _keycolor_bytes = self.cursor.read_u16::<LE>()?;

        // Unknown field.
        self.cursor.read_u16::<LE>()?;

        // NOTE: optional palette header should be read here. Skipping that
        // here, since the original game files don't use this.

        // Unknown fields.
        self.cursor.read_u16::<LE>()?;
        self.cursor.read_u16::<LE>()?;

        let _compressed_size = self.cursor.read_u32::<LE>()? - 12;
        let _uncompressed_size = self.cursor.read_u32::<LE>()?;

        let data = TgaRleIterator::new(&mut self.cursor)
            .take(width * height)
            .flat_map(|v| {
                // For a Pixel Depth of 15 and 16 bit, each pixel is stored with 5 bits
                // per color. If the pixel depth is 16 bits, the topmost bit is reserved
                // for transparency.
                if v & 0b1000_0000_0000_0000 != 0 {
                    vec![0, 0, 0, 0]
                } else {
                    let r = (((v & 0b0111_1100_0000_0000) >> 10) << 3) as u8;
                    let g = (((v & 0b0000_0011_1110_0000) >> 5) << 3) as u8;
                    let b = ((v & 0b0000_0000_0001_1111) << 3) as u8;
                    vec![r, g, b, 255]
                }
            })
            .collect::<Vec<u8>>();

        // For some reason one byte is not consumed?
        self.cursor.read_u8()?;

        println!("{}x{} {}", width, height, data.len() as usize);

        Ok(Image {
            width,
            height,
            data,
        })
    }

    fn read_image_data(&mut self) -> Result<Texture> {
        let mut frames = vec![];
        while self.cursor.position() < self.file_end {
            let item = self.read_item()?;
            if item.signature == *b"FRAM" {
                frames.push(self.parse_frame(item)?);
            } else {
                item.skip(&mut self.cursor)?;
            }
        }

        let (width, height) = match frames.get(0) {
            Some(first) => (first.width, first.height),
            None => bail!("No frames in ANI file"),
        };

        for (i, frame) in frames.iter().enumerate() {
            println!("{} {}x{}", i, frame.width, frame.height);
        }

        if !frames
            .iter()
            .all(|i| i.width == width && i.height == height)
        {
            bail!("Frames must all have the same dimensions");
        }

        // Build sprite sheet in vertical direction. Vertical because that
        // allows us to just concat all the image data together.
        let num_frames = frames.len() as u32;
        let image_data = frames.drain(0..).flat_map(|i| i.data).collect();

        let texture = Texture::new(
            Extent3d::new(width as u32, height as u32 * num_frames, 1),
            TextureDimension::D2,
            image_data,
            TextureFormat::Rgba8UnormSrgb,
        );

        Ok(texture)
    }
}
