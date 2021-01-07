use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, AssetPath, Handle, LoadContext, LoadedAsset},
    reflect::TypeUuid,
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
pub struct AnimationAssetLoader;

#[derive(Debug)]
pub struct Animation {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<usize>,
}

#[derive(Debug, TypeUuid)]
#[uuid = "56c38dde-6ab4-4d02-93c1-976a7fa8dea2"]
pub struct AnimationBundle {
    pub texture: Handle<Texture>,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_count: usize,
    pub animations: Vec<Animation>,
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

impl Item {
    fn signature_str(&self) -> String {
        String::from_utf8(self.signature.into()).unwrap()
    }

    fn end(&self) -> u64 {
        self.start + self.length as u64
    }

    fn dump(&self, cursor: &mut Cursor<&[u8]>) -> Result<()> {
        let mut buf = vec![0; self.length as usize];
        cursor.read_exact(&mut buf)?;
        println!("{} {:?}", self.signature_str(), self);
        print_bytes(&buf);
        Ok(())
    }

    fn skip(&self, cursor: &mut Cursor<&[u8]>) -> Result<()> {
        cursor.seek(SeekFrom::Current(self.length as i64))?;
        Ok(())
    }
}

struct Frame {
    width: u32,
    height: u32,
    /// 32bit RGBA encoded pixels.
    data: Vec<u8>,
}

struct Seq {
    name: String,
    indices: Vec<usize>,
}

#[derive(Default, Debug)]
struct Decoder<'a> {
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

/// TGA run length encoding iterator
///
/// If hightest bit of the control byte is set, the next 16 bits are a pixel
/// value that is to be repeated `control - highest bit` times.
///
/// Otherwise `control` is interpreted as a count of consecutive 16 bits pixel
/// values that are read in order. This is called a "raw packet" in TGA.
///
/// For more info see: http://www.ludorg.net/amnesia/TGA_File_Format_Spec.html
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
    fn parse_frame(&mut self, item: Item) -> Result<Frame> {
        // Ignore HEAD and FNAM.
        self.parse_item(b"HEAD")?.skip(&mut self.cursor)?;
        self.parse_item(b"FNAM")?.skip(&mut self.cursor)?;

        let item = self.parse_item(b"CIMG")?;

        if item.length < 32 {
            bail!("CIMG is too small: {} < 32", item.length);
        }

        if self.cursor.read_u16::<LE>()? != 0x0004 {
            bail!("CIMG type must be 0x0004 (16 bits per pixel)");
        }

        // Unknown field.
        let unknown1 = self.cursor.read_u16::<LE>()?;

        let additional_size = self.cursor.read_u32::<LE>()?;
        if additional_size >= 32 {
            bail!("CIMG palette header not supported");
        }

        // Unknown field.
        let unknown2 = self.cursor.read_u32::<LE>()?;

        let width = self.cursor.read_u16::<LE>()? as u32;
        let height = self.cursor.read_u16::<LE>()? as u32;
        let hotspot_x = self.cursor.read_u16::<LE>()? as u32;
        let hotspot_y = self.cursor.read_u16::<LE>()? as u32;
        let keycolor_bytes = self.cursor.read_u16::<LE>()?;

        // Unknown field.
        let unknown3 = self.cursor.read_u16::<LE>()?;

        // NOTE: optional palette header should be read here. Skipping that
        // here, since the original game files don't use this.

        // Unknown fields.
        let unknown4 = self.cursor.read_u16::<LE>()?;
        let unknown5 = self.cursor.read_u16::<LE>()?;

        let _compressed_size = self.cursor.read_u32::<LE>()? - 12;
        let _uncompressed_size = self.cursor.read_u32::<LE>()?;

        // println!("{} {} {}", hotspot_x, hotspot_y, keycolor_bytes);
        // println!("{} {} {} {} {}", unknown1, unknown2, unknown3, unknown4, unknown5);

        // Decode the TGA RLE encoding into raw data.
        let raw_data = TgaRleIterator::new(&mut self.cursor)
            .take((width * height) as usize)
            .collect::<Vec<u16>>();

        let hotspot_idx = hotspot_x + width * hotspot_y;
        let hotspot = raw_data[hotspot_idx as usize];
        // let hotspot = 0x0000;

        // Convert the 16 bit color values to 32bit RGBA. Pixels that match the
        // hotspot are made transparent. The 16 bit pixel format has support for
        // an alpha channel (the highest bit, but that is not used).
        let data = raw_data
            .iter()
            .flat_map(|v| {
                if *v == hotspot || v & 0b1000_0000_0000_0000 != 0 {
                    // if v == 0x4210 || v == 0x7f7f || v == 0x7f5f {
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

        Ok(Frame {
            width,
            height,
            data,
        })
    }

    /// Sequences are composed of a HEAD followed by one or more STAT items.
    /// The HEAD contains the "name". Each STAT item contains a HEAD (which we
    /// ignore) and a FRAM. The FRAM contains a "frame index".
    fn parse_animation(&mut self, seq: Item, frames: &[Frame]) -> Result<Animation> {
        let mut item = self.read_item()?;
        if item.signature != *b"HEAD" {
            bail!("Expected a HEAD item inside SEQ");
        }

        // We're only interested in the nul-terminated string in HEAD.
        let mut buf = vec![0; item.length as usize];
        self.cursor.read_exact(&mut buf)?;
        let str_bytes = buf.iter().cloned().take_while(|b| *b != 0).collect();
        let name = String::from_utf8(str_bytes).unwrap();

        let mut frame_indices = vec![];
        let mut first_frame: Option<&Frame> = None;

        while self.cursor.position() < seq.end() {
            item = self.read_item()?;
            if item.signature != *b"STAT" {
                bail!("Expected STAT items after HEAD in SEQ, got {:?}", item.signature);
            }

            self.parse_item(b"HEAD")?.skip(&mut self.cursor)?;
            self.parse_item(b"FRAM")?;

            let _id = self.cursor.read_u16::<LE>()?;
            let index = self.cursor.read_u16::<LE>()? as usize;

            // Maybe this is direction?
            // 0x0000 = forward
            // 0x0001 = backward
            // 0xffff = ???
            let _unknown1 = self.cursor.read_u16::<LE>()?;

            // No idea what this means?
            // Values seen: 0x0006, 0x0000, 0xffff, 0x0001
            let _unknown2 = self.cursor.read_u16::<LE>()?;

            // Padding? Is always 0x00000000
            let _unknown3 = self.cursor.read_u32::<LE>()?;

            let frame = &frames[index];
            if let Some(first) = first_frame {
                if frame.width != first.width || frame.height != first.height {
                    println!(
                        "Frames of different dimensions in SEQ: {}x{} and {}x{}",
                        frame.width, frame.height, first.width, first.height
                    );
                }
            } else {
                first_frame = Some(frame);
            }

            frame_indices.push(index);
        }

        Ok(Animation {
            name,
            width: first_frame.unwrap().width,
            height: first_frame.unwrap().height,
            frames: frame_indices,
        })
    }

    fn load_animation_bundle(&mut self, load_context: &mut LoadContext) -> Result<AnimationBundle> {
        for sig in &[b"HEAD", b"PAL ", b"TPAL", b"CBOX"] {
            self.parse_item(*sig)?.skip(&mut self.cursor)?;
        }

        let mut frames = vec![];
        let mut item = self.read_item()?;
        while item.signature == *b"FRAM" {
            frames.push(self.parse_frame(item)?);
            item = self.read_item()?;
        }

        let mut animations = vec![];
        while item.signature == *b"SEQ " {
            animations.push(self.parse_animation(item, &frames)?);
            if self.cursor.position() >= self.file_end {
                break;
            }
            item = self.read_item()?;
        }

        let tile_width = frames.iter().map(|i| i.width).max().unwrap();
        let tile_height = frames.iter().map(|i| i.height).max().unwrap();
        let tile_count = frames.len();

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
                    frame.data.extend_from_slice(&vec![
                        0;
                        (4 * tile_width * (tile_height - frame.height))
                            as usize
                    ]);
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

        let texture_label = "texture";
        load_context.set_labeled_asset(texture_label, LoadedAsset::new(texture));
        let texture_path = AssetPath::new_ref(load_context.path(), Some(texture_label));

        Ok(AnimationBundle {
            animations,
            tile_width,
            tile_height,
            tile_count,
            texture: load_context.get_handle(texture_path),
        })
    }
}
