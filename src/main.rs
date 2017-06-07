/*Pastel by Robby 21-05-2017
simple image editor in Rust for Redox
*/



extern crate orbtk;

extern crate orbimage;

extern crate image;

extern crate orbclient;

use orbtk::{Color, Action, Button, Image, Label, Menu, Point, ProgressBar, Rect, Separator,
            TextBox, Window, Renderer};
use orbtk::traits::{Border, Click, Enter, Place, Text};

use std::rc::Rc;
use std::cell::RefCell;
use std::process;
use std::process::Command;

use std::path::Path;
use std::env;

use std::slice;
/*
enum Tools {
    pen,
    line,
}
*/
struct MySize {
    x: u32,
    y: u32,
}


fn main() {

    let mut size = MySize{x: 500, y:500};    
    let mut x = 10;
    let mut y = 56;

    let mut filename;

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

    //load existing file or create new with filename size
    let mut canvas = load_image(&filename, &size);



    let tool = Label::new();
    tool.position(0, 0).size(400, 16).text("pen");

    //let mytool = Tools::pen;

    //implement GUI

    //resizable main window
    let mut window = Window::new_flags(Rect::new(100, 100, 1024, 768),
                                       "Pastel",
                                       &[orbclient::WindowFlag::Resizable]);


    let red_label = Label::new();
    red_label.text("Red: 0%").position(x, y).size(400, 16);
    window.add(&red_label);

    y += red_label.rect.get().height as i32 + 2;

    let red_bar = ProgressBar::new();
    red_bar
        .position(x, y)
        .size(400, 16)
        .on_click(move |red_bar: &ProgressBar, point: Point| {
                      let progress = point.x * 100 / red_bar.rect.get().width as i32;
                      red_label.text.set(format!("Red: {}%", progress));
                      red_bar.value.set(progress);
                  });
    window.add(&red_bar);

    y += red_bar.rect.get().height as i32 + 2;

    let green_label = Label::new();
    green_label.text("Green: 0%").position(x, y).size(400, 16);
    window.add(&green_label);

    y += green_label.rect.get().height as i32 + 2;

    let green_bar = ProgressBar::new();
    green_bar
        .position(x, y)
        .size(400, 16)
        .on_click(move |green_bar: &ProgressBar, point: Point| {
                      let progress = point.x * 100 / green_bar.rect.get().width as i32;
                      green_label.text.set(format!("Green: {}%", progress));
                      green_bar.value.set(progress);
                  });
    window.add(&green_bar);

    y += green_bar.rect.get().height as i32 + 2;


    let blue_label = Label::new();
    blue_label.text("Blue: 0%").position(x, y).size(400, 16);
    window.add(&blue_label);

    y += blue_label.rect.get().height as i32 + 2;

    let blue_bar = ProgressBar::new();
    blue_bar
        .position(x, y)
        .size(400, 16)
        .on_click(move |blue_bar: &ProgressBar, point: Point| {
                      let progress = point.x * 100 / blue_bar.rect.get().width as i32;
                      blue_label.text.set(format!("Blue: {}%", progress));
                      blue_bar.value.set(progress);
                  });
    window.add(&blue_bar);

    y += blue_bar.rect.get().height as i32 + 10;


    //clickable icon
    match Image::from_path("res/pastel100.png") {
        Ok(image) => {
            //let tool_clone = tool.clone();
            image.position(900, 10);
            image.on_click(move |_image: &Image, _point: Point| {
                               println!("Clicked");
                               //tool_clone.text.set("pen".to_owned());
                           });
            window.add(&image);

            //y += image.rect.get().height as i32 + 10;
        }
        Err(err) => {
            let label = Label::new();
            label.position(x, y).size(400, 16).text(err);
            window.add(&label);

            //y += label.rect.get().height as i32 + 10;
        }
    }

    // tools panel

    let y = 25;
    match Image::from_path("res/pencil1.png") {
        Ok(image) => {
            image.position(x, y);
            let tool_clone = tool.clone();
            image.on_click(move |_image: &Image, _point: Point| {
                               println!("Pencil clicked");
                               tool_clone.text.set("pen".to_owned());
                           });
            window.add(&image);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading tools panel");


        }
    }

    match Image::from_path("res/line.png") {
        Ok(image) => {
            image.position(x, y);
            let tool_clone = tool.clone();
            image.on_click(move |_image: &Image, _point: Point| {
                               println!("Line clicked");
                               tool_clone.text.set("line".to_owned());
                           });
            window.add(&image);

            x += image.rect.get().width as i32 + 2;
        }
        Err(err) => {
            println!("Error loading tools panel");


        }
    }

    x = 10;



    //Menu file

    let menu = Menu::new("File");
    menu.position(10, 0).size(32, 16);

    //menu entries for file

    {
        let action = Action::new("New");
        //let offset_label_clone = offset_label.clone();

        action.on_click(move |_action: &Action, _point: Point| {
            
                            let output = Command::new("./target/release/pastel")
                                                .arg("new.png")
                                                .arg("200x200")
                                                .spawn()
                                                .expect("Command executed with failing error code");
                           
                            println!("New window opened.");



                        });

        menu.add(&action);
    }

    {
        let action = Action::new("Open");
        //let offset_label_clone = offset_label.clone();
        action.on_click(move |_action: &Action, _point: Point| {
            //offset_label_clone.text.set("Open".to_owned());
            //Open Popup
            //let response = popup("Open","path:").unwrap();
            match dialog("Open", "path:") {
                Some(response) => println!("Open {}", response),
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


    //Menu image
    let menuimage = Menu::new("Image");
    menuimage.position (100,0).size (48,16);
    
    //Menu entries for image
    {
            let action = Action::new("Rotate 180");
            let canvas_clone = canvas.clone();
            action.on_click(move |_action: &Action, _point: Point| {
                            canvas_clone.rotate180();
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
                                  "Pastel, simple image editor \n for Redox OS by Robby Cerantola");
                        });
        help.add(&action);
    }



    // add menues
    window.add(&menu);
    window.add(&tools);
    window.add(&menuimage);
    window.add(&help);


    let click_pos: Rc<RefCell<Option<Point>>> = Rc::new(RefCell::new(None));

    let red_bar_clone = red_bar.clone();
    let green_bar_clone = green_bar.clone();
    let blue_bar_clone = blue_bar.clone();
    let tool_clone = tool.clone();

    canvas
        .position(0, 250)
        .on_click(move |canvas: &Image, point: Point| {
            let click = click_pos.clone();
            {

                let mut prev_opt = click.borrow_mut();

                if let Some(prev_position) = *prev_opt {
                    let mut image = canvas.image.borrow_mut();
                    let r = (red_bar_clone.value.get() as f32 * 2.55) as u8;
                    let g = (green_bar_clone.value.get() as f32 * 2.55) as u8;
                    let b = (blue_bar_clone.value.get() as f32 * 2.55) as u8;
                    //println!("r{} g{} b{}",r,g,b);
                    if tool_clone.text.get() == "line" {

                        image.line(prev_position.x,
                                   prev_position.y,
                                   point.x,
                                   point.y,
                                   orbtk::Color::rgb(r, g, b));
                    }
                    if tool_clone.text.get() == "pen" {

                        image.pixel(point.x, point.y, orbtk::Color::rgb(r, g, b));
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
        .position(x + 40, y)
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
    let mut new_window = Window::new(Rect::new(200, 200, 300, 100), title);
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

trait Improvements {
    fn save(&self, filename: &String);
    fn rotate180(&self);
}

impl Improvements for orbtk::Image {
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
        // we have to flip r with b but dont know why!!

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
    fn rotate180(&self){
        println!("Rotate180 not implemented yet");
       //Self::from_image(image::imageops::rotate180(&self.image.into_raw()));
    }

}
