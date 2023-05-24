use image::{GenericImage, Rgba, Pixel};
use imageproc::drawing::Canvas;

pub struct MyBlend<I>(pub I);

impl<I: GenericImage<Pixel = Rgba<u8>>> Canvas for MyBlend<I> {
  type Pixel = Rgba<u8>;

  fn dimensions(&self) -> (u32, u32) {
    self.0.dimensions()
  }

  fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
    self.0.get_pixel(x, y)
  }

  fn draw_pixel(&mut self, x: u32, y: u32, color: Self::Pixel) {
    let mut pix = self.0.get_pixel(x, y);
    let a = pix.0[3];
    pix.blend(&color);
    pix.0[3] = a;
    self.0.put_pixel(x, y, pix);
  }
}