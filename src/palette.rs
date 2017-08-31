extern crate orbtk;

use orbtk::{Color, Action, Button, Image, Label, Menu, Point, ProgressBar,
            ControlKnob,Toolbar, ToolbarIcon, Rect, Separator,
            TextBox, Window, Renderer, ColorSwatch}; //Toolbar
use orbtk::traits::{Click, Place, Text};  //Border, Enter
use std;
use std::cell::{Cell, RefCell};
use std::sync::Arc;

const SWATCH_SIZE :i32 = 24;

#[derive(Clone)]
pub struct Palette {
    pub swatches : RefCell<Vec<Color>>,
    rect : Cell<Rect>,
    current_swatch: RefCell<std::sync::Arc<orbtk::ColorSwatch>> ,
    pub order: Cell<usize>,
}


impl Palette {
///draw a palette
    pub fn new (x: i32,y:i32, width:u32,height:u32, swatch: std::sync::Arc<orbtk::ColorSwatch>) ->Arc<Self> {
        
       Arc::new(Palette {
            
            swatches : RefCell::new(vec![
                Color::rgb(0,0,0),
                Color::rgb(255,255,255),
                Color::rgb(100,100,100),
                Color::rgb(255,0,0),
                Color::rgb(0,255,0),
                Color::rgb(0,0,255),
                Color::rgb(12,132,166),
                Color::rgb(13,111,136),
                Color::rgb(11,94,112),
                Color::rgb(12,74,89),
                Color::rgb(7,49,61),
                Color::rgb(100,200,30),    
                ]),
            
            rect: Cell::new(Rect::new(x,y,width,height)),
            current_swatch:RefCell::new(swatch),
            order: Cell::new(0)   
        })
        
        
    }

/*        
    fn init(&mut self) {
            
        let default = vec![
                Color::rgb(0,0,0),
                Color::rgb(255,255,255),
                Color::rgb(100,100,100),
                Color::rgb(255,0,0),
                Color::rgb(0,255,0),
                Color::rgb(0,0,255),
                Color::rgb(12,132,166),
                Color::rgb(13,111,136),
                Color::rgb(11,94,112),
                Color::rgb(12,74,89),
                Color::rgb(7,49,61),
                Color::rgb(100,200,30),    
            ];
            
        for v in default {
            self.swatches.push(v);
        }
        
    }
*/
    
    pub fn draw (&self,  window: &Window) {
         
        let mut s: std::sync::Arc<orbtk::ColorSwatch>;
        let mut color: Color;
        let mut x: i32;
        let mut y: i32;
                
        
        for k  in 0..self.swatches.borrow().len() {
            color = self.swatches.borrow()[k as usize];
            s = ColorSwatch::new();
            
            x = self.rect.get().x + SWATCH_SIZE*(k) as i32;
            y = self.rect.get().y;
            
            
            s.position(x,y)
            .size(SWATCH_SIZE as u32, SWATCH_SIZE as u32)
            .color(color);
            
            let swatch_clone = self.current_swatch.clone();
            s.on_click(move |_swatch: &ColorSwatch, _point: Point| {
                
                swatch_clone.borrow_mut().color(color);
            });
        
            window.add(&s);
        }
    }
  
    pub fn add (&self, color: Color, window: &mut Window) {
        
        //let s: std::sync::Arc<orbtk::ColorSwatch>;
        let mut x: i32;
        let mut y: i32;
        
        x = self.rect.get().x + SWATCH_SIZE*self.swatches.borrow().len() as i32;
        y = self.rect.get().y;
        
        {
        self.swatches.borrow_mut().push(color);
        }
        
        let s=ColorSwatch::new();
        s.position(x ,y)
        .size(SWATCH_SIZE as u32, SWATCH_SIZE as u32)
        .color(color);
            
            
            let swatch_clone = self.current_swatch.clone(); 
            s.on_click(move |_swatch: &ColorSwatch, _point: Point| {
                
                swatch_clone.borrow_mut().color(color);
            });
            
            window.add(&s);
            
    }
    pub fn count (&self) -> usize {
        self.swatches.borrow().len()
        
    }
    pub fn hello(&self) {
        println!("Hello...");
    }
}
