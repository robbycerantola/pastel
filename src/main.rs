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
            Toolbar, ToolbarIcon, Rect, Separator,
             Window, Renderer, ColorSwatch};  //TextBox,ControlKnob,InnerWindow,
use orbtk::dialogs::FileDialog;
use orbtk::traits::{Click, Place, Text};  //Border, Enter
use orbtk::cell::CloneCell;

//use orbclient::EventOption;

use std::rc::Rc;
use std::cell::{Cell, RefCell}; //, RefMut
use std::sync::Arc;
use std::process;
use std::process::Command;
use std::env;
use std::collections::HashMap;
use std::path::{Path,PathBuf};
use std::fs;

mod dialogs;
use dialogs::{dialog,popup,new_dialog};

mod palette;
use palette::Palette;

mod addons;
use addons::AddOnsToOrbimage;

mod canvas;
use canvas::Canvas;

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

#[derive(Clone)]
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
                home_dir.push_str("/");
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
    canvas.undo_save();
    size.x = canvas.rect.get().width;
    size.y = canvas.rect.get().height;

    //Tools and properties 
    //create new tool with some properties and initial values
    let mut ntools = HashMap::new();
    ntools.insert("pen",vec![Property::new("Size",1),Property::new("Opacity",100)]);
    ntools.insert("line",vec![Property::new("Opacity",100)]);
    ntools.insert("polyline",vec![Property::new("Size",1),Property::new("Opacity",100)]); 
    ntools.insert("brush",vec![Property::new("Size",4),Property::new("Opacity",100),Property::new("Shape",0)]); //
    ntools.insert("fill",vec![Property::new("Opacity",100)]);
    ntools.insert("rectangle",vec![Property::new("Opacity",100),Property::new("Filled",1)]);
    ntools.insert("circle",vec![Property::new("Opacity",100),Property::new("Filled",0)]);

    //use invisible Label for storing current active tool
    let tool = Label::new();
    tool.text("pen");
    
    //define current selection
    let selection :  Rc<RefCell<Option<Rect>>> = Rc::new(RefCell::new(Some(Rect::new(0,0,size.x,size.y))));  //Rect::new(0,0,0,0);
    
    
    //if pastel_copy_buffer.png exists load it into canvas copy_buffer
    //for copy/paste between instances 
    //let buffer: Rc<RefCell<orbimage::Image>> = Rc::new(RefCell::new(load_buffer("/tmp/pastel_copy_buffer.png")));
    *canvas.copy_buffer.borrow_mut() = load_buffer("/tmp/pastel_copy_buffer.png");
    
    //implement GUI
    
    let mut x = 10;
    let mut y = 56;
    
    let title = format!("Pastel: {}", filename);
    //resizable main window
    let mut window = Window::new_flags(Rect::new(100, 100, 1024, 718),
                                       &title.to_owned(),
                                       &[orbclient::WindowFlag::Resizable ]);
    
        
    /*
    //2nd method to open a new window
    let win : Rc<RefCell<Window>> = Rc::new(RefCell::new(Window::new_flags(
                                            Rect::new(1134,100,400,200),
                                            "Palette",
                                            &[orbclient::WindowFlag::Resizable ])));
    */
    
    
    /* TESTING floating window
    //3rd method to open a new window
    let mut orb_window = Some(InnerWindow::new(1130, 100, 300, 200, "Test floating window").unwrap());
    let mut win = Box::new(Window::from_inner(orb_window.take().unwrap()));
    */
    

    // current color swatch 
    let swatch = ColorSwatch::new();
    swatch.position(320,56).size(24,35);
    swatch.color(orbtk::Color::rgb(0,0,0));
    window.add(&swatch);
    let swatch_clone=swatch.clone();

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

    // create a new palette at x,y,width,height linked to swatch 
    let palette=Palette::new(10,120,window.width(),50,swatch_clone,red_bar,green_bar,blue_bar );
    // show on window the standard palette
    palette.prepare(&window);
    
    /* TESTING floating window
    //draw something on 2nd window
    palette.draw(& win);
    win.draw_if_needed();

    // test possibilities to add a swatch to palette
    {
    
    // add new color to palette on window by reference 
    let window_clone = &mut window as *mut Window;
    unsafe{palette.clone().add(Color::rgb(200,100,50),&mut *window_clone);}//here works but not inside a closure !!
    unsafe{palette.clone().add(Color::rgb(100,200,150),&mut *window_clone);}
    }
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
    
    //button for adding current color to custom palette
    let add_button = Button::new();
    let swatch_clone = swatch.clone();
    let palette_clone = palette.clone();
    

    add_button.position(320,93)
        .size(24, 16)
        .text("+")
        .text_offset(8, 0)
        .on_click(move |_button: &Button, _point: Point| {
            if cfg!(feature = "debug"){println!("Add custom color to palette");}
            palette_clone.change(palette_clone.next(),swatch_clone.read());
        });
    window.add(&add_button);
    
    
    //manually implement toolbar object (old fashion...)
    // implement toolbars by multiple clickable images loaded in widget ToolbarIcon  
    let mut toolbar_obj = vec![];    //here we save all Toolbar widgets clones so we can manage 'selected' property
    let mut toolbar2_obj = vec![];   //create Toolbar2 here so we can manage 'selected','visible' properties from Toolbar
    
    //use new Toolbar widget to implement 3rd Toolbar
    let parent_window = &mut window as *mut Window;  //we need a pointer to the parent window to add the icons to
    let mut toolbar3 = Toolbar::new();
    
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
                               let o = property_get(&ntools_clone["rectangle"],"Opacity").unwrap();
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

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("hollow_circle.png") {
        Ok(image) => {
            image.position(x, y)                
                 .text("Draws circles".to_owned());
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
                               tool_clone.text.set("circle".to_owned());
                               
                               //get previous settings
                               size_bar_clone.visible.set(false);
                               size_label_clone.visible.set(false);
                               let o = property_get(&ntools_clone["circle"],"Opacity").unwrap();
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

    match ToolbarIcon::from_path("smooth_circle.png") {
        Ok(item) => {
            let ntools_clone = ntools.clone();
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<ToolbarIcon>>;
            item.position(x, y)
                 .text("Smooth edges circular shape".to_owned())
                 .on_click(move |_image: &ToolbarIcon, _point: Point| {
                               property_set(&ntools_clone["brush"],"Shape",3);
                               
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
                 .text("custom brush from buffer".to_owned())
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
                               property_set(&ntools_clone["circle"],"Filled",0);
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
                               property_set(&ntools_clone["circle"],"Filled",1);
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

    let menufile = Menu::new("File");
    menufile.position(10, 0).size(32, 16);

    //menu entries for file
    {
        let action = Action::new("New");
        action.on_click(move |_action: &Action, _point: Point| {
                           match new_dialog(&"New file".to_owned()) { 
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

        menufile.add(&action);
    }

    {
        let action = Action::new("Open");
        let home_dir_clone = home_dir.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            //match dialog("Open", "path:",&home_dir_clone[..]) {
              let mut f = FileDialog::new();
                f.path=PathBuf::from(home_dir_clone.to_owned());
              match f.exec() {
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
        menufile.add(&action);
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
        menufile.add(&action);
    }

    {
        let action = Action::new("Save As");
        let canvas_clone = canvas.clone();
        let home_dir_clone = home_dir.clone();
        //FIXME change filename after a SaveAs 
        action.on_click(move |_action: &Action, _point: Point| {
                            match dialog("Save As", "path:",&home_dir_clone[..]) {
                            Some(response) => {
                                match canvas_clone.save(&(String::from(response))){
                                    Ok(_) => (),
                                    Err(e) => popup("Error",&format!("{}",e)[..]),
                                }
                                
                                },
                            None => {println!("Cancelled");},
                            }
                        });
        menufile.add(&action);
    }

    menufile.add(&Separator::new());

    {
        let action = Action::new("Exit");
        action.on_click(move |_action: &Action, _point: Point| {
                            println!("Bye bye...");
                            process::exit(0x0f00);
                        });
        menufile.add(&action);
    }

    //Menu edit
    let menuedit = Menu::new("Edit");
        menuedit.position(50, 0).size(32, 16);

    //Menu entries for edit
    
    {
        let action = Action::new("Undo");
        let canvas_clone = canvas.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.undo();
                          });
        menuedit.add(&action);
    }

    menuedit.add(&Separator::new());

    {
        let action = Action::new("Select");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        tool_clone.text.set("marquee".to_owned());
                          });
        menuedit.add(&action);
    }

    {
        let action = Action::new("Clear selection");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        let size_clone = size.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        
                        println!("{:?}",selection_clone);
                        
                        /*match selection_clone.borrow() {
                            Some(_) => {
                                        canvas_clone.undo();
                                        *selection_clone.borrow_mut()=None;
                                        },
                            None    => (),
                        }*/
                        
                        *selection_clone.borrow_mut()=Some(Rect{x:0, y:0, width:size_clone.x, height: size_clone.y});
                        
                        });
        menuedit.add(&action);
    }

    menuedit.add(&Separator::new());

    {
        let action = Action::new("Copy");
        let tool_clone = tool.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        tool_clone.text.set("copy".to_owned());
                        
                          });
        menuedit.add(&action);
    }
    
    {
        let action = Action::new("Paste");
        let tool_clone = tool.clone();
        let ntools_clone = ntools.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        property_set(&ntools_clone["brush"],"Shape",2);
                        tool_clone.text.set("brush".to_owned());
                          });
        menuedit.add(&action);
    }

    menuedit.add(&Separator::new());
    
    {
        let action = Action::new("Load Buffer");
        let home_dir_clone = home_dir.clone();
        //let buffer_clone = buffer.clone();
        let canvas_clone = canvas.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        let mut f= FileDialog::new();
                        f.title="Load Buffer from file".to_owned();
                        f.path=PathBuf::from(home_dir_clone.to_owned());
                        match f.exec() {
                        Some(response) => {
                            let bf = load_buffer(&(response.display().to_string())[..]);
                            //*buffer_clone.borrow_mut() = bf; 
                            *canvas_clone.copy_buffer.borrow_mut() = bf;
                            },
                        None => println!("Cancelled"),
                        }
                          });
        menuedit.add(&action);
    }
    
    
    {
        let action = Action::new("Save Buffer");
        let home_dir_clone = home_dir.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        match dialog("Save Buffer", "path:",&home_dir_clone[..]) {
                            Some(response) => {
                                if let Ok(_) = fs::copy("/tmp/pastel_copy_buffer.png",&(response.to_string())[..] ) {}
                            },
                        
                            None => {println!("Cancelled");},
                        }
                        });
        menuedit.add(&action);
    }
    

    //Menu tool
    let menutools = Menu::new("Tools");
    menutools.position(90, 0).size(48, 16);

    //Menu entries for tools
    {
        let action = Action::new("Pen");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("pen".to_owned());
                        });
        menutools.add(&action);
    }

    {
        let action = Action::new("Line");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {

                            tool_clone.text.set("line".to_owned());
                        });
        menutools.add(&action);
    }

    {
        let action = Action::new("Polyline");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {

                            tool_clone.text.set("polyline".to_owned());
                        });
        menutools.add(&action);
    }

    {
        let action = Action::new("Brush");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("brush".to_owned());
                        });
        menutools.add(&action);
    }
    
    {
        let action = Action::new("Fill");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("fill".to_owned());
                        });
        menutools.add(&action);
    }
    
    {
        let action = Action::new("Rectangle");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("rectangle".to_owned());
                        });
        menutools.add(&action);
    }
    
    {
        let action = Action::new("Circle");
        let tool_clone = tool.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tool_clone.text.set("circle".to_owned());
                        });
        menutools.add(&action);
    }
    

    //Menu image
    let menuimage = Menu::new("Image");
    menuimage.position (140,0).size (48,16);
    
    //Menu entries for image
    
    {
        let action = Action::new("Blur");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        //canvas_clone.transformation("blur",0,0);
                        canvas_clone.trans_selection(selection_clone.borrow().unwrap(),"blur",0,0);
                    });
        menuimage.add(&action);
    }
    
    {
        let action = Action::new("Unsharpen");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow().unwrap(),"unsharpen",0,0);
                    });
        menuimage.add(&action);
    }
    
    {
        let action = Action::new("Verical flip");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow().unwrap(),"flip_vertical",0,0);
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Horizontal flip");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow().unwrap(),"flip_horizontal",0,0);
                    });
        menuimage.add(&action);
    }
    
    {
        let action = Action::new("Rotate 90");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow().unwrap(),"rotate90",0,0);
                    });
        menuimage.add(&action);
    }
    
    {
        let action = Action::new("Brighten");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow().unwrap(),"brighten",0,0);
                    });
        menuimage.add(&action);
    }
    
    {
        let action = Action::new("Darken");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow().unwrap(),"darken",0,0);
                    });
        menuimage.add(&action);
    }
    
    menuimage.add(&Separator::new());

    {
        let action = Action::new("Grayscale");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.transformation("grayscale",0,0);
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Resize");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        
                        match new_dialog(&"Resize".to_owned()) { 
                                Some(resolution) => {
                                    let val: Vec<&str> = resolution.split("x").collect();
                                    let x: i32 = val[0].parse().unwrap_or(640);
                                    let y: i32 = val[1].parse().unwrap_or(480);
                                    canvas_clone.transformation("resize",x,y);
                                                },
                                    None => println!("Resize cancelled"),
                                }
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Clear");
        let canvas_clone = canvas.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.clear();
                    });
        menuimage.add(&action);
    }

    //Menu palette
    let menupalette = Menu::new("Palette");
        menupalette.position (190, 0).size(64, 16);

    //Menu entries for palette
    {
        let action = Action::new("Load");
        let home_dir_clone = home_dir.clone();
        let palette_clone = palette.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            //match dialog("Open", "path:",&home_dir_clone[..]) {
                            let mut f= FileDialog::new();
                            f.title="Load palette".to_owned();
                            f.path=PathBuf::from(home_dir_clone.to_owned());
                            match f.exec() {
                            Some(response) => {
                                    println!("Loaded palette {:?} ", response);
                                    match palette_clone.load(&response){
                                        Ok(_) =>(),
                                        Err(e) => popup("Error",&format!("{}",e)[..]),
                                        }
                                },
                            None => println!("Cancelled"),
                            }
        });
        menupalette.add(&action);
    }    

    {
        let action = Action::new("Save");
        let palette_clone=palette.clone();
        let home_dir_clone = home_dir.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            match dialog("Save palette", "path:",&home_dir_clone[..]) {
                            Some(response) => {
                                match palette_clone.save(&(String::from(response))){
                                    Ok(_) => (),
                                    Err(e) => popup("Error",&format!("{}",e)[..]),
                                }
                                
                                },
                            None => {println!("Cancelled");},
                            }
                        });
        menupalette.add(&action);
    }
    menupalette.add(&Separator::new());

    {
        let action = Action::new("Add swatch");
        let swatch_clone = swatch.clone();
        let palette_clone = palette.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        palette_clone.change(palette_clone.next(),swatch_clone.read());
                        if cfg!(feature = "debug"){println!("{:?}, {:?}",swatch_clone.read(), palette_clone.swatches.borrow());}
                          });
        menupalette.add(&action);
    }
    menupalette.add(&Separator::new());

     {
        let action = Action::new("Reset");
        let palette_clone = palette.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        palette_clone.reset();
                          });
        menupalette.add(&action);
    }

    //Menu help

    let menuhelp = Menu::new("Help");
    menuhelp.position(260, 0).size(32, 16);

    //menu entries for help

    {
        let action = Action::new("Info");
        action.on_click(move |_action: &Action, _point: Point| {
                            popup("Info",
                                  "Pastel v0.0.20, simple bitmap editor \n for Redox OS by Robby Cerantola");
                        });
        menuhelp.add(&action);
    }

    // add menus
    window.add(&menufile);
    window.add(&menuedit);
    window.add(&menutools);
    window.add(&menuimage);
    window.add(&menupalette);
    window.add(&menuhelp);

    // paint on canvas
    let click_pos: Rc<RefCell<Option<Point>>> = Rc::new(RefCell::new(None));
    let window_clone = &mut window as *mut Window;
    let click_pos_clone = click_pos.clone();
    //let mut selection_clone = selection.clone();
    canvas
        .position(0, CANVASOFFSET) 
        .on_right_click(move |_ , point:Point|{
                if cfg!(feature = "debug"){
                    println!("Right click not implemented yet");
                }
                })
        .on_clear_click(move |_ , point:Point|{
            // clears last cursor position 
                let mut ck=click_pos_clone.borrow_mut();
                *ck = None;
                })
        .on_click(move |canvas: &Canvas, point: Point| {

            let click = click_pos.clone();
            let size = size_bar.clone().value.get();
            //let buffer_clone = buffer.clone();
            let swatch_clone = swatch.clone();
            let u = tool.clone().text.get();
            let mut selection_clone = selection.clone();
            {
                let mut prev_opt = click.borrow_mut();
                //let mut bf = buffer_clone.borrow_mut();
                //let r = (red_bar.clone().value.get() as f32 * 2.55) as u8;
                //let g = (green_bar.clone().value.get() as f32 * 2.55) as u8;
                //let b = (blue_bar.clone().value.get() as f32 * 2.55) as u8;
                let a = (trans_bar.clone().value.get() as f32 * 2.55) as u8;
                let swc = swatch_clone.read();
                let color = Color::rgba(swc.r(),swc.g(),swc.b(),a);
                
                //tools that dont need prev_position
                match tool.clone().text.get().as_ref() {
                    
                    "pen"  => canvas.image.borrow_mut().pixel(point.x, point.y, color),
                    "brush"=> {
                                match property_get(&ntools.clone()["brush"],"Shape") {
                                    Some(0) => canvas.image.borrow_mut().circle(point.x, point.y,-size,
                                                    color),
                                    Some(1) => canvas.image.borrow_mut().rect(point.x ,point.y,size as u32, size as u32,
                                                    color),
                                    Some(2) => //canvas.image.borrow_mut().paste_selection(point.x,point.y,
                                               //     a.clone(), bf.clone()),
                                               canvas.paste_buffer(point.x,point.y,
                                                    a.clone()),
                                    Some(3) => canvas.image.borrow_mut().smooth_circle(point.x,point.y,
                                                    size as u32, color),
                             None | Some(_) => println!("no Shape match!"),
                                    }
                                },
                    "fill" => canvas.fill(point.x, point.y,color),
               "rectangle" => {                   
                                canvas.undo_save();
                                let filled = property_get(&ntools.clone()["rectangle"],"Filled").unwrap();
                                unsafe{
                                    canvas.image.borrow_mut().interact_rect(point.x,
                                                    point.y,
                                                    color,
                                                    filled == 1,
                                                    &mut *window_clone
                                                    );
                                }
                               },
                "polyline" => {canvas.undo_save();
                                let width = property_get(&ntools.clone()["polyline"],"Size").unwrap();
                                unsafe{
                                        canvas.image.borrow_mut().interact_line(point.x,
                                                    point.y,
                                                    color,
                                                    width,
                                                    &mut *window_clone
                                                    );
                                    }   
                                },
                    "copy" =>  {
                                    let mut image = canvas.image.borrow_mut();
                                    if let Some(selection) = unsafe { 
                                                                image.select_rect(point.x,
                                                                    point.y,&mut *window_clone)
                                                             } {
                                         //*bf = image.copy_selection(selection.x,selection.y,selection.width,selection.height);
                                         *canvas.copy_buffer.borrow_mut() = image.copy_selection(selection.x,selection.y,selection.width,selection.height);
                                         //save buffer to disk as pastel_copy_buffer.png so we can reload when starting new program instance
                                         let newcanvas= Canvas::from_image(canvas.copy_buffer.borrow().clone());
                                         let path = "/tmp/pastel_copy_buffer.png".to_string();
                                         if let Ok(_) = newcanvas.save(&path){}
                                    }
                                },
                   "marquee"=> {    canvas.undo_save();
                                    let mut image = canvas.image.borrow_mut();
                                    if let Some(selection) = unsafe{image.select_rect(point.x,
                                                    point.y,&mut *window_clone)}
                                        {
                                                    *selection_clone.borrow_mut()= Some(selection);
                                                    //image.rect(selection.x, selection.y, selection.width, selection.height, orbtk::Color::rgba(100, 000, 000, 100));
                                                    /*let image_selection = image.copy_selection(selection.x, selection.y, selection.width, selection.height);
                                                    let new_image = canvas.trans_image(image_selection, "blur",0,0);
                                                    //draw slice into canvas at position x y 
                                                    image.image(selection.x, selection.y, selection.width, selection.height, &new_image[..]);
                                                    */
                                         }
                                    },
                    "paste" => //canvas.paste_selection(point.x,point.y, a.clone(), bf.clone()),
                                canvas.paste_buffer(point.x,point.y, a.clone()),
                    "circle" => {
                                    canvas.undo_save();
                                    let filled = property_get(&ntools.clone()["circle"],"Filled").unwrap();
                                    unsafe{
                                        canvas.image.borrow_mut().interact_circle(point.x,
                                                    point.y,
                                                    color,
                                                    filled == 1,
                                                    &mut *window_clone
                                                    );
                                    }
                                },
                    
                           _ => (),
                    }
                
                //tools that need prev_position to work
                if let Some(prev_position) = *prev_opt {
                    match tool.clone().text.get().as_ref() {
                        "line" => {
                                    canvas.image.borrow_mut().line(prev_position.x,
                                                prev_position.y,
                                                point.x,
                                                point.y,
                                                color);
                                   },
                              _ => (),          
                    }
                    *prev_opt = Some(point);     
                } else {
                    *prev_opt = Some(point);
                    if u == "line" || u =="pen" || u =="brush" {canvas.undo_save();} //prepare for undo
                }
            }
        });
        
    window.add(&canvas);
    window.exec();
}

//Helper functions

///Load an image from path if exists, otherwise create new empty canvas
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
fn load_buffer(path: &str) -> orbimage::Image {
    
    //let path="/tmp/pastel_copy_buffer.png".to_string();
    
    if cfg!(feature = "debug"){print!("Loading copy buffer from:  {} .....", path);}
    match orbimage::Image::from_path(path.to_string()) {
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
    
///set visibility for all toolbar items
fn visible_toolbar (toolbar_obj: &mut Vec<Arc<ToolbarIcon>>, v: bool) {
    for i in 0..toolbar_obj.len(){
        if let Some(toolbar) = toolbar_obj.get(i) {
            toolbar.visible.set(v);
        }
    }
}

