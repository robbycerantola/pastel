/*Pastel by Robby 21-05-2017
simple image editor in Rust for Redox
*/
extern crate orbtk;
extern crate orbimage;
extern crate image;
extern crate orbclient;

use orbtk::{Color, Action, Button, Image, Label, Menu, Point, ProgressBar, ControlKnob, Rect, Separator,
            TextBox, Window, Renderer};
use orbtk::traits::{Click, Enter, Place, Text};  //Border
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use orbtk::cell::CloneCell;
use std::sync::Arc;
use std::process;
use std::process::Command;
use std::path::Path;
use std::env;
use std::slice;
use std::collections::HashMap;


/*
enum Tools {
    pen,
    line,
}
*/

//implements structures to create new tools and store properties 
#[allow(dead_code)]
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
    #[allow(dead_code)]
    fn name<S: Into<String>>(&self, text: S) -> &Self {
        self.name.set(text.into());
        self
    }
    #[allow(dead_code)]
    fn value(&self, value: i32) -> &Self {
        self.value.set(value);
        self
    }
}

#[allow(dead_code)]
struct Settings {
    description: CloneCell<String>,     //tool's long description to be used in help popup
    size: Cell<i32>,                    
    //property: Vec<Cell<&Property>>,
    selected: Cell<bool>,
}

impl Settings {
    fn new() -> Arc<Self> {
        Arc::new(Settings {
            description: CloneCell::new(String::new()),
            size: Cell::new(0),
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
}

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

    //use Hash to save tools properties
    let mut tools = HashMap::new();

    //create tools and save initial size property

    let pen_tool = Settings::new();
    pen_tool.description("pen").size(1);
    tools.insert("pen",pen_tool);
    
    let line_tool = Settings::new();
    line_tool.description("line").size(1);
    tools.insert("line",line_tool);
    
    let brush_tool = Settings::new();
    brush_tool.description("brush").size(20);
    tools.insert("brush",brush_tool);
    
    let fill_tool = Settings::new();
    fill_tool.description("fill").size(0);
    tools.insert("fill",fill_tool);
    
    //TODO case for tools with many properties
    //create new tool with some properties and initial values
    let mut ntools = HashMap::new();
    ntools.insert("pen",vec![Property::new("Size",1),Property::new("Transparency",0)]);
    ntools.insert("line",vec![]); //no properties
    ntools.insert("brush",vec![Property::new("size",4)]);
    ntools.insert("fill",vec![]);
    
    //println!("{}",tools.get(&"pen").unwrap().name.get());
    //println!("{}",tools.get(&"pen").unwrap().size.get());


    //temporary use invisible Label for storing curent active tool
    let tool = Label::new();
    tool.text("pen");

    //implement GUI

    //resizable main window
    let mut window = Window::new_flags(Rect::new(100, 100, 1024, 768),
                                       "Pastel",
                                       &[orbclient::WindowFlag::Resizable ]);

    // color swatch
    let swatch = Label::new();
    swatch.text("■").position(320,80).size(56,16);
    //swatch.fg.set(orbtk::Color::rgb(r,g,b));
    window.add(&swatch);
    
    //color picker
    let red_bar = ProgressBar::new();
    let green_bar = ProgressBar::new();
    let blue_bar = ProgressBar::new();

    let red_label = Label::new();
    red_label.text("R: 0").position(x, y).size(48, 16);
    red_label.fg.set(orbtk::Color::rgb(255,0,0));
    window.add(&red_label);

    // use forked version of orbtk to get ProgressBar rendered in colors setting fg
    // compile with cargo flag --features colored
    if cfg!(feature = "colored"){red_bar.fg.set(orbtk::Color::rgb(255,0,0));}  
    
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
    
    if cfg!(feature = "colored"){green_bar.fg.set(orbtk::Color::rgb(0,255,0));}
    
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
    
    if cfg!(feature = "colored") {blue_bar.fg.set(orbtk::Color::rgb(0,0,255));}
    
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
    let tools_clone=tools.clone();
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
                      let cur_tool = tool_clone.text.get();
                      let a: &str = &cur_tool[..];  //FIXME workarround to convert String into &str                      
                      tools_clone[a].size(progress);
                      
                  });
    window.add(&size_bar);

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
            //let id=window.add(&image);
            //window.remove(id);  //test widget deletion from window
            
        }
        Err(err) => {
            let label = Label::new();
            label.position(x, y).size(400, 16).text(err);
            window.add(&label);
            
        }
    }
    
    
    
    // tools panel
    let y = 25;
    match Image::from_path("res/pencil1.png") {
        Ok(image) => {
            image.position(x, y);
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let tools_clone = tools.clone();
            let ntools_clone = ntools.clone();
            let size_label_clone = size_label.clone();
            let window_clone = &mut window as *mut Window;
            image.on_click(move |_image: &Image, _point: Point| {
                               println!("Pencil clicked");
                               tool_clone.text.set("pen".to_owned());
                               let v=tools_clone.get(&"pen").unwrap().size.get();
                               
                               size_bar_clone.value.set(v);
                               size_label_clone.text(format!("Size: {}",v));
                               size_bar_clone.visible.set(false);
                               size_label_clone.visible.set(false);
                               
                               //TODO clear window area reserved for tools properties
                               //    draw widgets for tool properties
                               //unsafe{prop_area(&ntools_clone["pen"],&mut *window_clone, 11);}
                               
                               
                               
                           });
            window.add(&image);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading tools panel {}",err);
        }
    }

    match Image::from_path("res/pencil2.png") {
        Ok(image) => {
            image.position(x, y);
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let tools_clone = tools.clone();
            let ntools_clone = ntools.clone();
            let window_clone = &mut window as *mut Window;
            image.on_click(move |_image: &Image, _point: Point| {
                               //set curent tool
                               println!("Line clicked");
                               tool_clone.text.set("line".to_owned());
                               //get previous settings
                               let v=tools_clone.get(&"line").unwrap().size.get();
                               size_bar_clone.value.set(v);
                               size_label_clone.text(format!("Size: {}",v));
                               
                               //    draw widgets for tool properties
                               //unsafe {prop_area(&ntools_clone["line"],&mut *window_clone);}
                               
                           });
            window.add(&image);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading tools panel {}",err);
        }
    }

    match Image::from_path("res/brush.png") {
        Ok(image) => {
            image.position(x, y);
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let tools_clone = tools.clone();
            let ntools_clone = ntools.clone();
            let window_clone = &mut window as *mut Window;
            image.on_click(move |_image: &Image, _point: Point| {
                               println!("Brush clicked");
                               tool_clone.text.set("brush".to_owned());
                               size_label_clone.visible.set(true);
                               size_bar_clone.visible.set(true);
                               let v=tools_clone.get(&"brush").unwrap().size.get();
                               size_bar_clone.value.set(v);
                               size_label_clone.text(format!("Size: {}",v));
                               
                               //    draw widgets for tool properties
                               //unsafe {prop_area(&ntools_clone["brush"],&mut *window_clone);}
                           });
            window.add(&image);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading tools panel {}",err);
        }
    }

    match Image::from_path("res/fillbucket.png") {
        Ok(image) => {
            image.position(x, y);
            let tool_clone = tool.clone();
            let size_bar_clone = size_bar.clone();
            let size_label_clone = size_label.clone();
            let tools_clone = tools.clone();
            let ntools_clone = ntools.clone();
            let window_clone = &mut window as *mut Window;
            image.on_click(move |_image: &Image, _point: Point| {
                               println!("Fill clicked");
                               tool_clone.text.set("fill".to_owned());
                               size_label_clone.visible.set(false);
                               size_bar_clone.visible.set(false);
                               //let v=tools_clone.get(&"fill").unwrap().size.get();
                               //size_bar_clone.value.set(v);
                               //size_label_clone.text(format!("Size: {}",v));
                               
                               //    draw widgets for tool properties
                               //unsafe {prop_area(&ntools_clone["brush"],&mut *window_clone);}
                           });
            window.add(&image);

            //x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading tools panel {}",err);
        }
    }

    //x = 10;

    //Menu file

    let menu = Menu::new("File");
    menu.position(10, 0).size(32, 16);

    //menu entries for file
            //TODO ask for new dimensions
    {
        let action = Action::new("New");
        action.on_click(move |_action: &Action, _point: Point| {
            
                           Command::new("./target/release/pastel")
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
                                    Command::new("./target/release/pastel")
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

    canvas
        .position(0, 250)
        .on_click(move |canvas: &Image, point: Point| {

            let click = click_pos.clone();
            let size = size_bar.clone().value.get();
            {
                let mut prev_opt = click.borrow_mut();

                if let Some(prev_position) = *prev_opt {
                    let mut image = canvas.image.borrow_mut();
                    let r = (red_bar.clone().value.get() as f32 * 2.56) as u8;
                    let g = (green_bar.clone().value.get() as f32 * 2.56) as u8;
                    let b = (blue_bar.clone().value.get() as f32 * 2.56) as u8;
                    
                    match tool.clone().text.get().as_ref() {
                        "line"  => {
                                    image.line(prev_position.x,
                                                prev_position.y,
                                                point.x,
                                                point.y,
                                                orbtk::Color::rgb(r, g, b));
                                   },
                         "pen"  => image.pixel(point.x, point.y, orbtk::Color::rgb(r, g, b)),
                         "brush"=> image.circle(point.x, point.y,-size,orbtk::Color::rgb(r, g, b)),
                         "fill" => image.fill(point.x, point.y,orbtk::Color::rgb(r, g, b)),
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

/*
    'event: while window.running.get() {
            window.drain_events();
            window.draw_if_needed();
            window.drain_orbital_events();

        }
*/    
}

//Load an image from path if exists, other way create new empty canvas
fn load_image(path: &str, size: &MySize) -> std::sync::Arc<orbtk::Image> {
    print!("Loading image from:  {} .....", path);
    match Image::from_path(&path) {
        Ok(image) => {
            println!(" OK");
            image
        }
        Err(err) => {
            println!("Failed: {} \n Creating new one ", err);
            let image = Image::from_color(size.x, size.y, Color::rgb(255, 255, 255));
            image
        }
    }
}

fn prop_area(properties: &Vec<Arc<Property>>, window: &mut orbtk::Window, id: usize) {
    ///This is the tool properties area that shows different widgets for different tools.
    
    //window.add(&label);//does not work , panics at runtime, because we cannot add widgets at runtime

    //But knowing the widget id we can hide or unhide it at runtime
    
    window.remove(id); //It works !!

    //window.add(&label);//does not work , panics at runtime, 
    //because we cannot add widgets when rendering the window

    //window.close(); //works
    for prop in properties {
        println!("Drawing widget for property {}",prop.name.get());
    }
        
}


//dialog window
fn dialog(title: &str, text: &str) -> Option<String> {
    let mut new_window = Window::new(Rect::new(200, 200, 320, 100), title);

    let x = 10;
    let mut y = 10;

    let label = Label::new();
    label.position(x, y).size(290, 16).text(text);
    new_window.add(&label);

    y += label.rect.get().height as i32 + 2;

    let text_box = TextBox::new();
    text_box.position(x, y).size(290, 28).text_offset(6, 6);

    //pressing enter in text_box closes popup window
    {
        let text_box = text_box.clone();
        let new_window_clone = &mut new_window as *mut Window;
        //let label = label.clone();
        text_box.on_enter(move |_| {
            //text_box: &TextBox

            unsafe {
                (&mut *new_window_clone).close();
            }
        });
    }
    new_window.add(&text_box);

    y += text_box.rect.get().height as i32 + 8;

    //OK button
    let ok_button = Button::new();
    ok_button
        .position(x, y)
        .size(48 + 12, text_box.rect.get().height)
        .text("OK")
        .text_offset(6, 6);

    {
        let text_box = text_box.clone();
        let button = ok_button.clone();
        button.on_click(move |_button: &Button, _point: Point| { text_box.emit_enter(); });
    }
    new_window.add(&ok_button);

    //Cancell button
    let cancel_button = Button::new();
    cancel_button
        .position(x + 64, y)
        .size(48 + 12, text_box.rect.get().height)
        .text("Cancel")
        .text_offset(6, 6);

    {
        let text_box = text_box.clone();
        let button = cancel_button.clone();
        button.on_click(move |_button: &Button, _point: Point| {
                            text_box.emit_enter();
                            text_box.text.set("".to_owned());

                        });
    }
    new_window.add(&cancel_button);
    new_window.exec();

    match text_box.text.get().len() {
        0 => None,
        _ => Some(text_box.text.get()),
    }
}

//popup window
fn popup(title: &str, text: &str) {
            
    let mut new_window = Window::new_flags(Rect::new(200, 200, 300, 100),
                                    title,&[orbclient::WindowFlag::Resizable,orbclient::WindowFlag::Async ]);
    let x = 10;
    let mut y = 10;

    let label = Label::new();
    label.position(x, y).size(400, 32).text(text);
    new_window.add(&label);

    y += label.rect.get().height as i32 + 12;

    //Close button
    let close_button = Button::new();
    close_button
        .position(x + 80, y)
        .size(48 + 12, 24)
        .text("Close")
        .text_offset(6, 6);
    {
        let button = close_button.clone();
        let new_window_clone = &mut new_window as *mut Window;
        button.on_click(move |_button: &Button, _point: Point| unsafe {
                            (&mut *new_window_clone).close();
                        });
    }

    new_window.add(&close_button);
    new_window.exec();

   
}

// come implementare nuove funzioni a crates già esistenti (non modificabili direttamente)

trait AddOnsToImage {
    fn save(&self, filename: &String);
    fn clear(&self);

    
}

impl AddOnsToImage for orbtk::Image {
    fn save(&self, filename: &String) {
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
                .unwrap();
        println!("Saved");
    }
    fn clear(&self){
       let mut image = self.image.borrow_mut();
       //image.clear();
       image.set(Color::rgb(255, 255, 255));
       
    }

}

/*
//remove a widget from window by id number (added to orbtk fork)
trait AddOnsToWindows {
    fn remove(&self, id: usize);
}

impl AddOnsToWindows for orbtk::Window {
    fn remove(&self, id: usize){
    let mut widgets = self.widgets.borrow_mut();
    widgets.remove(id);
    
    }
}
*/

// in future to be added directly to orbclient
trait AddOnsToOrbimage {
        fn fill(&mut self, x: i32 , y: i32, color: Color);
        fn flood_fill4 ( &mut self, x:i32, y:i32, new_color: u32 , old_color: u32);
        fn pixcol(&self, x:i32, y:i32) -> Color;
        
    }

impl AddOnsToOrbimage for orbimage::Image {
    ///return rgba color of pixel at position (x,y)
    fn pixcol(&self, x:i32, y:i32) -> Color {
        let p = self.width()as i32 * y + x;
        let rgba = self.data()[p as usize];
        rgba
    }
    
    
    fn fill(&mut self, x: i32, y: i32 , color: Color){
        //get curent pixel color 
        let rgba = self.pixcol(x,y);
        self.flood_fill4(x,y,color.data,rgba.data);  //use rgba and color as i32 values 
    }

    ///Recursive 4-way floodfill (be aware of stack overflow !!)
    fn flood_fill4 ( &mut self, x:i32, y:i32, new_color: u32 , old_color: u32) {
        if x >= 0 && x < self.width()as i32 && y >= 0 && y < self.height() as i32 
            && self.pixcol(x,y).data == old_color && self.pixcol(x,y).data != new_color {
            
            self.pixel(x,y, Color{data:new_color});
            
            self.flood_fill4(x+1,y,new_color,old_color);
            self.flood_fill4(x-1,y,new_color,old_color);
            self.flood_fill4(x,y+1,new_color,old_color);
            self.flood_fill4(x,y-1,new_color,old_color);
        }
    }


}
