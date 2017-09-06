extern crate orbtk;

use orbtk::{Color, Action, Button, Image, Label, Menu, Point, ProgressBar,
            ControlKnob,Toolbar, ToolbarIcon, Rect, Separator,
            TextBox, Window, Renderer, ColorSwatch}; //Toolbar
use orbtk::traits::{Click, Place, Text};  //Border, Enter
use std;
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::io::Error;
use std::path::{Path, PathBuf};

use std::fs::File;
use std::io::prelude::*;

const SWATCH_SIZE :i32 = 24;
const SWATCH_MAX :usize = 109; //67


#[derive(Clone)]
pub struct Palette {
    pub swatches : RefCell<Vec<Color>>,
    pub objects : RefCell<Vec<Arc<ColorSwatch>>>,
    rect : Cell<Rect>,
    current_swatch: RefCell<std::sync::Arc<orbtk::ColorSwatch>> ,
    pub order: Cell<usize>,
    pub red_bar: RefCell<Arc<orbtk::ProgressBar>>,
    pub green_bar: RefCell<Arc<orbtk::ProgressBar>>,
    pub blue_bar: RefCell<Arc<orbtk::ProgressBar>>,
}


impl Palette {

    pub fn new (x: i32, y:i32, width:u32, height:u32, 
                swatch: std::sync::Arc<orbtk::ColorSwatch>,
                 red_bar: Arc<ProgressBar>,
                 green_bar: Arc<ProgressBar>,
                 blue_bar: Arc<ProgressBar> ) ->Arc<Self> {

       Arc::new(Palette {
            swatches : RefCell::new(Vec::new()),
            objects: RefCell::new(Vec::new()),
            rect: Cell::new(Rect::new(x,y,width,height)),
            current_swatch:RefCell::new(swatch),
            order: Cell::new(16),
            red_bar: RefCell::new(red_bar),
            green_bar: RefCell::new(green_bar),
            blue_bar: RefCell::new(blue_bar),
        })
    }
    
    pub fn prepare (&self,  window: &Window) {
                 
        let mut s: std::sync::Arc<orbtk::ColorSwatch>;
        let mut x: i32;
        let mut y: i32;
                
        //default part of colors to be inserted into palette, not customizable (16 colors VGA)
        let mut default = vec![
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
                ];
        
        //customizable part prepared with empy (white) swatches        
        for i in 0..SWATCH_MAX {
            default.push(Color::rgb(255,255,255));
        }
        
        let mut id;
        let mut k=0;
        let max: i32 = self.rect.get().width as i32/ SWATCH_SIZE;  //max swatches per line in allocated space
        //add all colors to palette
        for color  in default {
                       
            s = ColorSwatch::new();
            
            x = self.rect.get().x + SWATCH_SIZE*(k) as i32;
            y = self.rect.get().y;
            
            if x > self.rect.get().width as i32 - SWATCH_SIZE { 
                x = self.rect.get().x + SWATCH_SIZE*(self.swatches.borrow().len() as i32 - max +1) ;
                y = self.rect.get().y + SWATCH_SIZE;
                if x > self.rect.get().width as i32 - SWATCH_SIZE {
                    x = self.rect.get().x + SWATCH_SIZE*(self.swatches.borrow().len() as i32 - 2*max +1) ;
                    y = self.rect.get().y + 2*SWATCH_SIZE;
                }
            }
            
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
                swatch_clone.borrow().color(color);
                red_bar_clone.borrow().value.set((s_clone.read().r() as f32 /2.55) as i32);
                green_bar_clone.borrow().value.set((s_clone.read().g() as f32 /2.55) as i32);
                blue_bar_clone.borrow().value.set((s_clone.read().b() as f32 /2.55) as i32);
            });
        
            id = window.add(&s);
            s.id(id);
            self.objects.borrow_mut().push(s); 
            self.swatches.borrow_mut().push(color);  //
            
            k +=1;
        }
    }
    
    pub fn change(&self, id: usize, color: Color){
        //change color to element of palette by id
        self.objects.borrow_mut()[id].color(color);  //#TODO why register same value in 2 places?
        self.swatches.borrow_mut()[id] = color;
    }
    
    ///add custom swatch color to palette
    pub fn add (&self, color: Color, window: &Window) {
         
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
        self.objects.borrow_mut().push(s); 
        
    }
    
    pub fn reset (&self) {
        for k in 16..SWATCH_MAX {   
                self.change(k,Color::rgb(255,255,255));
        }
        self.order.set(16);    
    }
    
    pub fn count (&self) -> usize {
        self.swatches.borrow().len()
        
    }
    ///find next empty custom swatch
    pub fn next (&self) -> usize {
        
        let n = self.order.get();
        if n < SWATCH_MAX {
            self.order.set(n+1);
        }
        n
    }

    pub fn save(&self, filename: &String ) -> Result <i32, Error>{
        
        let mut palette_data = self.swatches.clone().into_inner();
        let mut payload = String::new();
        let mut r;
        let mut g;
        let mut b;
        palette_data=palette_data[16..].to_vec(); //not saving first 16 default swatches 
        //serialize
        for col in palette_data {
            r= col.r();
            g= col.g();
            b= col.b();
            payload.push_str(&r.to_string());
            payload.push_str(",");
            payload.push_str(&g.to_string());
            payload.push_str(",");
            payload.push_str(&b.to_string());
            payload.push_str(",");
        }
        
        payload.pop(); //remove last colon    
        
        if cfg!(feature = "debug"){
            println!("Save palette ");
            println!("{:?}",payload);
        }
        
        let path = Path::new(&filename);
        let display = path.display();
        
        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}",display,why),
            Ok(file) => file,
        };

        // Write payload to `file`, returns `io::Result<()>`
        match file.write_all(payload.as_bytes()) {
            Err(why) => {
                println!("couldn't write to {}: {}", display,why);
                Err(why)
            },
            Ok(_) => {
                println!("successfully wrote to {}", display);
                Ok(0)
                },
        }
    }
        
    pub fn load(&self, filename: &PathBuf ) -> Result <i32, Error>{
        
        let path = Path::new(&filename);
        let display = path.display();
        let mut file = match File::open(&path) {
            Err(why) => return Err(why), //panic!("couldn't open {}: {}", display,why),
            Ok(file) => file,
        };
        let mut payload = String::new();
        let mut colors :Vec<u8> = Vec::new();
        match file.read_to_string(&mut payload) {
            Err(why) => return Err(why) ,// panic!("couldn't read {}: {}", display,why),
            Ok(_) => {
                //deserialize
                colors = payload.split(",").map(|payload| payload.parse::<u8>().unwrap()).collect();
                let mut i=0;
                let mut sw=0;
                let mut last=SWATCH_MAX;
                if colors.len() <SWATCH_MAX {last=colors.len()/3}
                //append loaded palette to current one
                for k in self.next()..last {   
                    self.objects.borrow_mut()[k].color(Color::rgb(colors[i],colors[i+1],colors[i+2]));
                    self.swatches.borrow_mut()[k] = Color::rgb(colors[i],colors[i+1],colors[i+2]);
                    //find empty swatch 
                    if sw==0 && colors[i] ==  255 && colors[i+1] == 255 && colors[i+2] == 255 {
                        sw = k;
                    }
                    
                    i +=3;
                }
                self.order.set(sw); //set next empty swatch
                
                },
        }
        Ok(0)
    }
}
