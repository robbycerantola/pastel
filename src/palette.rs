extern crate orbtk;

use orbtk::{Color, Action, Button, Image, Label, Menu, Point, ProgressBar,
            ControlKnob,Toolbar, ToolbarIcon, Rect, Separator,
            TextBox, Window, Renderer, ColorSwatch}; //Toolbar
use orbtk::traits::{Click, Place, Text};  //Border, Enter
use std;
use std::borrow::BorrowMut;



///draw a palette
pub(crate) fn palette (start_y: i32, max_swatches: u32, window: &Window, swatch: std::sync::Arc<orbtk::ColorSwatch>) {
    
    let default_palette = vec![
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
    Color::rgb(7,49,61)    
    ];
    
    let mut s: std::sync::Arc<orbtk::ColorSwatch>;
    let mut color: Color;
    
    
    for k  in 0..max_swatches {
        color=default_palette[k as usize];
        s=ColorSwatch::new();
        s.position(24*(k+1) as i32,start_y)
        .size(24, 24)
        .color(color);
        
        let swatch_clone = swatch.clone(); 
        s.on_click(move |_swatch: &ColorSwatch, _point: Point| {
            
            swatch_clone.color(color);
        });
    
        window.add(&s);
    }
}
