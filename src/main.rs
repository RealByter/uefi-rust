#![no_main]
#![no_std]

extern crate alloc;

use core::mem;

use alloc::vec;
use alloc::vec::Vec;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::rng::Rng;
use uefi::table::boot::SearchType;
use uefi::{Identify, Result};

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();

    // print_image_path(boot_services).unwrap();
    draw_sierpinski(boot_services).unwrap();

    boot_services.stall(10_000_000);
    Status::SUCCESS
}

#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

struct Buffer {
    width: usize,
    height: usize,
    pixels: Vec<BltPixel>,
}

impl Buffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![BltPixel::new(0, 0, 0); width * height],
        }
    }

    fn pixel(&mut self, x: usize, y: usize) -> Option<&mut BltPixel> {
        self.pixels.get_mut(y * self.width + x)
    }

    fn blit(&self, gop: &mut GraphicsOutput) -> Result {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.pixels,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (self.width, self.height),
        })
    }

    fn blit_pixel(&self, gop: &mut GraphicsOutput, coords: (usize, usize)) -> Result {
        gop.blt(BltOp::BufferToVideo {
            buffer: &self.pixels,
            src: BltRegion::SubRectangle {
                coords: coords,
                px_stride: self.width,
            },
            dest: coords,
            dims: (1, 1),
        })
    }
}

fn get_random_usize(rng: &mut Rng) -> usize {
    let mut buf = [0; mem::size_of::<usize>()];
    rng.get_rng(None, &mut buf).expect("get_rng failed");
    usize::from_le_bytes(buf)
}

fn draw_sierpinski(bt: &BootServices) -> Result {
    let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = bt.open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let rng_handle = bt.get_handle_for_protocol::<Rng>()?;
    let mut rng = bt.open_protocol_exclusive::<Rng>(rng_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let mut buffer = Buffer::new(width, height);

    for y in 0..height {
        let r = ((y as f32) / ((height - 1) as f32)) * 255.0;
        for x in 0..width {
            let g = ((x as f32) / ((width -1) as f32)) * 255.0;
            let pixel = buffer.pixel(x, y).unwrap();
            pixel.red = r as u8;
            pixel.green = g as u8;
            pixel.blue = 255;
        }
    }

    buffer.blit(&mut gop)?;

    let size = Point::new(width as f32, height as f32);

    let border = 20.0;
    let triangle = [
        Point::new(size.x / 2.0, border),
        Point::new(border, size.y - border),
        Point::new(size.x - border, size.y - border),
    ];

    let mut p = Point::new(size.x / 2.0, size.y / 2.0);

    loop {
        let v = triangle[get_random_usize(&mut rng) % 3];

        p.x = (p.x + v.x) * 0.5;
        p.y = (p.y + v.y) * 0.5;

        let pixel = buffer.pixel(p.x as usize, p.y as usize).unwrap();
        pixel.red = 0;
        pixel.green = 100;
        pixel.blue = 0;

        buffer.blit_pixel(&mut gop, (p.x as usize, p.y as usize))?;
    }
}

fn print_image_path(boot_services: &BootServices) -> Result {
    let loaded_image =
        boot_services.open_protocol_exclusive::<LoadedImage>(boot_services.image_handle())?;

    let device_path_to_text_handle = *boot_services
        .locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))?
        .first()
        .expect("DevicePathToText is missing");

    let device_path_to_text =
        boot_services.open_protocol_exclusive::<DevicePathToText>(device_path_to_text_handle)?;

    let image_device_path = loaded_image.file_path().expect("File path is not set");
    let image_device_path_text = device_path_to_text
        .convert_device_path_to_text(
            boot_services,
            image_device_path,
            DisplayOnly(true),
            AllowShortcuts(false),
        )
        .expect("convert_device_path_to_text failed");

    info!("Image path: {}", &*image_device_path_text);

    Ok(())
}
