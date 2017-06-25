extern crate orbtk;
//extern crate orbimage;
extern crate orbclient;

use orbtk::{Color, Action, Button, Image, Label, Menu, Point, ProgressBar, ControlKnob, Rect, Separator,
            TextBox, Window, Renderer};
use orbtk::traits::{Click, Enter, Place, Text};  //Border


//dialog window
pub fn dialog(title: &str, text: &str) -> Option<String> {
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
pub fn popup(title: &str, text: &str) {
            
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
