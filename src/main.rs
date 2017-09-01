/*Pastel by Robby 21-05-2017
simple image editor in Rust for Redox
*/

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate orbtk;
extern crate orbimage;
extern crate image;
extern crate orbclient;

use orbtk::{Color, Action, Button, Image, Label, Menu, Point, ProgressBar,
            ControlKnob,Toolbar, ToolbarIcon, Rect, Separator,
            TextBox, Window, Renderer, ColorSwatch};
use orbtk::dialogs::FileDialog;
use orbtk::traits::{Click, Place, Text};  //Border, Enter
use orbtk::cell::CloneCell;

use orbclient::EventOption;

use std::rc::Rc;
use std::cell::{Cell, RefCell}; //, RefMut
use std::sync::Arc;
use std::process;
use std::process::Command;
use std::env;
use std::collections::HashMap;
use std::path::Path;

//use std::borrow::Borrow;
//use std::borrow::BorrowMut;

mod dialogs;
use dialogs::{dialog,popup,new_dialog};

mod canvas;
use canvas::{Canvas};

mod palette;
use palette::Palette;

//structure to store tools properties 
struct Property{
    name: CloneCell<String>,
    value: Cell<i32>,
}

impl Property {
    fn new(name: &str, value: i32) -> Arc<Self> {
        Arc::new(Property {
        name: CloneCell::new(name.to_owned()),
        value: Cell::new(value),
        })
    }

    fn name<S: Into<String>>(&self, text: S) -> &Self {
        self.name.set(text.into());
        self
    }

    fn value(&self, value: i32) -> &Self {
        self.value.set(value);
        self
    }
}

struct MySize {
    x: u32,
    y: u32,
}

//canvas position
const CANVASOFFSET: i32 = 200;

fn main() {

    // deal with icons path under diferent os
    #[cfg(target_os = "linux")]
    let root = Path::new("./res/");
    
    #[cfg(target_os = "redox")]
    let root = Path::new("/ui/pastel/");

    #[cfg(target_os = "windows")]
    let root = Path::new("./res/");
    
    if let Ok(_) = env::set_current_dir(&root) {}
    
    //get user home directory (writable) 
    let mut home_dir = String::new();
    match env::home_dir() {
        Some(path) => {
                home_dir.push_str(path.to_str().unwrap());
                if cfg!(feature = "debug"){println!("Home path:{}", home_dir);}                        
                },
        None => println!("Impossible to get your home dir!"),
    }

    //canvas default size
    let mut size = MySize{x: 1024, y:500};    

    let filename;          //FIXME change filename type to Box so we can update

    //deal with command line arguments
    let args: Vec<String> = env::args().collect();
    
    //only name given
    if args.len() > 1 {

        filename = args[1].clone();
    } else {
        filename = String::from("../test.png");  //no name
    }
    
    //size given
    if args.len() > 2 {
       let k: Vec<_> = args[2].split("x").collect();
       size.x = (*k[0]).parse().unwrap() ;
       size.y = (*k[1]).parse().unwrap() ;
    }

    //load canvas from existing file or create new one with filename size
    let canvas = load_image(&filename, &size);

    //Tools and properties 
    //create new tool with some properties and initial values
    let mut ntools = HashMap::new();
    ntools.insert("pen",vec![Property::new("Size",1),Property::new("Opacity",100)]);
    ntools.insert("line",vec![Property::new("Opacity",100)]);
    ntools.insert("polyline",vec![Property::new("Size",1),Property::new("Opacity",100)]); 
    ntools.insert("brush",vec![Property::new("Size",4),Property::new("Opacity",100),Property::new("Shape",0)]);
    ntools.insert("fill",vec![Property::new("Opacity",100)]);
    ntools.insert("rectangle",vec![Property::new("Opacity",100),Property::new("Filled",1)]);
    ntools.insert("circle",vec![Property::new("Opacity",100)]);

    //use invisible Label for storing current active tool
    let tool = Label::new();
    tool.text("pen");
    
    //define current selection
    let selection = Rect::new(0,0,0,0);
    
    
    //if pastel_copy_buffer.png exists load it into buffer
    //for copy/paste between instances 
    
    let buffer: Rc<RefCell<orbimage::Image>> = Rc::new(RefCell::new(load_buffer()));
    
    //implement GUI
    
    let mut x = 10;
    let mut y = 56;
    
    let title = format!("Pastel: {}", filename);
    //resizable main window
    let mut window = Window::new_flags(Rect::new(100, 100, 1024, 718),
                                       &title.to_owned(),
                                       &[orbclient::WindowFlag::Resizable ]);

    // color swatch 
    let swatch = ColorSwatch::new();
    swatch.position(320,56).size(24,48);
    swatch.color(orbtk::Color::rgb(0,0,0));
    window.add(&swatch);
    let swatch_clone=swatch.clone();
    
    // create a new palette at x,y,width,height linked to swatch 
    let palette=Palette::new(20,120,window.width(),50,swatch_clone);


    // show on window the palette
    palette.draw(&window);
    //palette.add(Color::rgb(010,020,230),&mut window); // add swatch to palette 
    
    /*
    {
    // add new color to palette on window 
    let window_clone = &mut window as *mut Window;
    unsafe{palette.clone().add(Color::rgb(200,100,50),&mut *window_clone);}//here works but not inside a closure !!
    unsafe{palette.clone().add(Color::rgb(100,200,150),&mut *window_clone);}
    }
    */



    // use forked version of orbtk to get ProgressBar rendered in colors setting fg
    //color picker
    let red_bar = ProgressBar::new();
    let green_bar = ProgressBar::new();
    let blue_bar = ProgressBar::new();
    let red_label = Label::new();
    red_label.text("R: 0").position(x, y).size(48, 16);
    red_label.fg.set(orbtk::Color::rgb(255,0,0));
    window.add(&red_label);
    
    {
    red_bar.fg.set(orbtk::Color::rgb(255,0,0));  
    let swatch_clone = swatch.clone();
    let green_bar_clone_r = green_bar.clone();
    let blue_bar_clone_r = blue_bar.clone();
    red_bar
        .position(x+48, y)
        .size(256, 16)
        .on_click(move |red_bar: &ProgressBar, point: Point| {
                      let progress = point.x * 100 / red_bar.rect.get().width as i32;
                      red_label.text.set(format!("R: {}%", progress));
                      red_bar.value.set(progress);
                      //refresh color swatch
                      let r = (progress as f32 * 2.56) as u8;
                      let g = (green_bar_clone_r.value.get() as f32 * 2.56) as u8;
                      let b = (blue_bar_clone_r.value.get() as f32 * 2.56) as u8;
                      
                      swatch_clone.bg.set(orbtk::Color::rgb(r,g,b));
                  });
    window.add(&red_bar);
    }
    y += red_bar.rect.get().height as i32 + 2;

    let green_label = Label::new();
    green_label.text("G: 0").position(x, y).size(48, 16);
    green_label.fg.set(orbtk::Color::rgb(0,255,0));
    window.add(&green_label);

    {
    green_bar.fg.set(orbtk::Color::rgb(0,255,0));
    let swatch_clone = swatch.clone();
    let red_bar_clone_g = red_bar.clone();
    let blue_bar_clone_g = blue_bar.clone();
    green_bar
        .position(x+48, y)
        .size(256, 16)
        .on_click(move |green_bar: &ProgressBar, point: Point| {
                      let progress = point.x * 100 / green_bar.rect.get().width as i32;
                      green_label.text.set(format!("G: {}%", progress ));
                      green_bar.value.set(progress);
                      //refresh color swatch
                      let g = (progress as f32 * 2.56) as u8;
                      let r = (red_bar_clone_g.value.get() as f32 * 2.56) as u8;
                      let b = (blue_bar_clone_g.value.get() as f32 * 2.56) as u8;
                      swatch_clone.bg.set(orbtk::Color::rgb(r,g,b));
                  });
    window.add(&green_bar);
    }
    y += green_bar.rect.get().height as i32 + 2;


    let blue_label = Label::new();
    blue_label.text("B: 0").position(x, y).size(48, 16);
    blue_label.fg.set(orbtk::Color::rgb(0,0,255));
    window.add(&blue_label);
    
    {
    blue_bar.fg.set(orbtk::Color::rgb(0,0,255));
    let swatch_clone = swatch.clone();
    let green_bar_clone_b = green_bar.clone();
    let red_bar_clone_b = red_bar.clone();
    blue_bar
        .position(x+48, y)
        .size(256, 16)
        .on_click(move |blue_bar: &ProgressBar, point: Point| {
                      let progress = point.x * 100 / blue_bar.rect.get().width as i32;
                      blue_label.text.set(format!("B: {}%", progress));
                      blue_bar.value.set(progress);
                      //refresh color swatch
                      let b = (progress as f32 * 2.56) as u8;
                      let r = (red_bar_clone_b.value.get() as f32 * 2.56) as u8;
                      let g = (green_bar_clone_b.value.get() as f32 * 2.56) as u8;
                      //swatch_clone.bg.set(orbtk::Color::rgb(r,g,b));
                      swatch_clone.color(orbtk::Color::rgb(r,g,b));
                      
                  });
    window.add(&blue_bar);
    }
    y += blue_bar.rect.get().height as i32 + 10;
    
    // tool size bar
    let size_label = Label::new();
    size_label.text("Size: 1").position(x+380, 56).size(64, 16);
    size_label.visible.set(false);
    window.add(&size_label);

    let size_bar = ProgressBar::new();
    let tool_clone = tool.clone();
    let ntools_clone=ntools.clone();
    let size_label_clone = size_label.clone();
    size_bar.value.set(1);
    size_bar.visible.set(false);
    size_bar
        .position(x+450, 56)
        .size(256, 16)
        .on_click(move |size_bar: &ProgressBar, point: Point| {
                      let progress = point.x * 100 / size_bar.rect.get().width as i32;
                      size_label_clone.text.set(format!("Size: {}", progress));
                      size_bar.value.set(progress);
                      
                      //save size value for current tool
                      let cur_tool = tool_clone.text.get();
                      let a: &str = &cur_tool[..];  //workarround to convert String into &str                      
                      property_set(&ntools_clone[a],"Size",progress);
                      
                  });
    window.add(&size_bar);
    
    // tool transparency bar
    let trans_label = Label::new();
    trans_label.text("Opacity: 100%").position(x+340, 90).size(120, 16);
    trans_label.visible.set(true);
    //blue_label.fg.set(orbtk::Color::rgb(0,0,255));
    window.add(&trans_label);

    let trans_bar = ProgressBar::new();
    let tool_clone = tool.clone();
    //let tools_clone = tools.clone();
    let ntools_clone = ntools.clone();
    let trans_label_clone = trans_label.clone();
    trans_bar.value.set(100);
    trans_bar.visible.set(true);
    trans_bar
        .position(x+450, 90)
        .size(256, 16)
        .on_click(move |trans_bar: &ProgressBar, point: Point| {
                      let progress = 1 + point.x * 100 / trans_bar.rect.get().width as i32;
                      trans_label_clone.text.set(format!("Opacity: {}%", progress));
                      trans_bar.value.set(progress);
                      
                      //save Opacity (transparency) value for current tool
                      let cur_tool = tool_clone.text.get();
                      let a: &str = &cur_tool[..];  //workarround to convert String into &str                      
                      property_set(&ntools_clone[a],"Opacity",progress);
                      
                  });
    window.add(&trans_bar);

/*
    // tool Volume nob
    let volume_label = Label::new();
    volume_label.text("Volume: 1").position(x+380, 90).size(128, 16);
    //size_label.fg.set(orbtk::Color::rgb(0,0,255));
    window.add(&volume_label);
    
    let volume = ControlKnob::new(); //try widget control_knob
    let tool_clone = tool.clone();
    let tools_clone=tools.clone();
    let volume_label_clone = volume_label.clone();
    
    volume.border.set(true);
    volume.position(x+500, 120)
        .size(40, 40)   //size.x must be equal to size.y so the circle is exactly inside the rect 
        .on_click(move |volume: &ControlKnob, point: Point| {
                      let progress = Point{ x: point.x ,
                                            y:point.y};
                      volume_label_clone.text.set(format!("Volume: {} {}", progress.x , progress.y));
                      volume.value.set(progress);
                      //let cur_tool = tool_clone.text.get();
                      //let a: &str = &cur_tool[..];  // workarround to convert String into &str                      
                      //tools_clone[a].size(progress);
                      
                  });
    window.add(&volume);
*/
 
    //clickable icon
    match Image::from_path( "pastel100.png" ) {
        Ok(image) => {
            image.position(900, 10);
            image.on_click(move |_image: &Image, _point: Point| {
                               popup("Ciao",
                                  "Pastel is work in progress,\nplease be patient....");
                           });
            window.add(&image);
        }
        Err(err) => {
            let label = Label::new();
            label.position(x, y).size(400, 16).text(err);
            window.add(&label);
        }
    }
    
    // implement toolbars by multiple clickable images loaded in widget ToolbarIcon  
    let mut toolbar_obj = vec![];   //here save all Toolbar widgets clones so we can manage 'selected' property
    let mut toolbar2_obj = vec![];   //create Toolbar2 here so we can manage 'selected','visible' properties from Toolbar
    //TODO let toolbar = Toolbar::new(&window); must specify parent window !!
    let parent_window = &mut window as *mut Window;  //pointer to the parent window
    let mut toolbar3 = Toolbar::new();   //work in progress
    
    let y = 25;
    match ToolbarIcon::from_path("pencil1.png") {
        Ok(image) => {
            image.position(x, y)
                .text("Draft painting".to_owned())
                .selected(true);
                
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let trans_bar_clone = trans_bar.clone();
            let ntools_clone = ntools.clone();
            let size_label_clone = size_label.clone();
            let trans_label_clone = trans_label.clone();
            //let window_clone = &mut window as *mut Window;
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.on_click(move |_image: &ToolbarIcon, _point: Point| {
                               tool_clone.text.set("pen".to_owned());

                               size_bar_clone.visible.set(false);
                               size_label_clone.visible.set(false);

                               let o = property_get(&ntools_clone["pen"],"Opacity").unwrap();
                               trans_bar_clone.value.set(o);
                               trans_label_clone.text(format!("Opacity: {}%",o));
                               //toggle tool in toolbar TODO move into Toolbar
                               unsafe {toggle_toolbar(&mut *toolbar_obj_clone);}
                               //make invisible toolbar2  TODO move into Toolbar
                               unsafe{visible_toolbar(&mut *toolbar2_obj_clone,false);}
                               //make toolbar3 invisible
                               unsafe{(&mut *toolbar3_clone).visible(false);}
                           });
            
            window.add(&image);
            toolbar_obj.push(image.clone());  //TODO toolbar.add(&image);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("pencil2.png") {
        Ok(image) => {
            image.position(x, y)                
                 .text("Draw lines".to_owned());
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let ntools_clone = ntools.clone();
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.on_click(move |_image: &ToolbarIcon, _point: Point| {
                               //set current tool
                               tool_clone.text.set("line".to_owned());
                               
                               //get previous settings
                               size_bar_clone.visible.set(false);
                               size_label_clone.visible.set(false);
                               let o = property_get(&ntools_clone["line"],"Opacity").unwrap();
                               trans_bar_clone.value.set(o);
                               trans_label_clone.text(format!("Opacity: {}%",o));
                               
                               //toggle tool in toolbar
                               unsafe {toggle_toolbar(&mut *toolbar_obj_clone);}
                               //make invisible toolbar2
                               unsafe{visible_toolbar(&mut *toolbar2_obj_clone,false);}
                               //make toolbar3 invisible
                               unsafe{(&mut *toolbar3_clone).visible(false);}
                               });
            window.add(&image);
            toolbar_obj.push(image.clone());

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }


    match ToolbarIcon::from_path("brush.png") {
        Ok(image) => {
            image.position(x, y)
                 .text("Paint brush".to_owned());
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let ntools_clone = ntools.clone();
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.on_click(move |_image: &ToolbarIcon, _point: Point| {
                               tool_clone.text.set("brush".to_owned());
                               size_label_clone.visible.set(true);
                               size_bar_clone.visible.set(true);
                               //let v=tools_clone.get(&"brush").unwrap().size.get();
                               let v = property_get(&ntools_clone["brush"],"Size").unwrap();
                               size_bar_clone.value.set(v);
                               size_label_clone.text(format!("Size: {}",v));
                               
                               let o = property_get(&ntools_clone["brush"],"Opacity").unwrap();
                               trans_bar_clone.value.set(o);
                               trans_label_clone.text(format!("Opacity: {}%",o));
                               
                               //toggle tool in toolbar
                               unsafe {toggle_toolbar(&mut *toolbar_obj_clone);}
                               //make visible toolbar2
                               unsafe{visible_toolbar(&mut *toolbar2_obj_clone,true);}
                               //make toolbar3 invisible
                               unsafe{(&mut *toolbar3_clone).visible(false);}
                               });
            window.add(&image);
            toolbar_obj.push(image.clone());

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("fillbucket.png") {
        Ok(item) => {
            
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let ntools_clone = ntools.clone();
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            item.position(x, y)
                 .text("Fill up area with color".to_owned())
                 .on_click(move |_image: &ToolbarIcon, _point: Point| {
                               tool_clone.text.set("fill".to_owned());
                               size_label_clone.visible.set(false);
                               size_bar_clone.visible.set(false);
                               
                               let o = property_get(&ntools_clone["fill"],"Opacity").unwrap();
                               trans_bar_clone.value.set(o);
                               trans_label_clone.text(format!("Opacity: {}%",o));
                               
                               //toggle tool in toolbar
                               unsafe {toggle_toolbar(&mut *toolbar_obj_clone);}
                               //make invisible toolbar2
                               unsafe{visible_toolbar(&mut *toolbar2_obj_clone,false);}
                               //make toolbar3 invisible
                               unsafe{(&mut *toolbar3_clone).visible(false);}
                               });
            window.add(&item);
            toolbar_obj.push(item.clone());
            
            x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }


    match ToolbarIcon::from_path("polyline.png") {
        Ok(image) => {
            image.position(x, y)                
                 .text("Draw polylines".to_owned());
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let ntools_clone = ntools.clone();
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.on_click(move |_image: &ToolbarIcon, _point: Point| {
                               //set current tool
                               tool_clone.text.set("polyline".to_owned());
                               
                               //get previous settings
                               size_bar_clone.visible.set(true);
                               size_label_clone.visible.set(true);
                               let o = property_get(&ntools_clone["polyline"],"Opacity").unwrap();
                               trans_bar_clone.value.set(o);
                               trans_label_clone.text(format!("Opacity: {}%",o));
                               let s = property_get(&ntools_clone["polyline"],"Size").unwrap();
                               size_bar_clone.value.set(s);
                               size_label_clone.text(format!("Size: {}",s));
                               
                               //toggle tool in toolbar
                               unsafe {toggle_toolbar(&mut *toolbar_obj_clone);}
                               //make visible toolbar2
                               unsafe{visible_toolbar(&mut *toolbar2_obj_clone,false);}
                               //make toolbar3 invisible
                               unsafe{(&mut *toolbar3_clone).visible(false);}
                               });
            window.add(&image);
            toolbar_obj.push(image.clone());

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("rectangle.png") {
        Ok(image) => {
            image.position(x, y)                
                 .text("Draw rectangles".to_owned());
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let ntools_clone = ntools.clone();
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.on_click(move |_image: &ToolbarIcon, _point: Point| {
                               //set current tool
                               tool_clone.text.set("rectangle".to_owned());
                               
                               //get previous settings
                               size_bar_clone.visible.set(false);
                               size_label_clone.visible.set(false);
                               let o = property_get(&ntools_clone["polyline"],"Opacity").unwrap();
                               trans_bar_clone.value.set(o);
                               trans_label_clone.text(format!("Opacity: {}%",o));
                               
                               //toggle tool in toolbar
                               unsafe {toggle_toolbar(&mut *toolbar_obj_clone);}
                               //make invisible toolbar2
                               unsafe{visible_toolbar(&mut *toolbar2_obj_clone,false);}
                               //make toolbar3 visible
                               unsafe{(&mut *toolbar3_clone).visible(true);}
                               });
            window.add(&image);
            toolbar_obj.push(image.clone());

            //x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

//2nd toolbar
    x=500;
    
    match ToolbarIcon::from_path("circle.png") {
        Ok(item) => {
            let ntools_clone = ntools.clone();
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            item.position(x, y)
                 .text("Circular shape".to_owned())
                 .on_click(move |_image: &ToolbarIcon, _point: Point| {
                               property_set(&ntools_clone["brush"],"Shape",0);
                               
                               //toggle shape in toolbar2
                               unsafe {toggle_toolbar(&mut *toolbar2_obj_clone);}
                               });
            window.add(&item);
            toolbar2_obj.push(item.clone());
            
            x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("block.png") {
        Ok(item) => {
            let ntools_clone = ntools.clone();
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            item.position(x, y)
                 .text("Blocky shape".to_owned())
                 .on_click(move |_image: &ToolbarIcon, _point: Point| {
                               property_set(&ntools_clone["brush"],"Shape",1);
                               
                               //toggle shape in toolbar2
                               unsafe {toggle_toolbar(&mut *toolbar2_obj_clone);}
                               });
            window.add(&item);
            toolbar2_obj.push(item.clone());
            
            x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("buffer.png") {
        Ok(item) => {
            let ntools_clone = ntools.clone();
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            item.position(x, y)
                 .text("from buffer".to_owned())
                 .on_click(move |_image: &ToolbarIcon, _point: Point| {
                               property_set(&ntools_clone["brush"],"Shape",2);
                               
                               //toggle shape in toolbar2
                               unsafe {toggle_toolbar(&mut *toolbar2_obj_clone);}
                               });
            window.add(&item);
            toolbar2_obj.push(item.clone());
            
            //x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    // set 2nd toolbar not visible at start
    let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
    unsafe{visible_toolbar(&mut *toolbar2_obj_clone,false);}

    x = 500;

    //3rd toolbar new api 
    match ToolbarIcon::from_path("rectangle.png") {
        Ok(item) => {
            let ntools_clone = ntools.clone();
            //let toolbar3_clone = toolbar3.clone(); //does not work properly!!
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            item.position(x, y)
                 .text("Not filled".to_owned()) 
                 .on_click(move |_image: &ToolbarIcon, _point: Point| {
                               property_set(&ntools_clone["rectangle"],"Filled",0);
                               
                               //toggle item in toolbar3
                               //toolbar3_clone.toggle(); //does not work properly !!
                               unsafe{(&mut *toolbar3_clone).toggle();}  
                               });

            toolbar3.add(&item,parent_window);
            
            x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("filled.png") {
        Ok(item) => {
            let ntools_clone = ntools.clone();
            //let toolbar3_clone = toolbar3.clone();
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            item.position(x, y)
                 .text("Filled".to_owned())
                 .on_click(move |_image: &ToolbarIcon, _point: Point| {
                               property_set(&ntools_clone["rectangle"],"Filled",1);
                               
                               //toggle item in toolbar3  
                               //toolbar3_clone.toggle();
                               unsafe{(&mut *toolbar3_clone).toggle();}  
                               });

            toolbar3.add(&item,parent_window);
            
            //x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    //toolbar3 not visibile at start
    toolbar3.visible(false);

    //Menu file

    let menu = Menu::new("File");
    menu.position(10, 0).size(32, 16);

    //menu entries for file
            //TODO ask for new dimensions
    {
        let action = Action::new("New");
        action.on_click(move |_action: &Action, _point: Point| {
                           match new_dialog() { 
                                Some(resolution) => {
                                            let path: &str; //="";
                                            if cfg!(target_os = "redox"){
                                                path="/ui/bin/pastel";
                                            } else{
                                                path="../target/release/pastel"; 
                                            }
                                                Command::new(&path)
                                                .arg("new.png")
                                                .arg(resolution.to_owned())
                                                .spawn()
                                                .expect("Command executed with failing error code");
                           
                                                println!("New image opened.");
                                                },
                                                
                                    None => println!("New image cancelled"),
                                }
                        });

        menu.add(&action);
    }

    {
        let action = Action::new("Open");
        let home_dir_clone = home_dir.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            //match dialog("Open", "path:",&home_dir_clone[..]) {
              match FileDialog::new().exec() {
                Some(response) => {
                                    println!("Open {:?} ", response);
                                    let path: &str ;//="";
                                    if cfg!(target_os = "redox"){
                                        path="/ui/bin/pastel";
                                        } else{
                                            path="../target/release/pastel"; 
                                        }
                                    
                                    Command::new(&path)
                                                .arg(response)
                                                .spawn()
                                                .expect("Command executed with failing error code");
                                    },
                None => println!("Cancelled"),
            }
        });
        menu.add(&action);
    }

    {
        let action = Action::new("Save");
        let canvas_clone = canvas.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            match canvas_clone.save(&filename){
                                Ok(_) => (),
                                Err(e) => popup("Error",&format!("{}",e)[..]),
                                }  
                        });
        menu.add(&action);
    }

    {
        let action = Action::new("Save As");
        let canvas_clone = canvas.clone();
        //FIXME change filename after a SaveAs 
        action.on_click(move |_action: &Action, _point: Point| {
                            match dialog("Save As", "path:",&home_dir[..]) {
                            Some(response) => {
                                match canvas_clone.save(&(String::from(response))){
                                    Ok(_) => (),
                                    Err(e) => popup("Error",&format!("{}",e)[..]),
                                }
                                
                                },
                            None => {println!("Cancelled");},
                            }
                        });
        menu.add(&action);
    }

    menu.add(&Separator::new());

    {
        let action = Action::new("Exit");
        action.on_click(move |_action: &Action, _point: Point| {
                            println!("Bye bye...");
                            process::exit(0x0f00);
                        });
        menu.add(&action);
    }

    //Menu edit
    let edit = Menu::new("Edit");
        edit.position(50, 0).size(32, 16);

    //Menu entries for edit



    {
            let action = Action::new("Select");
            //let canvas_clone = canvas.clone();
            let tool_clone = tool.clone();
            //let window_clone = &mut window as *mut Window;
            action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("marquee".to_owned());
                              });
            edit.add(&action);
    }

    edit.add(&Separator::new());


    {
            let action = Action::new("Copy");
            let tool_clone = tool.clone();
            //let mut buffer_clone = buffer.clone();
            //let image= canvas.clone();
            let selection_clone = selection.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("copy".to_owned());
                            
                              });
            edit.add(&action);
    }

    
    {
            let action = Action::new("Paste");
            let tool_clone = tool.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("paste".to_owned());
                              });
            edit.add(&action);
    }

    edit.add(&Separator::new());
    
        {
            let action = Action::new("Add swatch");
            
            let swatch_clone = swatch.clone();
            let palette_clone = palette.clone();
            let window_clone = &mut window as *mut Window;
            
            action.on_click(move |_action: &Action, _point: Point| {
                            let color = swatch_clone.read();
                            
                            unsafe{palette.add(color, &mut *window_clone);}  //RUST bug ??
                //thread 'main' panicked at 'already borrowed: BorrowMutError', /checkout/src/libcore/result.rs:860:4
                            palette.swatches.borrow_mut().push(color);
                            
                            
                            //unsafe{test(s,&mut *window_clone);}
                            println!("{:?}, {:?}",swatch_clone.read(), palette.swatches.borrow());
                            
                              });
            edit.add(&action);
    }

    //Menu tool
    let tools = Menu::new("Tools");
    tools.position(90, 0).size(48, 16);

    //Menu entries for tools
    {
        let action = Action::new("Pen");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("pen".to_owned());
                        });
        tools.add(&action);
    }

    {
        let action = Action::new("Line");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {

                            tool_clone.text.set("line".to_owned());
                        });
        tools.add(&action);
    }

    {
        let action = Action::new("Polyline");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {

                            tool_clone.text.set("polyline".to_owned());
                        });
        tools.add(&action);
    }

    {
        let action = Action::new("Brush");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("brush".to_owned());
                        });
        tools.add(&action);
    }
    
        {
        let action = Action::new("Fill");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("fill".to_owned());
                        });
        tools.add(&action);
    }
    
    {
        let action = Action::new("Rectangle");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("rectangle".to_owned());
                        });
        tools.add(&action);
    }
    
    {
        let action = Action::new("Circle");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("circle".to_owned());
                        });
        tools.add(&action);
    }
    

    //Menu image
    let menuimage = Menu::new("Image");
    menuimage.position (140,0).size (48,16);
    
    //Menu entries for image
    {
            let action = Action::new("Clear");
            let canvas_clone = canvas.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.clear();
                        });
        menuimage.add(&action);
    }
    
    {
            let action = Action::new("Blur");
            let canvas_clone = canvas.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.transformation("blur");
                        });
        menuimage.add(&action);
    }
    
    {
            let action = Action::new("Unsharpen");
            let canvas_clone = canvas.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.transformation("unsharpen");
                        });
        menuimage.add(&action);
    }
    
    {
            let action = Action::new("Verical flip");
            let canvas_clone = canvas.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.transformation("flip_vertical");
                        });
        menuimage.add(&action);
    }

    {
            let action = Action::new("Horizontal flip");
            let canvas_clone = canvas.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.transformation("flip_horizontal");
                        });
        menuimage.add(&action);
    }
    
    {
            let action = Action::new("Brighten");
            let canvas_clone = canvas.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.transformation("brighten");
                        });
        menuimage.add(&action);
    }
    
    {
            let action = Action::new("Darken");
            let canvas_clone = canvas.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.transformation("darken");
                        });
        menuimage.add(&action);
    }

    //Menu help

    let help = Menu::new("Help");
    help.position(190, 0).size(32, 16);

    //menu entries for help

    {
        let action = Action::new("Info");
        action.on_click(move |_action: &Action, _point: Point| {
                            popup("Info",
                                  "Pastel v0.0.9, simple bitmap editor \n for Redox OS by Robby Cerantola");
                        });
        help.add(&action);
    }

    // add menus
    window.add(&menu);
    window.add(&edit);
    window.add(&tools);
    window.add(&menuimage);
    window.add(&help);

    // paint on canvas
    let click_pos: Rc<RefCell<Option<Point>>> = Rc::new(RefCell::new(None));
    let window_clone = &mut window as *mut Window;
    let click_pos_clone = click_pos.clone();
    
    
    canvas
        .position(0, CANVASOFFSET) 
        .on_right_click(move |_ , point:Point|{
            // right click clears last cursor position 
                let mut ck=click_pos_clone.borrow_mut();
                if cfg!(feature = "debug"){
                println!("Right click {:?}",ck);}
                *ck = None;
                })
        .on_click(move |canvas: &Canvas, point: Point| {

            let click = click_pos.clone();
            let size = size_bar.clone().value.get();
            let buffer_clone = buffer.clone();
            let swatch_clone = swatch.clone();
            
            let mut selection_clone = selection.clone();
            
            {
                let mut prev_opt = click.borrow_mut();
                let mut bf = buffer_clone.borrow_mut();
                if let Some(prev_position) = *prev_opt {
                    let mut image = canvas.image.borrow_mut();
                    //let r = (red_bar.clone().value.get() as f32 * 2.55) as u8;
                    //let g = (green_bar.clone().value.get() as f32 * 2.55) as u8;
                    //let b = (blue_bar.clone().value.get() as f32 * 2.55) as u8;
                    let a = (trans_bar.clone().value.get() as f32 * 2.55) as u8;
                    let swc = swatch_clone.read();
                    let color = Color::rgba(swc.r(),swc.g(),swc.b(),a);
                    
                    
                    match tool.clone().text.get().as_ref() {
                        "line"  => {
                                    image.line(prev_position.x,
                                                prev_position.y,
                                                point.x,
                                                point.y,
                                                color);
                                   },
                         "pen"  => image.pixel(point.x, point.y, color),
                         "brush"=> { 
                                    match property_get(&ntools.clone()["brush"],"Shape") {
                                           Some(0) => image.circle(point.x, point.y,-size,
                                                        color),
                                           Some(1) => image.rect(point.x ,point.y,size as u32, size as u32,
                                                        color),
                                           Some(2) =>  image.paste_selection(point.x,point.y,bf.clone()), 
                                           None | Some(_)   => println!("no Shape match!"),
                                        }
                                    },
                         "fill" => image.fill(point.x, point.y,color),
                    "rectangle" => {
                                    let filled = property_get(&ntools.clone()["rectangle"],"Filled").unwrap();
                                    unsafe{
                                            image.interact_rect(point.x,
                                                        point.y,
                                                        color,
                                                        filled == 1,
                                                        &mut *window_clone
                                                        );
                                        }
                                    },
                    "polyline" => {let width = property_get(&ntools.clone()["polyline"],"Size").unwrap();
                                    unsafe{
                                            image.interact_line(point.x,
                                                        point.y,
                                                        color,
                                                        width,
                                                        &mut *window_clone
                                                        );
                                        }   
                                    },
                        "circle"=> image.circle(prev_position.x, prev_position.y,
                                                2*(((point.x-prev_position.x)^2+
                                                (point.y-prev_position.y)^2) as f64).sqrt() as i32,
                                                 color),
                       "marquee"=> {
                                    if let Some(selection) = unsafe{image.select_rect(point.x,
                                                        point.y,&mut *window_clone)}
                                            {
                                                        selection_clone = selection;
                                                        println!("{:?}",selection_clone);
                                                        //*bf = image.copy_selection(selection_clone.x,selection_clone.y,selection_clone.width,selection_clone.height);
                                                        
                                             }
                                        },
                        "copy" =>  {
                                    if let Some(selection) = unsafe{image.select_rect(point.x,
                                                        point.y,&mut *window_clone)}
                                            {
                                             *bf = image.copy_selection(selection.x,selection.y,selection.width,selection.height);
                                             //save buffer to disk as pastel_copy_buffer.png so we can reload when starting new program instance
                                             let newcanvas= Canvas::from_image(bf.clone());
                                             let path = "/tmp/pastel_copy_buffer.png".to_string();
                                             if let Ok(_) = newcanvas.save(&path){}
                                             
                                            }
                                        },
                        "paste" => image.paste_selection(point.x,point.y,bf.clone()),
                              _ => println!("No match!"),          
                    }

                    *prev_opt = Some(point);     
                } else {
                    *prev_opt = Some(point);
                }
            }
        });
    window.add(&canvas);
    window.exec();
}

//Helper functions

fn test (widget: Arc<orbtk::ColorSwatch>, window: &mut orbtk::Window) {
    window.add(&widget);
}


///Load an image from path if exists, other way create new empty canvas
fn load_image(path: &str, size: &MySize) -> Arc<canvas::Canvas> {  
    if cfg!(feature = "debug"){print!("Loading image from:  {} .....", path);}
    match Canvas::from_path(&path) {
        Ok(image) => {
            if cfg!(feature = "debug"){println!(" OK");}
            image
        }
        Err(err) => {
            if cfg!(feature = "debug"){println!("Failed: {} \n Creating new one ", err);}
            let image = Canvas::from_color(size.x, size.y, Color::rgb(255, 255, 255));
            image
        }
    }
}

///load pastel_copy_buffer if exists
fn load_buffer() -> orbimage::Image {
    
    let path="/tmp/pastel_copy_buffer.png".to_string();
    
    if cfg!(feature = "debug"){print!("Loading copy buffer from:  {} .....", path);}
    match orbimage::Image::from_path(&path) {
        Ok(image) => {
            if cfg!(feature = "debug"){println!(" OK");}
            image
        }
        Err(err) => {
            if cfg!(feature = "debug"){println!("Failed: {} \n Creating empty one ", err);}
            let image = orbimage::Image::new(10,10);
            image
        }
    }
}    

///get tool property value
fn property_get( properties: &Vec<Arc<Property>>  , name: &str) -> Option<i32> {
    for a in properties {
        if &a.name.get() == name {
            return Some(a.value.get());
        }
    } 
    None
}

///set tool property value
fn property_set( properties: &Vec<Arc<Property>>  , name: &str, value: i32) {
    for a in properties {
        if &a.name.get() == name {
            a.value.set(value);
        }
    } 
}

///unselect all toolbar items
fn toggle_toolbar (toolbar_obj: &mut Vec<Arc<ToolbarIcon>>) {
    for i in 0..toolbar_obj.len(){
        if let Some(toolbar) = toolbar_obj.get(i) {
            toolbar.selected(false);
        }
    }
}
    
///set visibility for all toolvar items
fn visible_toolbar (toolbar_obj: &mut Vec<Arc<ToolbarIcon>>, v: bool) {
    for i in 0..toolbar_obj.len(){
        if let Some(toolbar) = toolbar_obj.get(i) {
            toolbar.visible.set(v);
        }
    }
}

/*
///draw a palette
fn palette (start_y: i32, max_swatches: u32, window: &Window) {
    
    let mut s: std::sync::Arc<orbtk::ColorSwatch>;
    
    for k in 1..max_swatches {
        s=ColorSwatch::new();
        s.position(24*k as i32,start_y)
        .size(24, 24)
        .color(Color::rgb(100,100,100));
    
        window.add(&s);
    }
}
*/






//  to be added directly to orbclient ?
trait AddOnsToOrbimage {
        fn fill(&mut self, x: i32 , y: i32, color: Color);
        fn flood_fill4(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
        fn flood_fill_scanline(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
        fn flood_fill_line(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
        fn pixcol(&self, x:i32, y:i32) -> Color;
        fn pixraw(&self, x:i32, y:i32) -> u32;
        fn interact_rect(&mut self, x: i32 , y: i32, color: Color, filled: bool, window: &mut orbtk::Window);
        fn interact_line(&mut self, x: i32 , y: i32, color: Color,width: i32, window: &mut orbtk::Window);
        fn select_rect(&mut self, x: i32 , y: i32, window: &mut orbtk::Window) ->Option<Rect>;
        fn copy_selection(&self, x: i32,y: i32,w: u32, h: u32) -> orbimage::Image;
        fn paste_selection (&mut self, x: i32, y:i32,buffer: orbimage::Image);
    }

impl AddOnsToOrbimage for orbimage::Image {
    ///return rgba color of image pixel at position (x,y)
    fn pixcol(&self, x:i32, y:i32) -> Color {
        let p = self.width()as i32 * y + x;
        let rgba = self.data()[p as usize];
        rgba
    }
    
    fn pixraw (&self, x:i32, y:i32) -> u32 {
        self.pixcol(x,y).data 
    }
    ///wrapper for flood fill 
    fn fill(&mut self, x: i32, y: i32 , color: Color) {
        //get current pixel color 
        let rgba = self.pixcol(x,y);
        //self.flood_fill4(x,y,color.data,rgba.data);  //use rgba and color as i32 values 
        //println!("Old color {}", rgba.data);
        self.flood_fill_scanline(x,y,color.data,rgba.data);  //use rgba and color as i32 values 
    }

    /*Recursive 4-way floodfill works with transparency but be aware of stack overflow !! 
    credits to http://lodev.org/cgtutor/floodfill.html
    Added stacker crate to mitigate overflow issue , does it work also for Redox?
    */
    fn flood_fill4(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32) {
        //stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        
        if x >= 0 && x < self.width()as i32 && y >= 0 && y < self.height() as i32 
            && self.pixcol(x,y).data == old_color && self.pixcol(x,y).data != new_color {
            
            self.pixel(x,y, Color{data:new_color});
            
            self.flood_fill4(x+1,y,new_color,old_color);
            self.flood_fill4(x-1,y,new_color,old_color);
            self.flood_fill4(x,y+1,new_color,old_color);
            self.flood_fill4(x,y-1,new_color,old_color);
        }
        //});
    }
    
    ///stack friendly and fast floodfill algorithm that works with transparency too 
    fn flood_fill_scanline( &mut self, x:i32, y:i32, new_color: u32, old_color:u32) {
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
        //println!("newcol {} pixcol {}",new_color, self.pixcol(x1,y).data);
        
        while x1 < w && self.pixcol(x1,y).data  == res_color  { 
        
            //println!("Test above {} {} ", self.pixcol(x1,y).data,old_color);
            if y > 0 && self.pixcol(x1,y-1).data  == old_color  {
              self.flood_fill_scanline(x1, y - 1, new_color, old_color);
            }
            x1 += 1;
          }
        x1 = x - 1;
        //println!("2) x1 {} y {} w {} ",x1,y,w);
        while x1 >= 0 && self.pixcol(x1,y).data == res_color {
            if y > 0 && self.pixcol(x1,y - 1).data  == old_color  {
              self.flood_fill_scanline(x1, y - 1, new_color, old_color);
            }
            x1 += -1;
          }
         
         //test for new scanlines below
        x1 = x;
        //println!("Test below ");
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

    fn flood_fill_line(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32) {
        if x >= 1 && x < self.width()as i32 && y >= 0 && y < self.height() as i32 
            && self.pixcol(x,y).data == old_color && self.pixcol(x,y).data != new_color {
           
           let mut x1=x; 
           loop {
                if x1>=0 && x1< self.width() as i32 && self.pixcol(x1,y).data == old_color{
                    self.pixel(x1,y, Color{data:new_color});
                    x1 +=1;
                } else {break}  
            }
                
            //self.flood_fill4(x+1,y,new_color,old_color);
           
           x1=x-1; 
           loop {
                if x1>=0 && x1< self.width() as i32 && self.pixcol(x1,y).data == old_color{
                    self.pixel(x1,y, Color{data:new_color});
                    x1 +=-1;
                } else {break}  
            }
                        
            //self.flood_fill4(x-1,y,new_color,old_color);
            

            
            self.flood_fill_line(x,y+1,new_color,old_color);
            
            self.flood_fill_line(x,y-1,new_color,old_color);
        }
    }

    ///crop new image from current image (copy) tranforming pure white into transparent
    fn copy_selection(&self, x: i32,y: i32,w: u32, h: u32) -> orbimage::Image {
       
        //let data = self.data();
        let mut vec = vec![];
        let mut col : Color;
        
        for y1 in y..y+h as i32 {
            for x1 in x..x+w as i32 {
                col=self.pixcol(x1,y1);
                if col.r()==255 && col.g()==255 && col.b()==255 {
                    col = Color::rgba(0,0,0,0);
                }
                vec.push(col);
            }
        }
        //println!("len {} w*h {}",vec.len(), w*h);
        orbimage::Image::from_data(w ,h ,vec.into_boxed_slice()).unwrap()
    }

    ///draws an image into current image starting at x,y (paste)
    fn paste_selection (&mut self, x: i32, y:i32,buffer: orbimage::Image){
        
        let w = buffer.width() as i32;
        let h = buffer.height() as i32;
        let data = buffer.into_data();
        let mut i:usize = 0;
        let x1:i32;
        let y1:i32;
        
        for y1 in y..y+h {
            for x1 in x..x+w {
                if i < data.len(){
                    self.pixel(x1,y1,data[i]);
                }
                i += 1;
            }
        }
    }

    /// interactive selection (rectangle)  
    fn select_rect(&mut self, x: i32 , y: i32, window: &mut orbtk::Window) ->Option<Rect> {
    
         //gets events from orbclient and render helping lines directly into orbclient window 
         let mut orbclient = window.inner.borrow_mut();
         let mut lx = 0;
         let mut ly = 0;
         let mut w = false;
        'events: loop{
            for event in orbclient.events() { 
                match event.to_option() {
                    EventOption::Key(key_event) => {println!("{:?}",key_event); break 'events},
                    EventOption::Quit(_quit_event) => break 'events,
                    EventOption::Scroll(scroll_event) => println!("Scroll not implemented yet..{:?}",scroll_event),
                    EventOption::Mouse(evt) => {
                                                if evt.y < CANVASOFFSET{
                                                    break 'events;
                                                    
                                                };
                                                if w {
                                                    orbclient.rect_marquee(x,
                                                    y+CANVASOFFSET,
                                                    lx,
                                                    ly+CANVASOFFSET,
                                                    orbtk::Color::rgba(100, 100, 100, 0));
                                                }
                                                w=true;
                                                
                                                orbclient.rect_marquee(x,
                                                y+CANVASOFFSET,
                                                evt.x,
                                                evt.y,
                                                orbtk::Color::rgba(100, 100, 100, 0));
                                                lx=evt.x;
                                                ly=evt.y-CANVASOFFSET;
                                                
                                                orbclient.sync();
                                                   
                                                },
                    EventOption::Button(btn) => {if btn.left {
                                                    let dx=lx-x;
                                                    let dy=ly-y;
                                                    return Some(Rect::new(x,y,dx as u32, dy as u32))
                                                    }
                                                
                                                if btn.right{
                                                        break 'events;
                                                        //TODO show menu with actions upon selection
                                                    }
                                                },
                    event_option => if cfg!(feature = "debug"){println!("{:?}", event_option)}
                                    else{ ()}
                }
          }
        }
      None  
    }


    /// draws interactive rectangle 
    fn interact_rect(&mut self, x: i32 , y: i32, color: Color,filled:bool, window: &mut orbtk::Window) {
    
         //gets events from orbclient and render helping lines directly into orbclient window 
         let mut orbclient = window.inner.borrow_mut();
         let mut lx = 0;
         let mut ly = 0;
         let mut w = false;
        'events: loop{
            for event in orbclient.events() { 
                match event.to_option() {
                    EventOption::Key(key_event) => {println!("{:?}",key_event); break 'events;},
                    EventOption::Quit(_quit_event) => break 'events,
                    EventOption::Scroll(scroll_event) => println!("Scroll not implemented yet..{:?}",scroll_event),
                    EventOption::Mouse(evt) => {
                                                if evt.y < CANVASOFFSET{
                                                    break 'events
                                                };
                                                if w {
                                                    orbclient.rect_marquee(x,
                                                    y+CANVASOFFSET,
                                                    lx,
                                                    ly+CANVASOFFSET,
                                                    orbtk::Color::rgba(100, 100, 100, 0));
                                                }
                                                w=true;
                                                
                                                orbclient.rect_marquee(x,
                                                y+CANVASOFFSET,
                                                evt.x,
                                                evt.y,
                                                orbtk::Color::rgba(100, 100, 100, 0));
                                                lx=evt.x;
                                                ly=evt.y-CANVASOFFSET;
                                                
                                                orbclient.sync();
                                                   
                                                },
                    EventOption::Button(btn) => {if btn.left {
                                                    if filled {
                                                        let dx=lx-x;
                                                        let dy=ly-y;
                                                        if dx >0 && dy>0 {
                                                            self.rect(x ,y,dx as u32, dy as u32 ,color);
                                                        }
                                                        if dx<0 && dy > 0 {
                                                            self.rect(x+dx ,y ,-dx as u32, dy as u32, color);
                                                        }
                                                        if dx<0 && dy <0 {
                                                            self.rect(x+dx ,y+dy ,-dx as u32, -dy as u32, color);
                                                        }
                                                        if dx>0 && dy <0 {
                                                            self.rect(x ,y+dy ,dx as u32, (-dy) as u32, color);
                                                        }
                                                        break 'events
                                                    } else {
                                                        self.line(x,y,lx,y,color);
                                                        self.line(lx,y,lx,ly,color);
                                                        self.line(lx,ly,x,ly,color);
                                                        self.line(x,ly,x,y,color);
                                                        break 'events
                                                    }
                                                }
                                                if btn.right{
                                                        break 'events
                                                    }
                                                },
                    event_option => if cfg!(feature = "debug"){println!("{:?}", event_option)}
                                    else{ ()}
                }
          }
        }
        
    }
    
    /// draws interactive polyline 
    fn interact_line(&mut self, x: i32 , y: i32, color: Color, width: i32, window: &mut orbtk::Window) {
    
         //gets events from orbclient and render helping lines directly into orbclient window 
         let mut orbclient = window.inner.borrow_mut();
         let mut lx =0;
         let mut ly =0;
         let mut ox = x;
         let mut oy = y;
         let mut w = false;
        'events: loop{
            for event in orbclient.events() { 
                match event.to_option() {
                    EventOption::Key(key_event) => break 'events,
                    EventOption::Quit(_quit_event) => break 'events,
                    EventOption::Scroll(scroll_event) => println!("Scroll not implemented yet.."),
                    EventOption::Mouse(evt) => {
                                                if evt.y < CANVASOFFSET{
                                                    break 'events
                                                };
                                                if w {
                                                    orbclient.ant_line(ox,
                                                    oy+CANVASOFFSET,
                                                    lx,
                                                    ly+CANVASOFFSET,
                                                    orbtk::Color::rgba(100, 100, 100, 0));//alfa has to be 0 for trick to work
                                                }
                                                w=true;
                                                lx=evt.x;
                                                ly=evt.y-CANVASOFFSET;
                                                 
                                                orbclient.ant_line(ox,
                                                oy+CANVASOFFSET,
                                                evt.x,
                                                evt.y,
                                                orbtk::Color::rgba(100, 100, 100, 0));//alfa has to be 0 for trick to work
                                                
                                                orbclient.sync();
                                                
                                                     
                                                },
                    EventOption::Button(btn) => {
                                                    if btn.left {
                                                        //quick and dirty workaround to trace thick lines
                                                        //TODO implement new line method to deal with thickness properly
                                                        for d in 0..width {
                                                            self.line(ox+d ,oy,lx+d, ly ,color); //update image
                                                            orbclient.line(ox+d ,oy+CANVASOFFSET,lx+d, ly+CANVASOFFSET ,color); //update preview 
                                                        }
                                                        orbclient.sync();
                                                        ox=lx;
                                                        oy=ly;
                                                        w =false;
                                                    }
                                                    if btn.right{
                                                        break 'events
                                                    }
                                                },
                    event_option => if cfg!(feature = "debug"){println!("{:?}", event_option)}
                                    else{ ()}
                }
          }
        }
        
    }
    
}


trait AddOnsToOrbclient {
    fn pixcol(&self, x:i32, y:i32) -> Color;
    fn ant_line(&mut self, argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color);
    fn rect_marquee(&mut self , argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color);
}
impl AddOnsToOrbclient for orbclient::Window{
    
    ///gets pixel Color at x,y
    fn pixcol(&self, x:i32, y:i32) -> Color {
        let p = self.width()as i32 * y + x;
        let rgba = self.data()[p as usize];
        rgba
    }
    
    /// Draws ant_line - - -   
    fn ant_line(&mut self, argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color) {
        let mut x = argx1;
        let mut y = argy1;
                
        let dx = if argx1 > argx2 { argx1 - argx2 } else { argx2 - argx1 };
        let dy = if argy1 > argy2 { argy1 - argy2 } else { argy2 - argy1 };

        let sx = if argx1 < argx2 { 1 } else { -1 };
        let sy = if argy1 < argy2 { 1 } else { -1 };

        let mut err = if dx > dy { dx } else {-dy} / 2;
        let mut err_tolerance;

        let mut old_color : orbtk::Color ;
        
        let mut ct = 0;

        loop {
            if ct == 0 {
            old_color = self.pixcol(x,y);
            // rgb bitwise xor between old and new pixel color
            // New faster implementation xor-ing 32 bit internal color data   
            // Attention :trick does not work as intended xor-ing entire 32bit color data, if new color alfa > 0!!
            self.pixel(x,y,Color{data: (&old_color.data ^ &color.data)}); 
            
            }
            
            if x == argx2 && y == argy2 { break };

            err_tolerance = 2 * err;

            if err_tolerance > -dx { err -= dy; x += sx; }
            if err_tolerance < dy { err += dx; y += sy; }
            
            if ct<3 {ct += 1;}  
            else {ct = 0;}            
        }
        //self.sync();
        
    }
    
    ///draws rectangular selection marquee
    fn rect_marquee(&mut self , argx1: i32, argy1: i32, argx2: i32, argy2: i32, color: Color) {
        self.ant_line(argx1,argy1,argx2,argy1,color);
        self.ant_line(argx2,argy1,argx2,argy2,color);
        self.ant_line(argx2,argy2,argx1,argy2,color);
        self.ant_line(argx1,argy2,argx1,argy1,color);
        //self.sync();
    }
        
}

