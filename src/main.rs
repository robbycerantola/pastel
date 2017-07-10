/*Pastel by Robby 21-05-2017
simple image editor in Rust for Redox
*/
extern crate orbtk;
extern crate orbimage;
//extern crate image;
extern crate orbclient;

use orbtk::{Color, Action, Button, Image, Label, Menu, Point, ProgressBar,
            ControlKnob, Toolbar, Rect, Separator,
            TextBox, Window, Renderer};
use orbtk::traits::{Click, Enter, Place, Text};  //Border
//use orbtk::event::Event;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use orbtk::cell::CloneCell;
use std::sync::Arc;
use std::process;
use std::process::Command;
//use std::path::Path;
use std::env;
use std::collections::HashMap;
//use orbclient::EventOption;

mod dialogs;
use dialogs::{dialog,popup};

mod canvas;
use canvas::{Canvas};

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

/*
#[allow(dead_code)]
struct Settings {
    description: CloneCell<String>,     //tool's long description to be used in help popup
    size: Cell<i32>,
    transparency: Cell<i32>,                    
    //property: Vec<Cell<&Property>>,
    selected: Cell<bool>,
}

impl Settings {
    fn new() -> Arc<Self> {
        Arc::new(Settings {
            description: CloneCell::new(String::new()),
            size: Cell::new(0),
            transparency: Cell::new(0),
            //property: Cell::new(Property::new("lucentezza")),
            selected: Cell::new(false),
            
        })
    }
    fn description<S: Into<String>>(&self, text: S) -> &Self {
        self.description.set(text.into());
        self
    }
    fn size(&self, size: i32) -> &Self {
        self.size.set(size);
        self
    }
    fn transparency(&self, transparency: i32) -> &Self {
        self.transparency.set(transparency);
        self
    }
}
*/

struct MySize {
    x: u32,
    y: u32,
}

fn main() {

    let mut size = MySize{x: 1024, y:500};    
    let mut x = 10;
    let mut y = 56;

    let filename;

    //deal with comand line arguments
    let args: Vec<String> = env::args().collect();
    
    //only name given
    if args.len() > 1 {

        filename = args[1].clone();
    } else {
        filename = String::from("test.png");  //no name
    }
    
    //size given
    if args.len() > 2 {
       let k: Vec<_> = args[2].split("x").collect();
       size.x = (*k[0]).parse().unwrap() ;
       size.y = (*k[1]).parse().unwrap() ;
    }

    //load canvas from existing file or create new one with filename size
    let canvas = load_image(&filename, &size);


    //Tools and properties for tools
    //create new tool with some properties and initial values
    let mut ntools = HashMap::new();
    ntools.insert("pen",vec![Property::new("Size",1),Property::new("Opacity",100)]);
    ntools.insert("line",vec![Property::new("Opacity",100)]); 
    ntools.insert("brush",vec![Property::new("Size",4),Property::new("Opacity",100),Property::new("Shape",0)]);
    ntools.insert("fill",vec![Property::new("Opacity",100)]);
    ntools.insert("rectangle",vec![Property::new("Opacity",100)]);
    ntools.insert("circle",vec![Property::new("Opacity",100)]);

    //use invisible Label for storing current active tool
    let tool = Label::new();
    tool.text("pen");


    //implement GUI

    //resizable main window
    let mut window = Window::new_flags(Rect::new(100, 100, 1024, 718),
                                       "Pastel",
                                       &[orbclient::WindowFlag::Resizable ]);

    // color swatch
    let swatch = Label::new();
    swatch.text("■■").position(320,80).size(56,16);
    //swatch.fg.set(orbtk::Color::rgb(r,g,b));
    window.add(&swatch);

    // use forked version of orbtk to get ProgressBar rendered in colors setting fg
    //color picker
    let red_bar = ProgressBar::new();
    let green_bar = ProgressBar::new();
    let blue_bar = ProgressBar::new();
    let red_label = Label::new();
    red_label.text("R: 0").position(x, y).size(48, 16);
    red_label.fg.set(orbtk::Color::rgb(255,0,0));
    window.add(&red_label);
    
    red_bar.fg.set(orbtk::Color::rgb(255,0,0));  
    let swatch_clone_r = swatch.clone();
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
                      swatch_clone_r.fg.set(orbtk::Color::rgb(r,g,b));
                  });
    window.add(&red_bar);

    y += red_bar.rect.get().height as i32 + 2;

    let green_label = Label::new();
    green_label.text("G: 0").position(x, y).size(48, 16);
    green_label.fg.set(orbtk::Color::rgb(0,255,0));
    window.add(&green_label);
    
    green_bar.fg.set(orbtk::Color::rgb(0,255,0));
    let swatch_clone_g = swatch.clone();
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
                      swatch_clone_g.fg.set(orbtk::Color::rgb(r,g,b));
                  });
    window.add(&green_bar);

    y += green_bar.rect.get().height as i32 + 2;


    let blue_label = Label::new();
    blue_label.text("B: 0").position(x, y).size(48, 16);
    blue_label.fg.set(orbtk::Color::rgb(0,0,255));
    window.add(&blue_label);
    
    blue_bar.fg.set(orbtk::Color::rgb(0,0,255));
    let swatch_clone_b = swatch.clone();
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
                      swatch_clone_b.fg.set(orbtk::Color::rgb(r,g,b));
                      
                  });
    window.add(&blue_bar);

    y += blue_bar.rect.get().height as i32 + 10;
    
    // tool size bar
    let size_label = Label::new();
    size_label.text("Size: 1").position(x+380, 56).size(64, 16);
    size_label.visible.set(false);
    //blue_label.fg.set(orbtk::Color::rgb(0,0,255));
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
                      let a: &str = &cur_tool[..];  //FIXME workarround to convert String into &str                      
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
                      let a: &str = &cur_tool[..];  //FIXME workarround to convert String into &str                      
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
                      //let a: &str = &cur_tool[..];  //FIXME workarround to convert String into &str                      
                      //tools_clone[a].size(progress);
                      
                  });
    window.add(&volume);
*/

    //clickable icon
    match Image::from_path("res/pastel100.png") {
        Ok(image) => {
            image.position(900, 10);
            image.on_click(move |_image: &Image, _point: Point| {
                               popup("Ciao",
                                  "Pastel is work in progress....");
                           });
            window.add(&image);
        }
        Err(err) => {
            let label = Label::new();
            label.position(x, y).size(400, 16).text(err);
            window.add(&label);
        }
    }
    
    // implement multiple toolbars by multiple clickable images loaded in widget Toolbar 
    let mut toolbar_obj = vec![];   //here save all Toolbar widgets clones so we can manage 'selected' property
    let mut toolbar2_obj = vec![];   //here save Toolbar widgets clones so we can manage 'selected','visible' properties
    let y = 25;
    match Toolbar::from_path("res/pencil1.png") {
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
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<Toolbar>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<Toolbar>>;
            image.on_click(move |_image: &Toolbar, _point: Point| {
                               tool_clone.text.set("pen".to_owned());

                               size_bar_clone.visible.set(false);
                               size_label_clone.visible.set(false);

                               let o = property_get(&ntools_clone["pen"],"Opacity").unwrap();
                               trans_bar_clone.value.set(o);
                               trans_label_clone.text(format!("Opacity: {}%",o));
                               //toggle tool in toolbar
                               unsafe {toggle_toolbar(&mut *toolbar_obj_clone);}
                               //make invisible toolbar2
                               unsafe{visible_toolbar(&mut *toolbar2_obj_clone,false);}
                               //TODO clear window area reserved for tools properties
                               //    draw widgets for tool properties
                               //unsafe{prop_area(&ntools_clone["pen"],&mut *window_clone, 11);}
                           });
            
            window.add(&image);
            toolbar_obj.push(image.clone());

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match Toolbar::from_path("res/pencil2.png") {
        Ok(image) => {
            image.position(x, y)                
                 .text("Draw lines".to_owned());
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let ntools_clone = ntools.clone();
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<Toolbar>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<Toolbar>>;
            image.on_click(move |_image: &Toolbar, _point: Point| {
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
                               });
            window.add(&image);
            toolbar_obj.push(image.clone());

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match Toolbar::from_path("res/brush.png") {
        Ok(image) => {
            image.position(x, y)
                 .text("Paint brush".to_owned());
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let ntools_clone = ntools.clone();
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<Toolbar>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<Toolbar>>;
            image.on_click(move |_image: &Toolbar, _point: Point| {
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
                               });
            window.add(&image);
            toolbar_obj.push(image.clone());

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match Toolbar::from_path("res/fillbucket.png") {
        Ok(item) => {
            
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let ntools_clone = ntools.clone();
            let toolbar_obj_clone = &mut toolbar_obj as *mut Vec<Arc<Toolbar>>;
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<Toolbar>>;
            item.position(x, y)
                 .text("Fill up area with color".to_owned())
                 .on_click(move |_image: &Toolbar, _point: Point| {
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
                               });
            window.add(&item);
            toolbar_obj.push(item.clone());
            
            x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }


//2nd toolbar

    match Toolbar::from_path("res/circle.png") {
        Ok(item) => {
            let ntools_clone = ntools.clone();
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<Toolbar>>;
            item.position(x+320, y)
                 .text("Circular shape".to_owned())
                 .on_click(move |_image: &Toolbar, _point: Point| {
                               property_set(&ntools_clone["brush"],"Shape",0);
                               
                               //toggle tool in toolbar
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

    match Toolbar::from_path("res/block.png") {
        Ok(item) => {
            let ntools_clone = ntools.clone();
            let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<Toolbar>>;
            item.position(x+320, y)
                 .text("Blocky shape".to_owned())
                 .on_click(move |_image: &Toolbar, _point: Point| {
                               property_set(&ntools_clone["brush"],"Shape",1);
                               
                               //toggle tool in toolbar
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

// set 2nd toolbar as not visible at start
let toolbar2_obj_clone = &mut toolbar2_obj as *mut Vec<Arc<Toolbar>>;
unsafe{visible_toolbar(&mut *toolbar2_obj_clone,false);}

    //x = 10;

    //Menu file

    let menu = Menu::new("File");
    menu.position(10, 0).size(32, 16);

    //menu entries for file
            //TODO ask for new dimensions
    {
        let action = Action::new("New");
        action.on_click(move |_action: &Action, _point: Point| {
                           let mut path="";
                           if cfg!(target_os = "redox"){
                               path="/ui/bin/pastel";
                           } else{
                               path="./target/release/pastel"; 
                           }
                           Command::new(&path)
                                                .arg("new.png")
                                                .arg("1024x500")
                                                .spawn()
                                                .expect("Command executed with failing error code");
                           
                            println!("New window opened.");
                        });

        menu.add(&action);
    }

    {
        let action = Action::new("Open");
        action.on_click(move |_action: &Action, _point: Point| {
            match dialog("Open", "path:") {
                Some(response) => {
                                    println!("Open {} ", response);
                                    let mut path="";
                                    if cfg!(target_os = "redox"){
                                        path="/ui/bin/pastel";
                                        } else{
                                            path="./target/release/pastel"; 
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
                            canvas_clone.save(&filename);
                        });
        menu.add(&action);
    }

    {
        let action = Action::new("Save As");
        let canvas_clone = canvas.clone();
        
        action.on_click(move |_action: &Action, _point: Point| {
                            match dialog("Save As", "path:") {
                            Some(response) => canvas_clone.save(&(String::from(response))),
                            None => println!("Cancelled"),
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


    //Menu tool
    let tools = Menu::new("Tools");
    tools.position(50, 0).size(48, 16);

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
    menuimage.position (100,0).size (48,16);
    
    //Menu entries for image
    {
            let action = Action::new("Clear");
            let canvas_clone = canvas.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.clear();
                        });
        menuimage.add(&action);
    }
    
    //Menu help

    let help = Menu::new("Help");
    help.position(150, 0).size(32, 16);

    //menu entries for help

    {
        let action = Action::new("Info");
        action.on_click(move |_action: &Action, _point: Point| {
                            popup("Info",
                                  "Pastel, simple bitmap editor \n for Redox OS ");
                        });
        help.add(&action);
    }

    // add menues
    window.add(&menu);
    window.add(&tools);
    window.add(&menuimage);
    window.add(&help);

    // paint on canvas

    let click_pos: Rc<RefCell<Option<Point>>> = Rc::new(RefCell::new(None));
    let window_clone = &mut window as *mut Window;
    
    canvas
        .position(0, 200)
        .on_click(move |canvas: &Canvas, point: Point| {

            let click = click_pos.clone();
            let size = size_bar.clone().value.get();
            
            {
                let mut prev_opt = click.borrow_mut();

                if let Some(prev_position) = *prev_opt {
                    let mut image = canvas.image.borrow_mut();
                    let r = (red_bar.clone().value.get() as f32 * 2.55) as u8;
                    let g = (green_bar.clone().value.get() as f32 * 2.55) as u8;
                    let b = (blue_bar.clone().value.get() as f32 * 2.55) as u8;
                    let a = (trans_bar.clone().value.get() as f32 * 2.55) as u8;
                    
                    match tool.clone().text.get().as_ref() {
                        "line"  => {
                                    image.line(prev_position.x,
                                                prev_position.y,
                                                point.x,
                                                point.y,
                                                orbtk::Color::rgba(r, g, b, a));
                                   },
                         "pen"  => {                             
                                    image.pixel(point.x, point.y, orbtk::Color::rgba(r, g, b, a))
                                   },
                         "brush"=> { 
                                    match property_get(&ntools.clone()["brush"],"Shape") {
                                           Some(0) => image.circle(point.x, point.y,-size,orbtk::Color::rgba(r, g, b, a)),
                                           Some(1) => image.rect(point.x ,point.y,size as u32, size as u32, orbtk::Color::rgba(r, g, b, a)),
                                           None | Some(_)   => println!("no Shape match!"),
                                        }
                                    },
                         "fill" => image.fill(point.x, point.y,orbtk::Color::rgba(r, g, b, a)),
                    "rectangle" => unsafe{
                                    image.interact_rect(prev_position.x,
                                                        prev_position.y,
                                                        point.x,
                                                        point.y,
                                                        orbtk::Color::rgba(r, g, b, a),
                                                        &mut *window_clone
                                                        );
                                    },
                        "circle"=> image.circle(prev_position.x, prev_position.y,
                                                (2.0*((prev_position.x-point.x)^2+(prev_position.y-point.y)^2) as f64).sqrt() as i32,
                                                 orbtk::Color::rgba(r, g, b, a)),
                              _ => println!("No match!"),          
                    }

                    *prev_opt = Some(point);     //FIXME clear last position after un-click
                } else {
                    *prev_opt = Some(point);
                }
            }
        });
    window.add(&canvas);
    
    window.exec();
}

//Helper functions

//Load an image from path if exists, other way create new empty canvas
fn load_image(path: &str, size: &MySize) -> Arc<canvas::Canvas> {  
    print!("Loading image from:  {} .....", path);
    match Canvas::from_path(&path) {
        Ok(image) => {
            println!(" OK");
            image
        }
        Err(err) => {
            println!("Failed: {} \n Creating new one ", err);
            let image = Canvas::from_color(size.x, size.y, Color::rgb(255, 255, 255));
            image
        }
    }
}

//get tool property value
fn property_get( properties: &Vec<Arc<Property>>  , name: &str) -> Option<i32> {
    for a in properties {
        if &a.name.get() == name {
            return Some(a.value.get());
        }
    } 
    None
}
//set tool property value
fn property_set( properties: &Vec<Arc<Property>>  , name: &str, value: i32) {
    for a in properties {
        if &a.name.get() == name {
            a.value.set(value);
        }
    } 
}

fn toggle_toolbar (toolbar_obj: &mut Vec<Arc<Toolbar>>) {
    //deselect all items from toolbar 
    for i in 0..toolbar_obj.len(){
        if let Some(toolbar) = toolbar_obj.get(i) {
            toolbar.selected(false);
        }
    }
}

fn visible_toolbar (toolbar_obj: &mut Vec<Arc<Toolbar>>, v: bool) {
    //set visibility for all items of toolbar 
    for i in 0..toolbar_obj.len(){
        if let Some(toolbar) = toolbar_obj.get(i) {
            toolbar.visible.set(v);
        }
    }
}

//  to be added directly to orbclient ?
trait AddOnsToOrbimage {
        fn fill(&mut self, x: i32 , y: i32, color: Color);
        fn flood_fill4(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
        fn flood_fill_scanline(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
        fn flood_fill_line(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
        fn pixcol(&self, x:i32, y:i32) -> Color;
        fn pixraw(&self, x:i32, y:i32) -> u32;
        fn interact_rect(&mut self,px: i32, py: i32, x: i32 , y: i32, color: Color, window: &mut orbtk::Window);
    }

impl AddOnsToOrbimage for orbimage::Image {
    ///return rgba color of pixel at position (x,y)
    fn pixcol(&self, x:i32, y:i32) -> Color {
        let p = self.width()as i32 * y + x;
        let rgba = self.data()[p as usize];
        rgba
    }
    
    fn pixraw (&self, x:i32, y:i32) -> u32 {
        self.pixcol(x,y).data 
    }

    fn fill(&mut self, x: i32, y: i32 , color: Color) {
        //get current pixel color 
        let rgba = self.pixcol(x,y);
        //self.flood_fill_line(x,y,color.data,rgba.data);  //use rgba and color as i32 values 
        //println!("Old color {}", rgba.data);
        self.flood_fill_scanline(x,y,color.data,rgba.data);  //use rgba and color as i32 values 
    }

    /*Recursive 4-way floodfill works with transparency but be aware of stack overflow !! 
    credits to http://lodev.org/cgtutor/floodfill.html
    */
    fn flood_fill4(&mut self, x:i32, y:i32, new_color: u32 , old_color: u32) {
        if x >= 0 && x < self.width()as i32 && y >= 0 && y < self.height() as i32 
            && self.pixcol(x,y).data == old_color && self.pixcol(x,y).data != new_color {
            
            self.pixel(x,y, Color{data:new_color});
            
            self.flood_fill4(x+1,y,new_color,old_color);
            self.flood_fill4(x-1,y,new_color,old_color);
            self.flood_fill4(x,y+1,new_color,old_color);
            self.flood_fill4(x,y-1,new_color,old_color);
        }
    }
    
    //stack friendly and fast floodfill algorithm
    //Fixed now does work with transparency ;) 
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


    // draw interactive rectangle 
    fn interact_rect(&mut self, px: i32, py: i32, x: i32 , y: i32, color: Color, window: &mut orbtk::Window) {
        self.rect(x ,y,(px -x) as u32 ,(py -y) as u32 ,color);
    }
}
