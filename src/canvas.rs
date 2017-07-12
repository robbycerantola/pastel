//canvas widget based on image widget

extern crate image;

use orbtk;

use orbclient::{Color, Renderer};
use orbimage;
use std::cell::{Cell, RefCell};
use std::path::Path;
use std::sync::Arc;

use orbtk::event::Event;
use orbtk::point::Point;
use orbtk::rect::Rect;
use orbtk::traits::{Click, Place};
use orbtk::widgets::Widget;

use std::slice;

pub struct Canvas {
    pub rect: Cell<Rect>,
    pub image: RefCell<orbimage::Image>,
    click_callback: RefCell<Option<Arc<Fn(&Canvas, Point)>>>,
    right_click_callback: RefCell<Option<Arc<Fn(&Canvas, Point)>>>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Arc<Self> {
        Self::from_image(orbimage::Image::new(width, height))
    }

    pub fn from_color(width: u32, height: u32, color: Color) -> Arc<Self> {
        Self::from_image(orbimage::Image::from_color(width, height, color))
    }

    pub fn from_image(image: orbimage::Image) -> Arc<Self> {
        Arc::new(Canvas {
            rect: Cell::new(Rect::new(0, 0, image.width(), image.height())),
            image: RefCell::new(image),
            click_callback: RefCell::new(None),
            right_click_callback: RefCell::new(None)
        })
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Arc<Self>, String> {
        Ok(Self::from_image(orbimage::Image::from_path(path)?))
    }
    
    pub fn save(&self, filename: &String) {
        let width = self.rect.get().width as u32;
        let height = self.rect.get().height as u32;
        //get image data in form of [Color] slice
        let image_data = self.image.clone().into_inner().into_data();

        // convert u32 values to 4 * u8 (r g b a) values
        let image_buffer = unsafe {
            slice::from_raw_parts(image_data.as_ptr() as *const u8, 4 * image_data.len())
        };

        //To save corectly the image with image::save_buffer
        // we have to switch r with b but dont know why!!
        

        let mut new_image_buffer = Vec::new();

        let mut i = 0;

        while i <= image_buffer.len() - 4 {

            new_image_buffer.push(image_buffer[i + 2]);
            new_image_buffer.push(image_buffer[i + 1]);
            new_image_buffer.push(image_buffer[i]);
            new_image_buffer.push(image_buffer[i + 3]);

            i = i + 4;
        }

        println!("Saving {}", &filename);
        println!("x{} y{} len={}", width, height, image_data.len());
        image::save_buffer(&Path::new(&filename),
                           &new_image_buffer,
                           width,
                           height,
                           image::RGBA(8))
                .unwrap();  //TODO gestione errori
        println!("Saved");
    }

    pub fn clear(&self){
       let mut image = self.image.borrow_mut();
       //image.clear();
       image.set(Color::rgb(255, 255, 255));
    }
    pub fn on_right_click<T: Fn(&Self, Point) + 'static>(&self, func: T) -> &Self {
        *self.right_click_callback.borrow_mut() = Some(Arc::new(func));
        self
    }
    pub fn emit_right_click(&self, point: Point) {
        if let Some(ref right_click_callback) = *self.right_click_callback.borrow() {
            right_click_callback(self, point);
        }
    }
}

impl Click for Canvas {
    fn emit_click(&self, point: Point) {
        if let Some(ref click_callback) = *self.click_callback.borrow() {
            click_callback(self, point);
        }
    }

    fn on_click<T: Fn(&Self, Point) + 'static>(&self, func: T) -> &Self {
        *self.click_callback.borrow_mut() = Some(Arc::new(func));
        self
    }
}

impl Place for Canvas {}

impl Widget for Canvas {
    fn rect(&self) -> &Cell<Rect> {
        &self.rect
    }

    fn draw(&self, renderer: &mut Renderer, _focused: bool) {
        let rect = self.rect.get();
        let image = self.image.borrow();
        renderer.image(rect.x, rect.y, image.width(), image.height(), image.data());
    }

    fn event(&self, event: Event, focused: bool, redraw: &mut bool) -> bool {
        match event {
         /*   Event::Mouse { point, left_button, .. } => {
                let rect = self.rect.get();
                if rect.contains(point) && left_button {
                    let click_point: Point = point - rect.point();
                    self.emit_click(click_point);
                    *redraw = true;
                }
            }*/
            
            Event::Mouse {point, right_button, left_button, middle_button, ..} => {
                let rect = self.rect.get();
                if rect.contains(point) {
                    let click_point: Point = point - rect.point();
                    if right_button {
                        //println!("Right_button");
                        let click_point: Point = point - rect.point();
                        self.emit_right_click(click_point);
                        *redraw = true;
                        }
                    if left_button {
                        let click_point: Point = point - rect.point();
                        self.emit_click(click_point);
                        *redraw = true;
                        }
                    if middle_button {println!("Middle_button");}
                    }
                }
            _ => println!("{:?}", event),  //(),
        }

        focused
    }

    fn visible(&self, flag: bool){
        !flag;
    }


}
