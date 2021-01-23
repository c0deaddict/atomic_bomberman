use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, Handle, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render::texture::{Extent3d, TextureDimension, TextureFormat},
    utils::BoxedFuture,
};
use byteorder::{ReadBytesExt, LE};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{self, Cursor, Read, Seek, SeekFrom};

/// Convert Bomberman specific ANI format to a sprite sheet.
///
/// Inspired by:
/// https://github.com/mmatyas/ab_aniex/blob/master/src/AniFile.cpp
/// https://github.com/image-rs/image/blob/master/src/codecs/bmp/decoder.rs
#[derive(Default)]
pub struct AnimationAssetLoader;

#[derive(Debug, Clone)]
pub struct Frame {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    pub offset_x: i32,
    pub offset_y: i32,
}

#[derive(Debug, Clone)]
pub struct AnimationAtlas {
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_count: usize,
    pub texture: Handle<TextureAtlas>,
}

#[derive(Debug, TypeUuid)]
#[uuid = "e1e9f49e-4fcd-464d-bb04-c8e60cf00422"]
pub struct Animation {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Frame>,
    pub atlas: AnimationAtlas,
}

#[derive(Debug, TypeUuid)]
#[uuid = "56c38dde-6ab4-4d02-93c1-976a7fa8dea2"]
pub struct AnimationBundle {
    pub animations: HashMap<String, Handle<Animation>>,
}

impl AssetLoader for AnimationAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut decoder = Decoder::new(bytes)?;
            let bundle = match decoder.load_animation_bundle(load_context) {
                Ok(bundle) => bundle,
                Err(err) => bail!("Error in {:?}: {:?}", load_context.path(), err),
            };
            load_context.set_default_asset(LoadedAsset::new(bundle));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ani", "ANI"]
    }
}

#[derive(Debug, Default)]
struct Item {
    signature: [u8; 4],
    id: u16,
    length: u32,
    start: u64,
}

fn print_bytes(bytes: &[u8]) {
    bytes.chunks(32).for_each(|chunk| {
        println!(
            "{}",
            chunk
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ")
        );
    });
}

type ByteCursor<'a> = Cursor<&'a [u8]>;

impl Item {
    fn signature_str(&self) -> String {
        String::from_utf8(self.signature.into()).unwrap()
    }

    fn end(&self) -> u64 {
        self.start + self.length as u64
    }

    fn dump(&self, cursor: &mut ByteCursor) -> Result<()> {
        let mut buf = vec![0; self.length as usize];
        cursor.read_exact(&mut buf)?;
        println!("{} {:?}", self.signature_str(), self);
        print_bytes(&buf);
        Ok(())
    }

    fn skip(&self, cursor: &mut ByteCursor) -> Result<()> {
        cursor.seek(SeekFrom::Current(self.length as i64))?;
        Ok(())
    }
}

struct FrameImage {
    width: u32,
    height: u32,
    /// 32bit RGBA encoded pixels.
    data: Vec<u8>,
}

#[derive(Default, Debug)]
struct Decoder<'a> {
    cursor: ByteCursor<'a>,
    has_loaded_metadata: bool,
    id: u16,
    file_end: u64,
}

trait TgaReadPixel<T> {
    fn read(cursor: &mut ByteCursor) -> io::Result<T>;
}

impl TgaReadPixel<u8> for u8 {
    fn read(cursor: &mut ByteCursor) -> io::Result<u8> {
        cursor.read_u8()
    }
}

impl TgaReadPixel<u16> for u16 {
    fn read(cursor: &mut ByteCursor) -> io::Result<u16> {
        cursor.read_u16::<LE>()
    }
}

struct TgaRleIterator<'a, 'b, T> {
    cursor: &'a mut ByteCursor<'b>,
    rle_data: Option<T>,
    rle_len: u8,
    raw_len: u8,
}

impl<'a, 'b, T> TgaRleIterator<'a, 'b, T> {
    fn new(cursor: &'a mut ByteCursor<'b>) -> Self {
        TgaRleIterator {
            cursor,
            rle_data: None,
            rle_len: 0,
            raw_len: 0,
        }
    }
}

/// TGA run length encoding iterator.
///
/// If hightest bit of the control byte is set, the next 16 bits are a pixel
/// value that is to be repeated `control - highest bit` times.
///
/// Otherwise `control` is interpreted as a count of consecutive 16 bits pixel
/// values that are read in order. This is called a "raw packet" in TGA.
///
/// For more info see: http://www.ludorg.net/amnesia/TGA_File_Format_Spec.html
impl<'a, 'b, T> Iterator for TgaRleIterator<'a, 'b, T>
where
    T: Copy + TgaReadPixel<T>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.rle_len > 0 {
            self.rle_len -= 1;
            return self.rle_data;
        } else if self.raw_len > 0 {
            self.raw_len -= 1;
            return T::read(&mut self.cursor).ok();
        }

        let control = match self.cursor.read_u8() {
            Ok(b) => b,
            Err(_) => return None,
        };

        // For both RLE and Raw packets we need to the next pixel value.
        let len = control & 0b01111111;
        let data = match T::read(&mut self.cursor) {
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

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Result<Self> {
        let mut decoder = Decoder {
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

    fn read_item(&mut self) -> Result<Item> {
        let mut item = Item::default();
        self.cursor.read_exact(&mut item.signature)?;
        item.length = self.cursor.read_u32::<LE>()?;
        item.id = self.cursor.read_u16::<LE>()?;
        item.start = self.cursor.position();
        Ok(item)
    }

    fn parse_item(&mut self, signature: &[u8]) -> Result<Item> {
        let item = self.read_item()?;
        if item.signature != *signature {
            bail!(
                "Expected {} item, found {}",
                String::from_utf8(signature.into()).unwrap(),
                item.signature_str()
            );
        }

        Ok(item)
    }

    /// Each frame consists of a HEAD, followed by a FNAM and a CIMG.
    fn parse_frame(&mut self) -> Result<FrameImage> {
        // Ignore HEAD.
        self.parse_item(b"HEAD")?.skip(&mut self.cursor)?;

        // Ignore FNAM, if present. It's missing in CLASSICS.ANI
        let mut item = self.read_item()?;
        if item.signature == *b"FNAM" {
            item.skip(&mut self.cursor)?;
            item = self.read_item()?;
        }
        if item.signature != *b"CIMG" {
            bail!("Expected CIMG item in frame");
        }

        if item.length < 32 {
            bail!("CIMG is too small: {} < 32", item.length);
        }

        let bits_per_pixel = match self.cursor.read_u16::<LE>()? {
            0x0004 => 16,
            0x000b => 8,
            other => bail!("CIMG type {:#06x} is not supported", other),
        };

        // Unknown field.
        let _unknown1 = self.cursor.read_u16::<LE>()?;

        let mut palette_size = None;
        let additional_size = self.cursor.read_u32::<LE>()? as usize;
        if additional_size >= 32 {
            if additional_size > 32 {
                palette_size = Some(additional_size - 32);
            } else {
                bail!("CIMG palette is missing!");
            }
        };

        // Unknown field.
        let _unknown2 = self.cursor.read_u32::<LE>()?;

        let width = self.cursor.read_u16::<LE>()? as u32;
        let height = self.cursor.read_u16::<LE>()? as u32;
        let hotspot_x = self.cursor.read_u16::<LE>()? as u32;
        let hotspot_y = self.cursor.read_u16::<LE>()? as u32;
        let transparent = self.cursor.read_u16::<LE>()?;

        // println!("hotspot = {}x{} transparent {:#06x}", hotspot_x, hotspot_y, transparent);

        // Unknown field.
        let _unknown3 = self.cursor.read_u16::<LE>()?;

        let mut palette: Option<Vec<Vec<u8>>> = None;
        if let Some(size) = palette_size {
            let _unknown4 = self.cursor.read_u32::<LE>()?;
            let _unknown5 = self.cursor.read_u32::<LE>()?;

            let mut buf = vec![0; size];
            self.cursor.read_exact(&mut buf)?;
            palette = Some(buf.chunks(4).map(|c| c.to_vec()).collect());
        }

        // Unknown fields.
        let _unknown6 = self.cursor.read_u16::<LE>()?;
        let _unknown7 = self.cursor.read_u16::<LE>()?;

        // println!("{:#06x} {:#06x}", _unknown6, _unknown7);

        let _compressed_size = self.cursor.read_u32::<LE>()? - 12;
        let _uncompressed_size = self.cursor.read_u32::<LE>()?;

        let data: Vec<u8> = if bits_per_pixel == 16 {
            if palette_size.is_some() {
                // println!("{:?}", palette.unwrap());
                // println!(
                //     "CIMG 16bpp expected no palette, size found {}",
                //     palette_size.unwrap()
                // );
            }

            // Convert the 16 bit color values to 32bit RGBA. The 16 bit pixel
            // format has support for an alpha channel (the highest bit) but
            // that is not used. Instead alpha is set to all pixels that match
            // `transparent`.
            TgaRleIterator::new(&mut self.cursor)
                .take((width * height) as usize)
                .flat_map(|v: u16| {
                    if v == transparent || v & 0b1000_0000_0000_0000 != 0 {
                        vec![0, 0, 0, 0]
                    } else {
                        let r = (((v & 0b0111_1100_0000_0000) >> 10) << 3) as u8;
                        let g = (((v & 0b0000_0011_1110_0000) >> 5) << 3) as u8;
                        let b = ((v & 0b0000_0000_0001_1111) << 3) as u8;
                        vec![r, g, b, 255]
                    }
                })
                .collect()
        } else {
            if palette_size.is_none() || palette_size.unwrap() != 1024 {
                bail!("CIMG 8bpp expected a palette size of 1024");
            }

            // Decode the TGA RLE encoding into 8bpp raw data.
            let raw_data = TgaRleIterator::new(&mut self.cursor)
                .take((width * height) as usize)
                .collect::<Vec<u8>>();

            let hotspot_idx = hotspot_x + width * hotspot_y;
            let hotspot = raw_data[hotspot_idx as usize];

            let palette = palette.unwrap();

            // Convert the 8 bit color values via the palette to 32 bits RGBA.
            raw_data
                .iter()
                .flat_map(|v| {
                    if *v == hotspot {
                        vec![0, 0, 0, 0]
                    } else {
                        palette[*v as usize].clone()
                    }
                })
                .collect()
        };

        // For some reason one byte is not consumed?
        self.cursor.read_u8()?;

        Ok(FrameImage {
            width,
            height,
            data,
        })
    }

    /// Sequences are composed of a HEAD followed by one or more STAT items.
    /// The HEAD contains the "name". Each STAT item contains a HEAD (which we
    /// ignore) and a FRAM. The FRAM contains a "frame index".
    fn parse_animation(
        &mut self,
        seq: Item,
        frame_images: &[FrameImage],
        atlas: AnimationAtlas,
    ) -> Result<Animation> {
        let mut item = self.read_item()?;
        if item.signature != *b"HEAD" {
            bail!("Expected a HEAD item inside SEQ");
        }

        // We're only interested in the nul-terminated string in HEAD.
        let mut buf = vec![0; item.length as usize];
        self.cursor.read_exact(&mut buf)?;
        let str_bytes = buf.iter().cloned().take_while(|b| *b != 0).collect();
        let name = String::from_utf8(str_bytes).unwrap();

        let mut frames = vec![];

        while self.cursor.position() < seq.end() {
            item = self.read_item()?;
            if item.signature != *b"STAT" {
                bail!(
                    "Expected STAT items after HEAD in SEQ, got {:?}",
                    item.signature
                );
            }

            self.parse_item(b"HEAD")?.skip(&mut self.cursor)?;
            let fram_item = self.parse_item(b"FRAM")?;

            let _id = self.cursor.read_u16::<LE>()?;
            let index = self.cursor.read_u16::<LE>()? as usize;
            // XXX: for one ANI file an index is too large, this fixes it.
            let index = index.min(frame_images.len() - 1);

            let offset_x = self.cursor.read_i16::<LE>()? as i32;
            let offset_y = self.cursor.read_i16::<LE>()? as i32;

            // Padding? Is always 0x00000000
            let _unknown3 = self.cursor.read_u32::<LE>()?;

            // Sometimes the FRAM is longer, usually it's 12 bytes, but there
            // are cases of 22 bytes.
            self.cursor.seek(SeekFrom::Start(fram_item.end()))?;

            let image = &frame_images[index];

            frames.push(Frame {
                index,
                offset_x,
                offset_y,
                width: image.width,
                height: image.height,
            });
        }

        let width = frames.iter().map(|f| f.width).max().unwrap();
        let height = frames.iter().map(|f| f.height).max().unwrap();

        println!("loaded animation {} ({}x{})", name, width, height);

        Ok(Animation {
            name,
            frames,
            width,
            height,
            atlas,
        })
    }

    fn load_animation_bundle(&mut self, load_context: &mut LoadContext) -> Result<AnimationBundle> {
        for sig in &[b"HEAD", b"PAL ", b"TPAL", b"CBOX"] {
            let item = self.parse_item(*sig)?;
            if item.signature == *b"PAL " {
                // item.dump(&mut self.cursor)?;
                item.skip(&mut self.cursor)?;
            } else {
                item.skip(&mut self.cursor)?;
            }
        }

        let mut frames = vec![];
        let mut item = self.read_item()?;
        while item.signature == *b"FRAM" {
            frames.push(self.parse_frame()?);
            item = self.read_item()?;
        }

        let tile_width = frames.iter().map(|i| i.width).max().unwrap();
        let tile_height = frames.iter().map(|i| i.height).max().unwrap();
        let tile_count = frames.len();

        let atlas = AnimationAtlas {
            tile_width,
            tile_height,
            tile_count,
            texture: Default::default(),
        };

        let mut animations = vec![];
        while item.signature == *b"SEQ " {
            animations.push(self.parse_animation(item, &frames, atlas.clone())?);
            if self.cursor.position() >= self.file_end {
                break;
            }
            item = self.read_item()?;
        }

        // Build sprite sheet in vertical direction. Vertical because that
        // allows us to just concat all the image data together.
        //
        // Each frame is right-paded with transparent pixels to match it's width
        // to the widest frame. And bottom-padded with transparent rows to match
        // the height of the highest frame. We need a constant tile size for the
        // texture atlas to work.
        let image_data = frames
            .drain(0..)
            .flat_map(|mut frame| {
                if frame.width < tile_width {
                    for y in 0..frame.height {
                        let idx = frame.width + (y * tile_width);
                        let idx = 4 * idx as usize;
                        let extra = vec![0; 4 * (tile_width - frame.width) as usize];
                        frame.data.splice(idx..idx, extra);
                    }
                }
                if frame.height < tile_height {
                    let rows = tile_height - frame.height;
                    frame
                        .data
                        .extend_from_slice(&vec![0; (4 * tile_width * rows) as usize]);
                }

                frame.data
            })
            .collect();

        let texture = Texture::new(
            Extent3d::new(tile_width, tile_height * tile_count as u32, 1),
            TextureDimension::D2,
            image_data,
            TextureFormat::Rgba8UnormSrgb,
        );

        let texture = load_context.set_labeled_asset("texture", LoadedAsset::new(texture));
        let texture_atlas = TextureAtlas::from_grid(
            texture,
            Vec2::new(tile_width as f32, tile_height as f32),
            1,
            tile_count,
        );

        let texture_atlas_handle =
            load_context.set_labeled_asset("texture_atlas", LoadedAsset::new(texture_atlas));

        // Bind texture atlas in all animations and wrap them in an asset.
        let animations = animations
            .drain(..)
            .map(|mut animation| {
                animation.atlas.texture = texture_atlas_handle.clone();
                let name = animation.name.clone();
                let handle = load_context.set_labeled_asset(&name, LoadedAsset::new(animation));
                (name, handle)
            })
            .collect();

        Ok(AnimationBundle { animations })
    }
}
