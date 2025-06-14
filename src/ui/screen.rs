use sdl2::image::{LoadSurface, LoadTexture};
pub use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
pub use sdl2::rect::{Point, Rect};
use sdl2::render::TextureQuery;
use sdl2::surface::Surface;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

#[derive(Copy, Clone)]
pub enum FontSize {
    Body,
    Title,
}

pub const SHADOW_DELTA: u32 = 2;

pub struct Screen<'a, 'b> {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    width: u32,
    height: u32,
    body_font: sdl2::ttf::Font<'a, 'b>,
    title_font: sdl2::ttf::Font<'a, 'b>,
}

impl<'a, 'b> Screen<'a, 'b> {
    pub(super) fn new(
        canvas: sdl2::render::Canvas<sdl2::video::Window>,
        width: u32,
        height: u32,
        ttf_context: &'a sdl2::ttf::Sdl2TtfContext,
        font_path: &Path,
    ) -> Self {
        let body_font = ttf_context.load_font(font_path, 14).unwrap();
        let title_font = ttf_context.load_font(font_path, 18).unwrap();

        Self {
            canvas,
            width,
            height,
            body_font,
            title_font,
        }
    }

    pub(super) fn clear(&mut self, color: Color) {
        let canvas = &mut self.canvas;
        canvas.set_draw_color(color);
        canvas.clear();
    }

    pub(super) fn present(&mut self) {
        let canvas = &mut self.canvas;
        canvas.present();
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn draw_image(&mut self, path: &Path, source: Option<Rect>, target: Option<Rect>) {
        let canvas = &mut self.canvas;
        let texture_creator = canvas.texture_creator();
        let image = texture_creator.load_texture(path).unwrap();
        canvas.copy(&image, source, target).unwrap();
    }

    pub fn draw_background(&mut self, image: sdl2::surface::Surface, scale: f32, angle: f64) {
        let canvas = &mut self.canvas;
        let screen_width = self.width;
        let screen_height = self.height;
        let (width, height) = image.size();
        let texture_creator = canvas.texture_creator();
        let image = image.as_texture(&texture_creator).unwrap();

        let h = (screen_width as f32 * height as f32 / width as f32) as u32;
        let dh = screen_height as i32 - h as i32;

        canvas
            .copy_ex(
                &image,
                None,
                Some(Rect::new(
                    ((screen_width as f32) * (scale - 1.0) * -0.5) as i32,
                    (dh as f32 * scale / 2.0) as i32
                        + ((screen_height as f32) * (scale - 1.0) * -0.5) as i32,
                    ((screen_width as f32) * scale) as u32,
                    ((h as f32) * scale) as u32,
                )),
                angle,
                Point::new(
                    (screen_width as f32 * scale / 2.0) as i32,
                    (h as f32 * scale / 2.0) as i32,
                ),
                false,
                false,
            )
            .unwrap();
    }

    pub fn draw_rect(&mut self, color: Color, rect: Rect) {
        let canvas = &mut self.canvas;
        canvas.set_draw_color(color);
        canvas.fill_rect(rect).unwrap()
    }

    pub fn measure_text(&mut self, size: FontSize, text: &str) -> (u32, u32) {
        let font = match size {
            FontSize::Body => &self.body_font,
            FontSize::Title => &self.title_font,
        };

        let canvas = &mut self.canvas;
        let texture_creator = canvas.texture_creator();
        let surface = font.render(text).blended(Color::RGB(0, 0, 0)).unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let TextureQuery { width, height, .. } = texture.query();
        (width, height)
    }

    fn draw_text_source(
        &mut self,
        size: FontSize,
        text: &str,
        x: i32,
        y: i32,
        max_width: Option<u32>,
    ) {
        let font = match size {
            FontSize::Body => &self.body_font,
            FontSize::Title => &self.title_font,
        };

        let canvas = &mut self.canvas;
        let texture_creator = canvas.texture_creator();
        for (color, delta) in [
            (Color::RGBA(0, 0, 0, 255), SHADOW_DELTA),
            (Color::RGBA(255, 255, 255, 255), 0),
        ] {
            let surface = font.render(text).blended(color).unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .unwrap();
            let TextureQuery { width, height, .. } = texture.query();
            let width = match max_width {
                Some(w) if w < width => w,
                _ => width,
            };
            let target = Rect::new(delta as i32 + x, delta as i32 + y, width, height);
            canvas
                .copy(
                    &texture,
                    max_width.map(|w| Rect::new(0, 0, w, height)),
                    Some(target),
                )
                .unwrap();
        }
    }

    pub fn draw_text(&mut self, size: FontSize, text: &str, x: i32, y: i32) {
        self.draw_text_source(size, text, x, y, None);
    }

    pub fn draw_text_clipped(
        &mut self,
        size: FontSize,
        text: &str,
        x: i32,
        y: i32,
        max_width: u32,
    ) {
        self.draw_text_source(size, text, x, y, Some(max_width))
    }

    pub fn create_background(path: &Path, desaturate: u8, darken: u8) -> sdl2::surface::Surface {
        let mut surface = Surface::from_file(path).unwrap();
        let (width, height) = surface.size();
        let pitch = surface.pitch();
        if surface.pixel_format_enum() != PixelFormatEnum::RGB24 {
            panic!("Unexpected pixel format {:?}", surface.pixel_format_enum());
        }

        surface.with_lock_mut(|bytes: &mut [u8]| {
            // desaturate
            let amount = desaturate as f32 / 255.0;
            for y in 0..height {
                let yoffset = (y * pitch) as usize;
                for x in 0..width {
                    let offset = yoffset + (x * 3) as usize;
                    let r = bytes[offset] as f32 / 255.0;
                    let g = bytes[offset + 1] as f32 / 255.0;
                    let b = bytes[offset + 2] as f32 / 255.0;
                    let z = 0.2126 * r + 0.7152 * g + 0.0722 * b;

                    bytes[offset] = ((z * amount + r * (1.0 - amount)) * 255.0) as u8;
                    bytes[offset + 1] = ((z * amount + g * (1.0 - amount)) * 255.0) as u8;
                    bytes[offset + 2] = ((z * amount + b * (1.0 - amount)) * 255.0) as u8;
                }
            }

            // darken
            let amount = 1.0 - (darken as f32 / 255.0);
            for b in bytes {
                *b = (*b as f32 * amount) as u8;
            }
        });

        surface
    }

    pub fn recommended_margin(&self) -> u32 {
        ((self.width as f32) / 40.0) as u32
    }

    pub fn take_screenshot(&self) {
        let filename = format!(
            "{}.bmp",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let format = sdl2::pixels::PixelFormatEnum::ARGB8888;
        let (width, height) = self.size();
        let pitch = format.byte_size_of_pixels(width as usize) as u32;

        match self.canvas.read_pixels(None, format).and_then(|mut bytes| {
            let surface = sdl2::surface::Surface::from_data(
                bytes.as_mut_slice(),
                width,
                height,
                pitch,
                format,
            )
            .unwrap();
            surface
                .save_bmp(&filename)
                .map_err(|e| format!("Could not write screenshot: {e:?}"))
        }) {
            Ok(_) => {
                info!("Saved screenshot {filename:?}");
            }
            Err(e) => {
                error!("Could not save screenshot: {e:?}");
            }
        };
    }
}
