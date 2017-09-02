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
    pub red_bar: RefCell<Arc<orbtk::ProgressBar>>,
    pub green_bar: RefCell<Arc<orbtk::ProgressBar>>,
    pub blue_bar: RefCell<Arc<orbtk::ProgressBar>>,
}


impl Palette {
///draw a palette
    pub fn new (x: i32, y:i32, width:u32, height:u32, 
                swatch: std::sync::Arc<orbtk::ColorSwatch>,
                 red_bar: Arc<ProgressBar>,
                 green_bar: Arc<ProgressBar>,
                 blue_bar: Arc<ProgressBar> ) ->Arc<Self> {
       //default 16 colors VGA palette 
       Arc::new(Palette {
            
            swatches : RefCell::new(vec![
                Color::rgb(0,0,0),
                Color::rgb(255,255,255),
                Color::rgb(128,128,128),
                Color::rgb(255,0,0),
                Color::rgb(0,255,0),
                Color::rgb(0,0,255),
                Color::rgb(128,0,0),
                Color::rgb(0,128,0),
                Color::rgb(0,0,128),
                Color::rgb(255,255,0),
                Color::rgb(128,0,128),
                Color::rgb(0,255,255),
                Color::rgb(192,192,192),
                Color::rgb(128,128,0),
                Color::rgb(0,128,128),
                Color::rgb(255,0,255),
                    
                ]),
            
            rect: Cell::new(Rect::new(x,y,width,height)),
            current_swatch:RefCell::new(swatch),
            order: Cell::new(0),
            red_bar: RefCell::new(red_bar),
            green_bar: RefCell::new(green_bar),
            blue_bar: RefCell::new(blue_bar),   
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
        ///draw standard palette
         
        let mut s: std::sync::Arc<orbtk::ColorSwatch>;
        let mut color: Color;
        let mut x: i32;
        let mut y: i32;
                
        //not customizable part of palette
        for k  in 0..self.swatches.borrow().len() {
            color = self.swatches.borrow()[k as usize];
            
            
            s = ColorSwatch::new();
            
            x = self.rect.get().x + SWATCH_SIZE*(k) as i32;
            y = self.rect.get().y;
            
            
            s.position(x,y)
            .size(SWATCH_SIZE as u32, SWATCH_SIZE as u32)
            .color(color);
            
            let s_clone= s.clone();
            let red_bar_clone = self.red_bar.clone();
            let green_bar_clone = self.green_bar.clone();
            let blue_bar_clone = self.blue_bar.clone();
            
            //on click change current color 
            let swatch_clone = self.current_swatch.clone();
            s.on_click(move |_swatch: &ColorSwatch, _point: Point| {
                
                swatch_clone.borrow_mut().color(color);
                red_bar_clone.borrow_mut().value.set((s_clone.read().r() as f32 /2.55) as i32);
                green_bar_clone.borrow_mut().value.set((s_clone.read().g() as f32 /2.55) as i32);
                blue_bar_clone.borrow_mut().value.set((s_clone.read().b() as f32 /2.55) as i32);
            });
        
            window.add(&s);
            

        
        }
    }
  
    pub fn add (&self, color: Color, window: &Window) -> Arc<ColorSwatch> {
        ///add custom swatch color to palette
        
        let mut x: i32;
        let mut y: i32;
        let max: i32 = self.rect.get().width as i32/ SWATCH_SIZE;
        
        x = self.rect.get().x + SWATCH_SIZE*self.swatches.borrow().len() as i32;
        
        y = self.rect.get().y;
        
        if x > self.rect.get().width as i32 - SWATCH_SIZE { 
            x = self.rect.get().x + SWATCH_SIZE*(self.swatches.borrow().len() as i32 - max +1) ;
            y = self.rect.get().y + SWATCH_SIZE;
        }
        
        {
        self.swatches.borrow_mut().push(color);
        }
        
        let s=ColorSwatch::new();
        s.position(x ,y)
        .size(SWATCH_SIZE as u32, SWATCH_SIZE as u32)
        .color(color);
        
        //on click change current color and rgb cursors    
        let s_clone= s.clone();
        let swatch_clone = self.current_swatch.clone();
        let red_bar_clone = self.red_bar.clone();
        let green_bar_clone = self.green_bar.clone();
        let blue_bar_clone = self.blue_bar.clone();
         
        s.on_click(move |_swatch: &ColorSwatch, _point: Point| {
            
            swatch_clone.borrow_mut().color(s_clone.read());
            red_bar_clone.borrow_mut().value.set((s_clone.read().r() as f32 /2.55) as i32);
            green_bar_clone.borrow_mut().value.set((s_clone.read().g() as f32 /2.55) as i32);
            blue_bar_clone.borrow_mut().value.set((s_clone.read().b() as f32 /2.55) as i32);
        });
        
        let id = window.add(&s);
        s.id(id);
        
        s
            
    }
    pub fn count (&self) -> usize {
        self.swatches.borrow().len()
        
    }
    pub fn next (&self) -> usize {
        ///find next empty custom swatch
        let n = self.order.get();
        self.order.set(n+1);
        n
    }
    pub fn hello(&self) {
        println!("Hello...");
    }
}
