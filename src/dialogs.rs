use orbclient::WindowFlag;
use orbtk::{ Button, Label, Point, Rect, TextBox, Window, InnerWindow}; //Color, Action,ControlKnob, Image, Menu, ProgressBar, Separator, Renderer
use orbtk::traits::{Click, Enter, Place, Text};  //Border
use orbtk::dialogs::FileDialog;

use std::ops::Deref;
use std::path::PathBuf;

use DEFAULTFONT;

//generic dialog window
pub fn dialog(title: &str, text: &str, suggestion: &str) -> Option<String> {
    
    //let mut new_window = Window::new(Rect::new(100, 100, 320, 100), title); 
    
    let mut orb_window = Some(InnerWindow::new(100, 100, 320, 100, title).unwrap());
    let mut new_window = Box::new(Window::from_inner(orb_window.take().unwrap()));

    let x = 10;
    let mut y = 10;

    let label = Label::new();
    label.position(x, y).size(290, 16).text(text);
    new_window.add(&label);

    y += label.rect.get().height as i32 + 2;

    let text_box = TextBox::new();
    text_box.position(x, y)
    .size(290, 28)
    .text_offset(6, 6)
    .text(suggestion);

    //pressing enter in text_box closes popup window
    {
        let text_box = text_box.clone();
        //let new_window_clone = &mut new_window as *mut Window;
        let new_window_clone = new_window.deref() as *const Window;
        //let label = label.clone();
        text_box.on_enter(move |_| {
            //text_box: &TextBox

            unsafe {
                //(&mut *new_window_clone).close();
                (*new_window_clone).close();
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

//dialog window for text tool
pub fn text_dialog(title: &str, text: &str, suggestion: &str) -> Option<(String,String)> {
    let mut orb_window = Some(InnerWindow::new(100, 100, 480, 200, title).unwrap());
    let mut new_window = Box::new(Window::from_inner(orb_window.take().unwrap()));

    let x = 10;
    let mut y = 10;

    let label = Label::new();
    label.position(x, y).size(290, 16).text(text);
    new_window.add(&label);

    y += label.rect.get().height as i32 + 2;

    let text_box = TextBox::new();
    text_box.position(x, y)
    .size(390, 28)
    .text_offset(6, 6)
    .text(suggestion);

    //pressing enter in text_box closes popup window
    {
        let text_box = text_box.clone();
        let new_window_clone = new_window.deref() as *const Window;
        text_box.on_enter(move |_| {
            unsafe {
                (*new_window_clone).close();
            }
        });
    }
    new_window.add(&text_box);

    y += text_box.rect.get().height as i32 + 8;

    let label2 = Label::new();
    label2.position(x, y).size(290, 16).text("Font path:");
    new_window.add(&label2);

    y += label.rect.get().height as i32 + 2;


    let font_path = DEFAULTFONT;

    let text_box2 = TextBox::new();
    text_box2.position(x, y)
    .size(390, 28)
    .text_offset(6, 6)
    .text(font_path);

// select font name
    {
        let text_box2 = text_box2.clone();
        let new_window_clone = new_window.deref() as *const Window;
        text_box.on_enter(move |_| {
            unsafe {
                (*new_window_clone).close();
            }
        });
    }
    new_window.add(&text_box2);


    //Browse button
    let browse_button = Button::new();
    browse_button
        .position(x + 10 + text_box.rect.get().width as i32 , y)
        .size(70, text_box.rect.get().height)
        .text("Browse")
        .text_offset(6, 6);

    {
        let text_box2 = text_box2.clone();
        browse_button.on_click(move |_button: &Button, _point: Point| { 
            let mut p =PathBuf::from(font_path);// set font path accordingly with default font path
            p.pop(); //get rid of name.ext
            let mut f = FileDialog::new();
            f.title = "Font selection".to_owned(); 
            f.path = p; 
            match f.exec() {
                Some(response) => {
                    text_box2.text(response.to_string_lossy());
                },
                None => println!("Cancelled"),
            }
        });
    }
    new_window.add(&browse_button);

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
        ok_button.on_click(move |_button: &Button, _point: Point| { text_box.emit_enter(); });
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
        _ => Some((text_box.text.get(),text_box2.text.get())),
    }
}

pub fn new_dialog(title: &str) -> Option<String> {
    //Dialog to input dimensions of new image
    let mut new_window = Window::new(Rect::new(200, 300, 320, 200), title);

    let x = 10;
    let mut y = 10;

    let label = Label::new();
    label.position(x, y).size(290, 16).text("New image dimensions in pixels:".to_owned());
    new_window.add(&label);

    y += label.rect.get().height as i32 + 20;
    
    let labelx = Label::new();
    labelx.position(x, y+4).size(100, 16).text("Width".to_owned());
    new_window.add(&labelx);

    let text_box_x = TextBox::new();
    text_box_x.position(x+ 50, y)
    .size(50, 28)
    .text_offset(6, 6)
    .text("640");

    //pressing enter in text_box closes popup window
    {
        let text_box = text_box_x.clone();
        let new_window_clone = &mut new_window as *mut Window;
        //let label = label.clone();
        text_box.on_enter(move |_| {
            //text_box: &TextBox

            unsafe {
                (&mut *new_window_clone).close();
                
            }
        });
    }
    new_window.add(&text_box_x);

    y += text_box_x.rect.get().height as i32 + 8;

    let labely = Label::new();
    labely.position(x, y+4).size(100, 16).text("Height".to_owned());
    new_window.add(&labely);
    
    let text_box_y = TextBox::new();
    text_box_y.position(x+50, y)
    .size(50, 28)
    .text_offset(6, 6)
    .text("480");

    {
        let text_box = text_box_y.clone();
        let new_window_clone = &mut new_window as *mut Window;
        //let label = label.clone();
        text_box.on_enter(move |_| {
            //text_box: &TextBox

            unsafe {
                (&mut *new_window_clone).close();
            }
        });
    }
    new_window.add(&text_box_y);

    y += text_box_y.rect.get().height as i32 + 12;

    //OK button
    let ok_button = Button::new();
    ok_button
        .position(x, y)
        .size(48 + 12, text_box_y.rect.get().height)
        .text("OK")
        .text_offset(6, 6);

    {
        let text_box = text_box_x.clone();
        let button = ok_button.clone();
        button.on_click(move |_button: &Button, _point: Point| { text_box.emit_enter(); });
    }
    new_window.add(&ok_button);

    //Cancell button
    let cancel_button = Button::new();
    cancel_button
        .position(x + 64, y)
        .size(48 + 12, text_box_y.rect.get().height)
        .text("Cancel")
        .text_offset(6, 6);

    {
        let text_box = text_box_x.clone();
        let button = cancel_button.clone();
        button.on_click(move |_button: &Button, _point: Point| {
                            text_box.emit_enter();
                            text_box.text.set("".to_owned());
                            
                        });
    }
    new_window.add(&cancel_button);
    new_window.exec();
    
    let resolution = format!("{}x{}",text_box_x.text.get().trim(),text_box_y.text.get().trim());

    match text_box_x.text.get().len() {
        0 => None,
        _ => Some(resolution),
    }
}



//popup window
pub fn popup(title: &str, text: &str) {
            
    let mut new_window = Window::new_flags(Rect::new(200, 200, 300, 100),
                                    title,&[WindowFlag::Resizable,WindowFlag::Async ]);
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
