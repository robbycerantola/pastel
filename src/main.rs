/*Pastel by Robby 21-05-2017
simple image editor in Rust for Redox
*/
#![feature(const_fn)]  //needed for fixing theme 
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate orbtk;
extern crate orbimage;
extern crate image;
extern crate orbclient;

use orbtk::{Color, Action, Button, Image, Label, Menu, Point, Rect, Separator, Window, Widget};  //Renderer,TextBox,ControlKnob,InnerWindow,
use orbtk::dialogs::FileDialog;
use orbtk::traits::{ Click, Place, Text };  //Border, Enter

use std::rc::Rc;
use std::cell::RefCell; //, RefMut, Cell
use std::sync::Arc;
use std::process;
use std::process::Command;
use std::path::{ Path, PathBuf };
use std::{ env, fs, cmp };

mod dialogs;
use dialogs::{ dialog, popup, new_dialog, text_dialog };

mod palette;
use palette::Palette;

mod addons;
use addons::AddOnsToOrbimage;

mod canvas;
use canvas::Canvas;

mod marquee;
use marquee::Marquee;

mod color_swatch;
use color_swatch::ColorSwatch;

mod toolbar;
use toolbar::{ Toolbar, ToolbarIcon };

mod progress_bar;
use progress_bar::ProgressBar;

/*
mod control_knob;
use control_knob::ControlKnob;
*/

mod theme;  //#FIXME use local theme to temporary fix compilation with last orbtk 0.2.26 pull which has moved to css theme, pastel has to move to css too

mod tools;
use tools::{ Property, Tools };


#[derive(Clone)]
struct MySize {
    x: u32,
    y: u32,
}

//canvas y offset position relative to main window
const CANVASOFFSET: i32 = 200;

// undo stack depth
const UNDODEPTH: usize = 10;

// enable disable help and status line
const STATUSLINE: bool = true;

const ZOOMSTEP: f32 = 0.1;

//default font location
#[cfg(target_os = "linux")]
const DEFAULTFONT : &'static str = "/usr/share/fonts/gnu-free/FreeMonoBold.ttf";
#[cfg(target_os = "redox")]
const DEFAULTFONT : &'static str = "/ui/fonts/Mono/Fira/Bold.ttf";
#[cfg(target_os = "windows")]
const DEFAULTFONT : &'static str = "C:/Windows/Fonts/arial.ttf";

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
            if cfg!(feature = "debug") {
                println!("Home path:{}", home_dir);
            }
        },
        None => println!("Impossible to get your home dir!"),
    }

    //canvas default size
    let mut size = MySize { x: 1024, y:500 };

    let filename;//#FIXME change filename type to Box so we can update

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
    size.x = canvas.width();
    size.y = canvas.height();

    //Tools and properties 
    //create new tool with some properties and initial values
    let mut tools = Tools::new();
    tools.insert("pen",vec![Property::new("Size","1"),Property::new("Opacity","100")]);
    tools.insert("line",vec![Property::new("Opacity","100")]);
    tools.insert("polyline",vec![Property::new("Size","1"),Property::new("Opacity","100")]); 
    tools.insert("brush",vec![Property::new("Size","4"),Property::new("Opacity","100"),Property::new("Shape","0")]);
    tools.insert("fill",vec![Property::new("Opacity","100")]);
    tools.insert("rectangle",vec![Property::new("Opacity","100"),Property::new("Size","1"),Property::new("Filled","0")]);
    tools.insert("circle",vec![Property::new("Opacity","100"),Property::new("Size","1"),Property::new("Filled","0")]);
    tools.insert("paste",vec![Property::new("Opacity","100")]);
    tools.insert("marquee",vec![Property::new("Opacity","100")]);
    tools.insert("polygon",vec![Property::new("Opacity","100"),Property::new("Sides","6")]);
    tools.insert("text",vec![Property::new("Opacity","100"),Property::new("Size","8"),Property::new("Text","Pastel"),Property::new("Font",DEFAULTFONT)]);
    tools.insert("pan",vec![Property::new("Opacity","100")]);
    // not a real tool but a way to store general preferences
    tools.insert("preferences",vec![Property::new("Antialias","1")]); 
    // where to store current active tool
    tools.insert("tool",vec![Property::new("Current","pen")]); 

    //set initial active tool
    tools.select("pen");

    //define current selection
    let selection :  Rc<RefCell<Option<Rect>>> = Rc::new(RefCell::new(None));

    //if pastel_copy_buffer.png exists load it into canvas copy_buffer
    //for copy/paste between instances 
    *canvas.copy_buffer.borrow_mut() = load_buffer("/tmp/pastel_copy_buffer.png");

    //implement GUI
    
    let mut x = 10;
    let mut y = 56;
    let wy = &size.y + CANVASOFFSET as u32 + 18;
    let title = format!("Pastel: {}", filename);

    //resizable main window
    let mut window = Window::new_flags(
        Rect::new(100, 100, 1024, wy),
        &title.to_owned(),
        &[orbclient::WindowFlag::Resizable ]
    );

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

    // define status line
    let status = Label::new();
    status
        .position(4, (window.height()-18) as i32)
        .size(window.width(),16)
        .text("Ready")
        .visible(STATUSLINE);

    //define marquee widget (visible selection rectangle)
    let marquee = Marquee::new();
    marquee.visible(false);

    //define current color swatch 
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
    red_label
        .text("R: 0")
        .position(x, y)
        .size(48, 16);//        .fg.set(orbtk::Color::rgb(255,0,0));
    window.add(&red_label);

    {
        red_bar.fg.set(orbtk::Color::rgb(255,0,0));  
        let swatch_clone = swatch.clone();
        let green_bar_clone_r = green_bar.clone();
        let blue_bar_clone_r = blue_bar.clone();
        let red_label = red_label.clone();
        red_bar
            .position(x+48, y)
            .size(256, 16)
            .on_click(
                move |red_bar: &ProgressBar, point: Point| {
                  let progress = point.x * 100 / red_bar.rect.get().width as i32;
                  red_label.text.set(format!("R: {}%", progress));
                  red_bar.value.set(progress);
                  //refresh color swatch
                  let r = (progress as f32 * 2.56) as u8;
                  let g = (green_bar_clone_r.value.get() as f32 * 2.56) as u8;
                  let b = (blue_bar_clone_r.value.get() as f32 * 2.56) as u8;
                  
                  swatch_clone.bg.set(orbtk::Color::rgb(r,g,b));
                },
            );
        window.add(&red_bar);
    }
    y += red_bar.rect.get().height as i32 + 2;

    let green_label = Label::new();
    green_label.text("G: 0")
        .position(x, y)
        .size(48, 16);//.fg.set(orbtk::Color::rgb(0,255,0));
    window.add(&green_label);

    {
        green_bar.fg.set(orbtk::Color::rgb(0,255,0));
        let swatch_clone = swatch.clone();
        let red_bar_clone_g = red_bar.clone();
        let blue_bar_clone_g = blue_bar.clone();
        let green_label = green_label.clone();
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
    blue_label.text("B: 0")
        .position(x, y)
        .size(48, 16);//.fg.set(orbtk::Color::rgb(0,0,255));
    window.add(&blue_label);

    {
        blue_bar.fg.set(orbtk::Color::rgb(0,0,255));
        let swatch_clone = swatch.clone();
        let green_bar_clone_b = green_bar.clone();
        let red_bar_clone_b = red_bar.clone();
        let blue_label = blue_label.clone();
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
                          swatch_clone.color(orbtk::Color::rgb(r,g,b));
                          
                      });
        window.add(&blue_bar);
    }
    y += blue_bar.rect.get().height as i32 + 10;

    // tool size bar
    let size_label = Label::new();
    size_label.text("Size: 1")
        .position(x+380, 56)
        .size(64, 16)
        .visible(false);
    window.add(&size_label);

    let size_bar = ProgressBar::new();
    let tools_clone = tools.clone();
    let size_label_clone = size_label.clone();
    size_bar.value.set(1);
    size_bar.visible(false);
    size_bar
        .position(x+450, 56)
        .size(256, 16)
        .on_click(move |size_bar: &ProgressBar, point: Point| {
                      let progress = point.x * 100 / size_bar.rect.get().width as i32;
                      size_label_clone.text.set(format!("Size: {}", progress));
                      size_bar.value.set(progress);
                      
                      //save size value for current tool
                      let cur_tool = tools_clone.current();
                      let a: &str = &cur_tool[..];  //convert String into &str                      
                      tools_clone.set(a,"Size",progress);
                  });
    window.add(&size_bar);

    // tool transparency bar
    let trans_label = Label::new();
    trans_label.text("Opacity: 100%").position(x+340, 90).size(120, 16);
    trans_label.visible(true);
    window.add(&trans_label);

    let trans_bar = ProgressBar::new();
    let tools_clone = tools.clone();
    let trans_label_clone = trans_label.clone();
    trans_bar.value.set(100);
    trans_bar.visible(true);
    trans_bar
        .position(x+450, 90)
        .size(256, 16)
        .on_click(move |trans_bar: &ProgressBar, point: Point| {
                      let progress = 1 + point.x * 100 / trans_bar.rect.get().width as i32;
                      trans_label_clone.text.set(format!("Opacity: {}%", progress));
                      trans_bar.value.set(progress);
                      
                      //save Opacity (transparency) value for current tool
                      let cur_tool = tools_clone.current();
                      let a: &str = &cur_tool[..];  //convert String into &str
                      tools_clone.set(a,"Opacity", progress);
                  });
    window.add(&trans_bar);

/*
    // tool Volume knob
    let volume_label = Label::new();
    volume_label.text("Volume: 1").position(x+380, 90).size(128, 16);
    //size_label.fg.set(orbtk::Color::rgb(0,0,255));
    window.add(&volume_label);
    
    let volume = ControlKnob::new(); //try widget control_knob
    let tool_clone = tool.clone();
    //let tools_clone=tools.clone();
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
                      //let a: &str = &cur_tool[..];  // convert String into &str
                      //tools_clone[a].size(progress);
                      
                  });
    window.add(&volume);
*/

    // create a new palette at x,y,width,height linked to swatch and color picker
    let palette=Palette::new(10,120,window.width(),50,swatch_clone,red_bar,
                            green_bar,blue_bar, red_label, green_label, blue_label);

    // show on window the standard palette
    palette.prepare(&window);

    //clickable icon
    match Image::from_path( "pastel100.png" ) {
        Ok(image) => {
            image.position(900, 10);
            /*
            image.on_click(move |_image: &Image, _point: Point| {
                               popup("Ciao",
                                  "Pastel is work in progress,\nplease be patient....");
                           });
            */
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

    //use new Toolbar widget to implement Toolbars
    let parent_window = &mut window as *mut Window;  //we need a pointer to the parent window to add the icons to
    let mut toolbar1 = Toolbar::new();
    let mut toolbar2 = Toolbar::new();
    let mut toolbar3 = Toolbar::new();

    let y = 25;

    //toolbar items
    match ToolbarIcon::from_path("pencil1.png") {
        Ok(image) => {
            image
                .position(x, y)
                .tooltip("Draft painting".to_owned())
                .selected(true);

            let status_clone = status.clone();
            let size_bar_clone = size_bar.clone();
            let trans_bar_clone = trans_bar.clone();
            let tools_clone = tools.clone();
            let size_label_clone = size_label.clone();
            let trans_label_clone = trans_label.clone();
            let toolbar1_clone = &mut toolbar1 as *mut Toolbar;
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.on_click(move |_image: &ToolbarIcon, _point: Point| {
               tools_clone.select("pen");
               status_clone.text("");
               size_bar_clone.visible(false);
               size_label_clone.visible(false);
               let o = tools_clone.get("pen","Opacity").unwrap();
               trans_bar_clone.value.set(o);
               trans_label_clone.text(format!("Opacity: {}%",o));
               //toggle tool in toolbar TODO move into Toolbar
               unsafe{(*toolbar1_clone).toggle()}
               //make  toolbar2 invisible
               unsafe{(*toolbar2_clone).visible(false)}
               //make toolbar3 invisible
               unsafe{(*toolbar3_clone).visible(false)}
            });

            toolbar1.add(&image,parent_window);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("pencil2.png") {
        Ok(image) => {
            let status_clone = status.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let tools_clone = tools.clone();
            let toolbar1_clone = &mut toolbar1 as *mut Toolbar;
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image
                .position(x, y)
                .tooltip("Draw lines".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                   //set current tool
                   tools_clone.select("line");
                   status_clone.text("");
                   //get previous settings
                   size_bar_clone.visible(false);
                   size_label_clone.visible(false);
                   let o = tools_clone.get("line","Opacity").unwrap();
                   trans_bar_clone.value.set(o);
                   trans_label_clone.text(format!("Opacity: {}%",o));
                   
                   //toggle tool in toolbar
                   unsafe{(*toolbar1_clone).toggle();}
                   //make invisible toolbar2
                   unsafe{(*toolbar2_clone).visible(false);}
                   //make toolbar3 invisible
                   unsafe{(*toolbar3_clone).visible(false);}
                });

            toolbar1.add(&image,parent_window);
            
            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("brush.png") {
        Ok(image) => {
            let status_clone = status.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let tools_clone = tools.clone();
            let toolbar1_clone = &mut toolbar1 as *mut Toolbar;
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;

            image
                .position(x, y)
                .tooltip("Paint brush".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    tools_clone.select("brush");
                    status_clone.text("");
                    size_label_clone.visible(true);
                    size_bar_clone.visible(true);
                    let v = tools_clone.get("brush","Size").unwrap();
                    size_bar_clone.value.set(v);
                    size_label_clone.text(format!("Size: {}",v));

                    let o = tools_clone.get("brush","Opacity").unwrap();
                    trans_bar_clone.value.set(o);
                    trans_label_clone.text(format!("Opacity: {}%",o));

                    //toggle tool in toolbar
                    unsafe{(*toolbar1_clone).toggle();}
                    //make visible toolbar2
                    unsafe{(*toolbar2_clone).visible(true);}
                    //make toolbar3 invisible
                    unsafe{(*toolbar3_clone).visible(false);}
                });

            toolbar1.add(&image,parent_window);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("fillbucket.png") {
        Ok(item) => {
            let status_clone = status.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let tools_clone = tools.clone();
            let toolbar1_clone = &mut toolbar1 as *mut Toolbar;
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            item.position(x, y)
                .tooltip("Fill up area with color".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    tools_clone.select("fill");
                    status_clone.text("Filling...");
                    size_label_clone.visible(false);
                    size_bar_clone.visible(false);
                   
                    let o = tools_clone.get("fill","Opacity").unwrap();
                    trans_bar_clone.value.set(o);
                    trans_label_clone.text(format!("Opacity: {}%",o));
                   
                    //toggle tool in toolbar
                    unsafe{(*toolbar1_clone).toggle();}
                    //make invisible toolbar2
                    unsafe{(*toolbar2_clone).visible(false);}
                    //make toolbar3 invisible
                    unsafe{(*toolbar3_clone).visible(false);}
                });

            toolbar1.add(&item,parent_window);
            
            x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("polyline.png") {
        Ok(image) => {

            let status_clone = status.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let tools_clone = tools.clone();
            let toolbar1_clone = &mut toolbar1 as *mut Toolbar;
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.position(x, y)
                .tooltip("Draw polylines".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    //set current tool
                    tools_clone.select("polyline");
                    status_clone.text("Drawing polylines... right click to exit.");
                    //get previous settings
                    size_bar_clone.visible(true);
                    size_label_clone.visible(true);
                    let o = tools_clone.get("polyline","Opacity").unwrap();
                    trans_bar_clone.value.set(o);
                    trans_label_clone.text(format!("Opacity: {}%",o));
                    let s = tools_clone.get("polyline","Size").unwrap();
                    size_bar_clone.value.set(s);
                    size_label_clone.text(format!("Size: {}",s));

                    //toggle tool in toolbar
                    unsafe{(*toolbar1_clone).toggle();}
                    //make visible toolbar2
                    unsafe{(*toolbar2_clone).visible(false);}
                    //make toolbar3 invisible
                    unsafe{(*toolbar3_clone).visible(false);}
                });

            toolbar1.add(&image,parent_window);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("rectangle.png") {
        Ok(image) => {
            let status_clone = status.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let tools_clone = tools.clone();
            let toolbar1_clone = &mut toolbar1 as *mut Toolbar;
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.position(x, y)
                .tooltip("Draw rectangles".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    //set current tool
                    tools_clone.select("rectangle");
                    status_clone.text("Drawing rectangles...");
                    //get previous settings
                    size_bar_clone.visible(true);
                    size_label_clone.visible(true);
                    let o = tools_clone.get("rectangle","Opacity").unwrap();
                    trans_bar_clone.value.set(o);
                    trans_label_clone.text(format!("Opacity: {}%",o));
                    let w = tools_clone.get("rectangle","Size").unwrap();
                    size_bar_clone.value.set(w);
                    size_label_clone.text(format!("Size: {}",w));

                    //toggle tool in toolbar
                    unsafe{(*toolbar1_clone).toggle();}
                    //make invisible toolbar2
                    unsafe{(*toolbar2_clone).visible(false);}
                    //make toolbar3 visible
                    unsafe{(*toolbar3_clone).visible(true);}
                });

            toolbar1.add(&image,parent_window);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("hollow_circle.png") {
        Ok(image) => {
            let status_clone = status.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let trans_bar_clone = trans_bar.clone();
            let trans_label_clone = trans_label.clone();
            let tools_clone = tools.clone();
            let toolbar1_clone = &mut toolbar1 as *mut Toolbar;
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.position(x, y)
                .tooltip("Draw circles".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    //set current tool
                    tools_clone.select("circle");
                    status_clone.text("Drawing circles...click on center, move cursor to set radius, click again.");
                    //get previous settings
                    size_bar_clone.visible(false);
                    size_label_clone.visible(false);
                    let o = tools_clone.get("circle","Opacity").unwrap();
                    trans_bar_clone.value.set(o);
                    trans_label_clone.text(format!("Opacity: {}%",o));

                    //toggle tool in toolbar
                    unsafe{(*toolbar1_clone).toggle();}
                    //make invisible toolbar2
                    unsafe{(*toolbar2_clone).visible(false);}
                    //make toolbar3 visible
                    unsafe{(&mut *toolbar3_clone).visible(true);}
                });

            toolbar1.add(&image,parent_window);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }
/*
    match ToolbarIcon::from_path("text.png") {
        Ok(image) => {
            image.position(x, y)
            .text("Draw some text on canvas".to_owned());
            let canvas_clone = canvas.clone();    
            let tool_clone = tool.clone();
            let text_clone = text.clone();
            let status_clone = status.clone();
            let size_bar_clone = size_bar.clone();
            let trans_bar_clone = trans_bar.clone();
            let tools_clone = tools.clone();
            let size_label_clone = size_label.clone();
            let trans_label_clone = trans_label.clone();
            let toolbar1_clone = &mut toolbar1 as *mut Toolbar;
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.on_click(move |_image: &ToolbarIcon, point: Point| {
                               size_bar_clone.visible(true);
                               size_label_clone.visible(true);
                                match dialog("Text", "text:","") {
                                    Some(response) => {
                                    text_clone.text.set(response.to_owned());
                                    canvas_clone.emit_right_click(point); //fix click bug
                                },
                                    None => {println!("Cancelled");},
                                }   
                               tool_clone.text.set("text".to_owned());
                               tools_clone.set("tool","Current","text");
                               status_clone.text("Drawing some text...");


                               let o = toola_clone.get("text","Opacity").unwrap();
                               trans_bar_clone.value.set(o);
                               trans_label_clone.text(format!("Opacity: {}%",o));
                                let s = tools_clone.get("text","Size").unwrap();
                                size_bar_clone.value.set(s);
                                size_label_clone.text(format!("Size: {}",s));
                               //toggle tool in toolbar TODO move into Toolbar
                               //unsafe {toggle_toolbar(&mut *toolbar_obj_clone);}
                               unsafe{(*toolbar1_clone).toggle();}
                               //make invisible toolbar2  TODO move into Toolbar
                               //unsafe{visible_toolbar(&mut *toolbar2_obj_clone,false);}
                               unsafe{(*toolbar2_clone).visible(false);}
                               //make toolbar3 invisible
                               unsafe{(*toolbar3_clone).visible(false);}
                               
                           });
            
            //window.add(&image);
            //toolbar_obj.push(image.clone());  //TODO toolbar.add(&image);
            toolbar1.add(&image,parent_window);
           x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }
*/

    match ToolbarIcon::from_path("select.png") {
        Ok(image) => {

            let status_clone = status.clone();
            let size_bar_clone = size_bar.clone();
            let tools_clone = tools.clone();
            let size_label_clone = size_label.clone();
            let trans_label_clone = trans_label.clone();
            let toolbar1_clone = &mut toolbar1 as *mut Toolbar;
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            image.position(x, y)
                .tooltip("Select canvas region".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    tools_clone.select("marquee");
                    status_clone.text("Selecting...");
                    size_bar_clone.visible(false);
                    size_label_clone.visible(false);

                    //toggle tool in toolbar TODO move into Toolbar
                    unsafe{(*toolbar1_clone).toggle();}
                    //make invisible toolbar2  TODO move into Toolbar
                    unsafe{(*toolbar2_clone).visible(false);}
                });
            
            toolbar1.add(&image,parent_window);

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
            let tools_clone = tools.clone();
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            item.position(x, y)
                .tooltip("Circular shape".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    tools_clone.set("brush","Shape",0);
                    //toggle shape in toolbar2
                    unsafe{(*toolbar2_clone).toggle();}
                    });
            toolbar2.add(&item,parent_window);

            x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("smooth_circle.png") {
        Ok(item) => {
            let tools_clone = tools.clone();
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            item.position(x, y)
                .tooltip("Smooth edges circular shape".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    tools_clone.set("brush","Shape",3);

                    //toggle shape in toolbar2
                    unsafe{(*toolbar2_clone).toggle();}
                });

            toolbar2.add(&item,parent_window);

            x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("block.png") {
        Ok(item) => {
            let tools_clone = tools.clone();
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            item.position(x, y)
                .tooltip("Blocky shape".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    tools_clone.set("brush","Shape",1);

                    //toggle shape in toolbar2
                    unsafe{(*toolbar2_clone).toggle();}
                });

            toolbar2.add(&item,parent_window);

            x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    match ToolbarIcon::from_path("buffer.png") {
        Ok(item) => {
            let tools_clone = tools.clone();
            let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
            item.position(x, y)
                .tooltip("custom brush from buffer".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    tools_clone.set("brush","Shape",2);
                               
                    //toggle shape in toolbar2
                    unsafe{(*toolbar2_clone).toggle();}
                });

            toolbar2.add(&item,parent_window);
            
            //x += item.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading toolbar element {}",err);
        }
    }

    // set 2nd toolbar not visible at start
    toolbar2.visible(false);

    x = 500;

    //3rd toolbar new api 
    match ToolbarIcon::from_path("rectangle.png") {
        Ok(item) => {
            let tools_clone = tools.clone();
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            item.position(x, y)
                .tooltip("Not filled".to_owned()) 
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    tools_clone.set("rectangle","Filled",0);
                    tools_clone.set("circle","Filled",0);
                    //toggle item in toolbar3
                    unsafe{(*toolbar3_clone).toggle();}  
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
            let tools_clone = tools.clone();
            let toolbar3_clone = &mut toolbar3 as *mut Toolbar;
            item.position(x, y)
                .tooltip("Filled".to_owned())
                .on_click(move |_image: &ToolbarIcon, _point: Point| {
                    tools_clone.set("rectangle","Filled",1);
                    tools_clone.set("circle","Filled",1);
                    //toggle item in toolbar3  
                    unsafe{(*toolbar3_clone).toggle();} 
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
        let action = Action::new("Undo     Ctrl+Z");
        let canvas_clone = canvas.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            canvas_clone.undo();
        });
        menuedit.add(&action);
    }

    menuedit.add(&Separator::new());

    {
        let action = Action::new("Select");
        let tools_clone = tools.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            tools_clone.select("marquee");
            status_clone.text("Selecting... (click on canvas, move cursor to define a rectangle than click again)");
        });
        menuedit.add(&action);
    }

    {
        let action = Action::new("Clear selection");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        let size_clone = size.clone();
        let marquee_clone = marquee.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            *selection_clone.borrow_mut()=None;
            marquee_clone.visible(false);
        });
        menuedit.add(&action);
    }

    menuedit.add(&Separator::new());

    {
        let action = Action::new("Copy     Ctrl+C");
        let tools_clone = tools.clone();
        let canvas_clone = canvas.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            tools_clone.select("copy");
            canvas_clone.emit_click(Point{x:0,y:0}); //trigger buffer saving without clicking effectively on canvas
            status_clone.text("Copying...");
        });
        menuedit.add(&action);
    }
    
    {
        let action = Action::new("Paste    Ctrl+V");
        let tools_clone = tools.clone();
        let canvas_clone = canvas.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            *canvas_clone.copy_buffer.borrow_mut() = load_buffer("/tmp/pastel_copy_buffer.png");
            tools_clone.select("paste");
            status_clone.text("Pasting... (click on canvas, move cursor to refine the position than click again)");
        });
        menuedit.add(&action);
    }

    menuedit.add(&Separator::new());

    {
        let action = Action::new("Load Buffer");
        let home_dir_clone = home_dir.clone();
        let canvas_clone = canvas.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            let mut f= FileDialog::new();
            f.title="Load Buffer from file".to_owned();
            f.path=PathBuf::from(home_dir_clone.to_owned());
            match f.exec() {
                Some(response) => {
                    let bf = load_buffer(&(response.display().to_string())[..]);
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
        let status_clone = status.clone();
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

   // menuedit.add(&Separator::new());    


    //Menu mask
    let menumask = Menu::new("Mask");
    menumask.position (90,0).size (36,16);
    
    //Menu entries for mask
    {
        let action = Action::new("Edit          ");
        let canvas_clone = canvas.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.paint_on_mask();
                        if canvas_clone.mask_flag() {
                            _action.text("Back to canvas");
                            status_clone.text("Paint in black & white (or greys) to define the mask, than click 'Mask -> Back to canvas' to use it");
                        }else{
                            status_clone.text("");
                            _action.text("Edit");
                        }
                            
                          });
        menumask.add(&action);
    }
    
    {
        let action = Action::new("Clear");
        let canvas_clone = canvas.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.clear_mask();
                          });
        menumask.add(&action);
    }

    {
        let action = Action::new("Disable");
        let canvas_clone = canvas.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.enable_mask(false);
                          });
        menumask.add(&action);
    }
    
    {
        let action = Action::new("Enable");
        let canvas_clone = canvas.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.enable_mask(true);
                        
                          });
        menumask.add(&action);
    }



    //Menu tool
    let menutools = Menu::new("Tools");
    menutools.position(130, 0).size(48, 16);

    //Menu entries for tools
    {
        let action = Action::new("Pen");
        let tools_clone = tools.clone();
        let status_clone = status.clone();

        action.on_click(move |_action: &Action, _point: Point| {
            tools_clone.select("pen");
            status_clone.text("Drawing points ...");
        });
        menutools.add(&action);
    }

    {
        let action = Action::new("Line");
        let tools_clone = tools.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            tools_clone.select("line");
            status_clone .text("Drawing...");
        });
        menutools.add(&action);
    }

    {
        let action = Action::new("Polyline");
        let tools_clone = tools.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            tools_clone.select("polyline");
            status_clone.text("Drawing polylines...");
            
        });
        menutools.add(&action);
    }


    {
        let action = Action::new("Brush");
        let tools_clone = tools.clone();
        let status_clone = status.clone();
        let toolbar2_clone = &mut toolbar2 as *mut Toolbar;
        let size_bar_clone = size_bar.clone();
        let size_label_clone = size_label.clone();
        let trans_bar_clone = trans_bar.clone();
        let trans_label_clone = trans_label.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            tools_clone.select("brush");
            status_clone.text("Painting...");
            size_label_clone.visible(true);
            size_bar_clone.visible(true);
            let v = tools_clone.get("brush","Size").unwrap();
            size_bar_clone.value.set(v);
            size_label_clone.text(format!("Size: {}",v));
            let o = tools_clone.get("brush","Opacity").unwrap();
            trans_bar_clone.value.set(o);
            trans_label_clone.text(format!("Opacity: {}%",o));
            unsafe{(*toolbar2_clone).visible(true);}
        });
        menutools.add(&action);
    }

    {
        let action = Action::new("Fill");
        let tools_clone = tools.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tools_clone.select("fill");
                            status_clone.text("Filling...");
                        });
        menutools.add(&action);
    }
    
    {
        let action = Action::new("Rectangle");
        let tools_clone = tools.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tools_clone.select("rectangle");
                            status_clone.text("Drawing rectangles...");
                        });
        menutools.add(&action);
    }
    
    {
        let action = Action::new("Circle");
        let tools_clone = tools.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            tools_clone.select("circle");
                            status_clone.text("Drawing circles...");
                        });
        menutools.add(&action);
    }

    {
        let action = Action::new("Polygon");
        let status_clone = status.clone();
        let tools_clone = tools.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            match dialog("Regular polygon", "sides:","3") {
                            Some(response) => {
                                tools_clone.set("polygon","Sides",response.parse::<i32>().unwrap_or(3));
                                tools_clone.select("polygon");
                                status_clone.text("Drawing regular poligons...");
                            },
                            None => {println!("Cancelled");},
                        }
        });
        menutools.add(&action);
    }

    {
        let action = Action::new("Text");
        let status_clone = status.clone();
        let size_bar_clone = size_bar.clone();
        let size_label_clone = size_label.clone();
        let tools_clone = tools.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            match text_dialog("Text", "text:","") {
                                Some(response) => {
                                    tools_clone.select("text");
                                    tools_clone.set("text","Text",response.0.to_owned());
                                    tools_clone.set("text","Font",response.1.to_owned());
                                    let s = tools_clone.get("text","Size").unwrap();
                                    let o = tools_clone.get("text","Opacity").unwrap();
                                    size_bar_clone.visible(true);
                                    size_label_clone.visible(true);
                                    size_bar_clone.value.set(s);
                                    size_label_clone.text(format!("Size: {}",s));
                                    status_clone.text("Click on canvas to draw the text...(you can change color, size & opacity)");
                                },
                                None => {println!("Cancelled");},
                            }
        });
        menutools.add(&action);
    }



    menutools.add(&Separator::new());

    {
        let action = Action::new("Antialias \u{2611}");
        let status_clone = status.clone();
        let tools_clone = tools.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        let antialias = tools_clone.get("preferences","Antialias").unwrap();
                        if antialias == 1 {
                            tools_clone.set("preferences","Antialias",0);
                            _action.text("Antialias \u{2610}");
                            status_clone.text("Antialiasing disabled");
                        }else{
                            tools_clone.set("preferences","Antialias",1);
                            status_clone.text("Antialiasing enabled");
                            _action.text("Antialias \u{2611}");
                        }
        });
        menutools.add(&action);
    }

    //Menu image
    let menuimage = Menu::new("Image");
    menuimage.position (180,0).size (48,16);

    //Menu entries for image

    {
        let action = Action::new("Blur");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        match dialog("Blur", "quantity:","5.1") {
                            Some(response) => {
                                canvas_clone.trans_selection(selection_clone.borrow()
                                .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                                height: canvas_clone.rect.get().height-1}),
                                "blur",response.parse::<f32>().unwrap_or(0.0),0);
                            },
                            None => {println!("Cancelled");},
                        }
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Unsharpen");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        match dialog("Unsharpen", "quantity:","5.1") {
                            Some(response) => {
                                canvas_clone.trans_selection(selection_clone.borrow()
                                .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                                height: canvas_clone.rect.get().height-1}),
                                "unsharpen",response.parse::<f32>().unwrap_or(0.0),0);
                            },
                            None => {println!("Cancelled");},
                        }
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Verical flip");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow()
                        .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                             height: canvas_clone.rect.get().height-1}),"flip_vertical",0.0,0);
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Horizontal flip");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow()
                        .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                             height: canvas_clone.rect.get().height-1}),"flip_horizontal",0.0,0);
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Rotate 90");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        let marquee_clone = marquee.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow()
                        .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                             height: canvas_clone.rect.get().height-1}),"rotate90",0.0,0);
                        //rotate also selection if exists 
                        let rect = selection_clone.borrow()
                            .unwrap_or(Rect{x: 0, y: 0 , width: 0, height: 0});
                        if rect.width >0 {
                            *selection_clone.borrow_mut() = Some(Rect{x:rect.x, y:rect.y,
                                width: rect.height, height: rect.width});
                            //and marquee
                            marquee_clone.size(rect.height, rect.width);
                        }
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Rotate");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        let marquee_clone = marquee.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        match dialog("Rotate", "degrees:","45") {
                            Some(response) => {
                                canvas_clone.trans_selection(selection_clone.borrow()
                                .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                                height: canvas_clone.rect.get().height-1}),
                                "rotate",response.parse::<f32>().unwrap_or(0.0),0);
                            },
                            None => {println!("Cancelled");},
                        }
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Brighten");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow()
                        .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                             height: canvas_clone.rect.get().height-1}),"brighten",0.0,0);
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Darken");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.trans_selection(selection_clone.borrow()
                        .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                             height: canvas_clone.rect.get().height-1}),"darken",0.0,0);
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Contrast");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        let marquee_clone = marquee.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        match dialog("Contrast", "value (+/-):","10.0") {
                            Some(response) => {
                                canvas_clone.trans_selection(selection_clone.borrow()
                                .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                                height: canvas_clone.rect.get().height-1}),
                                "contrast",response.parse::<f32>().unwrap_or(0.0),0);
                            },
                            None => {println!("Cancelled");},
                        }
                    });
        menuimage.add(&action);
    }

    {
        let action = Action::new("Invert");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            canvas_clone.trans_selection(selection_clone.borrow()
                .unwrap_or(Rect{x:0,y:0, width: canvas_clone.rect.get().width -1 ,
                    height: canvas_clone.rect.get().height-1}),"invert",0.0,0);
        });
        menuimage.add(&action);
    }
    //menuimage.add(&Separator::new());

    {
        let action = Action::new("Grayscale");
        let canvas_clone = canvas.clone();
        let selection_clone = selection.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                        canvas_clone.transformation("grayscale",0.0,0);
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
                                    canvas_clone.transformation("resize",x as f32,y);
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
        menupalette.position (230, 0).size(64, 16);

    //Menu entries for palette
    {
        let action = Action::new("Load");
        let home_dir_clone = home_dir.clone();
        let palette_clone = palette.clone();
        action.on_click(move |_action: &Action, _point: Point| {
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
    
    menupalette.add(&Separator::new());

    //Menu view
    let menuview = Menu::new("View");
    menuview.position(295, 0).size(32, 16);
    
    //menu entries for view
    {
        let action = Action::new("Zoom in");
        let window_clone = &mut window as *mut Window;
        let size_clone = size.clone();
        //let tools_clone = tools.clone();
        let canvas_clone = canvas.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.zoom_in();
                            //resize window to fit image + status bar
                            let newx = *cmp::max(&size_clone.x, &canvas_clone.width());
                            let newy = *cmp::max(&size_clone.y, &(canvas_clone.height() + 18 + CANVASOFFSET as u32));
                            unsafe{ (*window_clone).set_size(newx, newy) }
                            status_clone.text(format!("Zoomed in ...({}%)",(100.0 * canvas_clone.zoom_factor.get()) as i32));
                            status_clone.position(4,(canvas_clone.height()as i32 + CANVASOFFSET ) );
                        });
        menuview.add(&action);
    }

    {
        let action = Action::new("Zoom out");
        let window_clone = &mut window as *mut Window;
        let size_clone = size.clone();
        //let tools_clone = tools.clone();
        let canvas_clone = canvas.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.zoom_out();
                            //resize window to fit image + status bar
                            let newx = *cmp::max(&size_clone.x, &canvas_clone.width());
                            let newy = *cmp::max(&size_clone.y, &(canvas_clone.height() + 18 + CANVASOFFSET as u32));
                            unsafe{ (*window_clone).set_size(newx, newy) }
                            status_clone.text(format!("Zoomed out ...({}%)",(100.0 * canvas_clone.zoom_factor.get())as i32));
                            status_clone.position(4,(canvas_clone.height()as i32 + CANVASOFFSET ) );
                        });
        menuview.add(&action);
    }

    {
        let action = Action::new("Pan");
        //let window_clone = &mut window as *mut Window;
        //let size_clone = size.clone();
        let tools_clone = tools.clone();
        //let canvas_clone = canvas.clone();
        let status_clone = status.clone();
        action.on_click(move |_action: &Action, _point: Point| {
                            //canvas_clone.pan(100,100);
                            tools_clone.select("pan");
                            status_clone.text("Panning ...");
                        });
        menuview.add(&action);
    }
    

    //Menu help

    let menuhelp = Menu::new("Help");
    menuhelp.position(335, 0).size(32, 16);

    //menu entries for help

    {
        let action = Action::new("Info");
        action.on_click(move |_action: &Action, _point: Point| {
                            popup("Info",
                                  "Pastel v0.0.32, simple bitmap editor \n for Redox OS by Robby Cerantola");
                        });
        menuhelp.add(&action);
    }

    // paint on canvas
    let click_pos: Rc<RefCell<Option<Point>>> = Rc::new(RefCell::new(None));
    let window_clone = &mut window as *mut Window;
    let click_pos_clone = click_pos.clone();
    let selection_clone = selection.clone();
    let marquee_clone = marquee.clone();
    let tools_clone = tools.clone();
    let status_clone = status.clone();

    canvas
        .position(0, CANVASOFFSET)
        .on_shortcut(move |canvas: &Canvas, key: char| {
            if cfg!(feature = "debug"){
                println!("Pressed {} key ",key);
            }
            //manage shortcuts
            match key {
                'v' => {
                        tools_clone.select("paste");
                        canvas.emit_click(Point{x:(canvas.rect.get().width/2) as i32 ,
                            y: (canvas.rect.get().height/2) as i32});
                        canvas.emit_click(Point{x:(canvas.rect.get().width/2) as i32 ,
                            y: (canvas.rect.get().height/2) as i32});
                },
                'c' => {
                        tools_clone.select("copy");
                        canvas.emit_click(Point{x: 0, y: 0});
                },
                'Q' => {
                        canvas.paint_on_mask();
                },
                '@' => {
                        status_clone.position(4,(canvas.height()as i32 + CANVASOFFSET ) );
                },
                _ => (),
            }
        
    })

    .on_right_click(move |_ , point:Point|{
        if cfg!(feature = "debug"){
            println!("Right click at {:?} not implemented yet",point);
        }
    })

    .on_clear_click(move |_ , _point:Point|{
        // clears last cursor position 
        let mut ck=click_pos_clone.borrow_mut();
        *ck = None;
    })

    .on_click(move |canvas: &Canvas, point: Point| {
        let click = click_pos.clone();
        let size = size_bar.clone().value.get();
        let swatch_clone = swatch.clone();
        {
            let mut prev_opt = click.borrow_mut();
            //let r = (red_bar.clone().value.get() as f32 * 2.55) as u8;
            //let g = (green_bar.clone().value.get() as f32 * 2.55) as u8;
            //let b = (blue_bar.clone().value.get() as f32 * 2.55) as u8;
            let a = (trans_bar.clone().value.get() as f32 * 2.55) as u8;
            let swc = swatch_clone.read();
            let color = Color::rgba(swc.r(),swc.g(),swc.b(),a);
            let tools_clone = tools.clone();
            let antialias = tools_clone.get("preferences","Antialias").unwrap();
            let selected_tool = tools_clone.current();

            //tools that dont need prev_position
            match selected_tool.as_ref() {
                "pen"  => canvas.pixel(point.x, point.y, color),
                "fill" => canvas.fill(point.x, point.y,color),
                "rectangle" => {
                    canvas.undo_save();
                    let filled = tools_clone.get("rectangle","Filled").unwrap();
                    let mut width = tools_clone.get("rectangle","Size").unwrap();
                    let mut myselection = Rect::new(0,0,0,0);
                    if let Some(selection) = unsafe {
                        canvas.image.borrow_mut().new_select_rect(
                            point.x,
                            point.y,
                            Color::rgb(100,100,100),
                            0,
                            &mut *window_clone
                        )
                        } {
                            myselection = selection;
                        }
                        if filled == 1 {
                            if myselection.height > myselection.width {
                                width = 1+(myselection.width/2) as i32;
                            } else {
                                width = 1+(myselection.height/2) as i32;
                            }
                        }
                        for i in 0..width {
                            canvas.rect(
                                myselection.x+i,
                                myselection.y+i,
                                myselection.width- (2*i) as u32,
                                myselection.height- (2*i) as u32,
                                color
                            );
                        }
                },
                "polyline" => { 
                    canvas.undo_save();
                    let width = tools_clone.get("polyline","Size").unwrap();
                    let mut tu = (point.x,point.y,point.x,point.y);
                    if let Some(tuple) = unsafe {
                        canvas.image.borrow_mut().interact_line(point.x,
                            point.y,
                            color,
                            width,
                            antialias == 1,
                            &mut *window_clone
                        )
                    } { tu = tuple }
                    let (x1, y1, x2, y2) = tu;
                    for d in 0..width {
                        if antialias == 1 {
                            canvas.wu_line(x1 + d, y1, x2 + d, y2, color);
                        } else {
                            canvas.line(x1 + d, y1, x2 + d, y2, color);
                        }
                    }
                },
                "copy" => {
                    let mut image = canvas.image.borrow_mut();
                    match *selection_clone.borrow() {
                        Some(selection) => {
                             *canvas.copy_buffer.borrow_mut() = image.copy_selection(
                                selection.x,
                                selection.y,
                                selection.width,
                                selection.height
                            );
                             //save buffer to disk as pastel_copy_buffer.png so we can reload when starting new program instance
                             let newcanvas = Canvas::from_image(canvas.copy_buffer.borrow().clone());
                             let path = "/tmp/pastel_copy_buffer.png".to_string();
                             if let Ok(_) = newcanvas.save(&path) {}
                         },
                        None => if let Some(selection) =
                            unsafe { image.select_rect(point.x, point.y,&mut *window_clone)} 
                        {       
                            *canvas.copy_buffer.borrow_mut() = image.copy_selection(
                                selection.x,
                                selection.y,
                                selection.width,
                                selection.height
                            );
                            //save buffer to disk as pastel_copy_buffer.png so we can reload when starting new program instance
                            let newcanvas = Canvas::from_image(canvas.copy_buffer.borrow().clone());
                            let path = "/tmp/pastel_copy_buffer.png".to_string();
                            if let Ok(_) = newcanvas.save(&path){}
                        },
                    }
                },
               "marquee"=> {
                    marquee_clone.visible(false);
                    canvas.undo_save();
                    let mut image = canvas.image.borrow_mut();
                    if let Some(selection) =
                        unsafe{ image.select_rect(point.x, point.y,&mut *window_clone) }
                    {
                        *selection_clone.borrow_mut() = Some(selection);
                        marquee_clone
                            .position(selection.x,selection.y+CANVASOFFSET)
                            .size(selection.width,selection.height)
                            .visible(true);
                    }
                },
                "paste" => {
                    canvas.undo_save();
                    if let Some(tuple) = unsafe {
                        canvas.interact_paste(
                            point.x,
                            point.y,
                            a.clone(),
                            &mut *window_clone
                        )
                    } {
                        canvas.paste_buffer(tuple.0, tuple.1 , a.clone());
                    }
                },
                "circle" => {
                    canvas.undo_save();
                    let filled = tools_clone.get("circle","Filled").unwrap();
                    let width = tools_clone.get("circle","Size").unwrap();
                    let radius;
                    let mut myr = 0;
                    {
                        let mut image = canvas.image.borrow_mut();
                        if let Some((r,angle)) = unsafe {
                            image.interact_circle(point.x, point.y, color, &mut *window_clone)
                        } {myr = r;}
                    }
                    {
                        if filled == 1 {
                            radius = -myr;
                            canvas.circle(point.x, point.y, radius, color);
                        }else{
                            radius=myr;
                            if antialias == 1 {
                                canvas.wu_circle(point.x, point.y, radius, color);
                            }else{
                                canvas.circle(point.x, point.y, radius, color);
                            }
                        }
                    }
                },
                "polygon" => {
                    let sides = tools_clone.get("polygon","Sides").unwrap();
                    canvas.undo_save();
                    let mut aangle=0_f32;
                    let mut rr=0;
                    {
                        let mut image = canvas.image.borrow_mut();
                        if let Some((r,angle)) = unsafe{
                            image.interact_circle(
                                point.x,
                                point.y,
                                color,
                                &mut *window_clone
                            )
                        } {aangle = angle; rr = r;  }
                    }
                    canvas.polygon(point.x,point.y,rr,sides as u32, aangle, color, antialias==1);
                },
                "text" => {
                    //let text = text_clone.text.get();
                    //let font_path = text_clone.font.get();
                    let text = tools_clone.get_str("text","Text").unwrap();
                    let font_path = tools_clone.get_str("text","Font").unwrap();
                    canvas.text(&text, &font_path, point.x, point.y - CANVASOFFSET, color, size );
                },
                _ => (),
                }

            //tools that need prev_position to work
            if let Some(prev_position) = *prev_opt {
                match selected_tool.as_ref() {
                     "pan" => {canvas.pan(point.x - prev_position.x, point.y - prev_position.y);},
                    "line" => { 
                        if antialias == 1 {
                            canvas.wu_line(prev_position.x,
                                            prev_position.y,
                                            point.x,
                                            point.y,
                                            color);
                        }else{
                            canvas.line(prev_position.x,
                                        prev_position.y,
                                        point.x,
                                        point.y,
                                        color);
                        }
                    },
                    "brush" => {
                        match tools.clone().get("brush","Shape") {
                            Some(0) => canvas.brush_line(prev_position.x,
                                            prev_position.y,
                                            point.x,
                                            point.y,
                                            -size,
                                            color),
                            Some(1) => canvas.rect_line(prev_position.x,
                                            prev_position.y,point.x ,point.y,size as u32, size as u32,
                                                    color),
                            Some(2) => canvas.paste_buffer(point.x,point.y,
                                                    a.clone()),
                            Some(3) => canvas.smooth_circle(point.x,point.y,
                                                    size as u32, color),
                            None | Some(_) => println!("no Shape match!"),
                        }
                    },
                    _ => (),
                    }
                *prev_opt = Some(point);
            } else {
                *prev_opt = Some(point);
                if selected_tool == "line" || selected_tool =="pen" || selected_tool =="brush"
                    || selected_tool=="brush_line"
                    || selected_tool=="text" {canvas.undo_save();} //prepare for undo
            }
        }
    });

    window.add(&canvas);
    window.add(&marquee);
    window.add(&status);

    // add menus
    window.add(&menufile);
    window.add(&menuedit);
    window.add(&menutools);
    window.add(&menumask);
    window.add(&menuimage);
    window.add(&menupalette);
    window.add(&menuview);
    window.add(&menuhelp);

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
