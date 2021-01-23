use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    render::texture::{Extent3d, Texture, TextureDimension, TextureFormat},
    utils::BoxedFuture,
};
use byteorder::{ReadBytesExt, LE};
use std::fmt::Debug;
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Inspired by:
/// https://github.com/image-rs/image/blob/master/src/codecs/bmp/decoder.rs
///
/// Format description:
/// https://en.wikipedia.org/wiki/PCX
/// http://fastgraph.com/help/pcx_header_format.html
#[derive(Default)]
pub struct PcxImageAssetLoader;

impl AssetLoader for PcxImageAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut decoder = PcxDecoder::new(bytes)?;
            let texture = decoder.read_image_data()?;
            load_context.set_default_asset(LoadedAsset::new(texture));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pcx", "PCX"]
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum PcxVersion {
    V0,
    V2,
    V3,
    V4,
    V5,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum PcxBitsPerPixelPlane {
    Bits1,
    Bits2,
    Bits4,
    Bits8,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum PcxColorPlanes {
    Single,
    RGB,
    RGBA,
}

#[derive(Default, Debug)]
struct PcxDecoder<'a> {
    cursor: Cursor<&'a [u8]>,
    has_loaded_metadata: bool,
    version: Option<PcxVersion>,
    rle: bool,
    bits_per_pixel_plane: Option<PcxBitsPerPixelPlane>,
    min_x: u16,
    min_y: u16,
    max_x: u16,
    max_y: u16,
    width: u16,
    height: u16,
    horz_dpi: u16,
    vert_dpi: u16,
    palette: Option<Vec<(u8, u8, u8)>>,
    color_planes: Option<PcxColorPlanes>,
    color_plane_bytes: u16,
    grayscale_palette: bool,
    source_horz_resolution: u16,
    source_vert_resolution: u16,
}

struct RleIterator<'a> {
    cursor: Cursor<&'a [u8]>,
    rle_data: Option<u8>,
    rle_len: u8,
}

impl<'a> RleIterator<'a> {
    fn new(cursor: Cursor<&'a [u8]>) -> Self {
        RleIterator {
            cursor,
            rle_data: None,
            rle_len: 0,
        }
    }
}

impl<'a> Iterator for RleIterator<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.rle_len > 0 {
            self.rle_len -= 1;
            return Some(self.rle_data.unwrap());
        }

        let control = match self.cursor.read_u8() {
            Ok(b) => b,
            Err(_) => return None,
        };

        // RLE sequence if highest two bits are set.
        if control & 0b11000000 == 0b11000000 {
            self.rle_len = (control & 0b00111111) - 1;
            let data = match self.cursor.read_u8() {
                Ok(b) => b,
                Err(_) => return None,
            };
            self.rle_data = Some(data);
            Some(data)
        } else {
            Some(control)
        }
    }
}

impl<'a> PcxDecoder<'a> {
    fn new(bytes: &'a [u8]) -> Result<Self> {
        let mut decoder = PcxDecoder {
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
        let signature = self.cursor.read_u8()?;
        if signature != 0x0A {
            bail!("PCX signature is invalid: {:#02x}", signature);
        }

        self.version = Some(match self.cursor.read_u8()? {
            0 => PcxVersion::V0,
            2 => PcxVersion::V2,
            3 => PcxVersion::V3,
            4 => PcxVersion::V4,
            5 => PcxVersion::V5,
            version => bail!("Unsupported PCX version {}", version),
        });

        self.rle = match self.cursor.read_u8()? {
            0 => false,
            1 => true,
            other => bail!("Unknown encoding method {}", other),
        };

        self.bits_per_pixel_plane = Some(match self.cursor.read_u8()? {
            1 => PcxBitsPerPixelPlane::Bits1,
            2 => PcxBitsPerPixelPlane::Bits2,
            4 => PcxBitsPerPixelPlane::Bits4,
            8 => PcxBitsPerPixelPlane::Bits8,
            other => bail!("Invalid bits per pixel plane: {}", other),
        });

        self.min_x = self.cursor.read_u16::<LE>()?;
        self.min_y = self.cursor.read_u16::<LE>()?;
        self.max_x = self.cursor.read_u16::<LE>()?;
        self.max_y = self.cursor.read_u16::<LE>()?;
        self.width = self.max_x - self.min_x + 1;
        self.height = self.max_y - self.min_y + 1;
        self.horz_dpi = self.cursor.read_u16::<LE>()?;
        self.vert_dpi = self.cursor.read_u16::<LE>()?;

        // 48 bytes of EGA pallette (only used if bits_per_pixel_plane = 4).
        let mut ega_palette = [0; 48];
        self.cursor.read_exact(&mut ega_palette)?;
        if self.bits_per_pixel_plane.unwrap() == PcxBitsPerPixelPlane::Bits4 {
            self.palette = Some(ega_palette.chunks(3).map(|c| (c[0], c[1], c[2])).collect());
        }

        // First reserved field.
        self.cursor.read_u8()?;

        self.color_planes = Some(match self.cursor.read_u8()? {
            1 => PcxColorPlanes::Single,
            3 => PcxColorPlanes::RGB,
            4 => PcxColorPlanes::RGBA,
            other => bail!("Unsupported number of color planes: {}", other),
        });

        self.color_plane_bytes = self.cursor.read_u16::<LE>()?;

        self.grayscale_palette = match self.cursor.read_u8()? {
            0 => false,
            1 => true,
            other => bail!("Unknown value for palette type: {}", other),
        };

        self.source_horz_resolution = self.cursor.read_u16::<LE>()?;
        self.source_vert_resolution = self.cursor.read_u16::<LE>()?;

        // 54 bytes of reserved field(s).
        self.cursor.seek(SeekFrom::Current(54))?;

        Ok(())
    }

    fn read_palette(&mut self) -> Result<()> {
        if self.palette.is_some() {
            return Ok(());
        }

        let bpp = self.bits_per_pixel_plane.unwrap();
        if bpp != PcxBitsPerPixelPlane::Bits8 {
            bail!("Don't know how to read palette for {:?}", bpp);
        }

        // 256 color palette is in the last 768 bytes of the file.
        // It is preceeded with a marker 0x0C.
        self.cursor.seek(SeekFrom::End(-769))?;

        if self.cursor.read_u8()? != 0x0C {
            bail!("256-color palette marker not found");
        }

        let mut palette_data = vec![(0, 0, 0); 256];
        for value in palette_data.iter_mut() {
            *value = (
                self.cursor.read_u8()?,
                self.cursor.read_u8()?,
                self.cursor.read_u8()?,
            );
        }

        self.palette = Some(palette_data);

        Ok(())
    }

    fn read_image_data(&mut self) -> Result<Texture> {
        if self.bits_per_pixel_plane.unwrap() != PcxBitsPerPixelPlane::Bits8 {
            bail!("Only 8 bits per pixel plane are supported");
        }

        if self.color_planes.unwrap() != PcxColorPlanes::Single {
            bail!("Only a single color plane is supported");
        }

        let data_size = self.width as usize * self.height as usize;
        let pixel_indices: Vec<u8> = if self.rle {
            RleIterator::new(self.cursor.clone())
                .take(data_size)
                .collect()
        } else {
            let mut data = vec![0; data_size];
            for value in data.iter_mut() {
                *value = self.cursor.read_u8()?;
            }
            data
        };

        self.read_palette()?;

        let palette = self.palette.as_ref().unwrap();
        let image_data: Vec<u8> = pixel_indices
            .iter()
            .flat_map(|i| {
                let (r, g, b) = palette.get(*i as usize).unwrap();
                vec![*r, *g, *b, 255]
            })
            .collect();

        let texture = Texture::new(
            Extent3d::new(self.width as u32, self.height as u32, 1),
            TextureDimension::D2,
            image_data,
            TextureFormat::Rgba8UnormSrgb,
        );

        Ok(texture)
    }
}
