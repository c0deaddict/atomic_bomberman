use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    render::texture::{Extent3d, Texture, TextureDimension, TextureFormat},
    utils::BoxedFuture,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::fmt::Debug;
use std::io::{Write, Cursor, Read, Seek, SeekFrom};
use std::fs::File;

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

#[derive(Default)]
struct CImg {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

#[derive(Default, Debug)]
struct AniDecoder<'a> {
    cursor: Cursor<&'a [u8]>,
    has_loaded_metadata: bool,
    id: u16,
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
        let file_id = self.cursor.read_u16::<LE>()?;
        let file_end = self.cursor.position() + file_length as u64;

        println!("{} {} {}", file_length, file_id, file_end);
        while self.cursor.position() < file_end {
            let item = self.read_item()?;
            println!(
                "item {} {:?}",
                String::from_utf8(item.signature.into())?,
                item
            );
            if item.signature == *b"FRAM" {
                self.parse_frame(item)?;
            } else {
                self.cursor.seek(SeekFrom::Current(item.length as i64))?;
            }
        }

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

    fn parse_frame(&mut self, frame: AniItem) -> Result<()> {
        let mut name = None;
        let mut cimg = None;

        while self.cursor.position() < frame.start + frame.length as u64 {
            let item = self.read_item()?;
            match &item.signature {
                b"FNAM" => {
                    let mut buf = vec![0; item.length as usize];
                    self.cursor.read_exact(&mut buf)?;
                    buf.pop(); // remove \0
                    name = Some(String::from_utf8(buf)?);
                }
                b"CIMG" => cimg = Some(self.parse_cimg(&name.as_ref().unwrap(), item)?),
                _ => item.skip(&mut self.cursor)?,
            }
        }

        if name.is_some() && cimg.is_some() {
            println!("read frame {} {}x{}", name.unwrap(), cimg.as_ref().unwrap().width, cimg.as_ref().unwrap().height);
        }

        Ok(())
    }

    fn parse_cimg(&mut self, name: &str, item: AniItem) -> Result<CImg> {
        if item.length < 32 {
            bail!("CIMG is too small: {} < 32", item.length);
        }

        let mut img = CImg::default();

        let bits_per_pixel = match self.cursor.read_u16::<LE>()? {
            0x0004 => 10,
            0x0005 => 24,
            0x000a => 4,
            other => bail!("Unknown CIMG type: {}", other),
        };

        if bits_per_pixel != 10 {
            bail!("CIMG bits per pixel {} is not supported", bits_per_pixel);
        }

        // Unknown field.
        self.cursor.read_u16::<LE>()?;

        let additional_size = self.cursor.read_u32::<LE>()?;
        if additional_size < 24 {
            bail!("CIMG special header is too small: {} < 24", additional_size);
        }
        if self.cursor.position() + additional_size as u64 > item.start + item.length as u64 {
            bail!("CIMG special header size mismatch");
        }
        if additional_size >= 32 {
            bail!("CIMG palette header not supported");
        }

        // Unknown field.
        self.cursor.read_u32::<LE>()?;

        img.width = self.cursor.read_u16::<LE>()? as usize;
        img.height = self.cursor.read_u16::<LE>()? as usize;
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

        let compressed_size = self.cursor.read_u32::<LE>()? - 12;
        let uncompressed_size = self.cursor.read_u32::<LE>()?;

        // data is RLE encoded TGA data.
        // bitsperpixel = 16 ??

        let mut data = vec![0; compressed_size as usize];
        self.cursor.read_exact(&mut data)?;

        let mut header = vec![
            0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ];

        let mut file = File::create(format!("out/{}", name))?;
        file.write_all(&header)?;
        file.write_u16::<LE>(img.width as u16)?;
        file.write_u16::<LE>(img.height as u16)?;
        file.write_u8(16)?; // 16 bits in a stored pixel index.
        file.write_u8(0x20)?; // screen origin: upper left hand.
        file.write_all(&data)?;

        Ok(img)
    }

    fn read_image_data(&mut self) -> Result<Texture> {
        // let data_size = self.width as usize * self.height as usize;
        // let pixel_indices: Vec<u8> = if self.rle {
        //     RleIterator::new(self.cursor.clone())
        //         .take(data_size)
        //         .collect()
        // } else {
        //     let mut data = vec![0; data_size];
        //     for value in data.iter_mut() {
        //         *value = self.cursor.read_u8()?;
        //     }
        //     data
        // };

        // self.read_palette()?;

        // let palette = self.palette.as_ref().unwrap();
        // let image_data: Vec<u8> = pixel_indices
        //     .iter()
        //     .flat_map(|i| {
        //         let (r, g, b) = palette.get(*i as usize).unwrap();
        //         vec![*r, *g, *b, 255]
        //     })
        //     .collect();

        let image_data = vec![0; 4];

        let texture = Texture::new(
            Extent3d::new(1, 1, 1),
            TextureDimension::D2,
            image_data,
            TextureFormat::Rgba8UnormSrgb,
        );

        Ok(texture)
    }
}
