//canvas widget based on image widget

extern crate rusttype;
extern crate imageproc;
extern crate conv;

use self::rusttype::{FontCollection, Scale, point};

use image;
use image::{GenericImage, ImageBuffer, Pixel};


use orbclient::{Color, Renderer};
use orbimage::Image;
use orbtk::Window;
use orbtk::event::Event;
use orbtk::point::Point;
use orbtk::rect::Rect;
use orbtk::traits::{Click, Place};
use orbtk::widgets::Widget;
use orbtk::theme::{Theme};

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::sync::Arc;
use std::slice;
use std::io::Error;
use std::f32::consts::PI;
//use std::io;
use std::io::prelude::*;
use std::fs::File;

use self::imageproc::math::cast;
use self::imageproc::definitions::Clamp;
use self::conv::ValueInto;


use AddOnsToOrbimage;

use UNDODEPTH;
use CANVASOFFSET;

pub struct Canvas {
    pub rect: Cell<Rect>,
    pub image: RefCell<Image>,
    newundo_image: RefCell<Vec<Image>>,
    mask: RefCell<Image>,
    mask_flag: Cell<bool>,
    mask_enabled: Cell<bool>,
    pub copy_buffer: RefCell<Image>,
    click_callback: RefCell<Option<Arc<Fn(&Canvas, Point)>>>,
    right_click_callback: RefCell<Option<Arc<Fn(&Canvas, Point)>>>,
    clear_click_callback: RefCell<Option<Arc<Fn(&Canvas, Point)>>>,
    shortcut_callback: RefCell<Option<Arc<Fn(&Canvas, char)>>>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Arc<Self> {
        Self::from_image(Image::new(width, height))
    }

    pub fn from_color(width: u32, height: u32, color: Color) -> Arc<Self> {
        Self::from_image(Image::from_color(width, height, color))
    }

    pub fn from_image(image: Image) -> Arc<Self> {
        Arc::new(Canvas {
            rect: Cell::new(Rect::new(0, 0, image.width(), image.height())),
            newundo_image: RefCell::new(vec!(Image::new(image.width(),image.height()))),
            mask: RefCell::new(Image::from_color(image.width(), image.height(), Color::rgba(255,0,0,50))),
            mask_flag: Cell::new(false),
            mask_enabled: Cell::new(false),
            image: RefCell::new(image),
            copy_buffer: RefCell::new(Image::new(0,0)),
            click_callback: RefCell::new(None),
            right_click_callback: RefCell::new(None),
            clear_click_callback: RefCell::new(None),
            shortcut_callback:RefCell::new(None), 
        })
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Arc<Self>, String> {
        Ok(Self::from_image(Image::from_path(path)?))
    }
    
    pub fn save(&self, filename: &String) -> Result <i32, Error>{
        let width = self.rect.get().width as u32;
        let height = self.rect.get().height as u32;
        
        //get image data in form of [Color] slice
        let image_data = self.image.clone().into_inner().into_data();

        // convert u32 values to 4 * u8 (r g b a) values
        let image_buffer = unsafe {
            slice::from_raw_parts(image_data.as_ptr() as *const u8, 4 * image_data.len())
        };
        //let () = image_buffer;
        //To save corectly the image with image::save_buffer
        // we have to take care of correct byte order (rgba <-> abgr)
        let mut new_image_buffer = Vec::new();
        let mut i = 0;
        while i <= image_buffer.len() - 4 {
            new_image_buffer.push(image_buffer[i + 2]);
            new_image_buffer.push(image_buffer[i + 1]);
            new_image_buffer.push(image_buffer[i]);
            new_image_buffer.push(image_buffer[i + 3]);
            i = i + 4;
        }

        if cfg!(feature = "debug"){
            println!("Saving {}", &filename);
            println!("x{} y{} len={}", width, height, image_data.len());
        }
        
        match image::save_buffer(&Path::new(&filename),
                           &new_image_buffer,
                           width,
                           height,
                           image::RGBA(8)){
                Ok(_)   => {
                            if cfg!(feature = "debug"){println!("Saved");}
                            Ok(0)
                            },               
                Err(e) => {
                            if cfg!(feature = "debug"){println!("Error: {}",e);}
                            Err(e)
                            },
                
        }
    }

    pub fn clear(&self){
        //first prepare for undo 
        self.undo_save();
        
       let mut image = self.image.borrow_mut();
       //image.clear();
       image.set(Color::rgba(255, 255, 255,255));
    }
    
    pub fn height(&self) -> u32 {
        self.image.borrow().height()
    }
    
    pub fn width(&self) -> u32 {
        self.image.borrow().width()
    }

/*
    ///crop new image from curent canvas (copy)
    pub fn copy_selection(&self, x: i32,y: i32,w: u32, h: u32) -> Image {
        let image = self.image.borrow();
        let data = image.data();
        let mut vec = vec![];
        
        for y1 in y..y+h as i32 {
            for x1 in x..x+w as i32 {
                vec.push(self.pixcol(x1,y1));
            }
        }
        //println!("len {} w*h {}",vec.len(), w*h);
        Image::from_data(w ,h ,vec.into_boxed_slice()).unwrap()
    }
   
    ///return rgba color of pixel at canvas position (x,y)
    pub fn pixcol(&self, x:i32, y:i32) -> Color {
        let image = self.image.borrow();
        let p = image.width()as i32 * y + x;
        let rgba = image.data()[p as usize];
        rgba
    }
*/

    ///apply some transformations to entire canvas
    pub fn transformation(&self, cod: &str, a: f32, b:i32){
        //first prepare for undo 
        self.undo_save();
     
        let mut width = self.rect.get().width as u32;
        let mut height = self.rect.get().height as u32;
        //get image data in form of [Color] slice
        let image_data = self.image.clone().into_inner().into_data();
        let new_slice = self.trans_from_slice(image_data,width,height,cod,a,b);
        let mut image = self.image.borrow_mut();
        
        if cod == "resize" {
            width = a as u32;
            height = b as u32;
        }
        
        if cod == "rotate90" {
            image.clear();
            image.image(0, 0, height, width, &new_slice[..]);
        }else{
            image.image(0, 0, width, height, &new_slice[..]);
        }
        
    }

    ///apply some transformations to canvas selection (in place)
    pub fn trans_selection(&self, selection: Rect, cod: &str, a: f32, b:i32){
        //first prepare for undo 
        self.undo_save();
        
        let mut width = selection.width;
        let mut height = selection.height;
        let x = selection.x;
        let y = selection.y;
        let mut image = self.image.borrow_mut();
        let image_selection = image.copy_selection(x, y, width, height);
        let new_image = self.trans_image(image_selection, cod,a,b);
        //clear only under selection
        image.rect(x,y,width,height,Color::rgba(255,255,255,255)); 
        
        if cod == "resize" {
            width = a as u32;
            height = b as u32;
        }
        
        if cod == "rotate90" {
            image.image(x, y, height, width, &new_image[..]);
        }else{
            image.image(x,y, width, height, &new_image[..]);
        }
    }


    /// apply some transformation to an image 
    pub fn trans_image (&self, image_selection: Image, cod: &str, a: f32, b: i32) -> Vec<Color> {
        let width = image_selection.width() as u32;
        let height = image_selection.height() as u32;
        //get image data in form of [Color] slice
        let image_data = image_selection.into_data();
        //apply transformation to slice
        let new_slice = self.trans_from_slice(image_data,width,height,cod,a,b);
        //here only return new_slice instead of render because of borrowing issue 
        new_slice
    }

    /// apply some transformation to an image slice
    fn trans_from_slice (&self, image_data: Box<[Color]>, width: u32, height: u32, cod: &str, a: f32, b:i32) -> Vec<Color> {
        //let mut width = width;
        //let mut height = height;
        let image_buffer = unsafe {
            slice::from_raw_parts(image_data.as_ptr() as *const u8, 4 * image_data.len())
        };
                
        let mut imgbuf : image::ImageBuffer<image::Rgba<u8>, _> = image::ImageBuffer::from_raw(width as u32, height as u32, image_buffer.to_vec()).unwrap();
        let vec_image_buffer:Vec<u8> = image::ImageBuffer::into_raw ( 
            match cod.as_ref() {
            
             "blur"            => image::imageops::blur(&imgbuf,a),
             "unsharpen"       => image::imageops::unsharpen(&imgbuf,a,10),
             "flip_vertical"   => image::imageops::flip_vertical(&imgbuf),
             "flip_horizontal" => image::imageops::flip_horizontal(&imgbuf),
             "rotate90"        => image::imageops::rotate90(&imgbuf),
             "rotate"          => self.rotate_center(&imgbuf,(a as f32 * PI/180.0)),
             //"rotate"          => imageproc::affine::rotate_about_center(&imgbuf,(a as f32 * PI/180.0),imageproc::affine::Interpolation::Bilinear),
             "brighten"        => image::imageops::colorops::brighten(&imgbuf, 10),
             "darken"          => image::imageops::colorops::brighten(&imgbuf, -10),
             "contrast"        => image::imageops::colorops::contrast(&imgbuf, a),
             "invert"          => {image::imageops::colorops::invert(&mut imgbuf);
                                    imgbuf},
             "grayscale"       => self.gray2rgba(image::imageops::colorops::grayscale(&imgbuf),
                                            1.2,1.2,1.2),
             "resize"          => { 
                                    self.image.borrow_mut().clear();
                                    image::imageops::resize(&imgbuf,a as u32,b as u32,image::FilterType::Nearest)
                                    },
                             _ => imgbuf,
         });
        
        //convert rgba u8 image buffer back into Color slice
        let mut i = 0 ;
        let mut r: u8 ;
        let mut g: u8 ;
        let mut b: u8 ;
        let mut a: u8 ;
        let mut new_slice = Vec::new();
        while i <= vec_image_buffer.len() - 4 {        
            
            r = vec_image_buffer[i];
            g = vec_image_buffer[i+1];
            b = vec_image_buffer[i+2];
            a = vec_image_buffer[i+3];
            new_slice.push(Color::rgba(b, g, r, a)); //taking care of byte order
            i += 4;
        }
        new_slice
    } 

/// convert grayscale format image to rgba format
    fn gray2rgba (&self, 
                    grayimage: image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
                    r_factor : f32,
                    g_factor : f32,
                    b_factor : f32
                    )
                    -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let mut r: u8 ;
        let mut g: u8 ;
        let mut b: u8 ;
        let mut a: u8 ;
        let mut new_buffer = Vec::new();
        let width = grayimage.width();
        let height = grayimage.height();
        
        for luma in image::ImageBuffer::into_raw (grayimage) {
            
            if luma == 255 {
                r=255;
                g=255;
                b=255;
            }else{
            r = (luma as f32 / r_factor) as u8;
            g = (luma as f32 / g_factor) as u8;
            b = (luma as f32 / b_factor) as u8;
            }
            a = 255;
            new_buffer.push(b);
            new_buffer.push(g);
            new_buffer.push(r);
            new_buffer.push(a);
        }
        let imgbuf : image::ImageBuffer<image::Rgba<u8>, _> = image::ImageBuffer::from_raw(width as u32, height as u32, new_buffer).unwrap();
            imgbuf
    }

    ///rotate image about center
    fn rotate_center<I: GenericImage + 'static>(&self, image: &I, theta: f32) 
        -> ImageBuffer<I::Pixel, Vec<<I::Pixel as Pixel>::Subpixel>>
        where I::Pixel: 'static,
          <I::Pixel as Pixel>::Subpixel: 'static  {
        let (width, height) = image.dimensions();
        let center = ((width/2) as f32, (height/2) as f32);
        self.rotate_nearest(image, center, theta)
    }
    
    ///rotate image using nearest interpolation
    fn rotate_nearest<I: GenericImage + 'static>(&self, image: &I, center: (f32, f32), theta: f32) 
        -> ImageBuffer<I::Pixel, Vec<<I::Pixel as Pixel>::Subpixel>>
        where I::Pixel: 'static,
          <I::Pixel as Pixel>::Subpixel: 'static  {
        let default :<I as image::GenericImage>::Pixel = unsafe { image.unsafe_get_pixel(0,0) };
        let (width, height) = image.dimensions();
        //#TODO calculate new dimensions to fit rotated image; change canvas dimensions too! 
        let mut out = ImageBuffer::new(width, height);

        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        let center_x = center.0;
        let center_y = center.1;

        for y in 0..height {
            let dy = y as f32 - center_y;
            let mut px = center_x + sin_theta * dy - cos_theta * center_x;
            let mut py = center_y + cos_theta * dy + sin_theta * center_x;

            for x in 0..width {

                unsafe {
                    let pix = self.nearest(image, px, py, default);
                    out.unsafe_put_pixel(x, y, pix);
                }

                px += cos_theta;
                py -= sin_theta;
            }
        }

        out
    }

    fn nearest<P: Pixel + 'static, I: GenericImage + 'static>(&self, image: &I, x: f32, y: f32, default: P)
        -> <I as image::GenericImage>::Pixel {
        let rx = x.round();
        let ry = y.round();

        // default if out of bound
        let (width, height) = image.dimensions();
        if rx < 0f32 || rx >= width as f32 || ry < 0f32 || ry >= height as f32 {
            unsafe { image.unsafe_get_pixel(0,0) }  //#FIXME default pixel has to be transparent !
        } else {
           unsafe { image.unsafe_get_pixel(rx as u32, ry as u32) }
        }
    }

/*
    fn interpolate<P: Pixel + 'static, I: GenericImage + 'static>(&self, image: &I, x: f32, y: f32, default: P)
     -> P
    where
        P: Pixel + 'static,
        <P as Pixel>::Subpixel: ValueInto<f32> + Clamp<f32>,
    {
        let left = x.floor();
        let right = left + 1f32;
        let top = y.floor();
        let bottom = top + 1f32;

        let right_weight = x - left;
        let bottom_weight = y - top;

        // default if out of bound
        let (width, height) = image.dimensions();
        if left < 0f32 || right >= width as f32 || top < 0f32 || bottom >= height as f32 {
            default
        } else {
            let (tl, tr, bl, br) = unsafe {
                (
                    image.unsafe_get_pixel(left as u32, top as u32),
                    image.unsafe_get_pixel(right as u32, top as u32),
                    image.unsafe_get_pixel(left as u32, bottom as u32),
                    image.unsafe_get_pixel(right as u32, bottom as u32),
                )
            };
            //self.blend(tl, tr, bl, br, right_weight, bottom_weight)
        }
    }
*/
    fn blend<P>(&self,
        top_left: P,
        top_right: P,
        bottom_left: P,
        bottom_right: P,
        right_weight: f32,
        bottom_weight: f32,
    ) -> P
    where
        P: Pixel,
        P::Subpixel: ValueInto<f32> + Clamp<f32>,
    {
        let top = top_left.map2(&top_right, |u, v| {
            P::Subpixel::clamp((1f32 - right_weight) * cast(u) + right_weight * cast(v))
        });

        let bottom = bottom_left.map2(&bottom_right, |u, v| {
            P::Subpixel::clamp((1f32 - right_weight) * cast(u) + right_weight * cast(v))
        });

        top.map2(&bottom, |u, v| {
            P::Subpixel::clamp((1f32 - bottom_weight) * cast(u) + bottom_weight * cast(v))
        })
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

    pub fn on_clear_click<T: Fn(&Self, Point) + 'static>(&self, func: T) -> &Self {
        *self.clear_click_callback.borrow_mut() = Some(Arc::new(func));
        self
    }

    pub fn emit_clear_click(&self, point: Point) {
        if let Some(ref clear_click_callback) = *self.clear_click_callback.borrow() {
            clear_click_callback(self, point);
        }
    }

    pub fn on_shortcut<T: Fn(&Self, char) + 'static>(&self, func: T) -> &Self {
        *self.shortcut_callback.borrow_mut() = Some(Arc::new(func));
        self
    }

    pub fn emit_shortcut(&self, c: char) {
        if let Some(ref shortcut_callback) = *self.shortcut_callback.borrow() {
            shortcut_callback(self, c);
        }
    }

    /// save image state to undo stack 
    pub fn undo_save(&self) {
        let image = self.image.borrow_mut();
        self.newundo_image.borrow_mut().push(image.clone());
        // prevents undo stack to grow too much!!
        if self.newundo_image.borrow_mut().len() > UNDODEPTH {
            self.newundo_image.borrow_mut().remove(0);
        }
    }

    /// retrive image from undo stack
    pub fn undo (&self) {
        let mut newundo_image = self.newundo_image.borrow_mut();
        let l = newundo_image.len();
        if l > 1 {
            let mut image = self.image.borrow_mut();
            *image=newundo_image[l-1].clone();
            newundo_image.pop();
        }
    }

   ///wrapper for filling an image within a canvas
    pub fn fill (&self, x: i32 , y: i32, color: Color){
        self.undo_save();  //save state for undo
        let mut image = self.image.borrow_mut();
        image.fill(x,y,color);
    }
   
    /// wrapper for paste_selection (paste an external image)
    pub fn paste_selection (&self, x: i32, y:i32, opacity: u8, newimage: Image, ){
        self.undo_save();  //save state for undo
        let mut image = self.image.borrow_mut();
        image.paste_selection(x,y,opacity,newimage);
    }

    /// paste internal copy buffer into canvas
    pub fn paste_buffer (&self, x: i32, y:i32, opacity: u8){
        let mut image = self.image.borrow_mut();
        image.paste_selection(x, y, opacity, self.copy_buffer.borrow().clone());
    }

    /// wrapper interactive paste
    pub fn interact_paste (&self, x: i32, y:i32, opacity: u8, window: &mut Window){
        let mut image = self.image.borrow_mut();
        image.interact_paste(x, y, opacity, self.copy_buffer.borrow().clone(), window);
        
    }

    /// wrapper for interactive circle
    pub fn interact_circle (&mut self, x: i32 , y: i32, color: Color, window: &mut Window) {
        self.undo_save();  //save state for undo
        let mut image = self.image.borrow_mut();
        image.interact_circle(x,y,color,window);
    }

    pub fn paint_on_mask(&self) {
        let mut image = self.image.borrow_mut();
        let image2 = image.clone();
        let mut mask = self.mask.borrow_mut();
        *image = mask.clone();
        *mask = image2;
        if self.mask_flag.get(){
            self.mask_flag.set(false); 
            self.enable_mask(true);
        }else{
            self.mask_flag.set(true);
            self.enable_mask(false);
        }
    }

    pub fn clear_mask(& self) {
        if self.mask_flag.get() {
             self.image.borrow_mut().set(Color::rgba(255, 0, 0,50));
        } else {
            self.mask.borrow_mut().set(Color::rgba(255, 0, 0,50));
        }
    }

    pub fn enable_mask(& self, status: bool){
        self.mask_enabled.set(status);
    }

    pub fn mask_flag(& self) -> bool {
        let flag = self.mask_flag.get();
        flag
    }

    ///Draw some text on canvas
    pub fn text(&self, text: &str, font_path: &str, x0: i32, y0: i32, color: Color, size: i32){
        //self.undo_save();  //save state for undo
        let text = text;
        let size = size as f32;
        //using rusttype to render text

/*        // Load the font at compile time !
        #[cfg(target_os = "linux")]
        let font_data = include_bytes!("/usr/share/fonts/gnu-free/FreeMonoBold.ttf");
        #[cfg(target_os = "redox")]
        let font_data = include_bytes!("/ui/fonts/Mono/Fira/Bold.ttf");
        let collection = FontCollection::from_bytes(font_data as &[u8]);
*/        
        //Load font at runtime
        let mut f = match File::open(font_path.to_owned()) {
            Err(e) => return,
            Ok(f) =>f,
        };
        let mut buffer = Vec::new();
        
        f.read_to_end(&mut buffer).unwrap();
        
        let collection = FontCollection::from_bytes(buffer);

        let font = collection.into_font().unwrap();
        let scale = Scale {x: size, y: size};
        let start = point(x0 as f32, (y0 + CANVASOFFSET) as f32);
        let opacity = color.a() as f32;
        for glyph in font.layout(text, scale, start) {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| self.image.borrow_mut().pixel(
                    x as i32 + bounding_box.min.x ,
                    y as i32 + bounding_box.min.y ,
                    Color::rgba(color.r(), color.g(), color.b(), (v * opacity) as u8)
                ));
            }
        }
    }

/* Here unfortunately I have to reimplement not only the pixel function to
   take care of mask but also the other graphics functions
   because in rust I cannot override the pixel function !! 
*/ 
    ///pixel function with mask support
    pub fn pixel(&self , x: i32, y: i32, color: Color) {
        let mut color = color;
        //if we are not painting the mask apply mask to pixel
        if self.mask_enabled.get(){
            //read from mask tranparency value 
            let alpha_mask = self.mask.borrow().pixcol(x,y).r();
            // add mask transparency to color
            color = Color::rgba(color.r(),color.g(),color.b(),alpha_mask & color.a());
        }
        self.image.borrow_mut().pixel(x, y, color);
    }

    ///return rgba color of image pixel at position (x,y)  NOT SAFE if x y are bigger than current image size, but very fast.
    fn pixcol(&self, x:i32, y:i32) -> Color {
        let p = self.width()as i32 * y + x;
        let rgba = self.image.borrow().data()[p as usize];
        rgba
    }

    ///circle with mask support
    pub fn circle(&self , x0: i32, y0: i32, radius: i32, color: Color) {
        //self.image.borrow_mut().circle(x0, y0, radius, color);
        let mut x = radius.abs();
        let mut y = 0;
        let mut err = -radius.abs();
        
        match radius {
            radius if radius > 0 => {
                err = 0;
                while x >= y {
                    self.pixel(x0 - x, y0 + y, color);
                    self.pixel(x0 + x, y0 + y, color);
                    self.pixel(x0 - y, y0 + x, color);
                    self.pixel(x0 + y, y0 + x, color);
                    self.pixel(x0 - x, y0 - y, color);
                    self.pixel(x0 + x, y0 - y, color);
                    self.pixel(x0 - y, y0 - x, color);
                    self.pixel(x0 + y, y0 - x, color);
                
                    y += 1;
                    err += 1 + 2*y;
                    if 2*(err-x) + 1 > 0 {
                        x -= 1;
                        err += 1 - 2*x;
                    }
                }      
            },
            
            radius if radius < 0 => {
                while x >= y {
                    let lasty = y;
                    err +=y;
                    y +=1;
                    err += y;
                    self.line4points(x0,y0,x,lasty,color);
                    if err >=0 {
                        if x != lasty{
                           self.line4points(x0,y0,lasty,x,color);
                        }
                        err -= x;
                        x -= 1;
                        err -= x;
                    }
                }

                },
                     _ => {
                            self.pixel(x0, y0, color);
                            
                        },
        }
    }
    
    fn line4points(&self, x0: i32, y0: i32, x: i32, y: i32, color: Color){
        self.line(x0 - x, y0 + y, (x+x0), y0 + y, color);
        //self.rect(x0 - x, y0 + y, x as u32 * 2 + 1, 1, color);
        if y != 0 {
            self.line(x0 - x, y0 - y, (x+x0), y0-y , color);
            //self.rect(x0 - x, y0 - y, x as u32 * 2 + 1, 1, color);
        }
    }

    ///Draws antialiased circle with mask support
    pub fn wu_circle (&self, x0: i32, y0: i32, radius: i32, color: Color){
        let r = color.r();
        let g = color.g();
        let b = color.b();
        let a = color.a();
        let mut y =0;
        let mut x = radius;
        let mut d =0_f64;
        
        self.pixel (x0+x,y0+y,color);
        self.pixel (x0-x,y0-y,color);
        self.pixel (x0+y,y0-x,color);
        self.pixel (x0-y,y0+x,color);
        
        while x > y {
            let di = dist(radius,y);
            if di < d { x -= 1;}
            let col = Color::rgba(r,g,b,(a as f64*(1.0-di)) as u8);
            let col2 = Color::rgba(r,g,b,(a as f64*di) as u8);
            
            self.pixel(x0+x, y0+y, col);
            self.pixel(x0+x-1, y0+y, col2);//-
            self.pixel(x0-x, y0+y, col);
            self.pixel(x0-x+1, y0+y, col2);//+
            self.pixel(x0+x, y0-y, col);
            self.pixel(x0+x-1, y0-y, col2);//-
            self.pixel(x0-x, y0-y, col);
            self.pixel(x0-x+1, y0-y, col2);//+
            
            self.pixel(x0+y, y0+x, col);
            self.pixel(x0+y, y0+x-1, col2);
            self.pixel(x0-y, y0+x, col);
            self.pixel(x0-y, y0+x-1, col2);
            self.pixel(x0+y, y0-x, col);
            self.pixel(x0+y, y0-x+1, col2);
            self.pixel(x0-y, y0-x, col);
            self.pixel(x0-y, y0-x+1, col2);
            d = di;
            y += 1;
        }
        
        fn dist(r: i32, y: i32) -> f64{
            let x :f64 = ((r*r-y*y)as f64).sqrt();
            x.ceil()-x
        }
    }
    
    
    pub fn smooth_circle( &self, x: i32, y: i32, radius: u32, color: Color){
        self.image.borrow_mut().smooth_circle(x, y, radius, color);
    }
    
    //rectangle with mask support
    pub fn rect(&self, x: i32, y: i32 ,lenght: u32, width: u32, color: Color){
        //self.image.borrow_mut().rect(x ,y, lenght, width, color);
        let lenght = lenght as i32;
        let width = width as i32;
        self.line(x, y, x+lenght, y, color);
        self.line(x, y+1, x, y+width, color);
        self.line(x+1 ,y+width, x+lenght-1, y+width, color);
        self.line(x+lenght,y+width,x+lenght, y+1, color);
    }
    
    ///line with mask support
    pub fn line(&self, argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color) {
        let mut x = argx1;
        let mut y = argy1;

        let dx = if argx1 > argx2 { argx1 - argx2 } else { argx2 - argx1 };
        let dy = if argy1 > argy2 { argy1 - argy2 } else { argy2 - argy1 };

        let sx = if argx1 < argx2 { 1 } else { -1 };
        let sy = if argy1 < argy2 { 1 } else { -1 };

        let mut err = if dx > dy { dx } else {-dy} / 2;
        let mut err_tolerance;

        loop {
            self.pixel(x, y, color);

            if x == argx2 && y == argy2 { break };

            err_tolerance = 2 * err;

            if err_tolerance > -dx { err -= dy; x += sx; }
            if err_tolerance < dy { err += dx; y += sy; }
        }
    }
    
    /// wu_line with mask support
    pub fn wu_line (&self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        
        let mut x0 = x0 as f64;
        let mut y0 = y0 as f64;
        let mut x1 = x1 as f64;
        let mut y1 = y1 as f64;
        let r = color.r();
        let g = color.g();
        let b = color.b();
        let a = color.a() as f64;
        
        fn ipart (x: f64) -> i32 {
            x.trunc() as i32
        }
        fn round (x: f64) -> i32 {
            ipart(x+0.5) as i32
        }
        fn fpart (x: f64) -> f64 {
            if x <0.0 { return 1.0-(x-x.floor());}
            x-x.floor() 
        }
        fn rfpart(x: f64) -> f64 {
            1.0-fpart(x)
        }
        fn chkalpha (mut alpha :f64) -> u8 {
             if alpha > 255.0 { alpha = 255.0};
             if alpha < 0.0 {alpha = 0.0};
             alpha as u8
        }
        
        let steep :bool = (y1-y0).abs() > (x1-x0).abs();
        let mut temp;
        if steep {
            temp = x0; x0 = y0; y0 = temp;
            temp = x1; x1 = y1; y1 = temp;
        }
        if x0 > x1 {
            temp = x0; x0 = x1; x1 = temp;
            temp = y0; y0 = y1; y1 = temp;
        }
        let dx = x1 -x0;
        let dy = y1- y0;
        let gradient = dy/dx;
        
        let mut xend: f64 = (x0 as f64).round() ;
        let mut yend: f64 = y0 + gradient * (xend - x0);
        let mut xgap: f64 = rfpart(x0+0.5);
        let xpixel1 = xend as i32;
        let ypixel1 = (ipart (yend)) as i32;
        
        if steep {
            self.pixel(ypixel1, xpixel1, Color::rgba(r,g,b,chkalpha(rfpart(yend)*xgap*a)));
            self.pixel(ypixel1+1, xpixel1, Color::rgba(r,g,b,chkalpha(fpart(yend)*xgap*a)));
        }else{
            self.pixel(xpixel1, ypixel1, Color::rgba(r,g,b,chkalpha(rfpart(yend)*xgap*a)));
            self.pixel(xpixel1+1, ypixel1, Color::rgba(r,g,b,chkalpha(fpart(yend)*xgap*a)));
        }
        let mut intery :f64 = yend + gradient;
        xend = x1.round();
        yend = y1 + gradient * (xend-x1);
        xgap = fpart(x1 + 0.5);
        let xpixel2 = xend as i32;
        let ypixel2 = ipart(yend) as i32;
        if steep {
            self.pixel(ypixel2, xpixel2, Color::rgba(r,g,b,chkalpha(rfpart(yend)*xgap*a)));
            self.pixel(ypixel2+1, xpixel2, Color::rgba(r,g,b,chkalpha(fpart(yend)*xgap*a)));
        }else{
            self.pixel(xpixel2, ypixel2, Color::rgba(r,g,b,chkalpha(rfpart(yend)*xgap*a)));
            self.pixel(xpixel2+1, ypixel2, Color::rgba(r,g,b,chkalpha(fpart(yend)*xgap*a)));
        }
        if steep {
            for x in (xpixel1+1)..(xpixel2) {
                self.pixel(ipart(intery) as i32 , x, Color::rgba(r,g,b,chkalpha(a*rfpart(intery))));
                self.pixel(ipart(intery) as i32 + 1, x, Color::rgba(r,g,b,chkalpha(a*fpart(intery))));
                intery += gradient;
            }
        }else{
            for x in (xpixel1+1)..(xpixel2) {
                self.pixel(x, ipart(intery) as i32, Color::rgba(r,g,b,chkalpha(a*rfpart(intery))));
                self.pixel(x, ipart(intery) as i32 + 1, Color::rgba(r,g,b,chkalpha(a*fpart(intery))));
                intery += gradient;
            } 
        }           
    }

    ///continuus brush circular shape with mask support
    pub fn brush_line(&self, argx1: i32, argy1: i32, argx2: i32, argy2: i32, radius: i32, color: Color) {
        let mut x = argx1;
        let mut y = argy1;

        let dx = if argx1 > argx2 { argx1 - argx2 } else { argx2 - argx1 };
        let dy = if argy1 > argy2 { argy1 - argy2 } else { argy2 - argy1 };

        let sx = if argx1 < argx2 { 1 } else { -1 };
        let sy = if argy1 < argy2 { 1 } else { -1 };

        let mut err = if dx > dy { dx } else {-dy} / 2;
        let mut err_tolerance;

        loop {
            self.circle(x, y, radius, color);

            if x == argx2 && y == argy2 { break };

            err_tolerance = 2 * err;

            if err_tolerance > -dx { err -= dy; x += sx; }
            if err_tolerance < dy { err += dx; y += sy; }
        }
    }
    
    ///continuus brush rectangular shape not yet with mask support
    pub fn rect_line(&self, argx1: i32, argy1: i32, argx2: i32, argy2: i32,lenght: u32, width: u32, color: Color) {
        let mut x = argx1;
        let mut y = argy1;

        let dx = if argx1 > argx2 { argx1 - argx2 } else { argx2 - argx1 };
        let dy = if argy1 > argy2 { argy1 - argy2 } else { argy2 - argy1 };

        let sx = if argx1 < argx2 { 1 } else { -1 };
        let sy = if argy1 < argy2 { 1 } else { -1 };

        let mut err = if dx > dy { dx } else {-dy} / 2;
        let mut err_tolerance;

        loop {
            self.rect(x, y, lenght, width, color);

            if x == argx2 && y == argy2 { break };

            err_tolerance = 2 * err;

            if err_tolerance > -dx { err -= dy; x += sx; }
            if err_tolerance < dy { err += dx; y += sy; }
        }
    }

     ///Draws a regular polygon with mask support
    pub fn polygon(&self, x0: i32, y0: i32, r: i32, sides: u32, angle: f32, color: Color, antialias: bool ) {
        let mut x:Vec<i32> = Vec::new();
        let mut y:Vec<i32> = Vec::new();
        let i :usize = 0;
        let sides = sides as usize;
        //find vertices
        for i in 0..sides+1 {
            let t :f32 =angle + 2.0*PI* i as f32 /sides as f32;
            x.push((r as f32 * t.cos()) as i32 + x0);
            y.push((r as f32 * t.sin()) as i32 + y0);
        }
        
        if antialias {
        for i in 0..sides {
            self.wu_line(x[i],y[i],x[i+1],y[i+1],color);
        }
        self.wu_line(x[sides],y[sides],x[0],y[0],color);    
        }else{
        for i in 0..sides {
            self.line(x[i],y[i],x[i+1],y[i+1],color);
        }
        self.line(x[sides],y[sides],x[0],y[0],color);
        }
    }
    
    ///#FIXME wrapper for flood fill with mask support DOES NOT WORK YET !!
    pub fn fill_mask(&self, x: i32, y: i32 , color: Color) {
        //get current pixel color 
        let rgba = self.pixcol(x,y);
        self.flood_fill_scanline(x,y,color.data,rgba.data);  //use rgba and color as i32 values 
    }
    ///stack friendly and fast floodfill algorithm that works with transparency too 
    fn flood_fill_scanline( &self, x:i32, y:i32, new_color: u32, old_color:u32) {
        if old_color == new_color {
            return;
        }
        if self.pixcol(x,y).data  != old_color  {
            return;
        }
        
        let w = self.width() as i32;
        let h = self.height() as i32;
        
        //draw current scanline from start position to the right
        let mut x1 = x;
        
        while x1 < w && self.pixcol(x1,y).data  == old_color  {
            self.pixel(x1,y,Color{data:new_color});
            x1 +=1;
        } 
        //get resulted color because of transparency and use this for comparison 
        let res_color = self.pixcol(x,y).data;
        
        //draw current scanline from start position to the left
        x1 = x -1;
        
        while x1 >= 0 && self.pixcol(x1,y).data  == old_color  {
            self.pixel(x1,y,Color{data:new_color});
            x1 += -1;
          }
        
        //test for new scanlines above
        x1 = x;
        
        while x1 < w && self.pixcol(x1,y).data  == res_color  { 
            if y > 0 && self.pixcol(x1,y-1).data  == old_color  {
              self.flood_fill_scanline(x1, y - 1, new_color, old_color);
            }
            x1 += 1;
          }
        x1 = x - 1;
        while x1 >= 0 && self.pixcol(x1,y).data == res_color {
            if y > 0 && self.pixcol(x1,y - 1).data  == old_color  {
              self.flood_fill_scanline(x1, y - 1, new_color, old_color);
            }
            x1 += -1;
          }
         
         //test for new scanlines below
        x1 = x;
        while x1 < w && self.pixcol(x1,y).data == res_color  {
            //println!("Test below {} {} ", self.pixcol(x1,y).data,old_color);
            if y < (h - 1) && self.pixcol(x1,y + 1).data == old_color {
                self.flood_fill_scanline(x1, y + 1, new_color, old_color);
            }
            x1 +=1;
        }
        x1 = x - 1;
        while x1 >= 0 && self.pixcol(x1,y).data == res_color {
            if y < (h - 1) && self.pixcol(x1,y + 1).data == old_color {
                self.flood_fill_scanline(x1, y + 1, new_color, old_color);
            }
            x1 += -1;
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

    fn draw(&self, renderer: &mut Renderer, _focused: bool, _theme: &Theme) {
        let rect = self.rect.get();
        let image = self.image.borrow();
        renderer.image(rect.x, rect.y, image.width(), image.height(), image.data());
        //#TODO render mask only when needed
       /*if self.mask_enabled.get() {
           let image = self.image.borrow();
           renderer.image(rect.x, rect.y, image.width(), image.height(), image.data());
           let mask = self.mask.borrow();
           renderer.image(rect.x, rect.y, mask.width(), mask.height(), mask.data());
       } else {
           let image = self.image.borrow();
           renderer.image(rect.x, rect.y, image.width(), image.height(), image.data());
       }*/
        
    }

    fn event(&self, event: Event, focused: bool, redraw: &mut bool) -> bool {
        match event {
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
                    //mouse is moving without clicking, emit clear previous click position
                    if !right_button && !left_button && !middle_button {
                        let click_point= Point{x:0,y:0};
                        self.emit_clear_click(click_point);
                    } 
                }
            },
             // Ctrl+z => Undo   
            Event::Text {c} => {
                if c == 'z' {
                    self.undo();
                    *redraw = true;
                }
                if ['v','c','x','Q'].contains(&c) {
                    self.emit_shortcut(c);
                }
            },
            _ => if cfg!(feature = "debug"){println!("CanvasEvent: {:?}", event)} else {()}, 
        }
        focused
    }

    fn visible(&self, flag: bool){
        !flag;
    }
    
    fn name(&self) -> &str {
        "Canvas"
    }
}
